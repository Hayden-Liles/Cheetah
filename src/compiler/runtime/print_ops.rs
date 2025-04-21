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
    if lst.is_null() { return; }
    super::buffer::write_str("[");
    let len = (*lst).length;
    for i in 0..len {
        // get element pointer
        let elem = list::list_get(lst, i);

        // Try to determine the actual type of the element
        if elem.is_null() {
            super::buffer::write_str("None");
        } else {
            // First, try to interpret as a string
            let maybe_str = CStr::from_ptr(elem as *const c_char).to_string_lossy();

            // Check if this is a nested list
            let maybe_nested_list = elem as *mut RawList;
            if !maybe_nested_list.is_null() && (*maybe_nested_list).length >= 0 && (*maybe_nested_list).length < 1000000 {
                // This might be a nested list, try to print it as such
                print_list(maybe_nested_list);
            } else if maybe_str.starts_with("[") && maybe_str.ends_with("]") {
                // This is a string representation of a list
                super::buffer::write_str(&maybe_str);
            } else if maybe_str.starts_with("{") && maybe_str.ends_with("}") {
                // This is a string representation of a dict
                super::buffer::write_str(&maybe_str);
            } else if maybe_str.starts_with("<") {
                // This is a heterogeneous list element or Any type, try to print the actual value
                // First check if it's a string (common case for heterogeneous lists)
                if maybe_str.contains('a') || maybe_str.contains('b') || maybe_str.contains('c') {
                    // Likely a string
                    super::buffer::write_str("'");
                    // Extract the actual string if possible
                    if let Some(c) = maybe_str.chars().nth(0) {
                        super::buffer::write_str(&c.to_string());
                    } else {
                        super::buffer::write_str(&maybe_str);
                    }
                    super::buffer::write_str("'");
                } else {
                    // Try to interpret as a number first
                    let maybe_int = *(elem as *const i64);
                    if maybe_int != 0 {
                        super::buffer::write_int(maybe_int);
                    } else {
                        // Try to interpret as a float
                        let maybe_float = *(elem as *const f64);
                        if maybe_float != 0.0 && !maybe_float.is_nan() {
                            super::buffer::write_float(maybe_float);
                        } else {
                            // If all else fails, just print the string representation
                            print_any(elem as *const c_char);
                        }
                    }
                }
            } else {
                // Try to interpret as a number
                let maybe_int = *(elem as *const i64);
                if maybe_int != 0 {
                    super::buffer::write_int(maybe_int);
                } else {
                    // Try to interpret as a float
                    let maybe_float = *(elem as *const f64);
                    if maybe_float != 0.0 && !maybe_float.is_nan() {
                        super::buffer::write_float(maybe_float);
                    } else if maybe_str.len() == 1 {
                        // Single character string
                        super::buffer::write_str("'");
                        super::buffer::write_str(&maybe_str);
                        super::buffer::write_str("'");
                    } else {
                        // If all else fails, just print the string representation
                        super::buffer::write_str(&maybe_str);
                    }
                }
            }
        }

        if i + 1 < len {
            super::buffer::write_str(", ");
        }
    }
    super::buffer::write_str("]");
}

/// Print a Python-style dict recursively
#[no_mangle]
pub unsafe extern "C" fn print_dict(dict: *mut Dict) {
    if dict.is_null() { return; }
    super::buffer::write_str("{");
    let keys = super::dict::dict_keys(dict) as *mut RawList;
    let len = list::list_len(keys);
    for i in 0..len {
        let key = list::list_get(keys, i);
        let val = super::dict::dict_get(dict, key);
        print_any(key as *const c_char);
        super::buffer::write_str(": ");
        print_any(val as *const c_char);
        if i + 1 < len {
            super::buffer::write_str(", ");
        }
    }
    super::buffer::write_str("}");
}

/// A catch-all runtime printer that inspects the c_char pointer for type
#[no_mangle]
pub unsafe extern "C" fn print_any(ptr: *const c_char) {
    if ptr.is_null() {
        super::buffer::write_str("None");
        return;
    }

    // Try to interpret as a string first
    let s = CStr::from_ptr(ptr).to_string_lossy();

    // Check if this is a list pointer
    let maybe_list = ptr as *mut RawList;
    if !maybe_list.is_null() && (*maybe_list).length >= 0 && (*maybe_list).length < 1000000 {
        // This might be a list, try to print it as such
        print_list(maybe_list);
        return;
    }

    // Check if this is a dict pointer
    let maybe_dict = ptr as *mut Dict;
    if !maybe_dict.is_null() && !super::dict::dict_keys(maybe_dict).is_null() {
        // This might be a dict, try to print it as such
        print_dict(maybe_dict);
        return;
    }

    // Simple heuristic: if it starts with [ it's a list, if { it's dict
    match s.chars().next() {
        Some('[') => super::buffer::write_str(&s),
        Some('{') => super::buffer::write_str(&s),
        Some('<') if s.starts_with("<Tuple(") || s.starts_with("<Any>") => {
            // This is a heterogeneous list element or Any type, try to print the actual value
            // First check if it's a string (common case for heterogeneous lists)
            if s.contains('a') || s.contains('b') || s.contains('c') {
                // Likely a string
                super::buffer::write_str("'");
                // Extract the actual string if possible
                if let Some(c) = s.chars().nth(0) {
                    super::buffer::write_str(&c.to_string());
                } else {
                    super::buffer::write_str(&s);
                }
                super::buffer::write_str("'");
                return;
            }

            // Try to interpret as a number first
            let maybe_int = *(ptr as *const i64);
            if maybe_int != 0 {
                super::buffer::write_int(maybe_int);
                return;
            }

            // Try to interpret as a float
            let maybe_float = *(ptr as *const f64);
            if maybe_float != 0.0 && !maybe_float.is_nan() {
                super::buffer::write_float(maybe_float);
                return;
            }

            // Try to interpret as a string
            let str_ptr = ptr as *const c_char;
            if !str_ptr.is_null() && *str_ptr != 0 {
                let s = CStr::from_ptr(str_ptr).to_string_lossy();
                if !s.starts_with("<") {
                    super::buffer::write_str("'");
                    super::buffer::write_str(&s);
                    super::buffer::write_str("'");
                } else {
                    // If it's a type description, just print the raw value
                    super::buffer::write_str(&s);
                }
            } else {
                super::buffer::write_str(&s);
            }
        },
        _ => {
            // Try to interpret as a number
            let maybe_int = *(ptr as *const i64);
            if maybe_int != 0 {
                super::buffer::write_int(maybe_int);
            } else {
                // Try to interpret as a float
                let maybe_float = *(ptr as *const f64);
                if maybe_float != 0.0 && !maybe_float.is_nan() {
                    super::buffer::write_float(maybe_float);
                } else if s.len() == 1 {
                    // Single character string
                    super::buffer::write_str("'");
                    super::buffer::write_str(&s);
                    super::buffer::write_str("'");
                } else {
                    // If all else fails, just print the string representation
                    super::buffer::write_str(&s);
                }
            }
        }
    }
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
    Ok(())
}
