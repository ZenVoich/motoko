#[non_incremental_gc]
pub mod copying;
#[non_incremental_gc]
pub mod generational;
#[incremental_gc]
pub mod incremental;
#[non_incremental_gc]
pub mod mark_compact;

use motoko_rts_macros::*;

#[cfg(feature = "ic")]
#[non_incremental_gc]
unsafe fn should_do_gc(max_live: crate::types::Bytes<u64>) -> bool {
    use crate::memory::ic::linear_memory::{getHP, LAST_HP};

    // A factor of last heap size. We allow at most this much allocation before doing GC.
    const HEAP_GROWTH_FACTOR: f64 = 1.5;

    let heap_limit = core::cmp::min(
        (f64::from(LAST_HP) * HEAP_GROWTH_FACTOR) as u64,
        (u64::from(LAST_HP) + max_live.0) / 2,
    );

    u64::from(getHP()) >= heap_limit
}
