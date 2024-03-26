import Prim "mo:prim";

actor {
   stable var number : Nat = 0;

   public func modify() : async () {
      number += 1;
   };

   public func print() : async () {
      Prim.debugPrint(debug_show (number));
   };
};
