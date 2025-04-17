// Runtime support module for the Cheetah compiler

pub mod list_ops;
pub mod list_ops_runtime;
pub mod string_ops;
pub mod string_ops_register;
pub mod dict_ops;
pub mod dict_methods;
pub mod int_ops;
pub mod exception_ops;
pub mod exception_state;
pub mod exception_runtime;
pub mod print_ops;
pub mod buffered_output;
pub mod debug_utils;
pub mod range_ops;
pub mod range_iterator;
pub mod circular_buffer;
pub mod memory_profiler;
pub mod parallel_ops;

use inkwell::context::Context;
use inkwell::module::Module;

/// Register all runtime functions in the module
pub fn register_runtime_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Register list operation functions
    list_ops::register_list_functions(context, module);

    // Register string operation functions
    string_ops_register::register_string_functions(context, module);

    // Register dictionary operation functions
    dict_ops::register_dict_functions(context, module);

    // Register integer operation functions
    int_ops::register_int_functions(context, module);

    // Register exception handling functions
    exception_ops::register_exception_functions(context, module);

    // Register exception state functions
    exception_state::register_exception_state_functions(context, module);

    // Register print functions
    print_ops::register_print_functions(context, module);

    // Register range functions
    range_ops::register_range_functions(context, module);

    // Register range iterator functions
    range_iterator::register_range_iterator_functions(context, module);

    // Register memory profiler functions
    memory_profiler::register_memory_functions(context, module);
}
