// list_ops.rs - Runtime support for list operations

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::AddressSpace;

/// Register list operation functions in the module
pub fn register_list_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let _list_struct_type = context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );

    let list_new_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[], false);
    module.add_function("list_new", list_new_type, None);

    let list_with_capacity_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("list_with_capacity", list_with_capacity_type, None);

    let list_get_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("list_get", list_get_type, None);

    let list_slice_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
            context.i64_type().into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("list_slice", list_slice_type, None);

    let list_set_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("list_set", list_set_type, None);

    let list_append_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("list_append", list_append_type, None);

    let list_concat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("list_concat", list_concat_type, None);

    let list_repeat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("list_repeat", list_repeat_type, None);

    let list_free_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("list_free", list_free_type, None);

    let list_len_type = context
        .i64_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("list_len", list_len_type, None);
}

/// Get the list struct type
pub fn get_list_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}

/// Get the list element pointer type
pub fn get_list_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context
        .ptr_type(AddressSpace::default())
        .as_basic_type_enum()
}
