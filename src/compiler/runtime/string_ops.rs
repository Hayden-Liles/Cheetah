// string_ops.rs - Runtime support for string operations

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Convert an integer to a string (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn int_to_string(value: i64) -> *mut c_char {
    let s = format!("{}", value);
    let c_str = CString::new(s).unwrap();
    c_str.into_raw() // Caller is responsible for freeing this memory
}

/// Convert a float to a string (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn float_to_string(value: f64) -> *mut c_char {
    let s = format!("{}", value);
    let c_str = CString::new(s).unwrap();
    c_str.into_raw()
}

/// Convert a boolean to a string (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn bool_to_string(value: i64) -> *mut c_char {
    let s = if value != 0 { "True" } else { "False" }.to_string();
    let c_str = CString::new(s).unwrap();
    c_str.into_raw()
}

/// Convert a string to an integer (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_to_int(value: *const c_char) -> i64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    s.parse::<i64>().unwrap_or(0)
}

/// Convert a string to a float (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_to_float(value: *const c_char) -> f64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    s.parse::<f64>().unwrap_or(0.0)
}

/// Convert a string to a boolean (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_to_bool(value: *const c_char) -> bool {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    match s.to_lowercase().as_str() {
        "true" | "1" => true,
        _ => false,
    }
}

/// Free memory allocated by the string functions (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
            // Memory is freed when CString is dropped
        }
    }
}
