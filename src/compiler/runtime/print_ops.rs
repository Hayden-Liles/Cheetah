// print_ops.rs - Runtime support for print function

use std::ffi::CStr;
use std::os::raw::c_char;
use std::io::Write;

/// Print a string to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn print_string(value: *const c_char) {
    unsafe {
        if !value.is_null() {
            let c_str = CStr::from_ptr(value);
            if let Ok(str_slice) = c_str.to_str() {
                print!("{}", str_slice);
                let _ = std::io::stdout().flush();
            }
        }
    }
}

/// Print a string with a newline to stdout (C-compatible wrapper)
/// This function is now implemented to avoid double newlines
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn println_string(value: *const c_char) {
    unsafe {
        if !value.is_null() {
            let c_str = CStr::from_ptr(value);
            if let Ok(str_slice) = c_str.to_str() {
                // Check if the string already ends with a newline
                if str_slice.ends_with('\n') {
                    print!("{}", str_slice);
                } else {
                    println!("{}", str_slice);
                }
                let _ = std::io::stdout().flush();
            }
        }
    }
}

/// Print an integer to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn print_int(value: i64) {
    print!("{}", value);
    let _ = std::io::stdout().flush();
}

/// Print a float to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn print_float(value: f64) {
    print!("{}", value);
    let _ = std::io::stdout().flush();
}

/// Print a boolean to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn print_bool(value: bool) {
    print!("{}", if value { "True" } else { "False" });
    let _ = std::io::stdout().flush();
}

/// Register print operation functions in the module
pub fn register_print_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    use inkwell::AddressSpace;

    // Create print_string function
    let print_string_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // string pointer
        false,
    );
    module.add_function("print_string", print_string_type, None);

    // Create println_string function
    let println_string_type = context.void_type().fn_type(
        &[context.ptr_type(AddressSpace::default()).into()], // string pointer
        false,
    );
    module.add_function("println_string", println_string_type, None);

    // Create print_int function
    let print_int_type = context.void_type().fn_type(
        &[context.i64_type().into()], // integer value
        false,
    );
    module.add_function("print_int", print_int_type, None);

    // Create print_float function
    let print_float_type = context.void_type().fn_type(
        &[context.f64_type().into()], // float value
        false,
    );
    module.add_function("print_float", print_float_type, None);

    // Create print_bool function
    let print_bool_type = context.void_type().fn_type(
        &[context.bool_type().into()], // boolean value
        false,
    );
    module.add_function("print_bool", print_bool_type, None);
}
