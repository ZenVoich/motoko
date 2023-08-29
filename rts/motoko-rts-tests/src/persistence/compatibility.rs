use motoko_rts::memory::{alloc_blob, Memory};
use motoko_rts::persistence::compatibility::{
    memory_compatible, MUTABLE_ENCODING_TAG, OBJECT_ENCODING_TAG, OPTION_ENCODING_TAG,
};
use motoko_rts::types::{Bytes, Value};
use std::hash::Hasher;
use std::{collections::hash_map::DefaultHasher, hash::Hash};

use crate::memory::{initialize_test_memory, reset_test_memory, TestMemory};

struct BinaryData {
    byte_sequence: Vec<u8>,
}

impl BinaryData {
    fn new() -> BinaryData {
        BinaryData {
            byte_sequence: vec![],
        }
    }

    fn write_i32(&mut self, value: i32) {
        for byte in value.to_le_bytes() {
            self.byte_sequence.push(byte);
        }
    }

    fn write_hash(&mut self, value: &str) {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        self.write_i32(hasher.finish() as i32);
    }

    unsafe fn make_blob<M: Memory>(&self, mem: &mut M) -> Value {
        let length = Bytes(self.byte_sequence.len() as u32);
        let value = alloc_blob(mem, length);
        let target = value.as_blob_mut();

        for index in 0..length.as_usize() {
            let byte = target.payload_addr().add(index);
            *byte = *self.byte_sequence.get(index).unwrap();
        }
        value
    }
}

#[derive(Clone)]
enum Type {
    Object(ObjectType),
    Mutable(MutableType),
    Option(OptionType),
}

impl Type {
    fn serialize(&self, output: &mut BinaryData) {
        match &self {
            Self::Object(object_type) => object_type.serialize(output),
            Self::Mutable(mutable_type) => mutable_type.serialize(output),
            Self::Option(option_type) => option_type.serialize(output),
        }
    }
}

#[derive(Clone)]
struct TypeReference {
    index: i32,
}

impl TypeReference {
    fn nat() -> TypeReference {
        TypeReference { index: -1 }
    }
}

#[derive(Clone)]
struct Field {
    name: String,
    field_type: TypeReference,
}

impl Field {
    fn serialize(&self, output: &mut BinaryData) {
        output.write_hash(&self.name);
        output.write_i32(self.field_type.index);
    }
}

#[derive(Clone)]
struct ObjectType {
    fields: Vec<Field>,
}

impl ObjectType {
    fn serialize(&self, output: &mut BinaryData) {
        output.write_i32(OBJECT_ENCODING_TAG);
        output.write_i32(self.fields.len() as i32);
        for field in &self.fields {
            field.serialize(output);
        }
    }
}

#[derive(Clone)]
struct MutableType {
    variable_type: TypeReference,
}

impl MutableType {
    fn serialize(&self, output: &mut BinaryData) {
        output.write_i32(MUTABLE_ENCODING_TAG);
        output.write_i32(self.variable_type.index);
    }
}

#[derive(Clone)]
struct OptionType {
    option_type: TypeReference,
}

impl OptionType {
    fn serialize(&self, output: &mut BinaryData) {
        output.write_i32(OPTION_ENCODING_TAG);
        output.write_i32(self.option_type.index);
    }
}

struct TypeTable {
    types: Vec<Type>,
}

impl TypeTable {
    fn new(types: Vec<Type>) -> TypeTable {
        TypeTable { types }
    }

    fn serialize(&self) -> BinaryData {
        let mut output = BinaryData::new();
        output.write_i32(self.types.len() as i32);
        for current_type in &self.types {
            current_type.serialize(&mut output);
        }
        output
    }
}

unsafe fn build<M: Memory>(mem: &mut M, types: Vec<Type>) -> Value {
    TypeTable::new(types).serialize().make_blob(mem)
}

unsafe fn are_compatible<M: Memory>(
    mem: &mut M,
    old_types: Vec<Type>,
    new_types: Vec<Type>,
) -> bool {
    let old_type_blob = build(mem, old_types);
    let new_type_blob = build(mem, new_types);
    memory_compatible(mem, old_type_blob, new_type_blob)
}

unsafe fn is_compatible<M: Memory>(mem: &mut M, old_type: Type, new_type: Type) -> bool {
    are_compatible(mem, vec![old_type], vec![new_type])
}

pub unsafe fn test() {
    println!("  Testing memory compatibility...");
    let mut heap = initialize_test_memory();
    test_sucessful_cases(&mut heap);
    test_failing_cases(&mut heap);
    reset_test_memory();
}

unsafe fn test_sucessful_cases(heap: &mut TestMemory) {
    test_empty_actor(heap);
    test_reordered_actor_fields(heap);
    test_removed_actor_fields(heap);
    test_mutable_fields(heap);
    test_added_actor_fields(heap);
    test_removed_object_fields(heap);
    test_direct_recursive_type(heap);
    test_indirect_recursive_type(heap);
    test_option_types(heap);
}

unsafe fn test_empty_actor(heap: &mut TestMemory) {
    let old_type = Type::Object(ObjectType { fields: vec![] });
    let new_type = Type::Object(ObjectType { fields: vec![] });
    assert!(is_compatible(heap, old_type, new_type));
}

unsafe fn test_reordered_actor_fields(heap: &mut TestMemory) {
    let field1 = Field {
        name: String::from("Field1"),
        field_type: TypeReference::nat(),
    };
    let field2 = Field {
        name: String::from("Field2"),
        field_type: TypeReference::nat(),
    };

    let old_type = Type::Object(ObjectType {
        fields: vec![field1.clone(), field2.clone()],
    });
    let new_type = Type::Object(ObjectType {
        fields: vec![field2.clone(), field1.clone()],
    });

    assert!(is_compatible(heap, old_type, new_type));
}

unsafe fn test_removed_actor_fields(heap: &mut TestMemory) {
    let field1 = Field {
        name: String::from("Field1"),
        field_type: TypeReference::nat(),
    };
    let field2 = Field {
        name: String::from("Field2"),
        field_type: TypeReference::nat(),
    };
    let field3 = Field {
        name: String::from("Field3"),
        field_type: TypeReference::nat(),
    };

    let old_type = Type::Object(ObjectType {
        fields: vec![field1.clone(), field2.clone(), field3.clone()],
    });
    let new_type = Type::Object(ObjectType {
        fields: vec![field2.clone()],
    });

    assert!(is_compatible(heap, old_type, new_type));
}

unsafe fn test_added_actor_fields(heap: &mut TestMemory) {
    let field1 = Field {
        name: String::from("Field1"),
        field_type: TypeReference::nat(),
    };
    let field2 = Field {
        name: String::from("Field2"),
        field_type: TypeReference::nat(),
    };
    let field3 = Field {
        name: String::from("Field3"),
        field_type: TypeReference::nat(),
    };

    let old_type = Type::Object(ObjectType {
        fields: vec![field2.clone()],
    });
    let new_type = Type::Object(ObjectType {
        fields: vec![field1.clone(), field2.clone(), field3.clone()],
    });

    assert!(is_compatible(heap, old_type, new_type));
}

unsafe fn test_mutable_fields(heap: &mut TestMemory) {
    let actor_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let mutable_type = Type::Mutable(MutableType {
        variable_type: TypeReference::nat(),
    });
    let old_types = vec![actor_type.clone(), mutable_type.clone()];
    let new_types = vec![actor_type.clone(), mutable_type.clone()];
    assert!(are_compatible(heap, old_types, new_types));
}

unsafe fn test_removed_object_fields(heap: &mut TestMemory) {
    let actor_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });

    let field1 = Field {
        name: String::from("Field1"),
        field_type: TypeReference::nat(),
    };
    let field2 = Field {
        name: String::from("Field2"),
        field_type: TypeReference::nat(),
    };
    let field3 = Field {
        name: String::from("Field3"),
        field_type: TypeReference::nat(),
    };

    let old_type = Type::Object(ObjectType {
        fields: vec![field1.clone(), field2.clone(), field3.clone()],
    });
    let new_type = Type::Object(ObjectType {
        fields: vec![field2.clone()],
    });

    let old_types = vec![actor_type.clone(), old_type];
    let new_types = vec![actor_type.clone(), new_type];
    assert!(are_compatible(heap, old_types, new_types));
}

unsafe fn test_direct_recursive_type(heap: &mut TestMemory) {
    let actor_field = Field {
        name: String::from("ActorField"),
        field_type: TypeReference { index: 1 },
    };
    let actor_type = Type::Object(ObjectType {
        fields: vec![actor_field],
    });
    let recursive_field = Field {
        name: String::from("RecursiveField"),
        field_type: TypeReference { index: 1 },
    };
    let recursive_type = Type::Object(ObjectType {
        fields: vec![recursive_field],
    });
    let types = vec![actor_type, recursive_type];
    assert!(are_compatible(heap, types.clone(), types.clone()));
}

unsafe fn test_indirect_recursive_type(heap: &mut TestMemory) {
    let actor_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let first_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("Field1"),
            field_type: TypeReference { index: 2 },
        }],
    });
    let second_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("Field2"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let types = vec![actor_type, first_type, second_type];
    assert!(are_compatible(heap, types.clone(), types.clone()));
}

unsafe fn test_option_types(heap: &mut TestMemory) {
    let actor_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("OptionalField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let option_type = Type::Option(OptionType {
        option_type: TypeReference::nat(),
    });
    let types = vec![actor_type, option_type];
    assert!(are_compatible(heap, types.clone(), types.clone()));
}

unsafe fn test_failing_cases(heap: &mut TestMemory) {
    test_added_object_fields(heap);
    test_mutable_mismatch(heap);
    test_immutable_mismatch(heap);
    test_recursion_mismatch(heap);
    test_option_mismatch(heap);
}

unsafe fn test_recursion_mismatch(heap: &mut TestMemory) {
    let old_actor_field = Field {
        name: String::from("ActorField"),
        field_type: TypeReference { index: 1 },
    };
    let old_actor = Type::Object(ObjectType {
        fields: vec![old_actor_field],
    });
    let recursive_field = Field {
        name: String::from("Field"),
        field_type: TypeReference { index: 1 },
    };
    let recursive_type = Type::Object(ObjectType {
        fields: vec![recursive_field],
    });
    let new_actor_field = Field {
        name: String::from("ActorField"),
        field_type: TypeReference { index: 1 },
    };
    let new_actor = Type::Object(ObjectType {
        fields: vec![new_actor_field],
    });
    let non_recursive_field = Field {
        name: String::from("Field"),
        field_type: TypeReference::nat(),
    };
    let non_recursive_type = Type::Object(ObjectType {
        fields: vec![non_recursive_field],
    });
    let old_types = vec![old_actor, recursive_type];
    let new_types = vec![new_actor, non_recursive_type];
    assert!(!are_compatible(heap, old_types, new_types));
}

unsafe fn test_added_object_fields(heap: &mut TestMemory) {
    let actor_type = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });

    let field1 = Field {
        name: String::from("Field1"),
        field_type: TypeReference::nat(),
    };
    let field2 = Field {
        name: String::from("Field2"),
        field_type: TypeReference::nat(),
    };
    let field3 = Field {
        name: String::from("Field3"),
        field_type: TypeReference::nat(),
    };

    let old_type = Type::Object(ObjectType {
        fields: vec![field2.clone()],
    });
    let new_type = Type::Object(ObjectType {
        fields: vec![field1.clone(), field2.clone(), field3.clone()],
    });

    let old_types = vec![actor_type.clone(), old_type];
    let new_types = vec![actor_type.clone(), new_type];
    assert!(!are_compatible(heap, old_types, new_types));
}

unsafe fn test_mutable_mismatch(heap: &mut TestMemory) {
    let old_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let mutable_type = Type::Mutable(MutableType {
        variable_type: TypeReference::nat(),
    });
    let new_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference::nat(),
        }],
    });
    let old_types = vec![old_actor, mutable_type];
    let new_types = vec![new_actor];
    assert!(!are_compatible(heap, old_types, new_types));
}

unsafe fn test_immutable_mismatch(heap: &mut TestMemory) {
    let old_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference::nat(),
        }],
    });
    let new_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("ActorField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let mutable_type = Type::Mutable(MutableType {
        variable_type: TypeReference::nat(),
    });
    let old_types = vec![old_actor];
    let new_types = vec![new_actor, mutable_type];
    assert!(!are_compatible(heap, old_types, new_types));
}

unsafe fn test_option_mismatch(heap: &mut TestMemory) {
    let old_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("OptionalField"),
            field_type: TypeReference { index: 1 },
        }],
    });
    let option_type = Type::Option(OptionType {
        option_type: TypeReference::nat(),
    });
    let new_actor = Type::Object(ObjectType {
        fields: vec![Field {
            name: String::from("OptionalField"),
            field_type: TypeReference::nat(),
        }],
    });
    let old_types = vec![old_actor, option_type];
    let new_types = vec![new_actor];
    assert!(!are_compatible(heap, old_types, new_types));
}
