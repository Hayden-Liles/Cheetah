// exception_ops.rs - Runtime support for exception handling

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;
use inkwell::types::StructType;

/// Register exception handling functions in the module
pub fn register_exception_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Define the exception struct type
    let _exception_struct_type = get_exception_struct_type(context);

    // Create exception_new function (creates a new exception)
    let exception_new_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // type name (string pointer)
            context.ptr_type(AddressSpace::default()).into(), // message (string pointer)
        ],
        false,
    );
    module.add_function("exception_new", exception_new_type, None);

    // Create exception_raise function (raises an exception)
    let exception_raise_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception pointer
        ],
        false,
    );
    module.add_function("exception_raise", exception_raise_type, None);

    // Create exception_check function (checks if an exception is of a given type)
    let exception_check_type = context.bool_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception pointer
            context.ptr_type(AddressSpace::default()).into(), // type name (string pointer)
        ],
        false,
    );
    module.add_function("exception_check", exception_check_type, None);

    // Create exception_get_message function (gets the message from an exception)
    let exception_get_message_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception pointer
        ],
        false,
    );
    module.add_function("exception_get_message", exception_get_message_type, None);

    // Create exception_get_type function (gets the type from an exception)
    let exception_get_type_type = context.ptr_type(AddressSpace::default()).fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception pointer
        ],
        false,
    );
    module.add_function("exception_get_type", exception_get_type_type, None);

    // Create exception_free function (frees an exception's memory)
    let exception_free_type = context.void_type().fn_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // exception pointer
        ],
        false,
    );
    module.add_function("exception_free", exception_free_type, None);
}

/// Get the exception struct type
pub fn get_exception_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(), // type name (string pointer)
            context.ptr_type(AddressSpace::default()).into(), // message (string pointer)
        ],
        false,
    )
}
