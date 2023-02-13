// The 2 palindrome implementations from
// "There and Back Again", by Olivier Danvy and Mayer Goldberg
//MOC-FLAG --force-gc
import { error; performanceCounter; rts_heap_size; debugPrint ; stableMemoryGrow; stableMemoryLoadBlob} = "mo:⛔";


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

    }
}
//SKIP run-low
//SKIP run
//SKIP run-ir
//SKIP ic-ref-run
//CALL ingress go 0x4449444C0000
//CALL ingress go 0x4449444C0000
//CALL ingress go 0x4449444C0000