// list_ops.rs - Runtime support for list operations

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::types::BasicType;
use inkwell::AddressSpace;

/// Register list operation functions in the module
pub fn register_list_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Define the list struct type
    let _list_struct_type = context.struct_type(
        &[
            context.i64_type().into(), // length
            context.i64_type().into(), // capacity
            context.ptr_type(AddressSpace::default()).into(), // data pointer
        ],
        false,
    );

    // Create list_new function (creates an empty list)
    let list_new_type = context.ptr_type(AddressSpace::default()).fn_type(&[], false);
    module.add_function("list_new", list_new_type, None);

    // Create list_with_capacity function (creates a list with pre-allocated capacity)
    let list_with_capacity_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("list_with_capacity", list_with_capacity_type, None);

    // Create list_get function (gets an item from a list)
    let list_get_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list pointer
            context.i64_type().into(),                        // index
        ],
        false,
    );
    module.add_function("list_get", list_get_type, None);

    // Create list_slice function (gets a slice from a list)
    let list_slice_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list pointer
            context.i64_type().into(),                        // start index
            context.i64_type().into(),                        // stop index
            context.i64_type().into(),                        // step
        ],
        false,
    );
    module.add_function("list_slice", list_slice_type, None);

    // Create list_set function (sets an item in a list)
    let list_set_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list pointer
            context.i64_type().into(),                        // index
            context.ptr_type(AddressSpace::default()).into(), // value
        ],
        false,
    );
    module.add_function("list_set", list_set_type, None);

    // Create list_append function (appends an item to a list)
    let list_append_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list pointer
            context.ptr_type(AddressSpace::default()).into(), // value
        ],
        false,
    );
    module.add_function("list_append", list_append_type, None);

    // Create list_concat function (concatenates two lists)
    let list_concat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list1 pointer
            context.ptr_type(AddressSpace::default()).into(), // list2 pointer
        ],
        false,
    );
    module.add_function("list_concat", list_concat_type, None);

    // Create list_repeat function (repeats a list n times)
    let list_repeat_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // list pointer
            context.i64_type().into(),                        // repeat count
        ],
        false,
    );
    module.add_function("list_repeat", list_repeat_type, None);

    // Create list_free function (frees a list's memory)
    let list_free_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // list pointer
        false,
    );
    module.add_function("list_free", list_free_type, None);

    // Create list_len function (gets the length of a list)
    let list_len_type = context.i64_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // list pointer
        false,
    );
    module.add_function("list_len", list_len_type, None);
}

/// Get the list struct type
pub fn get_list_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(), // length
            context.i64_type().into(), // capacity
            context.ptr_type(AddressSpace::default()).into(), // data pointer
        ],
        false,
    )
}

/// Get the list element pointer type
pub fn get_list_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context.ptr_type(AddressSpace::default()).as_basic_type_enum()
}
