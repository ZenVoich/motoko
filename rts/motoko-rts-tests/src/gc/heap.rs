use super::utils::{make_pointer, make_scalar, write_word, ObjectIdx, GC, WORD_SIZE};

use motoko_rts::memory::Memory;
use motoko_rts::types::*;

use std::cell::{Ref, RefCell};
use std::convert::TryFrom;
use std::rc::Rc;

use fxhash::{FxHashMap, FxHashSet};
use motoko_rts_macros::*;

/// Represents Motoko heaps. Reference counted (implements `Clone`) so we can clone and move values
/// of this type to GC callbacks.
#[derive(Clone)]
pub struct MotokoHeap {
    inner: Rc<RefCell<MotokoHeapInner>>,
}

impl Memory for MotokoHeap {
    unsafe fn alloc_words(&mut self, n: Words<u32>) -> Value {
        self.inner.borrow_mut().alloc_words(n)
    }

    unsafe fn grow_memory(&mut self, ptr: u64) {
        self.inner.borrow_mut().grow_memory(ptr as usize);
    }
}

impl MotokoHeap {
    /// Create a new Motoko heap from the given object graph and roots. `GC` argument is used to
    /// allocate as little space as possible for the dynamic heap.
    ///
    /// Note that for `GC::MarkCompact` we limit the upper bound on mark stack size as
    /// `super::MAX_MARK_STACK_SIZE`. In the worst case the size would be the same as the heap
    /// size, but that's not a realistic scenario.
    pub fn new(
        map: &[(ObjectIdx, Vec<ObjectIdx>)],
        roots: &[ObjectIdx],
        continuation_table: &[ObjectIdx],
        gc: GC,
    ) -> MotokoHeap {
        MotokoHeap {
            inner: Rc::new(RefCell::new(MotokoHeapInner::new(
                map,
                roots,
                continuation_table,
                gc,
            ))),
        }
    }

    /// Get the beginning of dynamic heap, as offset in the heap array
    pub fn heap_base_offset(&self) -> usize {
        self.inner.borrow().heap_base_offset
    }

    /// Get the heap pointer, as offset in the heap array
    pub fn heap_ptr_offset(&self) -> usize {
        self.inner.borrow().heap_ptr_offset
    }

    /// Get the heap pointer, as address in the current process. The address can be used to mutate
    /// the heap.
    pub fn heap_ptr_address(&self) -> usize {
        self.inner.borrow().heap_ptr_address()
    }

    /// Update the heap pointer given as an address in the current process.
    #[non_incremental_gc]
    pub fn set_heap_ptr_address(&self, address: usize) {
        self.inner.borrow_mut().set_heap_ptr_address(address)
    }

    /// Get the last heap pointer, as address in the current process. The address can be used to mutate
    /// the heap.
    #[non_incremental_gc]
    pub fn last_ptr_address(&self) -> usize {
        self.inner.borrow().last_ptr_address()
    }

    /// Update the last heap pointer given as an address in the current process.
    #[non_incremental_gc]
    pub fn set_last_ptr_address(&self, address: usize) {
        self.inner.borrow_mut().set_last_ptr_address(address)
    }

    /// Get the beginning of dynamic heap, as an address in the current process
    pub fn heap_base_address(&self) -> usize {
        self.inner.borrow().heap_base_address()
    }

    /// Get the offset of the variable pointing to the static root array.
    pub fn static_root_array_variable_offset(&self) -> usize {
        self.inner.borrow().static_root_array_variable_offset
    }

    /// Get the address of the variable pointing to the static root array.
    pub fn static_root_array_variable_address(&self) -> usize {
        self.inner.borrow().static_root_array_variable_address()
    }

    /// Get the offset of the variable pointing to the continuation table.
    pub fn continuation_table_variable_offset(&self) -> usize {
        self.inner.borrow().continuation_table_variable_offset
    }

    /// Get the address of the variable pointing to the continuation table.
    pub fn continuation_table_variable_address(&self) -> usize {
        self.inner.borrow().continuation_table_variable_address()
    }

    /// Get the heap as an array. Use `offset` values returned by the methods above to read.
    pub fn heap(&self) -> Ref<Box<[u8]>> {
        Ref::map(self.inner.borrow(), |heap| &heap.heap)
    }

    /// Print heap contents to stdout, for debugging purposes.
    #[allow(unused)]
    pub fn dump(&self) {
        unsafe {
            motoko_rts::debug::dump_heap(
                self.heap_base_address() as u32,
                self.heap_ptr_address() as u32,
                self.static_root_array_variable_address() as *mut Value,
                self.continuation_table_variable_address() as *mut Value,
            );
        }
    }
}

struct MotokoHeapInner {
    /// The heap. This is a boxed slice instead of a vector as growing this wouldn't make sense
    /// (all pointers would have to be updated).
    heap: Box<[u8]>,

    /// Where the dynamic heap starts
    heap_base_offset: usize,

    /// Last dynamic heap end, used for generational gc testing
    _heap_ptr_last: usize,

    /// Where the dynamic heap ends, i.e. the heap pointer
    heap_ptr_offset: usize,

    /// Offset of the static root array.
    ///
    /// Reminder: This location is in static memory and points to an array in the dynamic heap.
    static_root_array_variable_offset: usize,

    /// Offset of the continuation table pointer.
    ///
    /// Reminder: This location is in static memory and points to an array in the dynamic heap.
    continuation_table_variable_offset: usize,
}

impl MotokoHeapInner {
    fn address_to_offset(&self, address: usize) -> usize {
        address - self.heap.as_ptr() as usize
    }

    fn offset_to_address(&self, offset: usize) -> usize {
        offset + self.heap.as_ptr() as usize
    }

    /// Get heap base in the process's address space
    fn heap_base_address(&self) -> usize {
        self.offset_to_address(self.heap_base_offset)
    }

    /// Get heap pointer (i.e. where the dynamic heap ends) in the process's address space
    fn heap_ptr_address(&self) -> usize {
        self.offset_to_address(self.heap_ptr_offset)
    }

    /// Set heap pointer
    fn set_heap_ptr_address(&mut self, address: usize) {
        self.heap_ptr_offset = self.address_to_offset(address);
    }

    /// Get last heap pointer (i.e. where the dynamic heap ends last GC run) in the process's address space
    #[non_incremental_gc]
    fn last_ptr_address(&self) -> usize {
        self.offset_to_address(self._heap_ptr_last)
    }

    /// Set last heap pointer
    #[non_incremental_gc]
    fn set_last_ptr_address(&mut self, address: usize) {
        self._heap_ptr_last = self.address_to_offset(address);
    }

    /// Get the address of the variable pointing to the static root array.
    fn static_root_array_variable_address(&self) -> usize {
        self.offset_to_address(self.static_root_array_variable_offset)
    }

    /// Get the address of the variable pointing to the continuation table.
    fn continuation_table_variable_address(&self) -> usize {
        self.offset_to_address(self.continuation_table_variable_offset)
    }

    fn new(
        map: &[(ObjectIdx, Vec<ObjectIdx>)],
        roots: &[ObjectIdx],
        continuation_table: &[ObjectIdx],
        gc: GC,
    ) -> MotokoHeapInner {
        // Check test correctness: an object should appear at most once in `map`
        {
            let heap_objects: FxHashSet<ObjectIdx> = map.iter().map(|(obj, _)| *obj).collect();
            assert_eq!(
                heap_objects.len(),
                map.len(),
                "Invalid test heap: some objects appear multiple times"
            );
        }

        // Two pointers, one to the static root array, and the other to the continuation table.
        let root_pointers_size_bytes = 2 * WORD_SIZE;

        // Each object will have array header plus one word for id per object + one word for each reference. 
        // The static root is an array (header + length) with one element, one MutBox for each static variable. 
        let static_root_set_size_bytes = (size_of::<Array>().as_usize() + roots.len()
            + roots.len() * size_of::<MutBox>().as_usize())
            * WORD_SIZE;

        let continuation_table_size_byes = (size_of::<Array>() + Words(continuation_table.len() as u32)).to_bytes().as_usize();

        let dynamic_objects_size_bytes = {
            let object_headers_words = map.len() * (size_of::<Array>().as_usize() + 1);
            let references_words = map.iter().map(|(_, refs)| refs.len()).sum::<usize>();
            (object_headers_words + references_words) * WORD_SIZE
        };

        let dynamic_heap_size_bytes = dynamic_objects_size_bytes + static_root_set_size_bytes + continuation_table_size_byes;

        let total_heap_size_bytes = root_pointers_size_bytes + dynamic_heap_size_bytes;

        let heap_size = heap_size_for_gc(
            gc,
            root_pointers_size_bytes,
            dynamic_heap_size_bytes,
            map.len(),
        );

        // The Worst-case unalignment w.r.t. 32-byte alignment is 28 (assuming
        // that we have general word alignment). So we over-allocate 28 bytes.
        let mut heap = vec![0u8; heap_size + 28];

        // Align the dynamic heap starts at a 32-byte multiple.
        let realign = (32 - (heap.as_ptr() as usize + root_pointers_size_bytes) % 32) % 32;
        assert_eq!(realign % 4, 0);

        // Maps `ObjectIdx`s into their offsets in the heap
        let (static_root_array_address, continuation_table_address) = create_dynamic_heap(
            map,
            roots,
            continuation_table,
            &mut heap[root_pointers_size_bytes + realign..heap_size + realign],
        );

        // Root pointers in static memory space.
        let static_root_array_variable_offset = root_pointers_size_bytes - 2 * WORD_SIZE;
        let continuation_table_variable_offset = root_pointers_size_bytes - WORD_SIZE;
        create_static_memory(
            static_root_array_variable_offset,
            continuation_table_variable_offset,
            static_root_array_address,
            continuation_table_address,
            &mut heap[realign..root_pointers_size_bytes + realign],
        );

        MotokoHeapInner {
            heap: heap.into_boxed_slice(),
            heap_base_offset: root_pointers_size_bytes + realign,
            _heap_ptr_last: root_pointers_size_bytes + realign,
            heap_ptr_offset: total_heap_size_bytes + realign,
            static_root_array_variable_offset: static_root_array_variable_offset + realign,
            continuation_table_variable_offset: continuation_table_variable_offset + realign,
        }
    }

    #[non_incremental_gc]
    unsafe fn alloc_words(&mut self, n: Words<u32>) -> Value {
        self.linear_alloc_words(n)
    }

    #[incremental_gc]
    unsafe fn alloc_words(&mut self, n: Words<u32>) -> Value {
        let mut dummy_memory = DummyMemory {};
        let result =
            motoko_rts::gc::incremental::get_partitioned_heap().allocate(&mut dummy_memory, n);
        self.set_heap_ptr_address(result.get_ptr()); // realign on partition changes

        self.linear_alloc_words(n)
    }

    unsafe fn linear_alloc_words(&mut self, n: Words<u32>) -> Value {
        // Update heap pointer
        let old_hp = self.heap_ptr_address();
        let new_hp = old_hp + n.to_bytes().as_usize();
        self.heap_ptr_offset = new_hp - self.heap.as_ptr() as usize;

        // Grow memory if needed
        self.grow_memory(new_hp as usize);
        Value::from_ptr(old_hp)
    }

    unsafe fn grow_memory(&mut self, ptr: usize) {
        let heap_end = self.heap.as_ptr() as usize + self.heap.len();
        if ptr > heap_end {
            // We don't allow growing memory in tests, allocate large enough for the test
            panic!(
                "MotokoHeap::grow_memory called: heap_end={:#x}, grow_memory argument={:#x}",
                heap_end, ptr
            );
        }
    }
}

struct DummyMemory {}

impl Memory for DummyMemory {
    unsafe fn alloc_words(&mut self, _n: Words<u32>) -> Value {
        unreachable!()
    }

    unsafe fn grow_memory(&mut self, _ptr: u64) {}
}

/// Compute the size of the heap to be allocated for the GC test.
#[non_incremental_gc]
fn heap_size_for_gc(
    gc: GC,
    static_heap_size_bytes: usize,
    dynamic_heap_size_bytes: usize,
    n_objects: usize,
) -> usize {
    let total_heap_size_bytes = static_heap_size_bytes + dynamic_heap_size_bytes;
    match gc {
        GC::Copying => {
            let to_space_bytes = dynamic_heap_size_bytes;
            total_heap_size_bytes + to_space_bytes
        }
        GC::MarkCompact => {
            let bitmap_size_bytes = {
                let dynamic_heap_bytes = Bytes(dynamic_heap_size_bytes as u32);
                // `...to_words().to_bytes()` below effectively rounds up heap size to word size
                // then gets the bytes
                let dynamic_heap_words = dynamic_heap_bytes.to_words();
                let mark_bit_bytes = dynamic_heap_words.to_bytes();

                // The bitmap implementation rounds up to 64-bits to be able to read as many
                // bits as possible in one instruction and potentially skip 64 words in the
                // heap with single 64-bit comparison
                (((mark_bit_bytes.as_u32() + 7) / 8) * 8) + size_of::<Blob>().to_bytes().as_u32()
            };
            // In the worst case the entire heap will be pushed to the mark stack, but in tests
            // we limit the size
            let mark_stack_words = n_objects.clamp(
                motoko_rts::gc::mark_compact::mark_stack::INIT_STACK_SIZE.as_usize(),
                super::utils::MAX_MARK_STACK_SIZE,
            ) + size_of::<Blob>().as_usize();

            total_heap_size_bytes + bitmap_size_bytes as usize + (mark_stack_words * WORD_SIZE)
        }
        GC::Generational => {
            const ROUNDS: usize = 3;
            const REMEMBERED_SET_MAXIMUM_SIZE: usize = 1024 * 1024 * WORD_SIZE;
            let size = heap_size_for_gc(
                GC::MarkCompact,
                static_heap_size_bytes,
                dynamic_heap_size_bytes,
                n_objects,
            );
            size + ROUNDS * REMEMBERED_SET_MAXIMUM_SIZE
        }
    }
}

#[incremental_gc]
fn heap_size_for_gc(
    gc: GC,
    _static_heap_size_bytes: usize,
    _dynamic_heap_size_bytes: usize,
    _n_objects: usize,
) -> usize {
    match gc {
        GC::Incremental => 3 * motoko_rts::gc::incremental::partitioned_heap::PARTITION_SIZE,
    }
}

/// Given a heap description (as a map from objects to objects), and the dynamic part of the heap
/// (as an array), initialize the dynamic heap with objects.
///
/// Returns a pair containing the address of the static root array and the address of the continuation table.
fn create_dynamic_heap(
    refs: &[(ObjectIdx, Vec<ObjectIdx>)],
    static_roots: &[ObjectIdx],
    continuation_table: &[ObjectIdx],
    dynamic_heap: &mut [u8],
) -> (u32, u32) {
    let incremental = cfg!(feature = "incremental_gc");
    let heap_start = dynamic_heap.as_ptr() as usize;

    // Maps objects to their addresses
    let mut object_addrs: FxHashMap<ObjectIdx, usize> = Default::default();

    // First pass allocates objects without fields
    {
        let mut heap_offset = 0;
        for (obj, refs) in refs {
            object_addrs.insert(*obj, heap_start + heap_offset);
            
            // Store object header
            let address = u32::try_from(heap_start + heap_offset).unwrap();
            write_word(dynamic_heap, heap_offset, TAG_ARRAY);
            heap_offset += WORD_SIZE;

            if incremental {
                write_word(dynamic_heap, heap_offset, make_pointer(address)); // forwarding pointer
                heap_offset += WORD_SIZE;
            }

            // Store length: idx + refs
            write_word(
                dynamic_heap,
                heap_offset,
                u32::try_from(refs.len() + 1).unwrap(),
            );
            heap_offset += WORD_SIZE;

            // Store object value (idx)
            write_word(dynamic_heap, heap_offset, make_scalar(*obj));
            heap_offset += WORD_SIZE;
            
            // Leave space for the fields
            heap_offset += refs.len() * WORD_SIZE;
        }
    }

    // println!("object addresses={:#?}", object_addrs);

    // Second pass adds fields
    for (obj, refs) in refs {
        let obj_offset = object_addrs.get(obj).unwrap() - heap_start;
        for (ref_idx, ref_) in refs.iter().enumerate() {
            let ref_addr = make_pointer(*object_addrs.get(ref_).unwrap() as u32);
            let field_offset = obj_offset
                + (size_of::<Array>() + Words(1 + ref_idx as u32))
                    .to_bytes()
                    .as_usize();
            write_word(dynamic_heap, field_offset, u32::try_from(ref_addr).unwrap());
        }
    }

    // Add the static root table
    let n_objects = refs.len();
    // fields+1 for the scalar field (idx)
    let n_fields: usize = refs.iter().map(|(_, fields)| fields.len() + 1).sum();
    let root_section_offset = (size_of::<Array>() * n_objects as u32)
        .to_bytes()
        .as_usize()
        + n_fields * WORD_SIZE;

    let mut heap_offset = root_section_offset;
    let mut root_mutboxes = vec![];
    {
        for root_id in static_roots {
            let mutbox_address = u32::try_from(heap_start + heap_offset).unwrap();
            root_mutboxes.push(mutbox_address);
            write_word(dynamic_heap, heap_offset, TAG_MUTBOX);
            heap_offset += WORD_SIZE;

            if incremental {
                write_word(
                    dynamic_heap,
                    heap_offset,
                    make_pointer(mutbox_address),
                );
                heap_offset += WORD_SIZE;
            }

            let root_ptr = *object_addrs.get(root_id).unwrap();
            write_word(dynamic_heap, heap_offset, make_pointer(root_ptr as u32));
            heap_offset += WORD_SIZE;
        }
    }
    let static_root_array_address = u32::try_from(heap_start + heap_offset).unwrap();
    {
        write_word(dynamic_heap, heap_offset, TAG_ARRAY);
        heap_offset += WORD_SIZE;

        if incremental {
            write_word(
                dynamic_heap,
                heap_offset,
                make_pointer(static_root_array_address),
            );
            heap_offset += WORD_SIZE;
        }

        assert_eq!(static_roots.len(), root_mutboxes.len());
        write_word(dynamic_heap, heap_offset, root_mutboxes.len() as u32);
        heap_offset += WORD_SIZE;
        
        for mutbox_address in root_mutboxes {
            write_word(dynamic_heap, heap_offset, make_pointer(mutbox_address));
            heap_offset += WORD_SIZE;
        }
    }
    
    let continuation_table_address = u32::try_from(heap_start + heap_offset).unwrap();
    {
        write_word(dynamic_heap, heap_offset, TAG_ARRAY);
        heap_offset += WORD_SIZE;

        if incremental {
            write_word(
                dynamic_heap,
                heap_offset,
                make_pointer(continuation_table_address),
            );
            heap_offset += WORD_SIZE;
        }

        write_word(dynamic_heap, heap_offset, continuation_table.len() as u32);
        heap_offset += WORD_SIZE;

        for idx in continuation_table {
            let idx_ptr = *object_addrs.get(idx).unwrap();
            write_word(dynamic_heap, heap_offset, make_pointer(idx_ptr as u32));
            heap_offset += WORD_SIZE;
        }
    }

    (static_root_array_address, continuation_table_address)
}

/// Static memory part containing the root pointers.
fn create_static_memory(
    static_root_array_variable_offset: usize,
    continuation_table_variable_offset: usize,
    static_root_array_address: u32,
    continuation_table_address: u32,
    heap: &mut [u8],
) {
    // Write static array pointer as the second last word in static memory
    write_word(
        heap,
        static_root_array_variable_offset,
        make_pointer(static_root_array_address),
    );

    // Write continuation table pointer as the last word in static memory
    write_word(
        heap,
        continuation_table_variable_offset,
        make_pointer(continuation_table_address),
    );
}
