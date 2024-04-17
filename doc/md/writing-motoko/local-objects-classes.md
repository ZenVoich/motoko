# Objects and classes

<!--
TODO: Move examples into doc/modules/language-guide/examples
-->

## Overview

Motoko programs benefit from the ability to encapsulate state as objects with abstract types, as an `object` may encapsulate local state as `var`-bound variables by packaging this state with `public` methods that access and update it.

Motoko objects that include mutable state are not shareable as a critical security-oriented design decision. If they were shareable, that would expose two main security risks: conceptually moving a mobile object’s code among actors and executing it remotely, and sharing state with remote logic.

However, objects may be pure records which are shareable since they are free from mutable state. The [mutable state](mutable-state.md) introduces declarations of private mutable state in the form of `var`-bound variables and mutable array allocation.

To compensate for this necessary limitation, `actor` objects are shareable, but always execute remotely. They communicate with shareable Motoko data only. Local objects interact in less restricted ways with themselves, and can pass any Motoko data to each other’s methods, including other objects. In most other ways, local objects and classes are non-shareable counterparts to actor objects and classes.


## Object and actor classes

**Object classes** : A family of related objects to perform a task with a customizable initial state. Motoko provides a syntactical construct, called a `class` definition, which simplifies building objects of the same type and implementation.

**Actor classes** : An object class that exposes a [service](async-data.md) using asynchronous behavior. The corresponding Motoko construct is an [actor class](actor-classes.md), which follows a similar but distinct design.

## Example

The following example illustrates a general evolution path for Motoko programs. Each object has the potential to be refactored into a service by refactoring the local object into an actor object.

Consider the following object declaration of the object value `counter`:

``` motoko
object counter {
  var count = 0;
  public func inc() { count += 1 };
  public func read() : Nat { count };
  public func bump() : Nat {
    inc();
    read()
  };
};
```

This declaration introduces a single object instance named `counter`. The developer exposes three public functions `inc`, `read` and `bump` using keyword `public` to declare each in the object body. The body of the object, like a block expression, consists of a list of declarations.

In addition to these three functions, the object has one private mutable variable `count`, which holds the current count and is initially zero.

## Object types

This object `counter` has the following object type, written as a list of field-type pairs, enclosed in braces `{` and `}`:

``` motoko no-repl
{
  inc  : () -> () ;
  read : () -> Nat ;
  bump : () -> Nat ;
}
```

Each field type consists of an identifier, a colon `:`, and a type for the field content. Here, each field is a function, and thus has an arrow type form (`_ → _`).

In the declaration of `object`, the variable `count` was explicitly declared neither as `public` nor as `private`.

By default, all declarations in an object block are `private`. Consequently, the type for `count` does not appear in the type of the object. Its name and presence are both inaccessible from the outside.

By not exposing this implementation detail, the object has a more general type with fewer fields, and as a result, is interchangeable with objects that implement the same counter object type differently without using such a field.

To illustrate the point just above, consider this variation of the `counter` declaration above, of `byteCounter`:

``` motoko
import Nat8 "mo:base/Nat8";
object byteCounter {
  var count : Nat8 = 0;
  public func inc() { count += 1 };
  public func read() : Nat { Nat8.toNat(count) };
  public func bump() : Nat { inc(); read() };
};
```

This object has the same type as the previous one, and thus from the standpoint of type checking, this object is interchangeable with the prior one:

``` motoko no-repl
{
  inc  : () -> () ;
  read : () -> Nat ;
  bump : () -> Nat ;
}
```

This version does not use the same implementation of the counter field. Rather than use an ordinary natural `Nat`, this version uses a byte-sized natural number, type `Nat8`, whose size is always eight bits.

As such, the `inc` operation may fail with an overflow for this object but never the prior one, which may instead fill the program’s memory.

Neither implementation of a counter comes without some complexity. In this case, they share a common type.

A common type shared among two implementations of an object or service affords the potential for the internal implementation complexity to be factored away from the rest of the application.

Objects can also have [subtypes](object-subtyping.md).

## Object classes

In Motoko, an object encapsulates state, and an object `class` is a package of two entities that share a common name.

Consider this example `class` for counters that start at zero:

``` motoko name=counter
class Counter() {
  var c = 0;
  public func inc() : Nat {
    c += 1;
    return c;
  }
};
```

The value of this definition is that we can construct new counters, each starting with their own unique state, initially at zero:

``` motoko name=cinit include=counter
let c1 = Counter();
let c2 = Counter();
```

Each is independent:

``` motoko include=counter,cinit
let x = c1.inc();
let y = c2.inc();
(x, y)
```

You could achieve the same results by writing a function that returns an object:

``` motoko
func Counter() : { inc : () -> Nat } =
  object {
    var c = 0;
    public func inc() : Nat { c += 1; c }
  };
```

Notice the return type of this constructor function is an object type:

``` motoko no-repl
{ inc : () -> Nat }
```

You may want to name this type such as `Counter` for use in further type declarations:

``` motoko no-repl
type Counter = { inc : () -> Nat };
```

The `class` keyword syntax shown above is a shorthand for these two definitions of `Counter`: a factory function `Counter` that constructs objects, and the type `Counter` of these objects. Classes do not provide any new functionality beyond this convenience.

### Class constructor

An object class defines a constructor function that may carry zero or more data arguments and zero or more type arguments.

The `Counter` example above has zero of each.

The type arguments, if any, parameterize both the type and the constructor function for the class.

The data arguments, if any, parameterize only the constructor function for the class.

#### Data arguments

Suppose you want to initialize the counter with some non-zero value. You can supply that value as a data argument to the `class` constructor:

``` motoko
class Counter(init : Nat) {
  var c = init;
  public func inc() : Nat { c += 1; c };
};
```

This parameter is available to all methods. For instance, you can `reset` the `Counter` to its initial value, a parameter:

``` motoko
class Counter(init : Nat) {
  var c = init;
  public func inc() : Nat { c += 1; c };
  public func reset() { c := init };
};
```

#### Type arguments

Suppose you want the counter to actually carry data that it counts, like a specialized `Buffer`.

When classes use or contain data of arbitrary type, they carry a type argument. This is equivalent to a type parameter for an unknown type, just as with functions.

The scope of this type parameter covers the entire `class` with data parameters. As such, the methods of the class can use these type parameters without reintroducing them.

``` motoko
import Buffer "mo:base/Buffer";

class Counter<X>(init : Buffer.Buffer<X>) {
  var buffer = init.clone();
  public func add(x : X) : Nat {
    buffer.add(x);
    buffer.size()
  };

  public func reset() {
    buffer := init.clone()
  };
};
```

#### Type annotation

The class constructor may also carry a type annotation for its return type. When supplied, Motoko checks that this type annotation is compatible with the body of the class, which is an object definition. This check ensures that each object produced by the constructor meets the supplied specification.

For example, repeat the `Counter` as a buffer and annotate it with a more general type `Accum<X>` that permits adding, but not resetting, the counter. This annotation ensures that the objects are compatible with the type `Accum<X>`.

``` motoko
import Buffer "mo:base/Buffer";

type Accum<X> = { add : X -> Nat };

class Counter<X>(init : Buffer.Buffer<X>) : Accum<X> {
  var buffer = init.clone();
  public func add(x : X) : Nat { buffer.add(x); buffer.size() };
  public func reset() { buffer := init.clone() };
};
```

#### Full syntax

Classes are defined by the keyword `class`, followed by:

- A name for the constructor and type being defined. For example, `Counter`.

- Optional type arguments. For example, omitted, or `<X>`, or `<X, Y>`.

- An argument list. For example, `()`, or `(init : Nat)`, etc.

- An optional type annotation for the constructed objects. For example, omitted, or `Accum<X>`.

- The class "body" is an object definition, parameterized by the type and value arguments, if any.

The constituents of the body marked `public` contribute to the resulting objects' type and these types compared against the optional annotation, if given.

Consider the task of walking the bits of a natural `Nat` number. For this example, you could define the following:

``` motoko
class Bits(n : Nat) {
  var state = n;
  public func next() : ?Bool {
    if (state == 0) { return null };
    let prev = state;
    state /= 2;
    ?(state * 2 != prev)
  }
}
```

The above class definition is equivalent to the simultaneous definition of a structural type synonym and a factory function, both named `Bits`:

``` motoko no-repl
type Bits = {next : () -> ?Bool};
func Bits(n : Nat) : Bits = object {
  // class body
};
```


