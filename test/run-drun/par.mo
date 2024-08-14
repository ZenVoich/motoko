import { debugPrint } "mo:⛔";
import Cycles = "cycles/cycles";

actor {

    func foo(next : () -> async ()) : async () {
        await (with cycles = 3000) next()
    };

    func bar(next : () -> async ()) : async () = async {
        await (with cycles = 4000) next()
    };

    public func oneshot() {
    };

    public func test() : async () {
        debugPrint "test()";
        let message = "Hi!";

        func closA() : async Nat {
            message.size()
        };

        func closB() : async Nat = async {
            message.size()
        };
/*
         // this is ruled out
        func closC() : async Nat =
          if (1 == 2) async {
            message.size()
          } else async {
            message.size() + 1
          };
*/

/*      //Rule this out?
        func closD() : async Nat = (with cycles = 765) async {
            message.size()
        };    
*/

        assert 3 == (await (with cycles = 101) closA());
        assert 3 == (await (with cycles = 102) closB());

        await (with yeah = 8; timeout = 55; cycles = 1000)
        foo(func() : async () = async { assert message == "Hi!" });
        await (with cycles = 5000)
        bar(func() : async () = async { assert message == "Hi!" });
    };

    public func test2() : async () {
        debugPrint "test2()";
        await (with cycles = 1042) async { assert Cycles.available() == 1042 };
        await (with cycles = 3042) (with cycles = 4042) async { assert Cycles.available() == 3042/*FIXME: WHY?*/ };
    }
}

// testing
//SKIP run
//SKIP run-ir
//SKIP run-low

//CALL ingress test "DIDL\x00\x00"
//CALL ingress test2 "DIDL\x00\x00"
