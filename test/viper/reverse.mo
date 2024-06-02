import Array "mo:base/Array";
import Prim "mo:⛔";

// @verify
actor Reverse {
  var xarray : [var Nat] = [var 1, 2, 3, 4, 5];

  private func copy_xarray(): [var Nat] {
    assert:return (var:return).size() == xarray.size();
    let length = xarray.size();
    let t = Array.init<Nat>(length, 0);
    var i = 0;
    while (i < length) {
        assert:loop:invariant (i >= 0);
        assert:system (i < length);
        t[i] := xarray[i];
        i := i + 1;
    };
    return t;
  };

  private func reverseArray<T>(a : [var T]) : () {
    assert:return a.size() == (old (a.size()));
    assert:return Prim.forall<Nat>(
      func (k : Nat) = (0 <= k and k < a.size()) implies a[k] == (old (a[a.size() - 1 - k])));
    let b = [1, 2, 4]; // space variable to test loop invariant deducing
    var length = a.size();
    if (length == 0) {
      return;
    };
    // At this point the line below cannot trap
    var i = length - 1;
    var j = 0;
    while (i > j) {
      assert:loop:invariant (i < length and i >= 0);
      assert:loop:invariant (j < length and j >= 0);
      assert:loop:invariant (i == a.size() - 1 - j);
      assert:loop:invariant Prim.forall<Nat>(
        func (k : Nat) = (j <= k and k <= i) implies a[k] == (old (a[k])));
      assert:loop:invariant Prim.forall<Nat>(
        func (k : Nat) = (0 <= k and k < j) implies a[k] == (old (a[a.size() - 1 - k])));
      assert:loop:invariant Prim.forall<Nat>(
        func (k : Nat) = (i < k and k < a.size()) implies a[k] == (old (a[a.size() - 1 - k])));
      var tmp = a[i];
      a[i] := a[j];
      a[j] := tmp;
      i := i - 1;
      j := j + 1;
    };
    return;
  };

  public func reverse() : async () {
    var a = copy_xarray();
    reverseArray(a);
    xarray := a;
  };
};
