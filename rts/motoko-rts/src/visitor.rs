#[enhanced_orthogonal_persistence]
pub mod enhanced;

#[classical_persistence]
pub mod classical;

use motoko_rts_macros::{classical_persistence, enhanced_orthogonal_persistence};

use crate::types::*;

/// A visitor that passes field addresses of fields with pointers to dynamic heap to the given
/// callback
///
/// Arguments:
///
/// * `ctx`: any context passed to the `visit_*` callbacks
/// * `obj`: the heap object to be visited (note: its heap tag may be invalid)
/// * `tag`: the heap object's logical tag (or start of array object's suffix slice)
/// * `_heap_base`: start address of the dynamic heap, only used with classical persistence.
/// * `visit_ptr_field`: callback for individual fields
/// * `visit_field_range`: callback for determining the suffix slice
///   Arguments:
///   * `&mut C`: passed context
///   * `usize`: start index of array suffix slice being visited
///   * `*mut Array`: home object of the slice (its heap tag may be invalid)
///   Returns:
///   * `usize`: start of the suffix slice of fields not to be passed to `visit_ptr_field`;
///            it is the callback's responsibility to deal with the spanned slice
#[enhanced_orthogonal_persistence]
pub unsafe fn visit_pointer_fields<C, F, G>(
    ctx: &mut C,
    obj: *mut Obj,
    tag: Tag,
    _heap_base: usize,
    visit_ptr_field: F,
    visit_field_range: G,
) where
    F: Fn(&mut C, *mut Value),
    G: Fn(&mut C, usize, *mut Array) -> usize,
{
    self::enhanced::visit_pointer_fields(ctx, obj, tag, visit_ptr_field, visit_field_range);
}

#[classical_persistence]
pub unsafe fn visit_pointer_fields<C, F, G>(
    ctx: &mut C,
    obj: *mut Obj,
    tag: Tag,
    heap_base: usize,
    visit_ptr_field: F,
    visit_field_range: G,
) where
    F: Fn(&mut C, *mut Value),
    G: Fn(&mut C, usize, *mut Array) -> usize,
{
    self::classical::visit_pointer_fields(
        ctx,
        obj,
        tag,
        heap_base,
        visit_ptr_field,
        visit_field_range,
    );
}
