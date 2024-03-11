//MOC-FLAG --compacting-gc --rts-stack-pages 32 -measure-rts-stack
import { errorMessage; debugPrint; } = "mo:⛔";

actor {
    let expectedMinimumSize = 31_000;

    public func ser() : async () { await go(false) };
    public func deser() : async () { await go(true) };

    public func go(deserialize : Bool) : async () {
        var i = 0;
        type List = {
          #some : ((), List);
          #none
        };
        var l : List = #none;
        var done = false;
        while (not done) {
          try {
            await async {
              var c = 0;
              while (c < 1024) {
                l := (#some ((),l));
                i += 1;
                c += 1
              };
              let b = to_candid(l);

              let o : ?(List) =
               if deserialize
                 from_candid(b)
               else null;
              ()
            }
          } catch e {
            debugPrint(errorMessage(e));
            done := true
          }
        };
        
        assert(i > expectedMinimumSize);
        
        let b = to_candid(l);
        debugPrint("serialized");

        let _o : ?(List) =
          if deserialize
            from_candid(b)
          else null;

        if deserialize debugPrint("deserialized");
    }


}
//SKIP run-low
//SKIP run
//SKIP run-ir
//SKIP ic-ref-run
//CALL ingress ser 0x4449444C0000
//CALL ingress deser 0x4449444C0000
