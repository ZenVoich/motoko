import Prim "mo:⛔";
import Cycles = "cycles/cycles";
import Cs "actor-class-mgmt/C";

actor a {
  type Change_origin = {
      #from_user : {
          user_id : Principal;
      };
      #from_canister : {
          canister_id : Principal;
          canister_version : ?Nat64;
      };
  };

  type Change_details = {
      #creation : { controllers : [Principal] };
      #code_uninstall;
      #code_deployment : {
          mode : { #install; #reinstall; #upgrade};
          // module_hash : Blob; // introduces non-determinism when codegen improves
      };
      #controllers_change : {
          controllers : [Principal];
      };
  };

  type Change = {
      // timestamp_nanos : Nat64; // just omit this
      canister_version : Nat64;
      origin : Change_origin;
      details : Change_details;
  };

  let ic00 = actor "aaaaa-aa" :
    actor {
      create_canister : {
        settings : ? {
          controllers : ?[Principal];
          compute_allocation: ?Nat;
          memory_allocation: ?Nat;
        freezing_threshold: ?Nat;
       }
     } -> async { canister_id : Principal };

      canister_info : {
          canister_id : Principal;
          num_requested_changes : ?Nat64;
      } -> async {
          total_num_changes : Nat64;
          recent_changes : [Change];
          // module_hash : ?Blob;
          controllers : [Principal];
      };
   };

  let default_settings = { settings = null };
  // same as default but explicit
  let settings = { settings = ? {
     controllers = null;
     compute_allocation = null;
     freezing_threshold = null;
     memory_allocation = null;
    };
  };

  public func go () : async () {
    // To get lots of cycles in both drun and ic-ref-run
    if (Cycles.balance() == 0)
      await Cycles.provisional_top_up_actor(a, 100_000_000_000_000);

    do {
      Cycles.add<system>(2_000_000_000_000);
      let c0 = await
        Cs.C (0, ?(Prim.principalOfActor a));
      assert ({args = 0; upgrades = 0} == (await c0.observe()));

      Cycles.add<system>(2_000_000_000_000);
      let c1 = await
        (system Cs.C)(#new default_settings)(1, null);
      assert ({args = 1; upgrades = 0} == (await c1.observe()));
      assert (c1 != c0);

      Cycles.add<system>(2_000_000_000_000);
      let c2 = await
        (system Cs.C)(#new settings)(2, null);
      assert ({args = 2; upgrades = 0} == (await c2.observe()));
      assert (c2 != c1);

      Cycles.add<system>(2_000_000_000_000);
      let {canister_id = p} = await
         ic00.create_canister default_settings;
      // no need to add cycles
      let c3 = await
        (system Cs.C)(#install p)(3, null);
      assert ({args = 3; upgrades = 0} == (await c3.observe()));
      assert (Prim.principalOfActor c3 == p);
      assert (c3 != c2);

      // no need to add cycles
      let c4 = await
        (system Cs.C)(#upgrade c3)(4, null);
      assert ({args = 4; upgrades = 1} == (await c4.observe()));
      assert (c4 == c3);

      // no need to add cycles
      let c5 = await
        (system Cs.C)(#upgrade_with_persistence { wasm_memory_persistence = #Keep ; canister = c4 })(5, null);
      assert ({args = 5; upgrades = 2} == (await c5.observe()));
      assert (c5 == c4);

      // no need to add cycles
      let c6 = await
        (system Cs.C)(#reinstall c5)(6, null);
      assert ({args = 6; upgrades = 0} == (await c6.observe()));
      assert (c6 == c5);

      // no need to add cycles
      let c7 = await
        (system Cs.C)(#upgrade_with_persistence { wasm_memory_persistence = #Replace ; canister = c6 })(7, null);
      assert ({args = 7; upgrades = 0} == (await c7.observe()));
      assert (c7 == c6);

      let info = await ic00.canister_info {
          canister_id = p;
          num_requested_changes = ?4
      };

      Prim.debugPrint (debug_show info);
    };
  };

}

//CALL ingress go "DIDL\x00\x00"
//SKIP run
//SKIP run-ir
//SKIP run-low
