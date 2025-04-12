// dict_ops.rs - Runtime support for dictionary operations

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::types::BasicType;
use inkwell::AddressSpace;

/// Register dictionary operation functions in the module
pub fn register_dict_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Define the dictionary entry struct type (key-value pair)
    let _dict_entry_struct_type = context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // key pointer
            context.ptr_type(AddressSpace::default()).into(), // value pointer
            context.i64_type().into(),                        // hash value
        ],
        false,
    );

    // Define the dictionary struct type
    let _dict_struct_type = context.struct_type(
        &[
            context.i64_type().into(), // count
            context.i64_type().into(), // capacity
            context.ptr_type(AddressSpace::default()).into(), // entries pointer
        ],
        false,
    );

    // Create dict_new function (creates an empty dictionary)
    let dict_new_type = context.ptr_type(AddressSpace::default()).fn_type(&[], false);
    module.add_function("dict_new", dict_new_type, None);

    // Create dict_with_capacity function (creates a dictionary with pre-allocated capacity)
    let dict_with_capacity_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("dict_with_capacity", dict_with_capacity_type, None);

    // Create dict_get function (gets a value from a dictionary by key)
    let dict_get_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict pointer
            context.ptr_type(AddressSpace::default()).into(), // key pointer
        ],
        false,
    );
    module.add_function("dict_get", dict_get_type, None);

    // Create dict_set function (sets a value in a dictionary)
    let dict_set_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict pointer
            context.ptr_type(AddressSpace::default()).into(), // key pointer
            context.ptr_type(AddressSpace::default()).into(), // value pointer
        ],
        false,
    );
    module.add_function("dict_set", dict_set_type, None);

    // Create dict_contains function (checks if a key exists in a dictionary)
    let dict_contains_type = context.i8_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict pointer
            context.ptr_type(AddressSpace::default()).into(), // key pointer
        ],
        false,
    );
    module.add_function("dict_contains", dict_contains_type, None);

    // Create dict_remove function (removes a key-value pair from a dictionary)
    let dict_remove_type = context.i8_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict pointer
            context.ptr_type(AddressSpace::default()).into(), // key pointer
        ],
        false,
    );
    module.add_function("dict_remove", dict_remove_type, None);

    // Create dict_clear function (removes all key-value pairs from a dictionary)
    let dict_clear_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // dict pointer
        false,
    );
    module.add_function("dict_clear", dict_clear_type, None);

    // Create dict_len function (gets the number of key-value pairs in a dictionary)
    let dict_len_type = context.i64_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // dict pointer
        false,
    );
    module.add_function("dict_len", dict_len_type, None);

    // Create dict_free function (frees a dictionary's memory)
    let dict_free_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // dict pointer
        false,
    );
    module.add_function("dict_free", dict_free_type, None);

    // Create dict_merge function (merges two dictionaries)
    let dict_merge_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict1 pointer
            context.ptr_type(AddressSpace::default()).into(), // dict2 pointer
        ],
        false,
    );
    module.add_function("dict_merge", dict_merge_type, None);

    // Create dict_update function (updates a dictionary with another dictionary)
    let dict_update_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // dict1 pointer
            context.ptr_type(AddressSpace::default()).into(), // dict2 pointer
        ],
        false,
    );
    module.add_function("dict_update", dict_update_type, None);
}

/// Get the dictionary struct type
pub fn get_dict_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(), // count
            context.i64_type().into(), // capacity
            context.ptr_type(AddressSpace::default()).into(), // entries pointer
        ],
        false,
    )
}

/// Get the dictionary entry struct type
pub fn get_dict_entry_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // key pointer
            context.ptr_type(AddressSpace::default()).into(), // value pointer
            context.i64_type().into(),                        // hash value
        ],
        false,
    )
}

/// Get the dictionary element pointer type
pub fn get_dict_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context.ptr_type(AddressSpace::default()).as_basic_type_enum()
}
