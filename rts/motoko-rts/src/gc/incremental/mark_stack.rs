//! In-heap extendable mark stack for the incremental GC.
//!
//! The mark stack cannot grow contiguously as new objects can be allocated
//! during the GC run and thus during the mark phase. This is why the stack
//! is represented as multiple tables.
//!
//! Doubly linked list of stack tables, each containing a series of entries.
//! A table is represented as a blob with the following internal layout:
//!
//! ┌──────────┬─────────┬──────────┬─────────┬──────────────┬────────┐
//! │ previous │   next  | entry[0] |  ...    | entry[top-1] | (free) |
//! └──────────┴─────────┴──────────┴─────────┴──────────────┴────────┘
//!
//! The list is doubly linked for the following purpose (wihout indirection over
//! the object table):
//! * `previous` to return to the previous table with preceding entries.
//! * `next` avoid repeated allocations when the stack shrinks and regrows.
//!
//! Whenever a table is full and an entry needs to be pushed on the stack,
//! a new stack table is allocated and linked, unless there already exists
//! a next table. Only the last table can have free entry space.
//!
//! The entries represent ids (references) to objects to be visited by the GC.
//! (being conceptually gray in the incremental tri-color mark scheme).
//!
//! NOTES:
//! * The tables are blobs, as their entries must not be visited by the GC.
//! * The incremental mark write barrier may require new mark stack tables for
//!   recording old objects in the young generation. If this is the case, the
//!   corresponding mark stack tables are recorded in the young remembered set
//!   such that they are promoted to the old generation such that it remains
//!   available for the mark increment of the old generation.
//! * The mark stack must use object ids for referencing the previous/next
//!   tables, because the tables could be moved from the young generation to
//!   the old generation (as explained before).
//! * The stack tables become garbage and can be reclaimed. If they have been
//!   promoted back to the old generation (if the write barrier had to extend
//!   the old mark stack inside the young generation), they will be reclaimed
//!   in the subsequent GC run, and otherwise, in the same GC run.

use crate::gc::incremental::write_barrier::remember_old_object;
use crate::memory::{alloc_blob, Memory};
use crate::types::{size_of, Blob, Obj, Value, NULL_OBJECT_ID};

pub struct MarkStack {
    last: Value,
    top: usize, // Index of next free entry in the last stack table.
}

pub const STACK_TABLE_CAPACITY: usize = 256 * 1024;

#[repr(C)]
struct StackTable {
    header: Blob,
    previous: Value,
    next: Value,
    entries: [Value; STACK_TABLE_CAPACITY],
}

impl MarkStack {
    /// Create an empty mark stack that still needs to be allocated before use.
    /// To avoid slow `Option<MarkStack>`, the stack is allocated and freed by
    /// separate functions.
    pub const fn new() -> MarkStack {
        MarkStack {
            last: NULL_OBJECT_ID,
            top: 0,
        }
    }

    /// Allocate the mark stack before use.
    pub unsafe fn allocate<M: Memory>(&mut self, mem: &mut M, remember_table: bool) {
        debug_assert!(!self.is_allocated());
        self.last = Self::new_table(mem, NULL_OBJECT_ID, remember_table);
        debug_assert_eq!(self.top, 0);
    }

    /// Release the mark stack after use.
    pub unsafe fn free(&mut self) {
        #[cfg(debug_assertions)]
        self.assert_is_garbage();

        debug_assert!(self.is_allocated());
        debug_assert!(self.is_empty());
        debug_assert_eq!(self.top, 0);
        self.last = NULL_OBJECT_ID
        // Stack and their object ids are freed by the GC.
    }

    pub fn is_allocated(&self) -> bool {
        self.last != NULL_OBJECT_ID
    }

    /// Push an object address on the stack.
    /// Denote whether a created stack table should be also recorded in the young generation's
    /// remembered set. This is the case when the mark stack is extended for the old generation,
    /// while the mutator is running and the young generation exists.
    pub unsafe fn push<M: Memory>(&mut self, mem: &mut M, object: Value, remember_table: bool) {
        debug_assert!(object != NULL_OBJECT_ID);
        debug_assert!(self.is_allocated());
        let mut table = self.last.as_blob_mut() as *mut StackTable;
        if self.top == STACK_TABLE_CAPACITY {
            if (*table).next == NULL_OBJECT_ID {
                self.last = Self::new_table(mem, self.last, remember_table);
            } else {
                self.last = (*table).next;
            }
            table = self.last.as_blob_mut() as *mut StackTable;
            self.top = 0;
        }
        debug_assert!(self.top < STACK_TABLE_CAPACITY);
        (*table).entries[self.top] = object;
        self.top += 1;
    }

    /// Pop an object address off the stack, if it is not empty.
    /// Otherwise, if empty, returns `NULL_OBJECT_ID`.
    pub unsafe fn pop(&mut self) -> Value {
        debug_assert!(self.is_allocated());
        let mut table = self.last.as_blob_mut() as *mut StackTable;
        if self.top == 0 {
            if (*table).previous == NULL_OBJECT_ID {
                return NULL_OBJECT_ID;
            }
            self.last = (*table).previous;
            table = self.last.as_blob_mut() as *mut StackTable;
            self.top = STACK_TABLE_CAPACITY;
        }
        debug_assert!(self.top > 0);
        self.top -= 1;
        debug_assert!(self.top < STACK_TABLE_CAPACITY);
        let object = (*table).entries[self.top];
        debug_assert!(object != NULL_OBJECT_ID);
        object
    }

    /// Determine whether the stack is empty.
    pub unsafe fn is_empty(&self) -> bool {
        debug_assert!(self.is_allocated());
        let table = self.last.as_blob_mut() as *mut StackTable;
        self.top == 0 && (*table).previous == NULL_OBJECT_ID
    }

    /// `remember` denotes whether the created table should be also registered in
    /// young generation's remembered set.
    unsafe fn new_table<M: Memory>(mem: &mut M, previous: Value, remember: bool) -> Value {
        let table_id = alloc_blob(mem, size_of::<StackTable>().to_bytes());
        let table = table_id.as_blob_mut() as *mut StackTable;
        // No mark bit is set as the blob is to be reclaimeed by the current GC run.
        debug_assert!(!(table as *mut Obj).is_marked());
        (*table).previous = previous;
        (*table).next = NULL_OBJECT_ID;
        if previous != NULL_OBJECT_ID {
            let previous_table = previous.as_blob_mut() as *mut StackTable;
            (*previous_table).next = table_id;
        }
        if remember {
            // Retain an old generation stack table that is allocated in the young generation
            // by the mutator during the incremental mark phase
            remember_old_object(mem, table_id);
        }
        table_id
    }

    #[cfg(debug_assertions)]
    unsafe fn assert_is_garbage(&self) {
        assert!(self.is_allocated());
        let mut current = self.last;
        let mut table = current.as_blob_mut() as *mut StackTable;
        while (*table).previous != NULL_OBJECT_ID {
            current = (*table).previous;
            table = current.as_blob_mut() as *mut StackTable;
        }
        while current != NULL_OBJECT_ID {
            table = current.as_blob_mut() as *mut StackTable;
            assert!(!(table as *mut Obj).is_marked());
            current = (*table).next;
        }
    }
}