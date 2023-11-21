use crate::{
    stabilization::StableMemoryAccess,
    types::{MutBox, Obj, Value, TAG_MUTBOX},
};

use super::{Serializer, StableValue, StaticScanner};

// Note: The unaligned reads are needed because heap allocations are aligned to 32-bit,
// while the stable layout uses 64-bit values.

#[repr(C)]
pub struct StableMutBox {
    field: StableValue,
}

impl StaticScanner<StableValue> for StableMutBox {
    fn update_pointers<C: StableMemoryAccess, F: Fn(&mut C, StableValue) -> StableValue>(
        &mut self,
        context: &mut C,
        translate: &F,
    ) -> bool {
        self.field = translate(context, self.field);
        true
    }
}

impl StaticScanner<Value> for MutBox {
    fn update_pointers<C: StableMemoryAccess, F: Fn(&mut C, Value) -> Value>(
        &mut self,
        context: &mut C,
        translate: &F,
    ) -> bool {
        self.field = translate(context, self.field);
        true
    }
}

impl Serializer<MutBox> for StableMutBox {
    unsafe fn serialize_static_part(main_object: *mut MutBox) -> Self {
        StableMutBox {
            field: StableValue::serialize((*main_object).field),
        }
    }

    unsafe fn deserialize_static_part(stable_object: *mut Self, target_address: Value) -> MutBox {
        let field = stable_object.read_unaligned().field.deserialize();
        MutBox {
            header: Obj::new(TAG_MUTBOX, target_address),
            field,
        }
    }
}
