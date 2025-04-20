// min_max_ops.rs - Runtime support for min and max operations

use inkwell::context::Context;
use inkwell::module::Module;

/// Register min and max operation functions in the module
pub fn register_min_max_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Register min functions
    let min_int_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("min_int", min_int_type, None);

    let min_float_type = context.f64_type().fn_type(
        &[
            context.f64_type().into(),
            context.f64_type().into(),
        ],
        false,
    );
    module.add_function("min_float", min_float_type, None);

    // Register max functions
    let max_int_type = context.i64_type().fn_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
        ],
        false,
    );
    module.add_function("max_int", max_int_type, None);

    let max_float_type = context.f64_type().fn_type(
        &[
            context.f64_type().into(),
            context.f64_type().into(),
        ],
        false,
    );
    module.add_function("max_float", max_float_type, None);
}

/// Find the minimum of two integers (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn min_int(a: i64, b: i64) -> i64 {
    if a < b { a } else { b }
}

/// Find the minimum of two floats (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn min_float(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}

/// Find the maximum of two integers (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn max_int(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}

/// Find the maximum of two floats (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn max_float(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}
