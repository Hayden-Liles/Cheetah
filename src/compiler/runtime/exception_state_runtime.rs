// exception_state_runtime.rs - C runtime functions for exception state management

use crate::compiler::runtime::exception_runtime::Exception;

// Global variable to store the current exception
static mut CURRENT_EXCEPTION: *mut Exception = std::ptr::null_mut();

/// Get the current exception
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn get_current_exception() -> *mut Exception {
    unsafe { CURRENT_EXCEPTION }
}

/// Set the current exception
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn set_current_exception(exception: *mut Exception) {
    unsafe {
        // Free the previous exception if it exists
        if !CURRENT_EXCEPTION.is_null() {
            // In a real implementation, we would free the previous exception
            // For now, we'll just leak it to avoid double-free issues
        }

        CURRENT_EXCEPTION = exception;
    }
}

/// Clear the current exception
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn clear_current_exception() {
    unsafe {
        // Free the current exception if it exists
        if !CURRENT_EXCEPTION.is_null() {
            // In a real implementation, we would free the exception
            // For now, we'll just set it to null to avoid double-free issues
            CURRENT_EXCEPTION = std::ptr::null_mut();
        }
    }
}
