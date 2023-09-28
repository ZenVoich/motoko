# Orthogonal Persistence (Stable Heap)

This realizes the vision of keeping the canister main memory persistent even across upgrades and thus allows scalable upgrades.
Canister upgrades do no longer involve serialization and deserialization to and from secondary stable memory.

## Purpose
* **Instantenous upgrades**: New program versions simply resume on the existing main memory and have access to the memory-compatible data.
* **Scalable upgrades**: The upgrade mechanism scales with larger heaps and in contrast to serialization, does not hit IC instruction limits.

## Broader Vision
In the longer term, this approach aims to enable **true orthogonal persistence** that is simple, flexible, efficient, and scalable.
While this version implements the runtime support for 32-bit memory, this could be leveraged to 64-bit persistent main memory in future.
As a result, the use of secondary storage (explicit stable memory, dedicated stable data structures, DB-like storage abstractions) will no longer be necessary: 
Motoko developers could directly work on their normal object-oriented program structures that are automatically persisted and retained across program version changes.
With 64-bit main memory, large-scaled orthogonal persistence would be enabled, supported by the incremental GC that is designed to also scale in 64-bit.

## Design
The stable heap is based on the following main properties:
* Extension of the IC to retain main memory on upgrades.
* A long-term memory layout that is invariant to new compiled program versions.
* A fast memory compatibility check performed on each canister upgrade.
* Incremental garbage collection using a partitioned heap.

### IC Extension
As a prerequisite for the stable heap support, the IC runtime support has to be extended in order not to erase the main memory on upgrades.
This is realized in a specific IC branch (https://github.com/luc-blaeser/ic/tree/luc/stable-heap-on-release) that retains the main memory even on upgrades, similar to normal canister message execution. 
The only data that needs to be updated on an upgrade is the Wasm data segments specified in the Wasm binary, e.g. text literals. 
For this purpose, a special memory section is reserved for data segments.

### Memory Layout
In a co-design between the compiler and the runtime system, the main memory is arranged in the following structure, invariant of the compiled program version:
* Lower 2MB: Rust call stack.
* Space between 2MB and 6MB: Reserved space Wasm data segments.
* Between 6MB and 6.125MB: Persistent metadata.
* Thereafter: Dynamic heap space. Fix start address at 6.125MB.

### Persistent Metadata
The persistent metadata describes all anchor information for the program to resume after an upgrade. 
More specifically, it comprises:
* A stable heap version that allows evolving the persistent memory layout in the future.
* The stable subset of the main actor, containing all stable variables declared in the main actor.
* A descriptor of the stable static types to check memory compatibility on upgrades.
* The reference to the null singleton object.
* The runtime state of the garbage collector, including the dynamic heap metadata and memory statistics.
* A reserve for future metadata extensions.

### Compatibility Check
Upgrades are only permitted if the new program version is compatible to the old version, such that the runtime system guarantees a compatible memory structure.

Compatible changes for immutable types are equivalent to the allowed Motoko subtype relation, e.g.
* Adding or removing actor fields.
* Removing object fields.
* Adding variant fields.
* `Nat` to `Int`.
* Shared function parameter contravariance and return type covariance.

The existing IDL-subtype functionality is reused with some adjustments to check memory compatibility: The compiler generates the type descriptor, a type table, that is recorded in the persistent metadata. Upon an upgrade, the new type descriptor is compared against the existing type descriptor, and the upgrade only succeeds for compatible changes.

This compatibility check serves as an additional safety measure on top of the DFX Candid subtype check that can be bypassed by users (when ignoring a warning). Moreoever, the memory compatibility rules is in some aspects different to the Candid sub-type check:
* Types cannot be made optional.
* Mutable types (aliases) are supported with type invariance.

### Garbage Collection
The implementation focuses on the incremental GC and abandons the other GCs because the GCs use different memory layouts. For example, the incremental GC uses a partitioned heap with objects carrying a forwarding pointer.

The incremental GC is chosen because it is designed to scale on large heaps and the stable heap design also aims to increase scalability. Moreover, it is suited to scale on 64-bit memory in future.

The garbage collection state needs to be persisted and retained across upgrades. 
This is because the GC may not yet be completed at the time of an upgrade, such that object forwarding is still in use. The partition table is stored as part of the GC state.

The garbage collector uses two kinds of roots:
* Persistent roots: These refer to root objects that need to survive canister upgrades.
* Transient roots: These cover additional roots that are only valid in a specific version of a program and are discarded on an upgrade.

The persistent roots are registered in the persistent metadata and comprise:
* All stable variables of the main actor, only stored during an upgrade.
* The null singleton object.
* The stable type table.

The transient roots are referenced by the Wasm data segments and comprise:
* All canister variables of the current version, including flexible variables.

### Main Actor
On an upgrade, the main actor is recreated and existing stable variables are recovered from the persistent root.
The remaining actor variables, the flexible fields as well as new stable variables, are (re)initialized.
As a result, the GC can collect unreachable flexible objects of previous canister versions. 
Unused stable variables of former versions can also be reclaimed by the GC.

### Wasm Data Segments
Wasm data segments are reinitialized on upgrade. 
This is necessary because data segments may contain text literals or transient GC roots that are bound to a specific new Wasm binary.
The system reserves dedicated space for data segments, namely between 2MB and 6MB. 
Therefore, the data segments are limited to 4MB per canister Wasm in this design. 
Both the linker and the IC runtime system check that the data segments fit inside this reserved space.

There exist alternative design possibilities to handle data segments, see below.

### No Static Heap
The static heap is abandoned and former static objects need to be allocated in the dynamic heap.
This is because these objects may also need to survive upgrades and must not be not overwritten by new data segments. 

The incremental GC also operates on these objects, meaning that forwarding pointer resolution is also necessary for these objects.

The runtime systems avoids any global Wasm variables for state that needs to be preserved on upgrades.
Instead, such global runtime state is stored in the persistent metadata.

Sharing optimization (pooling) is possible for compile-time-known objects, see below.

### Null Singleton
As an optimization, the top-level `null`` singleton is allocated once in the dynamic heap and remembered in the persistent metadata across upgrades. 
This is necessary to implement null checks by pointer comparison (however, by first resolving pointer forwarding before the comparison).
The null singleton needs to be part of the persistent root set.

### Migration Path
When migrating from the old serialization-based stabilization to the new stable heap, the old data is deserialized one last time from stable memory and then placed in the new stable heap layout.
Once operating on the stable heap, the system prevents downgrade attempts to the old serialization-based persistence.

### Old Stable Memory
The old stable memory remains equally accessible as secondary memory with the new support.

## Possible Extensions
The following extensions or optimization could be applied in the future:
* Unlimited data segments: Using passive Wasm data segments and loading them at runtime to dynamically computed addresses in the heap. The IC would however need to be extended to support passive Wasm data segments.
* 64-bit memory: Extend the main memory to 64-bit by using Wasm64, see https://github.com/dfinity/motoko/pull/4136. The memory layout would need to be extended. Moreover, it would be beneficial to introduce a dynamic partition table for the GC. Ideally, stable heap support is directly rolled out for 64-bit to avoid complicated memory layout upgrades from 32-bit to 64-bit.
* Object pooling: Compile-time-known objects can be shared in the dynamic heap by remembering them in an additional pool table. The pool table needs to be registered as a transient GC root and is recreated on canister upgrades.

## Related PRs

* IC with stable main memory support: https://github.com/luc-blaeser/ic/tree/luc/stable-heap-on-release
* Wasm64 Support for Motoko: https://github.com/dfinity/motoko/pull/4136