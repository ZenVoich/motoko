//MOC-FLAG --sanity-checks
import P "mo:⛔";
import StableMemory "stable-mem/StableMemory";
actor {

  let 0 = StableMemory.grow(32768-128);
  let n = 2**30+16384;
  let o:Nat64 = 2**30+16384 - 1;
  StableMemory.storeNat8(o, 255);
  let b = StableMemory.loadBlob(0, n);
  StableMemory.storeNat8(o, 0);
  assert StableMemory.loadNat8(o) == 0;
  StableMemory.storeBlob(0, b);
  P.debugPrint(debug_show StableMemory.loadNat8(o));
  assert StableMemory.loadNat8(o) == 255;
  assert b.size() == n;
  P.debugPrint("ok");
  assert false;
}

//SKIP run
//SKIP run-low
//SKIP run-ir
// too slow on ic-ref-run:
//SKIP comp-ref


