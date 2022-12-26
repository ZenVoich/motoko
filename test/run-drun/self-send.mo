//MOC-ENV MOC_UNLOCK_PRIM=yesplease
import Prim "mo:⛔";

actor self {

  let raw_rand = (actor "aaaaa-aa" : actor { raw_rand : () -> async Blob }).raw_rand;

  public func request() : async () {
  };

  public func oneway() : () {
  };

  public func test1() : async () {
    var n = 0;
    while (n < 1000) {
//      Prim.debugPrint(debug_show n);
      ignore request();
      n += 1;
    }

  };

  public func test2() : async () {
    try {
      var n = 0;
      while (n < 1000) {
//        Prim.debugPrint(debug_show n);
        ignore request();
        n += 1;
      }
    } catch e {
      Prim.debugPrint("caught " # Prim.errorMessage(e));
      throw e;
    }
  };


  public func test3() : async () {
    var n = 0;
    while (n < 1000) {
//      Prim.debugPrint(debug_show n);
      oneway();
      n += 1;
    }

  };

  public func test4() : async () {
    try {
      var n = 0;
      while (n < 1000) {
//        Prim.debugPrint(debug_show n);
        oneway();
        n += 1;
      }
    } catch e {
      Prim.debugPrint("caught " # Prim.errorMessage(e));
      throw e;
    }
  };

  public func test5() : async () {
    var n = 0;
    while (n < 1000) {
//    Prim.debugPrint(debug_show n);
      // NB: calling
      // ignore Prim.call_raw(Prim.principalOfActor(self),"request", to_candid ());
      // is not enough to trigger message send failure, because the Prim.call_raw is an
      // eta-expansion of prim "call_raw", and introduces an additional await, draining the queue.
      // Instead, we need to call the raw primitive:
      ignore (prim "call_raw" : (Principal, Text, Blob) -> async Blob) (Prim.principalOfActor(self),"request", to_candid ());
      //ignore request();
      n += 1;
    }

  };

  public func test6() : async () {
    try {
      var n = 0;
      while (n < 1000) {
//        Prim.debugPrint(debug_show n);
      // NB: calling
      // ignore Prim.call_raw(Prim.principalOfActor(self),"request", to_candid ());
      // is not enough to trigger message send failure, because the Prim.call_raw is an
      // eta-expansion of prim "call_raw", and introduces an additional await, draining the queue.
      // Instead, we need to call the raw primitive:
      ignore (prim "call_raw" : (Principal, Text, Blob) -> async Blob) (Prim.principalOfActor(self),"request", to_candid ());
        n += 1;
      }
    } catch e {
      Prim.debugPrint("caught " # Prim.errorMessage(e));
      throw e;
    }
  };


  public func go() : async () {

    Prim.debugPrint("test1:");

    try {
      await test1();
      assert false;
    }
    catch e {
      Prim.debugPrint("test1: " # Prim.errorMessage(e));
    };

    let _ = await raw_rand(); // drain queues, can't use await async() as full!

    Prim.debugPrint("test2:");
    try {
      await test2();
    }
    catch e {
      Prim.debugPrint("test2: " # Prim.errorMessage(e));
    };

    Prim.debugPrint("test3:");

    let _ = await raw_rand(); // drain queues, can't use await async() as full!

    try {
      await test3();
      assert false;
    }
    catch e {
      Prim.debugPrint("test3: " # Prim.errorMessage(e));
    };

    let _ = await raw_rand(); // drain queues, can't use await async() as full!

    Prim.debugPrint("test4:");
    try {
      await test4();
    }
    catch e {
      Prim.debugPrint("test4: " # Prim.errorMessage(e));
    };

    let _ = await raw_rand(); // drain queues, can't use await async() as full!

    // call_raw
    Prim.debugPrint("test5:");

    try {
      await test5();
      assert false;
    }
    catch e {
      Prim.debugPrint("test5: " # Prim.errorMessage(e));
    };

    let _ = await raw_rand(); // drain queues, can't use await async() as full!

    Prim.debugPrint("test6:");
    try {
      await test6();
    }
    catch e {
      Prim.debugPrint("test6: " # Prim.errorMessage(e));
    };

  }

};

//SKIP run
//SKIP run-ir
//SKIP run-low
//SKIP ic-ref-run

//await a.go(); //OR-CALL ingress go "DIDL\x00\x00"