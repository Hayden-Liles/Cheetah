// Runtime support module for the Cheetah compiler

pub mod boxed_any;
pub mod boxed_any_ops;
pub mod boxed_dict;
pub mod boxed_list;
pub mod boxed_print_ops;
pub mod boxed_tuple;
pub mod buffer;
pub mod debug_utils;
pub mod dict;
pub mod exception;
pub mod int_ops;
pub mod list;
pub mod memory_profiler;
pub mod min_max_ops;
pub mod parallel_ops;
pub mod print_ops;
pub mod range;
pub mod string;

use inkwell::context::Context;
use inkwell::module::Module;

/// Register all runtime functions in the module
pub fn register_runtime_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Register BoxedAny functions
    boxed_any::register_boxed_any_functions(context, module);

    // Register BoxedAny operations
    boxed_any_ops::register_boxed_any_ops_functions(context, module);

    // Register BoxedList functions
    boxed_list::register_boxed_list_functions(context, module);

    // Register BoxedDict functions
    boxed_dict::register_boxed_dict_functions(context, module);

    // Register BoxedTuple functions
    boxed_tuple::register_boxed_tuple_functions(context, module);

    // Register BoxedAny print functions
    boxed_print_ops::register_boxed_print_functions(context, module);

    // Register list operation functions
    list::register_list_functions(context, module);

    // Register string operation functions
    string::register_string_functions(context, module);

    // Register dictionary operation functions
    dict::register_dict_functions(context, module);

    // Register integer operation functions
    int_ops::register_int_functions(context, module);

    // Register exception handling functions
    exception::register_exception_functions(context, module);

    // Register exception state functions
    exception::register_exception_state(context, module);

    // Register print functions
    print_ops::register_print_functions(context, module);

    // Register range functions
    range::register_range_functions(context, module);

    // Register memory profiler functions
    memory_profiler::register_memory_functions(context, module);

    // Register min and max functions
    min_max_ops::register_min_max_functions(context, module);
}
