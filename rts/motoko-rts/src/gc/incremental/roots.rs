use motoko_rts_macros::ic_mem_fn;

use crate::{types::Value, visitor::is_non_null_pointer_field};

/// An array referring to the static program variables, being
/// - All canister variables.
/// - Pooled shared objects.
/// The array constitutes a GC root that is reinitialized on each canister upgrade.
/// The scalar sentinel denotes an uninitialized root.
#[cfg(feature = "ic")]
static mut STATIC_VARIABLES: Value = Value::from_scalar(0);

/// GC root set.
pub type Roots = [*mut Value; 6];

#[cfg(feature = "ic")]
pub unsafe fn root_set() -> Roots {
    use crate::{
        continuation_table::continuation_table_loc,
        persistence::{stable_actor_location, stable_type_descriptor},
        region::region0_get_ptr_loc,
    };
    [
        static_variables_location(),
        continuation_table_loc(),
        stable_actor_location(),
        stable_type_descriptor().candid_data_location(),
        stable_type_descriptor().type_offsets_location(),
        region0_get_ptr_loc(),
    ]
}

pub unsafe fn visit_roots<C, V: Fn(&mut C, *mut Value)>(
    roots: Roots,
    context: &mut C,
    visit_field: V,
) {
    for location in roots {
        if is_non_null_pointer_field(location) {
            visit_field(context, location);
        }
    }
}

#[cfg(feature = "ic")]
unsafe fn static_variables_location() -> *mut Value {
    &mut STATIC_VARIABLES as *mut Value
}

#[ic_mem_fn(ic_only)]
pub unsafe fn set_static_variables<M: crate::memory::Memory>(mem: &mut M, array: Value) {
    use super::barriers::write_with_barrier;
    use crate::types::TAG_ARRAY;

    assert_eq!(array.tag(), TAG_ARRAY);
    let location = &mut STATIC_VARIABLES as *mut Value;
    write_with_barrier(mem, location, array);
}

#[no_mangle]
#[cfg(feature = "ic")]
pub unsafe extern "C" fn get_static_variable(index: usize) -> Value {
    debug_assert!(STATIC_VARIABLES.is_non_null_ptr());
    STATIC_VARIABLES.as_array().get(index)
}
