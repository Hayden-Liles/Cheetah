// dict_ops.rs - Runtime support for dictionary operations

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::AddressSpace;

/// Register dictionary operation functions in the module
pub fn register_dict_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let _dict_entry_struct_type = context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    );

    let _dict_struct_type = context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let dict_new_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[], false);
    module.add_function("dict_new", dict_new_type, None);

    let dict_with_capacity_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("dict_with_capacity", dict_with_capacity_type, None);

    let dict_get_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_get", dict_get_type, None);

    let dict_set_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_set", dict_set_type, None);

    let dict_contains_type = context.i8_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_contains", dict_contains_type, None);

    let dict_remove_type = context.i8_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_remove", dict_remove_type, None);

    let dict_clear_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_clear", dict_clear_type, None);

    let dict_len_type = context
        .i64_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_len", dict_len_type, None);

    let dict_free_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_free", dict_free_type, None);

    let dict_merge_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_merge", dict_merge_type, None);

    let dict_update_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("dict_update", dict_update_type, None);

    let dict_keys_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_keys", dict_keys_type, None);

    let dict_values_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_values", dict_values_type, None);

    let dict_items_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("dict_items", dict_items_type, None);
}

/// Get the dictionary struct type
pub fn get_dict_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}

/// Get the dictionary entry struct type
pub fn get_dict_entry_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    )
}

/// Get the dictionary element pointer type
pub fn get_dict_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context
        .ptr_type(AddressSpace::default())
        .as_basic_type_enum()
}
