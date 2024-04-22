---
sidebar_position: 4
---

# Verifying upgrade compatibility

## Overview

When upgrading a canister, it is important to verify that an upgrade can proceed without:

-   Breaking clients due to a Candid interface change.

-   Discarding the Motoko stable state due to a change in stable declarations.

Motoko checks these properties statically before attempting the upgrade.

## Upgrade example

The following is a simple example of how to declare a stateful counter:

``` motoko no-repl file=../examples/count-v0.mo
```

In this example, when the counter is upgraded, its state is lost.

To fix this, you can declare a stable variable that is retained across upgrades:


``` motoko no-repl file=../examples/count-v1.mo
```

If a variable is not marked `stable`, `state` would restart from `0` on upgrade.


## Evolving the Candid interface

In this example, old clients are still satisfied, while new ones get extra features such as the `read` query in this example.

``` motoko no-repl file=../examples/count-v2.mo
```

## Changing the stable interface

Let's take a look at an example where the counter is refactored from using `Int` to `Nat`.

``` motoko no-repl file=../examples/count-v3.mo
```

Now, the code has been upgraded, but the counter value is back to `0`. The state was lost in an upgrade.

This is because the Candid interface evolved safely​ but the stable types did not.

An upgrade must be able to:

-   Consume any stable variable value from its predecessor, or

-   Run the initializer for a new stable variable.

Since `Int </: Nat`, the upgrade logic discards the saved `Int` and re-runs the initializer instead. The upgrade silently "succeeded", resetting the counter to `0`.

## Stable type signatures

A stable type signature looks similar to the content within a Motoko actor type.

For example, `v2`'s stable types:

``` motoko no-repl file=../examples/count-v2.most
```

An upgrade from `v2` to `v3`'s stable types requires consuming an `Int` as a `Nat`, which is a **type error**.

``` motoko no-repl file=../examples/count-v3.most
```

## Dual interface evolution

An upgrade is safe provided that the Candid interface evolves to a subtype and the stable interface evolves to a compatible one: a stable variable must either be newly declared, or re-declared at a super type of its old type.

Consider the following four versions of the counter example:

Version `v0` with Candid interface `v0.did` and stable type interface `v0.most`:

``` candid file=../examples/count-v0.did
```

``` motoko no-repl file=../examples/count-v0.most
```

Version `v1` with Candid interface `v1.did` and stable type interface `v1.most`,

``` candid file=../examples/count-v1.did
```

``` motoko no-repl file=../examples/count-v1.most
```

Version `v2` with Candid interface `v2.did` and stable type interface `v2.most`,

``` candid file=../examples/count-v2.did
```

``` motoko no-repl file=../examples/count-v2.most
```

Version `v3` with Candid interface `v3.did` and stable type interface `v3.most`:

``` candid file=../examples/count-v3.did
```

``` motoko no-repl file=../examples/count-v3.most
```

The following table summarizes the (in)compatibilities between them:

|         |                  |                       |
|---------|------------------|-----------------------|
| Version | Candid interface | Stable type interface |
| `v0`    | `v0.did`         | `v0.most`             |
|         | :> ✓             | \<\<: ✓               |
| `v1`    | `v1.did`         | `v1.most`             |
|         | :> ✓             | \<\<: ✓               |
| `v2`    | `v2.did`         | `v2.most`             |
|         | :> ✓             | \<\<: *✗*             |
| `v3`    | `v3.did`         | `v3.most`             |

## Upgrade tooling

The Motoko compiler (`moc`) supports:

-   `moc --stable-types …​`: Emits stable types to a `.most` file.

-   `moc --stable-compatible <pre> <post>`: Checks two `.most` files for upgrade compatibility.

To upgrade from `cur.wasm` to `nxt.wasm` we need check that both the Candid interface and stable variables are compatible.

```
didc check nxt.did cur.did  // nxt <: cur
moc --stable-compatible cur.most nxt.most  // cur <<: nxt
```

Using the versions above, the upgrade from `v2` to `v3` fails this check:

```
> moc --stable-compatible v2.most v3.most
(unknown location): Compatibility error [M0170], stable variable state of previous type
  var Int
cannot be consumed at new type
  var Nat
```

Upgrades from `v2.wasm` to `v3.wasm` would fail and roll-back, avoiding data loss. If Candid is revised, an upgrade would now "succeed", but with data loss. This is the difference between a fail safe and a silent failure.

To upgrade correctly to change `state` to `Nat`, you can introduce a new stable variable, `newState`, initialized from the old one:

``` motoko no-repl file=../examples/count-v4.mo
```

``` motoko no-repl file=../examples/count-v4.most
```

## Incompatible upgrade example 

A common, real-world example of an incompatible upgrade can be found [on the forum](https://forum.dfinity.org/t/questions-about-data-structures-and-migrations/822/12?u=claudio/).

In that example, a user was attempting to add a field to the record payload of an array, by upgrading from stable type interface:

``` motoko no-repl
type Card = {
  title : Text
};
actor {
  stable var map: [(Nat32, Card)]
}
```

to *incompatible* stable type interface:

``` motoko no-repl
type Card = {
  title : Text;
  description : Text
};
actor {
  stable var map : [(Nat32, Card)]
}
```

Adding a new record field (to magic from nothing) does not work.
