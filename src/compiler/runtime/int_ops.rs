// int_ops.rs - Runtime support for integer operations

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Register integer operation functions in the module
pub fn register_int_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Create int_to_ptr function (converts an integer to a pointer)
    let int_to_ptr_type = context
        .ptr_type(AddressSpace::default())
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("int_to_ptr", int_to_ptr_type, None);
}
