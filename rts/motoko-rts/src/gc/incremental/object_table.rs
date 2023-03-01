//! Central object table used by the incremental GC.
//! Maps object ids to the correponding object addresses in the dynamic heap.
//! All references to objects in the dynamic heap are routed through this table.
//! This enables fast moving of objects in the incremental GC by only updating the
//! address of the corresponding object in the table.
//!
//! The table is allocated at the end of the static heap and before the dynamic heap.
//!
//! ┌────────────┬─────────────┬──────────────┬──────────────┬────────────┐
//! │ Rust stack │ Static heap │ Object table | Dynamic heap │ Free space │
//! └────────────┴─────────────┴──────────────┴──────────────┴────────────┘
//!                            ^              ^              ^
//!                            |              |              |
//!                        Table base     Heap base     Heap pointer
//!
//! Heap base is shifted on allocation and growth (shrinking) of the object table.
//! The base of the object table never moves to guarantee immutability of the object
//! ids that are encoded as pointers into the table. The GC does not reclaim the
//! object table as it is outside the dynamic heap space.
//!
//! The dynamic heap can be organized into generations, e.g. old and young generation
//! with `LAST_HP` splitting both generations. On each GC run, the young generation could
//! be first collected (classically), before the incremental collection of the (extended)
//! old generation continues. A mark bitmap and mark stack frames for incremental old
//! generation collection could be allocated at the end of the old generation (reclaimable
//! by GC), when the incremental GC run of the old generation starts. Young objects promoted
//! to the old generation behind the mark bitmap are implicitly considered marked (fitting to
//! snapshot-at-the-beginning marking strategy).
//!
//! The object table stores an id-to-address translation as an array. Each array element
//! can be used to represent object id with the address of an allocated object stored in
//! the element. Object ids are represented as skewed pointers to the corresponding array
//! element in central object table. Table elements are word-sized.
//!
//!                       Object table
//! Value (skewed)       ┌──────────────┐   
//!    |                 |     ...      |
//!    |   object id     |──────────────|                     Object
//!    └────────────────>|   address    |────────────────>┌─────────────┐
//!                      |──────────────|                 |     ...     |
//!                      |     ...      |                 └─────────────┘
//!                      └──────────────┘
//!
//! Free object ids are stored in a simple stack that is inlined in the array. The top
//! free pointer denotes a free object id, where the element of that id stores the next
//! free object id, and so on. The bottom of the free stack is represented by the sentinel
//! value `FREE_STACK_END`. Insertion and removal from the free id stack is `O(1)` at the
//! top of the stack.
//!
//!                       Object table
//! Top free             ┌──────────────┐   
//!    |                 |     ...      |
//!    |   object id     |──────────────|
//!    └────────────────>| next free id |─────┐
//!                      |─────────────-|     |
//!                      |     ...      |     |
//!                      |─────────────-|     |
//!                ┌─────| next free id |<────┘
//!                |     |─────────────-|
//!                └────>|   free end   |
//!                      └──────────────┘
//!
//! On dynamic allocation of a new object, a free object id has to be popped off the free
//! stack and the address to be recorded in the element. If the free stack is empty and the
//! object table is full, the table is extended (see below).
//!
//! When an object is freed by the GC, the corresponding object id is again pushed back on
//! the free stack.
//!
//! When the garbage collector moves an object, it determines the object id in the header of
//! the object and can then update the address for the corresponding object id in the table.
//! This allows atomic `O(1)` updating of incoming references and thus incremental heap
//! compaction, by moving alive objects, one after the other.
//!
//! Table growth:
//! When the table is full, i.e. the allocator encounters an empty free stack, the table is
//! extended at its end, which also shifts the beginning of the dynamic heap space.
//! Objects blocking the extension of the table can be easily moved to another place, because
//! of the `O(1)` object movement costs by changing their addresses in the table.
//! Note: If objects are moved to the young generation due to table extension, their object id
//! must be added to the remembered set of the young generation in order to retain the moved object.
//!
//! Table shrinking is generally not supported due to the fragmentation of the free slots in table,
//! i.e. free object ids can be spread across the entire table and do not necessarily manifest
//! at table end. If the table end contains a contiguous section with only free ids, it could be
//! shrunk by that size (currently not yet implemented). Otherwise, reassignment of ids would be
//! needed which is not supported as it would require updating fields/array elements storing that id,
//! with entails a full heap/memory scan.
//!
//! Exceptions:
//! * Static objects are not indirected via this table, but their object id directly
//!   store the skewed address in the static heap. This is done because static objects
//!   do not move and the compiler already generates the object ids in the static object
//!   header before this table has been allocated.
//! * Non-incremental GCs. The table is not used and all object ids are represented as
//!   skewed addresses of the corresponding objects.

use core::ops::Range;

use crate::{
    constants::WORD_SIZE,
    memory::Memory,
    rts_trap_with,
    types::{skew, unskew, Value, Words},
};

/// Central object table.
pub struct ObjectTable {
    /// Bottom of the table array.
    base: *mut usize,
    /// Number of table entries (words).
    length: usize,
    /// Top of stack for free object ids.
    free: Value,
}

const FREE_STACK_END: Value = Value::from_raw(skew(0) as u32);

impl ObjectTable {
    pub unsafe fn new<M: Memory>(mem: &mut M, length: usize) -> ObjectTable {
        assert!(length > 0);
        let size = Words(length as u32);
        let base = mem.alloc_words(size) as *mut usize;
        let mut table = ObjectTable {
            base,
            length,
            free: FREE_STACK_END,
        };
        table.add_free_range(0..length);
        table
    }

    fn add_free_range(&mut self, range: Range<usize>) {
        for index in range.rev() {
            let object_id = self.index_to_object_id(index);
            self.push_free_id(object_id);
        }
    }

    pub fn new_object_id(&mut self, address: usize) -> Value {
        let object_id = self.pop_free_id();
        self.write_element(object_id, address);
        object_id
    }

    pub fn get_object_address(&self, object_id: Value) -> usize {
        self.read_element(object_id)
    }

    fn index_to_object_id(&self, index: usize) -> Value {
        unsafe { Value::from_raw(skew(self.base.add(index) as usize) as u32) }
    }

    fn push_free_id(&mut self, object_id: Value) {
        assert!(object_id != FREE_STACK_END);
        self.write_element(object_id, self.free.get_raw() as usize);
        self.free = object_id;
    }

    fn pop_free_id(&mut self) -> Value {
        if self.free == FREE_STACK_END {
            unsafe { rts_trap_with("Full object table") }
        }
        let object_id = self.free;
        self.free = Value::from_raw(self.read_element(object_id) as u32);
        object_id
    }

    fn write_element(&self, object_id: Value, value: usize) {
        unsafe {
            let element = self.get_element(object_id);
            *element = value;
        }
    }

    fn read_element(&self, object_id: Value) -> usize {
        unsafe {
            let entry = self.get_element(object_id);
            *entry
        }
    }

    unsafe fn get_element(&self, object_id: Value) -> *mut usize {
        assert!(object_id.is_object_id());
        let element_address = unskew(object_id.get_raw() as usize);
        assert_eq!(element_address % WORD_SIZE as usize, 0);
        assert!(element_address >= self.base as usize);
        assert!(element_address < self.base.add(self.length) as usize);
        element_address as *mut usize
    }
}
