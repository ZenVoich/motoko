actor class Counter () {

  var count = 0;

  // ...

  public shared query func peek() : async Nat {
    count
  };

  public shared composite query func sum(counters : [Counter]) : async Nat {
    var sum = 0;
    for (counter in counters.vals())  {
      sum += await counter.peek();
    };
    sum
  }

}
