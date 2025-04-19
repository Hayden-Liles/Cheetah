// exception_ops.rs - Runtime support for exception handling

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;

/// Register exception handling functions in the module
pub fn register_exception_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let _exception_struct_type = get_exception_struct_type(context);

    let exception_new_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("exception_new", exception_new_type, None);

    let exception_raise_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("exception_raise", exception_raise_type, None);

    let exception_check_type = context.bool_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    module.add_function("exception_check", exception_check_type, None);

    let exception_get_message_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("exception_get_message", exception_get_message_type, None);

    let exception_get_type_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("exception_get_type", exception_get_type_type, None);

    let exception_free_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("exception_free", exception_free_type, None);
}

/// Get the exception struct type
pub fn get_exception_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}
