# Stable Region API

See StableMemory.md for context of the current experimental API.

This document aims to specify the API and memory representations for a generalization
of this API that permits multiple isolated _regions_ of stable memory, where each can be
grown independently.

The **region manager** is the state and logic to support this generalization.


## Role for "stable regions" in Motoko

The current stable memory module in `base` has been "experimental" for a long time, and requires a more composable API to graduate from this status.

Stable regions address the problem that today's `ExperimentalStableMemory` module only provides a single, monolithic memory that makes it unsuitable for directly building composable software parts.

Stable regions permit a new API that supports composable use cases.

Stable regions also bring Motoko closer to parity with Rust canister development support today, by giving a run-time-system-based analog of a special Rust library for stable data structures that allocates “pages” for them from stable memory in separate, isolated, memory regions.


## Design space

The design space for the page allocator is defined by at least two
tensions:

 1. fully-stable representation of allocator meta data **versus** fast load/store operations.

 2. Total scaling capacity **versus** minimum footprint for meta data.


**Tension 1** is introduced because we want to avoid relying on the Motoko heap as the "ground truth" about the allocator's state.  If this heap is lost, as it is during an upgrade, then a developer may still want to recover all of the regions' data and meta data.

Tension 1 is resolved by storing the ground truth in stable memory, keeping it in sync with heap structures that permit faster access operations.

Compared with the Rust version, we store enough extra meta data to permit:

 - Regions whose page blocks are in arbitrary order, not
   necessarily in order of smallest to highest address.

 - 32767 Regions max (instead of 255 Regions max).
   (Could expand this to 64k with current design, or beyond with larger Region IDs).

We want to permit arbitrary page block orders to make a smooth
transition to region reclamation and re-allocation in the near
future, with potential integration into the Motoko GC.  The
extra complexity is modest, and seems "worth" the cost.

We change the maximum region limit because 255 may be too small in
some extreme cases.

We address the question of whether the new limit of 32k regions is
"enough" in the Q&A section (it is, for all practical purposes)


**Tension 2** is introduced because we want a design that will continue
to work even when canisters can store more stable data than today (32GB).

Tension 2 is resolved by making prudent representation choices.

The representations we choose for regions and region identifiers
permit a scaling to 256GB of stable data while still permitting meta
data to be repeated in both stable and non-stable arenas.  These are
the same limits imposed by the Rust implementation, for the same
reasons.  See Q&A for more discussion.


## Definitions and constants

 - a **page** is 65536 bytes.
 - a **page block** is a contiguous sequence of 128 pages (~8MB).
 - a **page block index** is a 16 bit, index-based identifier for a page block.
 - a **region** is a sequence of (generally non-contiguous) **page blocks**.
 - the maximum number of page blocks is 32768.
 - the maximum amount of stable memory for all regions is 256GB.


## Questions and answers

### Q: What determines the 8MB non-empty region minimum?

This size comes from wanting to grow a region by more than one page at
a time (in terms of the way that the canister interacts with the
system API, at least).  Rather than actually grow by a single page,
the implementation grows by a "page block" (8MB) at a time.

This choice means that there are 128 pages per page block, and that
the maximum number of regions and region blocks are each relatively
small (32k each).  Consequently, they can each be identified with a
2-byte identifier, and we can pre-allocate tables to store certain
relations about them, which is critical.

### Q: Are 32767 regions enough?

A: Permitting more than 32k regions may seem theoretically
interesting, but is not practical given other parameters today that
control the minimal footprint of a region (8MB) and dictate the
maximum size of stable memory for a canister today (32GB).  With 32k
regions at 8MB each, well over the maximum stable memory size is used
(256GB compared to 32GB, the limit today)

### Q: When is stable memory becoming less costly?

Spring 2023.

### Q: How does the cheaper stable memory access cost compare with ordinary heap memory access cost?

2-3x slower than ordinary heap memory.


## Design details

### API

Internal region allocator operations:

 - `initialize` -- called by the RTS, not by the Motoko developer.

User-facing region allocator operations:

 - `region_new` -- create a dynamic region.
 - `region_grow` -- grow region by a specified number of pages.
 - `region_load` -- read some data from the region.
 - `region_store` -- store some data into the region.

And a special operation, for testing our design for future GC integration (bonus):

- `region_release` -- release region and reuse region's page blocks.

The `_release` operation is *not* part of the user-facing API nor part of the MVP,
but supporting it is important because it means we can transition quickly to an integration
with the ambient Motoko GC if we can support it.

Another special operation, for disaster recovery:

 - `rebuild` -- not needed unless we need to recreate all Region objects from their stable-memory counterparts.


## Internal footprint

The state of the allocator is stored in a combination of:

 - stable memory fields and tables and
 - stable heap memory, in the form of objects of type `Region`.

The stable memory state is sufficient to reconstitute the stable heap objects
(`rebuild` operation, described in a subsection below).

That means that even if the stable parts of the heap are lost, the
stable memory state can fully describe the region objects that will be rebuilt when it succeeds.

### stable memory fields

 - total allocated blocks, `Nat16`, max value is `32768`.
 - total allocated regions, `Nat16`, max value is `32767` (one region is reserved for "no region").
 - The `block-region` table (fixed size, about 131kb).
 - The `region` table (fixed size, about 262kb).

### representation of values of type `Region`

 - A singleton, heap-allocated object with mutable fields.
 - While being heap-allocated, the object is also `stable` (can be stored in a `stable var`, etc).
 - `RegionObject { size_in_pages: Nat64; id: Nat16; vec_capacity: Nat16; vec_ptr: Nat32 }`
 - Field `size_in_pages` gives the number of pages allocated to the Region.
 - Field `id` gives the Region's numerical id as an index into the `region` table.
 - Fields `vec_capacity` and `vec_ptr` work with `size_in_pages`
   to represent a growable vector that we call the region's **"access
   vector"** (because "blocks vector" sounds a bit strange, and its
   used to support O(1) access operations):
   - the access vector's address is held in `vec_ptr` and it has `vec_capacity` slots.
   - the first `size_in_pages + 127 / 128` slots of `vec_ptr` contain a valid page block ID for the region.
   - the access vector doubles when it grows.
   - no region has more than 32k page blocks, so a `Nat16` suffices for `capacity`,
   - during an upgrade, the access vectors get serialized and deserialized as data `Blobs` (as if no pointers are inside each).


### region-blocks relation

The `region-blocks` relation is not materialized into a table in stable memory (doing so with a pre-allocated table would be prohibitively large).

Instead, this relation is represented in two ways at the same time:
 1. by the set of heap-allocated region objects, and their access vectors.  The access vectors provide O(1) store and load support.
 2. by the `block-region` table and the `region` table, which together are sufficient to recreate all of the heap-allocated region objects.

In ordinary operation, the second feature is not required.  In the event of an upgrade failure, however, it could be vital (See `rebuild`).

### block-region table

 - purpose:
   - relate a block ID ("page block ID") to its region (if any) and its position in that region (see `rebuild`).

   - NB: The organization of this table is not useful for efficient
     access calculations of load or store (they require a linear
     search that would be prohibitively slow).  OTOH, this
     table is suitable to do a "batch" rebuild of the dynamic heap-allocated vectors
     in that table, if need be (see `rebuild`).

 - 32768 entries (statically sized).
 - 4 bytes per entry.
 - entry type = `BlockRegion { region : Nat16, position : Nat16 }`
 - the location of each entry gives its corresponding block ID.

### region table

 - purpose:
   - relate a region ID to its size in pages.

   - NB: This table exists as a "stable backup" of the per-region fields
     that exist in the set of region objects, except for the access vectors' elements (see `block-region` table for their data).

 - 32768 entries (statically sized).
 - 8 bytes per entry.
 - entry type = `RegionBlocks { size_in_pages: Nat64; }`
 - the location of each entry in the table gives its corresponding region ID.
 - the `size_in_pages` field measures the size of the region in terms of _pages_, not blocks.
 - to do: track bit per region of whether in-use or reclaimed-and-available, somehow.


### Overview of `rebuild`

When upgrades work as expected, stable `Regions` are serialized and deserialized just like other stable data.

For disaster recovery, we can **also** rebuild all of the region objects from data in stable memory.

We use the `block-region` and `region` tables in stable memory to rebuild the regions' objects:

 - The `region` table gives a size for each region, saying how large each vector for each region needs to be.
 - The `block-region` table gives a relative position and region ID for each block ID.

Once each regions' vectors have been allocated, the block-region table says how to fill them, one entry at a time.
Unlike the Rust design, vector entries can be populated out-of-order.


### Special (reserved) regions

  - Region 0 -- Anonymous region, for supporting the legacy API that we have today, which lacks `Region` values.
  - Region 1 -- "Reclaimed blocks" region that consists of blocks reclaimed from GC'd regions.
  - Regions 2-15 -- Future use by Motoko RTS (TBD).

### Overview of GC support (future work)

- Regions are reprensented (see special subsection) with heap objects that are `stable`, but mutable.
- They have special GC headers to recognize their special structure.
- The Region table keeps track of which available IDs are in use as Region heap values are GC'd.

#### Region allocation, re-allocation

A region with a particular ID is first initially allocated by a global, bumped counter reaching it.

When that happens, the region table is marked (TO DO) indicating that region is in use.

The region heap object with that ID witnesses its lifetime, and is reclaimed by GC when it goes out of scope.

When the bump counter reaches its maximum value, the allocation algorithm changes to scanning the regions,
looking for the first available one (with minimum ID).

#### Region reclamation

A heap object with a certain region ID lives until GC reclaims it, where the region table is re-marked (TO DO) to indicate that the region is free again.

Before reusing that region, each of its block are added to the special "free blocks" region.


### Migration from earlier designs into region system

#### Version overview

Including this design, there are three possible verions (`0`, `1`, or `2`):

 0. Stable vars only.
 1. Stable vars *plus* direct access to IC0 API, including `grow`.
    This access is exposed via the Experimental Stable Memory API.
 2. This Region system, where direct access still works through region zero.


#### Compiler flag

Initially, all of version 2 (the region system) is guarded by a compiler flag:

```
  --stable-regions
```

A section below describes the opt in mechanism, after some more relevant background.

#### Key points

- Migration from earlier versions requires an explicit opt-in with the compiler flag.
- (Our intention is to make this flag set automatically in the future, but not initially.)

- Version 0:
  - will never be forced to "migrate" to other versions (assuming no stable memory API use).
  - will never incur the space overhead of the region system.

- Migration to version 2 from version 0 occurs when the compiler flag is set, and either:
 - An initial region is allocated via `Region.new`.
 - Experimental Stable Memory function `grow` is invoked for the first time.

- Migration to version 2 from version 1 occurs on upgrade, when the compiler flag is set.

We summarize the conditions for "opt in" again below.

#### Format version details

The first 32 bits of stable memory record a "marker," which indicates how to determine the "version number"
for Motoko stable memory.  This version number is stored either:
  - *implicitly*, when the marker is non-zero, and version is `0`.
  - *explicitly*, when the marker is zero, and version is stored elsewhere (but currently always `1`).

Including this design, there are three possible verions (`0`, `1`, or `2`).  See preceeding section.

In the first cases (`0`) and (`1`), we wish to *selectively* migrate into the region system (`2`), with its own internal versioning.

#### Trade offs

The main factor involved in making the per-canister choice of stable memory format involves the mandatory block-sized reservation for current and future meta data (8MB).

For canisters that do not store an order of magnitude more than 8MB, it may make sense to
eschew regions in favor of stable vars for stable data, thereby saving that 8MB of otherwise-wasted space.

Canisters can do this by not using any stable memory API (only stable vars), or by using the experimental API and not enabling the compiler flag for regions.

#### Opt-in mechanism

The opt-in mechanism for using version 2 consists of doing both:

 - Enabling the compiler flag.
 - Using either `Region.new` or `StableMemory.grow` on a fresh canister,
   or "upgrading" from a canister using version 1.

Critically,

1. There is no provision for "downgrading" back to earlier, pre-region systems.

2. Since they opt into the system, it does not make sense to place calls to `R.new()` in library code without adequate documentation.  It may make sense to _never_ place them there, depending on the policy for controlling and managing regions.

Until checks about unexpected placements of `R.new()` can be implemented, without enabling this flag, the region `R.new` primitive will not be compiled by the compiler.  Hence, the absence of the flag acts as a master "off switch".

##### Version 0 migration.

To migrate from version 0 to version 2, there is nothing additional to do for existing data.

The region system detects this case by measuring the zero-sized stable memory during its initialization.

##### Version 1 migration.

Migrating version 1 stable memory renames it as "region 0" in Version 2.

Critically, to migrate from version 1, we must perserve existing data, but reorganize it slightly.

In effect, all existing data will retain its logical order as (pre-allocated) region 0.

To accomodate the meta data of the region system, we move the first block of region 0, physically.

Then, we reformat the first block of stable memory as the region meta data block.

The rest of the blocks become part of region 0, with its first block stored at the end of physical memory.
