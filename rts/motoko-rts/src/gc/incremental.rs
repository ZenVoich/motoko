use motoko_rts_macros::ic_mem_fn;

use crate::{memory::Memory, types::*, visitor::visit_pointer_fields};

use self::{
    mark_stack::MarkStack,
    partitioned_heap::PartitionedHeap,
    phases::{
        evacuation_increment::EvacuationIncrement, mark_increment::MarkIncrement,
        update_increment::UpdateIncrement,
    },
    roots::Roots,
    time::BoundedTime,
};

pub mod array_slicing;
pub mod barriers;
pub mod mark_stack;
pub mod partitioned_heap;
mod phases;
pub mod roots;
#[cfg(feature = "memory_check")]
pub mod sanity_checks;
pub mod time;

#[ic_mem_fn(ic_only)]
unsafe fn initialize_incremental_gc<M: Memory>(mem: &mut M) {
    use crate::memory::ic;
    ic::initialize_memory(true);
    assert_eq!(ic::HP, ic::get_aligned_heap_base()); // No dynamic heap allocations so far.
    IncrementalGC::<M>::initialize(mem, ic::get_aligned_heap_base() as usize);
}

#[ic_mem_fn(ic_only)]
unsafe fn schedule_incremental_gc<M: Memory>(mem: &mut M) {
    let running = PHASE != Phase::Pause && PHASE != Phase::Stop;
    if running || should_start() {
        incremental_gc(mem);
    }
}

/// Limits on the number of steps performed in a GC increment.
/// Distinguishing between two types of GC increments:
/// * Scheduled increments: Compiler-instrumented GC calls.
/// * Allocation increments: GC increment at periodic allocations.
pub const SCHEDULED_INCREMENT_LIMIT: usize = 2_500_000;
const ALLOCATION_INCREMENT_LIMIT: usize = 500_000;

#[ic_mem_fn(ic_only)]
unsafe fn incremental_gc<M: Memory>(mem: &mut M) {
    use self::roots::root_set;
    if PHASE == Phase::Pause {
        record_gc_start::<M>();
    }
    let time = BoundedTime::new(SCHEDULED_INCREMENT_LIMIT);
    IncrementalGC::instance(mem, time).empty_call_stack_increment(root_set());
    if PHASE == Phase::Pause {
        record_gc_stop::<M>();
    }
}

#[cfg(feature = "ic")]
static mut LAST_ALLOCATIONS: Bytes<u64> = Bytes(0u64);

#[cfg(feature = "ic")]
unsafe fn should_start() -> bool {
    use self::partitioned_heap::PARTITION_SIZE;
    use crate::memory::ic;

    const CRITICAL_HEAP_LIMIT: Bytes<u32> = Bytes(u32::MAX - 1024 * 1024 * 1024);
    const CRITICAL_GROWTH_THRESHOLD: f64 = 0.15;
    const NORMAL_GROWTH_THRESHOLD: f64 = 0.65;

    let heap_size = ic::get_heap_size();
    let growth_threshold = if heap_size > CRITICAL_HEAP_LIMIT {
        CRITICAL_GROWTH_THRESHOLD
    } else {
        NORMAL_GROWTH_THRESHOLD
    };

    let current_allocations = ic::get_total_allocations();
    debug_assert!(current_allocations >= LAST_ALLOCATIONS);
    let absolute_growth = current_allocations - LAST_ALLOCATIONS;
    let relative_growth = absolute_growth.0 as f64 / heap_size.as_usize() as f64;
    relative_growth > growth_threshold && heap_size.as_usize() >= PARTITION_SIZE
}

#[cfg(feature = "ic")]
unsafe fn record_gc_start<M: Memory>() {
    use crate::memory::ic;
    LAST_ALLOCATIONS = ic::get_total_allocations();
}

#[cfg(feature = "ic")]
unsafe fn record_gc_stop<M: Memory>() {
    use crate::memory::ic;

    let current_allocations = ic::get_total_allocations();
    debug_assert!(current_allocations >= LAST_ALLOCATIONS);
    let growth_during_gc = current_allocations - LAST_ALLOCATIONS;
    let heap_size = ic::get_heap_size();
    debug_assert!(growth_during_gc.0 <= heap_size.as_usize() as u64);
    let live_set = heap_size - Bytes(growth_during_gc.0 as u32);
    ic::MAX_LIVE = ::core::cmp::max(ic::MAX_LIVE, live_set);
}

/// GC phases per run. Each of the following phases is performed in potentially multiple increments.
/// 1. Marking: Incremental full-heap snapshot-at-the-beginning marking.
///    Must start on empty call stack.
///     * Concurrent allocations are conservatively marked.
///     * Concurrent pointer writes are handled by the write barrier.
/// 2. Evacuation: Incremental compacting evacuation of high-garbage partitions.
///     * Copying live objects out of the selected partitions to new partitions.
///     * Concurrent accesses to old object locations are handled by pointer forwarding.
/// 3. Updating: Incremental updates of all old pointers to their new forwarded addresses.
///    Must complete on empty call stack.
///     * Also clearing mark bit of all alive objects.
///     * Concurrent copying of old pointer values is intercepted to resolve forwarding.
/// Finally, all the evacuated partitions are freed.

// Performance note: Storing the phase-specific state in the enum would be nicer but it is much slower.
#[derive(PartialEq)]
enum Phase {
    Pause,    // Inactive, waiting for the next GC run.
    Mark,     // Incremental marking.
    Evacuate, // Incremental evacuation compact.
    Update,   // Incremental pointer updates.
    Stop,     // GC stopped on canister upgrade.
}

pub struct MarkState {
    mark_stack: MarkStack,
    complete: bool,
}

/// GC state retained over multiple GC increments.
static mut PHASE: Phase = Phase::Pause;
pub static mut PARTITIONED_HEAP: Option<PartitionedHeap> = None;

/// Incremental GC.
/// Each GC call has its new GC instance that shares the common GC states `PHASE` and `PARTITIONED_HEAP`.
pub struct IncrementalGC<'a, M: Memory> {
    mem: &'a mut M,
    time: BoundedTime,
}

impl<'a, M: Memory + 'a> IncrementalGC<'a, M> {
    /// (Re-)Initialize the entire incremental garbage collector.
    /// Called on a runtime system start with incremental GC and also during RTS testing.
    pub unsafe fn initialize(mem: &'a mut M, heap_base: usize) {
        PHASE = Phase::Pause;
        PARTITIONED_HEAP = Some(PartitionedHeap::new(mem, heap_base));
    }

    /// Each GC schedule point can get a new GC instance that shares the common GC state.
    /// This is because the memory implementation is not stored as global variable.
    pub unsafe fn instance(mem: &'a mut M, time: BoundedTime) -> IncrementalGC<'a, M> {
        IncrementalGC { mem, time }
    }

    /// Special GC increment invoked when the call stack is guaranteed to be empty.
    /// As the GC cannot scan or use write barriers on the call stack, we need to ensure:
    /// * The mark phase is only be started on an empty call stack.
    /// * The update phase can only be completed on an empty call stack.
    pub unsafe fn empty_call_stack_increment(&mut self, roots: Roots) {
        if self.pausing() {
            self.start_marking(roots);
        }
        self.increment();
        if self.mark_completed() {
            self.start_evacuating(roots);
            self.increment();
        }
        if self.evacuation_completed() {
            self.start_updating(roots);
            self.increment();
        }
        if self.updating_completed() {
            self.complete_run(roots);
        }
    }

    unsafe fn pausing(&mut self) -> bool {
        PHASE == Phase::Pause
    }

    unsafe fn increment(&mut self) {
        match PHASE {
            Phase::Pause | Phase::Stop => {}
            Phase::Mark => MarkIncrement::instance(self.mem, &mut self.time).run(),
            Phase::Evacuate => EvacuationIncrement::instance(self.mem, &mut self.time).run(),
            Phase::Update => UpdateIncrement::instance(&mut self.time).run(),
        }
    }

    /// Only to be called when the call stack is empty as pointers on stack are not collected as roots.
    unsafe fn start_marking(&mut self, roots: Roots) {
        debug_assert!(self.pausing());

        PHASE = Phase::Mark;
        MarkIncrement::start_phase(self.mem);
        let mut increment = MarkIncrement::instance(self.mem, &mut self.time);
        increment.mark_roots(roots);
    }

    unsafe fn mark_completed(&self) -> bool {
        PHASE == Phase::Mark && MarkIncrement::<M>::mark_completed()
    }

    unsafe fn check_mark_completion(&mut self, _roots: Roots) {
        #[cfg(feature = "memory_check")]
        {
            sanity_checks::check_memory(
                self.mem,
                _roots,
                sanity_checks::CheckerMode::MarkCompletion,
            );
        }
    }

    unsafe fn start_evacuating(&mut self, roots: Roots) {
        self.check_mark_completion(roots);
        debug_assert!(self.mark_completed());
        MarkIncrement::<M>::complete_phase();
        PHASE = Phase::Evacuate;
        EvacuationIncrement::<M>::start_phase();
    }

    unsafe fn evacuation_completed(&self) -> bool {
        PHASE == Phase::Evacuate && EvacuationIncrement::<M>::evacuation_completed()
    }

    unsafe fn start_updating(&mut self, roots: Roots) {
        debug_assert!(self.evacuation_completed());
        EvacuationIncrement::<M>::complete_phase();
        PHASE = Phase::Update;
        UpdateIncrement::start_phase();
        let mut increment = UpdateIncrement::instance(&mut self.time);
        increment.update_roots(roots);
    }

    unsafe fn updating_completed(&self) -> bool {
        PHASE == Phase::Update && UpdateIncrement::update_completed()
    }

    /// Only to be called when the call stack is empty as pointers on stack are not updated.
    unsafe fn complete_run(&mut self, roots: Roots) {
        debug_assert!(self.updating_completed());
        UpdateIncrement::complete_phase();
        PHASE = Phase::Pause;
        ALLOCATION_COUNT = 0;
        self.check_update_completion(roots);
    }

    unsafe fn check_update_completion(&mut self, _roots: Roots) {
        #[cfg(feature = "memory_check")]
        {
            sanity_checks::check_memory(
                self.mem,
                _roots,
                sanity_checks::CheckerMode::UpdateCompletion,
            );
        }
    }
}

/// Write barrier to be called BEFORE a potential overwrite of a pointer value.
/// `overwritten_value` (skewed if a pointer) denotes the value that will be overwritten.
/// The barrier can be conservatively called even if the overwritten value is not a pointer.
/// The barrier is only effective while the GC is in the mark phase.
#[inline(never)]
pub(crate) unsafe fn pre_write_barrier<M: Memory>(mem: &mut M, overwritten_value: Value) {
    if PHASE == Phase::Mark {
        let heap = PARTITIONED_HEAP.as_ref().unwrap();
        if overwritten_value.points_to_or_beyond(heap.base_address()) {
            if !MarkIncrement::<M>::mark_completed() {
                let mut time = BoundedTime::new(0);
                let mut increment = MarkIncrement::instance(mem, &mut time);
                increment.mark_object(overwritten_value);
            } else {
                debug_assert!(overwritten_value.as_obj().is_marked());
            }
        }
    }
}

/// Allocation barrier to be called AFTER a new object allocation.
/// `new_object` is the skewed pointer of the newly allocated and initialized object.
/// The new object needs to be fully initialized, except fot the payload of a blob.
/// The barrier is only effective during a running GC.
pub(crate) unsafe fn post_allocation_barrier(new_object: Value) {
    if PHASE == Phase::Mark || PHASE == Phase::Evacuate {
        mark_new_allocation(new_object);
    } else if PHASE == Phase::Update {
        update_new_allocation(new_object);
    }
}

/// Mark a new object during the mark phase and evacuation phase.
/// `new_object` is the skewed pointer of a newly allocated object.
///
/// Incremental GC allocation scheme:
/// * During the pause:
///   - No marking. New objects can be reclaimed in the next GC round if they become garbage by then.
/// * During the mark phase:
///   - New allocated objects are conservatively marked and cannot be reclaimed in the
///     current GC run. This is necessary because the incremental GC does neither scan
///     nor use write barriers on the call stack. The fields in the new allocated array
///     do not need to be visited during the mark phase due to the snapshot-at-the-beginning
///     consistency.
/// * During the evacuation phase:
///   - Mark new objects such that their fields are updated in the subsequent
///     update phase. The fields may still point to old object locations that are forwarded.
/// * During the update phase
///   - New objects must not be marked in this phase as the mark bits are reset.
/// * When GC is stopped on canister upgrade:
///   - The GC will not resume and thus marking is irrelevant.
unsafe fn mark_new_allocation(new_object: Value) {
    debug_assert!(PHASE == Phase::Mark || PHASE == Phase::Evacuate);
    let object = new_object.get_ptr() as *mut Obj;
    debug_assert!(!object.is_marked());
    object.mark();
    PARTITIONED_HEAP
        .as_mut()
        .unwrap()
        .record_marked_space(object);
}

/// Update the pointer fields during the update phase.
/// This is to ensure that new allocation do not contain any old pointers referring to
/// forwarded objects.
/// The object must be fully initialized, except for the payload of a blob.
/// `new_object` is the skewed pointer of a newly allocated and initialized object.
///
/// Incremental GC update scheme:
/// * During the mark phase and a pause:
///   - No pointers to forwarded pointers exist in alive objects.
/// * During the evacuation phase:
///   - The fields may point to old locations that are forwarded.
/// * During the update phase:
///   - All old pointers to forwarded objects must be updated to refer to the corresponding
///     new object locations. Since the mutator may copy old pointers around, all allocations
///     and pointer writes must be handled by barriers.
///   - Allocation barrier: Resolve the forwarding for all pointers in the new allocation.
///   - Write barrier: Resolve forwarding for the written pointer value.
/// * When the GC is stopped on canister upgrade:
///   - The GC will not resume and thus pointer updates are irrelevant. The runtime system
///     continues to resolve the forwarding for all remaining old pointers.
unsafe fn update_new_allocation(new_object: Value) {
    debug_assert!(PHASE == Phase::Update);
    let heap = PARTITIONED_HEAP.as_ref().unwrap();
    if heap.updates_needed() {
        let object = new_object.get_ptr() as *mut Obj;
        visit_pointer_fields(
            &mut (),
            object,
            object.tag(),
            heap.base_address(),
            |_, field| {
                *field = (*field).forward_if_possible();
            },
            |_, _, array| array.len(),
        );
    }
}

/// Number of allocations during a GC run.
static mut ALLOCATION_COUNT: usize = 0;

/// Number of allocations that triggers an additional GC allocation increment.
const ALLOCATION_INCREMENT_INTERVAL: usize = 5_000;

/// Additional increment, performed at certain allocation intervals to keep up with a high allocation rate.
unsafe fn allocation_increment<M: Memory>(_mem: &mut M) {
    ALLOCATION_COUNT += 1;
    if ALLOCATION_COUNT == ALLOCATION_INCREMENT_INTERVAL {
        ALLOCATION_COUNT = 0;
        let time = BoundedTime::new(ALLOCATION_INCREMENT_LIMIT);
        IncrementalGC::instance(_mem, time).increment();
    }
}

/// Stop the GC before performing upgrade. Otherwise, GC increments
/// on allocation and writes may interfere with the upgrade mechanism
/// that invalidates object tags during stream serialization.
#[no_mangle]
pub unsafe extern "C" fn stop_gc_on_upgrade() {
    PHASE = Phase::Stop;
}