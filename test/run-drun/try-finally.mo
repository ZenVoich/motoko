import { debugPrint; error; call_raw; principalOfActor } =  "mo:⛔";

actor A {
    func m() : async () {
    };

    public func raw() : async () {
    };

    func t0() : async () {
        try { debugPrint "IN"; await m(); assert false }
        finally { debugPrint "OUT" };
    };

    func t0r() : async () {
        let p = principalOfActor A;
        try { debugPrint "INr"; ignore await call_raw(p, "raw", to_candid()); assert false }
        finally { debugPrint "OUTr" };
    };

    func t0d() : async () {
        try { debugPrint "INd"; let fut = m(); await fut; debugPrint "AGAINd"; await fut; assert false }
        finally { debugPrint "OUTd" };
    };

/*  nested `try` won't work yet
    func t1() : async () {
        try {
            do {
                debugPrint "IN1";
                throw error "IN1";
            }
            finally { debugPrint "OUT1" };
        }
        catch _ { debugPrint "CAUGHT1" }
    };
*/

    func t2() : async () {
        try {
            debugPrint "IN2";
            throw error "IN2";
        }
        catch _ { debugPrint "CAUGHT2" }
        finally { debugPrint "OUT2" };
    };

    func t2i() : async Int {
        try {
            debugPrint "IN2i";
            await async ();
            throw error "IN2i";
        }
        catch _ { debugPrint "CAUGHT2i"; 42 }
        finally { debugPrint "OUT2i" };
    };

    func t2t() : async () {
        try {
            debugPrint "IN2t";
            await m();
            throw error "IN2t";
        }
        catch _ { debugPrint "CAUGHT2t"; assert false }
        finally { debugPrint "OUT2t" };
    };

    func t3() : async () {
        try {
            debugPrint "IN3";
            await m();
            return;
        }
        finally { debugPrint "OUT3" };
    };

    // check that finally is not running twice
    func t4() : async () {
        try {
            debugPrint "IN4";
            await m();
        }
        finally { debugPrint "OUT4" };
        return;
    };

    func t4f() : async () {
        try {
            debugPrint "IN4f";
            await m();
        }
        finally { debugPrint "OUT4f"; return };
    };

    func t5() : async () {
        debugPrint "BEFORE5";
        label out try {
            debugPrint "IN5";
            await m();
            break out;
            debugPrint "DEAD5";
        }
        finally { debugPrint "OUT5" };
        debugPrint "AFTER5"
    };

    func t6() : async () {
        debugPrint "BEFORE6";
        label out try {
            debugPrint "IN6";
            try {
                debugPrint "InnerIN6";
                await m();
                debugPrint "InnerLIVE6";
                break out;
                debugPrint "InnerDEAD6";
            } finally { debugPrint "InnerOUT6" };
            debugPrint "DEAD6";
        }
        finally { debugPrint "OUT6" };
        debugPrint "AFTER6"
    };

    func t6t() : async () {
        debugPrint "BEFORE6t";
        label out try {
            debugPrint "IN6t";
            try {
                debugPrint "InnerIN6t";
                await m();
                debugPrint "InnerLIVE6t";
                assert false;
                debugPrint "InnerDEAD6t";
            } finally { debugPrint "InnerOUT6t" };
            debugPrint "DEAD6t";
        }
        finally { debugPrint "OUT6t" };
        debugPrint "AFTER6t"
    };

    func t6c() : async () {
        debugPrint "BEFORE6c";
        label out try {
            debugPrint "IN6c";
            try {
                debugPrint "InnerIN6c";
                await m();
                debugPrint "InnerLIVE6c";
                throw error "InnerIN6c";
            } catch _ {
                debugPrint "InnerCATCH6c";
                assert false;
                debugPrint "DEADCATCH6c";
            } finally { debugPrint "InnerOUT6c" };
            debugPrint "DEAD6c";
        }
        finally { debugPrint "OUT6c" };
        debugPrint "AFTER6c"
    };

    func t6d() : async () {
        debugPrint "BEFORE6d";
        label out try {
            debugPrint "IN6d";
            try {
                debugPrint "InnerIN6d";
                let fut = m();
                await fut;
                debugPrint "InnerLIVE6d";
                await fut;
                debugPrint "InnerLIVESTILL6d";
                assert false;
                debugPrint "InnerDEAD6d";
            } finally { debugPrint "InnerOUT6d" };
            debugPrint "DEAD6d";
        }
        finally { debugPrint "OUT6d" };
        debugPrint "AFTERDEAD6d"
    };

    // `await*` tests
    func t7() : async* () {
        debugPrint "BEFORE7";
        label out try {
            debugPrint "IN7";
            try {
                debugPrint "InnerIN7";
                let fut = m();
                await fut;
                debugPrint "InnerLIVE7";
                await fut;
                debugPrint "InnerLIVESTILL7";
                assert false;
                debugPrint "InnerDEAD7";
            } finally { debugPrint "InnerOUT7" };
            debugPrint "DEAD7";
        }
        finally { debugPrint "OUT7" };
        debugPrint "AFTERDEAD7"
    };

    func t8() : async () {
        try {
            debugPrint "IN8";
            // await* async* throw error "IN8"
            // https://github.com/dfinity/motoko/issues/4578
            await* async* { throw error "IN8"; () }
        }
        catch _ { debugPrint "CAUGHT8" }
        finally { debugPrint "OUT8" };
    };

    func t8i() : async () {
        // see: https://github.com/dfinity/motoko/issues/4578
        func inner() : async* () = async* { throw error "IN8i" };

        try {
            debugPrint "IN8i";
            await* inner()
        }
        catch _ { debugPrint "CAUGHT8i" }
        finally { debugPrint "OUT8i" };
    };

    func t8t() : async () {
        func inner() : async* () = async* { debugPrint "InnerIN8t"; await m(); assert true };

        try {
            debugPrint "IN8t";
            await* inner()
        }
        finally { debugPrint "OUT8t" };
    };

    public func go() : async () {
        // These don't trap (for the interpreters)
        //await t1();
        await t2();
        ignore await t2i();
        await t3();
        await t4();
        await t4f();
        await t5();
        await t6();
        await t8();
        await t8i();

        // These trap, and only work on drun
        try /*ignore*/ await t0() catch _ {};
        try await t0r() catch _ {};
        try await t0d() catch _ {};
        try await t2t() catch _ {};
        try await t6c() catch _ {};
        try await t6t() catch _ {};
        try await t6d() catch _ {};
        try await t8t() catch _ {};
        /// caveat: t7 won't return!
        try await* t7() catch _ {} finally debugPrint "It's over";
    };

    public func go2() : async () {
        /// caveat: the `await*` won't return!
        try await* async* {
            await m();
            debugPrint "go2";
            assert false }
        catch _ {}
        finally debugPrint "It's so over";
    }
};

//SKIP ic-ref-run

A.go(); //OR-CALL ingress go "DIDL\x00\x00"
//CALL ingress go2 "DIDL\x00\x00"
