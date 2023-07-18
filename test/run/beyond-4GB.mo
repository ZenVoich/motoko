// test allocation beyond the 32-bit address space
import P "mo:⛔";
do {

  let expectedSize = 10 * 1024 * 1024 * 1024; // 10 GB
  var c = 5;

  while(c > 0) {
    let a : [var Nat8] = P.Array_init<Nat8>(1024*1024*1024/4, 0xFF);
    c -= 1;
  };

  
  assert(P.rts_memory_size() > expectedSize);
  assert(P.rts_heap_size() > expectedSize);
}

//SKIP run
//SKIP run-low
//SKIP run-ir


