func f() : async () { ignore await C(); };

actor class C() {

  let _ : Any = C;

  public shared ctxt func m () : async () {
     let _ : actor {} = await C();
  };

};

ignore (await C()).m() //OR-CALL ingress m RElETAAA

