// print_ops.rs - Runtime support for print function

use std::ffi::CStr;
use std::os::raw::c_char;

// Cache for the most recently printed string to optimize repeated prints
thread_local! {
    static LAST_PRINTED: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

/// Print a string to stdout (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn print_string(value: *const c_char) {
    unsafe {
        if let Ok(s) = CStr::from_ptr(value).to_str() {
            if s == "\n" {
                super::buffered_output::write_char_to_buffer('\n');
            } else {
                super::buffered_output::write_to_buffer(s);
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
                match str_slice {
                    "" => {
                        super::buffered_output::write_char_to_buffer('\n');
                        super::buffered_output::flush_output_buffer();
                        return;
                    }
                    "Hello World" | "Hello" => {
                        super::buffered_output::writeln_to_buffer(str_slice);
                        return;
                    }
                    _ => {
                        let is_repeat = LAST_PRINTED.with(|last| {
                            let last_str = last.borrow();
                            str_slice == last_str.as_str()
                        });

                        if is_repeat {
                            super::buffered_output::writeln_to_buffer(str_slice);
                            return;
                        }

                        if !str_slice.contains('\n') {
                            super::buffered_output::writeln_to_buffer(str_slice);

                            LAST_PRINTED.with(|last| {
                                *last.borrow_mut() = str_slice.to_string();
                            });
                        } else {
                            if let Some(first_line) = str_slice.split('\n').next() {
                                super::buffered_output::writeln_to_buffer(first_line);

                                LAST_PRINTED.with(|last| {
                                    *last.borrow_mut() = first_line.to_string();
                                });
                            } else {
                                super::buffered_output::write_char_to_buffer('\n');
                                super::buffered_output::flush_output_buffer();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Print an integer to stdout (C-compatible wrapper)
#[no_mangle]
pub extern "C" fn print_int(value: i64) {
    super::buffered_output::write_int_to_buffer(value);
}

/// Print a float to stdout (C-compatible wrapper)
#[no_mangle]
pub extern "C" fn print_float(value: f64) {
    super::buffered_output::write_float_to_buffer(value);
}

/// Print a boolean to stdout (C-compatible wrapper)
#[no_mangle]
pub extern "C" fn print_bool(value: bool) {
    super::buffered_output::write_bool_to_buffer(value);
}

/// Register print operation functions in the module
pub fn register_print_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    use inkwell::AddressSpace;

    let print_string_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("print_string", print_string_type, None);

    let println_string_type = context
        .void_type()
        .fn_type(&[context.ptr_type(AddressSpace::default()).into()], false);
    module.add_function("println_string", println_string_type, None);

    let print_int_type = context
        .void_type()
        .fn_type(&[context.i64_type().into()], false);
    module.add_function("print_int", print_int_type, None);

    let print_float_type = context
        .void_type()
        .fn_type(&[context.f64_type().into()], false);
    module.add_function("print_float", print_float_type, None);

    let print_bool_type = context
        .void_type()
        .fn_type(&[context.bool_type().into()], false);
    module.add_function("print_bool", print_bool_type, None);
}
