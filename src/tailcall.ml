open Source
open Ir
open Effect
open Type
open Construct
module S = Syntax

(* Optimize (self) tail calls to jumps, avoiding stack overflow
   in a single linear pass *)

(*
This is simple tail call optimizer that replaces tail calls to the current function by jumps.
It can  easily be extended to non-self tail calls, once supported by wasm.

For each function `f` whose `body[...]` has at least one self tailcall to `f<Ts>(es)`, apply the transformation:
```
    func f<Ts>(pat) = body[f<Ts>(es)+]
    ~~>
    func f<Ts>(args) = {
       var temp = args;
       loop {
         label l {
           let pat = temp;
           return body[{temp := es;break l;}+]
        }
      }
    }
```


It's implemented by a recursive traversal that maintains an environment recording whether the current term is in tail position,
and what its enclosing function (if any) is.

The enclosing function is forgotten when shadowed by a local binding (we don't assume all variables are distinct) and when
entering a function, class or actor constructor.

On little gotcha for functional programmers: the argument `e` to an early `return e` is *always* in tail position,
regardless of `return e`s own tail position.

TODO: optimize for multiple arguments using multiple temps (not a tuple).

*)

type func_info = { func: S.id;
                   typ_binds: typ_bind list;
                   temps: var list;
                   label: S.id;
                   tail_called: bool ref;
                 }

type env = { tail_pos:bool;          (* is the expression in tail position *)
             info: func_info option; (* the innermost enclosing func, if any *)
           }


let bind env i (info:func_info option) : env =
  match info with
  | Some _ ->
    { env with info = info; }
  | None ->
    match env.info with
    | Some { func; _}  when i.it = func.it ->
      { env with info = None } (* remove shadowed func info *)
    | _ -> env (* preserve existing, non-shadowed info *)


let are_generic_insts (tbs : typ_bind list) insts =
  List.for_all2 (fun (tb : typ_bind) inst ->
      match inst with
      | Con(c2,[]) -> Con.eq tb.it.con c2 (* conservative, but safe *)
      |  _ -> false
      ) tbs insts

let rec tailexp env e =
  {e with it = exp' env e}

and exp env e  : exp =
  {e with it = exp' {env with tail_pos = false}  e}

and assignEs vars exp : dec list =
  match vars, exp.it with
  | [v], _ -> [ expD (assignE v exp) ]
  | _, TupE es when List.length es = List.length vars ->
       List.map expD (List.map2 assignE vars es)
  | _, _ ->
    let tup = fresh_var "tup" (typ exp) in
    letD tup exp ::
    List.mapi (fun i v -> expD (assignE v (projE v i))) vars

and exp' env e  : exp' = match e.it with
  | VarE _
    | LitE _
    | PrimE _             -> e.it
  | UnE (ot, uo, e)      -> UnE (ot, uo, exp env e)
  | BinE (ot, e1, bo, e2)-> BinE (ot, exp env e1, bo, exp env e2)
  | RelE (ot, e1, ro, e2)-> RelE (ot, exp env e1, ro, exp env e2)
  | ShowE (ot, e)       -> ShowE (ot, exp env e)
  | TupE es             -> TupE (List.map (exp env) es)
  | ProjE (e, i)        -> ProjE (exp env e, i)
  | DotE (e, sn)        -> DotE (exp env e, sn)
  | ActorDotE (e, sn)   -> ActorDotE (exp env e, sn)
  | AssignE (e1, e2)    -> AssignE (exp env e1, exp env e2)
  | ArrayE (m,t,es)     -> ArrayE (m,t,(exps env es))
  | IdxE (e1, e2)       -> IdxE (exp env e1, exp env e2)
  | CallE (cc, e1, insts, e2)  ->
    begin
      match e1.it, env with
      | VarE f1, { tail_pos = true;
                   info = Some { func; typ_binds; temps; label; tail_called } }
           when f1.it = func.it && are_generic_insts typ_binds insts  ->
        tail_called := true;
        (blockE (assignEs temps (exp env e2))
                 (breakE label (tupE []))).it
      | _,_-> CallE(cc, exp env e1, insts, exp env e2)
    end
  | BlockE (ds, e)      -> BlockE (block env ds e)
  | IfE (e1, e2, e3)    -> IfE (exp env e1, tailexp env e2, tailexp env e3)
  | SwitchE (e, cs)     -> SwitchE (exp env e, cases env cs)
  | LoopE e1            -> LoopE (exp env e1)
  | LabelE (i, t, e)    -> let env1 = bind env i None in
                           LabelE(i, t, exp env1 e)
  | BreakE (i, e)       -> BreakE(i,exp env e)
  | RetE e              -> RetE (tailexp { env with tail_pos = true } e)
  (* NB:^ e is always in tailposition, regardless of fst env *)
  | AsyncE e            -> AsyncE (exp { tail_pos = true; info = None } e)
  | AwaitE e            -> AwaitE (exp env e)
  | AssertE e           -> AssertE (exp env e)
  | OptE e              -> OptE (exp env e)
  | VariantE (i, e)     -> VariantE (i, exp env e)
  | DeclareE (i, t, e)  -> let env1 = bind env i None in
                           DeclareE (i, t, tailexp env1 e)
  | DefineE (i, m, e)   -> DefineE (i, m, exp env e)
  | FuncE (x, cc, tbs, as_, ret_tys, exp0) ->
    let env1 = { tail_pos = true; info = None} in
    let env2 = args env1 as_ in
    let exp0' = tailexp env2 exp0 in
    FuncE (x, cc, tbs, as_, ret_tys, exp0')
  | ActorE (i, ds, fs, t) -> ActorE (i, ds, fs, t) (* TODO: descent into ds *)
  | NewObjE (s,is,t)    -> NewObjE (s, is, t)

and exps env es  = List.map (exp env) es

and args env as_ =
  List.fold_left (fun env a -> bind env a None) env as_

and pat env p =
  let env = pat' env p.it in
  env

and pat' env p = match p with
  | WildP         ->  env
  | VarP i        ->
    let env1 = bind env i None in
    env1
  | TupP ps       -> pats env ps
  | ObjP pfs      -> pats env (pats_of_obj_pat pfs)
  | LitP l        -> env
  | OptP p
  | VariantP (_, p) -> pat env p
  | AltP (p1, p2) -> assert(Freevars.S.is_empty (snd (Freevars.pat p1)));
                     assert(Freevars.S.is_empty (snd (Freevars.pat p2)));
                     env

and pats env ps  =
  match ps with
  | [] -> env
  | p :: ps ->
    let env1 = pat env p in
    pats env1 ps

and case env (c : case) =
  { c with it = case' env c.it }
and case' env {pat=p;exp=e} =
  let env1 = pat env p in
  let e' = tailexp env1 e in
  { pat=p; exp=e' }


and cases env cs = List.map (case env) cs

and dec env d =
  let (mk_d,env1) = dec' env d in
  ({d with it = mk_d}, env1)

and dec' env d =
  match d.it with
  (* A local let bound function, this is what we are looking for *)
  (* TODO: Do we need to detect more? A tuple of functions? *)
  | LetD (({it = VarP id;_} as id_pat),
          ({it = FuncE (x, ({ Value.sort = Local; _} as cc), tbs, as_, typT, exp0);_} as funexp)) ->
    let env = bind env id None in
    begin fun env1 ->
      let temps = fresh_vars "temp" (List.map (fun a -> Mut a.note) as_) in
      let label = fresh_id "tailcall" () in
      let tail_called = ref false in
      let env2 = { tail_pos = true;
                   info = Some { func = id;
                                 typ_binds = tbs;
                                 temps;
                                 label;
                                 tail_called } }
      in
      let env3 = args env2 as_ in (* shadow id if necessary *)
      let exp0' = tailexp env3 exp0 in
      let cs = List.map (fun (tb : typ_bind) -> Con (tb.it.con, [])) tbs in
      if !tail_called then
        let ids = match typ funexp with
          | Func( _, _, _, dom, _) ->
            fresh_vars "id" (List.map (fun t -> open_ cs t) dom)
          | _ -> assert false
        in
        let l_typ = Type.unit in
        let body =
          blockE (List.map2 (fun t i -> varD (id_of_exp t) i) temps ids) (
            loopE (
              labelE label l_typ (blockE
                (List.map2 (fun a t -> letD (exp_of_arg a) (immuteE t)) as_ temps)
                (retE exp0'))
            )
          )
        in
        LetD (id_pat, {funexp with it = FuncE (x, cc, tbs, List.map arg_of_exp ids, typT, body)})
      else
        LetD (id_pat, {funexp with it = FuncE (x, cc, tbs, as_, typT, exp0')})
    end,
    env
  | LetD (p, e) ->
    let env = pat env p in
    (fun env1 -> LetD(p,exp env1 e)),
    env
  | VarD (i, e) ->
    let env = bind env i None in
    (fun env1 -> VarD(i,exp env1 e)),
    env
  | TypD _ ->
    (fun env -> d.it),
    env

and decs env ds =
  let rec decs_aux env ds =
    match ds with
    | [] -> ([],env)
    | d::ds ->
      let (mk_d, env1) = dec env d in
      let (mk_ds, env2) = decs_aux env1 ds in
      (mk_d :: mk_ds,env2)
  in
  let mk_ds,env1 = decs_aux env ds in
  ( env1
  , List.map
      (fun mk_d ->
        let env2 = { env1 with tail_pos = false } in
        { mk_d with it = mk_d.it env2 })
      mk_ds)

and block env ds exp =
  let (env1, ds') = decs env ds in
  (ds', tailexp env1 exp)

and decss env = function
  | [] -> (env, [])
  | ds::dss ->
    let (env1, ds') = decs env ds in
    let (env2, dss') = decss env1 dss in
    (env2, ds' :: dss')

and prog ((as_, dss, fs), flavor) =
  let env0 = { tail_pos = false; info = None } in 
  let (env1, dss') = decss env0 dss in
  (as_, dss', fs), flavor

(* validation *)

let transform (p : prog) = prog p
