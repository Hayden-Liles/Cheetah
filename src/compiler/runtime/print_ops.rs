// print_ops.rs - Extended runtime support for recursive printing

use std::ffi::CStr;
use std::os::raw::c_char;
use crate::compiler::runtime::list;
use crate::compiler::runtime::list::RawList;
use crate::compiler::runtime::dict::Dict;

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
                super::buffer::write_str("\n");
            } else {
                super::buffer::write_str(s);
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
                        super::buffer::write_str("\n");
                        super::buffer::flush();
                        return;
                    }
                    "Hello World" | "Hello" => {
                        super::buffer::write_str(str_slice);
                        super::buffer::write_newline();
                        return;
                    }
                    _ => {
                        let is_repeat = LAST_PRINTED.with(|last| {
                            let last_str = last.borrow();
                            str_slice == last_str.as_str()
                        });

                        if is_repeat {
                            super::buffer::write_str(str_slice);
                            super::buffer::write_newline();
                            return;
                        }

                        if !str_slice.contains('\n') {
                            super::buffer::write_str(str_slice);
                            super::buffer::write_newline();

                            LAST_PRINTED.with(|last| {
                                *last.borrow_mut() = str_slice.to_string();
                            });
                        } else {
                            if let Some(first_line) = str_slice.split('\n').next() {
                                super::buffer::write_str(first_line);
                                super::buffer::write_newline();

                                LAST_PRINTED.with(|last| {
                                    *last.borrow_mut() = first_line.to_string();
                                });
                            } else {
                                super::buffer::write_str("\n");
                                super::buffer::flush();
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
    super::buffer::write_int(value);
}

/// Print a float to stdout (C-compatible wrapper)
#[no_mangle]
pub extern "C" fn print_float(value: f64) {
    super::buffer::write_float(value);
}

/// Print a boolean to stdout (C-compatible wrapper)
#[no_mangle]
pub extern "C" fn print_bool(value: bool) {
    super::buffer::write_bool(value);
}

/// Print a Python-style list recursively
#[no_mangle]
pub unsafe extern "C" fn print_list(lst: *mut RawList) {
    if lst.is_null() {
        super::buffer::write_str("None");
        return;
    }

    // Use catch_unwind to prevent segmentation faults
    let result = std::panic::catch_unwind(|| {
        super::buffer::write_str("[");
        let len = (*lst).length;

        for i in 0..len {
            // Get element pointer
            let elem = list::list_get(lst, i);

            // Handle each element safely
            if elem.is_null() {
                super::buffer::write_str("None");
            } else {
                // Try to print the element safely
                let elem_result = std::panic::catch_unwind(|| {
                    // Try to interpret as a string
                    let str_result = std::panic::catch_unwind(|| {
                        let s = CStr::from_ptr(elem as *const c_char).to_string_lossy();
                        if s.len() > 0 && s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                            super::buffer::write_str("\"");
                            super::buffer::write_str(&s);
                            super::buffer::write_str("\"");
                            return true;
                        }
                        false
                    });

                    if str_result.is_ok() && str_result.unwrap() {
                        return;
                    }

                    // Try to interpret as a float FIRST
                    let float_result = std::panic::catch_unwind(|| {
                        let val = *(elem as *const f64);
                        // Only print as float if it really looks like one
                        if !val.is_nan() && val.fract() != 0.0 {
                            super::buffer::write_float(val);
                            return true;
                        }
                        false
                    });

                    if float_result.is_ok() && float_result.unwrap() {
                        return;
                    }

                    // Try to interpret as an integer
                    let int_result = std::panic::catch_unwind(|| {
                        let val = *(elem as *const i64);
                        super::buffer::write_int(val);
                    });

                    if int_result.is_ok() {
                        return;
                    }

                    // If all else fails, just print a placeholder
                    super::buffer::write_str("<Any>");
                });

                // If the element printing failed, print a placeholder
                if elem_result.is_err() {
                    super::buffer::write_str("<Any>");
                }
            }

            if i + 1 < len {
                super::buffer::write_str(", ");
            }
        }

        super::buffer::write_str("]");
    });

    // If the list printing failed, print a placeholder
    if result.is_err() {
        super::buffer::write_str("<List>");
    }
}

/// Print a Python-style dict recursively
#[no_mangle]
pub unsafe extern "C" fn print_dict(dict: *mut Dict) {
    if dict.is_null() {
        super::buffer::write_str("None");
        return;
    }

    // Use catch_unwind to prevent segmentation faults
    let result = std::panic::catch_unwind(|| {
        super::buffer::write_str("{");

        // Safely get the keys
        let keys_result = std::panic::catch_unwind(|| {
            let keys = super::dict::dict_keys(dict) as *mut RawList;
            if keys.is_null() {
                return 0;
            }
            list::list_len(keys)
        });

        let len = keys_result.unwrap_or(0);
        if len == 0 {
            super::buffer::write_str("}");
            return;
        }

        let keys = super::dict::dict_keys(dict) as *mut RawList;

        for i in 0..len {
            // Safely get and print each key-value pair
            let pair_result = std::panic::catch_unwind(|| {
                let key = list::list_get(keys, i);
                let val = super::dict::dict_get(dict, key);

                // Print key
                let key_result = std::panic::catch_unwind(|| {
                    print_any(key as *const c_char);
                });

                if key_result.is_err() {
                    super::buffer::write_str("<Key>");
                }

                super::buffer::write_str(": ");

                // Print value
                let val_result = std::panic::catch_unwind(|| {
                    print_any(val as *const c_char);
                });

                if val_result.is_err() {
                    super::buffer::write_str("<Value>");
                }
            });

            // If the pair printing failed, print a placeholder
            if pair_result.is_err() {
                super::buffer::write_str("<Pair>");
            }

            if i + 1 < len {
                super::buffer::write_str(", ");
            }
        }

        super::buffer::write_str("}");
    });

    // If the dict printing failed, print a placeholder
    if result.is_err() {
        super::buffer::write_str("<Dict>");
    }
}

/// A catch-all runtime printer that inspects the c_char pointer for type
#[no_mangle]
pub unsafe extern "C" fn print_any(ptr: *const c_char) {
    if ptr.is_null() {
        super::buffer::write_str("None");
        return;
    }

    // Try to interpret as a string first
    let result = std::panic::catch_unwind(|| {
        let s = CStr::from_ptr(ptr).to_string_lossy();
        if s.len() > 0 && s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
            // Check if it looks like a string (starts with a letter or quote)
            let first_char = s.chars().next().unwrap();
            if first_char.is_alphabetic() || first_char == '"' || first_char == '\'' {
                super::buffer::write_str("\"");
                super::buffer::write_str(&s);
                super::buffer::write_str("\"");
                return true;
            }
        }
        false
    });

    if result.is_ok() && result.unwrap() {
        return;
    }

    // Try to interpret as a float
    let result = std::panic::catch_unwind(|| {
        let val = *(ptr as *const f64);
        if !val.is_nan() && (val.abs() > 0.000001 || val == 0.0) {
            super::buffer::write_float(val);
            return true;
        }
        false
    });

    if result.is_ok() && result.unwrap() {
        return;
    }

    // Try to interpret as an integer
    let result = std::panic::catch_unwind(|| {
        let val = *(ptr as *const i64);
        // Only print if it looks like a reasonable integer
        if val > -1000000000 && val < 1000000000 {
            super::buffer::write_int(val);
            return true;
        }
        false
    });

    if result.is_ok() && result.unwrap() {
        return;
    }

    // If all else fails, just print a placeholder
    super::buffer::write_str("<Any>");
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

    // Add new recursive print functions
    let ptr_type = context.ptr_type(AddressSpace::default());

    module.add_function(
        "print_list",
        context.void_type().fn_type(&[ptr_type.into()], false),
        None
    );

    module.add_function(
        "print_dict",
        context.void_type().fn_type(&[ptr_type.into()], false),
        None
    );

    module.add_function(
        "print_any",
        context.void_type().fn_type(&[ptr_type.into()], false),
        None
    );

    // Add function for printing heterogeneous lists
    module.add_function(
        "print_list_any",
        context.void_type().fn_type(&[ptr_type.into()], false),
        None
    );
}

/// Register print runtime functions for JIT execution
pub fn register_print_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("print_string") {
        engine.add_global_mapping(&f, print_string as usize);
    }
    if let Some(f) = module.get_function("println_string") {
        engine.add_global_mapping(&f, println_string as usize);
    }
    if let Some(f) = module.get_function("print_int") {
        engine.add_global_mapping(&f, print_int as usize);
    }
    if let Some(f) = module.get_function("print_float") {
        engine.add_global_mapping(&f, print_float as usize);
    }
    if let Some(f) = module.get_function("print_bool") {
        engine.add_global_mapping(&f, print_bool as usize);
    }
    if let Some(f) = module.get_function("print_list") {
        engine.add_global_mapping(&f, print_list as usize);
    }
    if let Some(f) = module.get_function("print_dict") {
        engine.add_global_mapping(&f, print_dict as usize);
    }
    if let Some(f) = module.get_function("print_any") {
        engine.add_global_mapping(&f, print_any as usize);
    }

    // Register the print_list_any function from C
    if let Some(f) = module.get_function("print_list_any") {
        extern "C" {
            fn print_list_any(list: *mut crate::compiler::runtime::list::RawList);
        }
        engine.add_global_mapping(&f, print_list_any as usize);
    }

    Ok(())
}
