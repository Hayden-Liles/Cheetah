// Runtime support module for the Cheetah compiler

pub mod list_ops;
pub mod string_ops;
pub mod string_ops_register;

use inkwell::context::Context;
use inkwell::module::Module;

/// Register all runtime functions in the module
pub fn register_runtime_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Register list operation functions
    list_ops::register_list_functions(context, module);

    // Register string operation functions
    string_ops_register::register_string_functions(context, module);
}
