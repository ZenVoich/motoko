shared(c) actor class C() = c {

  public shared ctxt func c1 () : async actor {} {
     return c;
  };

};

//contains features that can't be lowered (toplevel awaits)
//SKIP run-low

ignore (await C()).c1(); //OR-CALL ingress c1 RElETAAA
