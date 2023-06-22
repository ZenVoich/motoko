//MOC-FLAG --stable-regions

import P "mo:⛔";
import M "../stable-mem/StableMemory";
import Region "../stable-region/Region";

actor {

    // measure out three blocks' worth of bytes.
    let pageInBytes = 1 << 16 : Nat64;
    let blockInBytes = pageInBytes * 128 : Nat64;
    let size = blockInBytes * 3 : Nat64;
    var i = 0 : Nat64;

    // Check size for necessary number of pages.
    let reqPages = size / pageInBytes;
    assert M.size() == reqPages;

    // Load out previously-stored byte pattern, one byte at a time.
    // Check each byte is what we would have written, if we were repeating the same logic again.
    while (i < size) {
        let expected = P.natToNat8(P.nat64ToNat(i % 256)) : Nat8;
        let loaded = M.loadNat8(i);
        assert loaded == expected;
        i := i + 1;
    };

    P.debugPrint ("actor1: checked region0.");

    stable var r1 = Region.new();

    P.debugPrint ("actor1: allocated another region.");
}
