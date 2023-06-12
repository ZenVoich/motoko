import Prim "mo:⛔";


assert(Prim.textCompare("" , "") == 0);
assert(Prim.blobCompare("",  "") == 0);
assert(Prim.textCompare("a", "a") == 0);
assert(Prim.blobCompare("a", "a") == 0);
assert(Prim.textCompare("a", "") == 1);
assert(Prim.blobCompare("a", "") == 1);
assert(Prim.textCompare("a", "b") == -1);
assert(Prim.blobCompare("a", "b") == -1);
assert(Prim.textCompare("",  "b") == -1);
assert(Prim.blobCompare("",  "b") == -1);
assert(Prim.textCompare("x",  "x") == 0);
assert(Prim.blobCompare("x",  "x") == 0);
assert(Prim.textCompare("xa", "xa") == 0);
assert(Prim.blobCompare("xa", "xa") == 0);
assert(Prim.textCompare("xa", "x") == 1);
assert(Prim.blobCompare("xa", "x") == 1);
assert(Prim.textCompare("xa", "xb") == -1);
assert(Prim.blobCompare("xa", "xb") == -1);
assert(Prim.textCompare("x",  "xb") == -1);
assert(Prim.blobCompare("x",  "xb") == -1);

