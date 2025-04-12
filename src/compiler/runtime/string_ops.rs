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

/// Get a character from a string at a given index (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_get_char(value: *const c_char, index: i64) -> i64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");

    if index < 0 || index >= s.len() as i64 {
        return 0; // Return null character for out-of-bounds
    }

    // Get the character at the given index
    let index = index as usize;
    s.chars().nth(index).map(|c| c as i64).unwrap_or(0)
}

/// Get a slice of a string (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_slice(value: *const c_char, start: i64, stop: i64, step: i64) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");

    // Handle empty string case
    if s.is_empty() {
        let empty = CString::new("").unwrap();
        return empty.into_raw();
    }

    let len = s.len() as i64;

    // Normalize indices
    let start = if start < 0 { 0 } else if start > len { len } else { start };
    let stop = if stop < 0 { 0 } else if stop > len { len } else { stop };

    // Handle step = 0 case (invalid in Python, but we'll just return empty string)
    if step == 0 {
        let empty = CString::new("").unwrap();
        return empty.into_raw();
    }

    // Create the sliced string
    let mut result = String::new();

    if step > 0 {
        let mut i = start;
        while i < stop {
            if let Some(c) = s.chars().nth(i as usize) {
                result.push(c);
            }
            i += step;
        }
    } else { // step < 0
        let mut i = start;
        while i > stop {
            if let Some(c) = s.chars().nth(i as usize) {
                result.push(c);
            }
            i += step; // This will decrease i since step is negative
        }
    }

    // Convert to C string and return
    let c_result = CString::new(result).unwrap();
    c_result.into_raw() // Caller is responsible for freeing this memory
}

/// Get the length of a string (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn string_len(value: *const c_char) -> i64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    s.len() as i64
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
