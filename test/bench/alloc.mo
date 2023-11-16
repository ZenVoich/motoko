//MOC-FLAG --force-gc
import { performanceCounter; rts_heap_size; debugPrint } = "mo:⛔";
// this test becomes much more expensive due to smaller compact bignums.
actor alloc {

    func counters() : (Int, Nat64) = (rts_heap_size(), performanceCounter(0));

    public func go() : async () {
        let (m0, n0) = counters();
        var i = 0;
        type List = ?((), List);
        var l : List = null;
        while (i < 1024*1024*16) {
          l := ?((),l);
          i += 1;
        };
        let (m1, n1) = counters();
        debugPrint(debug_show (m1 - m0, n1 - n0));
        assert false; // rollback changes
    }
}
//SKIP run-low
//SKIP run
//SKIP run-ir
//SKIP ic-ref-run
//CALL ingress go 0x4449444C0000
//CALL ingress go 0x4449444C0000
//CALL ingress go 0x4449444C0000
