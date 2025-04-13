// exception_runtime.rs - C runtime functions for exception handling

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Exception struct definition
#[repr(C)]
pub struct Exception {
    typ: *mut c_char,   // Type name (e.g., "ValueError", "TypeError")
    message: *mut c_char, // Error message
}

/// Create a new exception (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn exception_new(typ: *const c_char, message: *const c_char) -> *mut Exception {
    let typ_str = unsafe { CStr::from_ptr(typ) };
    let message_str = unsafe { CStr::from_ptr(message) };
    
    // Clone the strings to ensure they remain valid
    let typ_owned = CString::new(typ_str.to_str().unwrap_or("Exception")).unwrap();
    let message_owned = CString::new(message_str.to_str().unwrap_or("")).unwrap();
    
    // Allocate memory for the exception
    let exception = Box::new(Exception {
        typ: typ_owned.into_raw(),
        message: message_owned.into_raw(),
    });
    
    // Convert to raw pointer and return
    Box::into_raw(exception)
}

/// Raise an exception (C-compatible wrapper)
/// In a real implementation, this would set a global exception state
#[unsafe(no_mangle)]
pub extern "C" fn exception_raise(exception: *mut Exception) {
    // In a real implementation, this would set a global exception state
    // For now, we'll just print the exception
    unsafe {
        if !exception.is_null() {
            let typ = CStr::from_ptr((*exception).typ);
            let message = CStr::from_ptr((*exception).message);
            
            eprintln!(
                "Exception raised: {} - {}",
                typ.to_str().unwrap_or("Unknown"),
                message.to_str().unwrap_or("")
            );
        }
    }
}

/// Check if an exception is of a given type (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn exception_check(exception: *mut Exception, typ: *const c_char) -> bool {
    unsafe {
        if exception.is_null() {
            return false;
        }
        
        let exception_typ = CStr::from_ptr((*exception).typ);
        let check_typ = CStr::from_ptr(typ);
        
        exception_typ.to_str().unwrap_or("") == check_typ.to_str().unwrap_or("")
    }
}

/// Get the message from an exception (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn exception_get_message(exception: *mut Exception) -> *const c_char {
    unsafe {
        if exception.is_null() {
            let empty = CString::new("").unwrap();
            return empty.into_raw();
        }
        
        (*exception).message
    }
}

/// Get the type from an exception (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn exception_get_type(exception: *mut Exception) -> *const c_char {
    unsafe {
        if exception.is_null() {
            let empty = CString::new("").unwrap();
            return empty.into_raw();
        }
        
        (*exception).typ
    }
}

/// Free an exception's memory (C-compatible wrapper)
#[unsafe(no_mangle)]
pub extern "C" fn exception_free(exception: *mut Exception) {
    unsafe {
        if !exception.is_null() {
            // Free the strings
            let _ = CString::from_raw((*exception).typ);
            let _ = CString::from_raw((*exception).message);
            
            // Free the exception
            let _ = Box::from_raw(exception);
        }
    }
}
