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

        if elem.is_null() {
            super::buffer::write_str("None");
        } else {
            // Try to determine the type and print accordingly
            // This is a simplified approach - in a real implementation you'd have proper type tags
            let elem_ptr = elem as *const u8;
            let first_byte = if !elem_ptr.is_null() { *elem_ptr } else { 0 };

            // Check for list or dict markers
            if first_byte == b'[' {
                // Looks like a list
                print_list(elem as *mut RawList);
            } else if first_byte == b'{' {
                // Looks like a dict
                print_dict(elem as *mut Dict);
            } else {
                // Try to interpret as a string first
                let c_str = elem as *const c_char;
                if !c_str.is_null() && CStr::from_ptr(c_str).to_str().is_ok() {
                    // It's a valid string
                    print_string(c_str);
                } else {
                    // Fall back to print_any
                    print_any(c_str);
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

        // Print the key with appropriate type handling
        if key.is_null() {
            super::buffer::write_str("None");
        } else {
            // Check if it's a list or dict (unlikely for keys, but handle it)
            let first_byte = *(key as *const u8);
            if first_byte == b'[' {
                print_list(key as *mut RawList);
            } else if first_byte == b'{' {
                print_dict(key as *mut Dict);
            } else {
                // Try to determine the type by examining the value
                let as_i64_ptr = key as *const i64;
                let as_f64_ptr = key as *const f64;
                let as_bool_ptr = key as *const bool;

                // Check if it looks like a string
                if first_byte >= 32 && first_byte <= 126 {
                    // Likely a string
                    print_string(key as *const c_char);
                } else if *as_bool_ptr == true || *as_bool_ptr == false {
                    // Likely a boolean
                    print_bool(*as_bool_ptr);
                } else if *as_f64_ptr > -1e10 && *as_f64_ptr < 1e10 {
                    // Likely a float
                    print_float(*as_f64_ptr);
                } else {
                    // Default to integer
                    print_int(*as_i64_ptr);
                }
            }
        }

        super::buffer::write_str(": ");

        // Print the value with appropriate type handling
        if val.is_null() {
            super::buffer::write_str("None");
        } else {
            // Check if it's a list or dict
            let first_byte = *(val as *const u8);
            if first_byte == b'[' {
                print_list(val as *mut RawList);
            } else if first_byte == b'{' {
                print_dict(val as *mut Dict);
            } else {
                // Try to determine the type by examining the value
                let as_i64_ptr = val as *const i64;
                let as_f64_ptr = val as *const f64;
                let as_bool_ptr = val as *const bool;

                // Check if it looks like a string
                if first_byte >= 32 && first_byte <= 126 {
                    // Likely a string
                    print_string(val as *const c_char);
                } else if *as_bool_ptr == true || *as_bool_ptr == false {
                    // Likely a boolean
                    print_bool(*as_bool_ptr);
                } else if *as_f64_ptr > -1e10 && *as_f64_ptr < 1e10 {
                    // Likely a float
                    print_float(*as_f64_ptr);
                } else {
                    // Default to integer
                    print_int(*as_i64_ptr);
                }
            }
        }

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
    let s = CStr::from_ptr(ptr).to_string_lossy();
    // simple heuristic: if it starts with [ it's a list, if { it's dict else string
    match s.chars().next() {
        Some('[') => super::buffer::write_str(&s),
        Some('{') => super::buffer::write_str(&s),
        _ => super::buffer::write_str(&s),
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
