actor A {
    func m() : async () {
    };

    func _t0() : async () {
        try { await m() }
        else catch _ {}
        finally { ignore m() } // BAD: no effects allowed!
    };

    func _t1() : async () {
        try { await m() }
        else catch _ {}
        finally { 42 } // BAD: should return unit.
    }
}
