// exception.rs - Combined exception operations, state management, and runtime

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use inkwell::context::Context;
use inkwell::module::Module;

use inkwell::AddressSpace;

/// Exception object
#[repr(C)]
pub struct Exception {
    typ: *mut c_char,
    message: *mut c_char,
}

// -------- C-compatible runtime functions --------

/// Create a new exception
#[unsafe(no_mangle)]
pub extern "C" fn exception_new(
    typ: *const c_char,
    message: *const c_char
) -> *mut Exception {
    let typ_str = unsafe { CStr::from_ptr(typ) };
    let msg_str = unsafe { CStr::from_ptr(message) };
    let typ_owned = CString::new(typ_str.to_str().unwrap_or("Exception")).unwrap();
    let msg_owned = CString::new(msg_str.to_str().unwrap_or("")).unwrap();
    let exc = Box::new(Exception {
        typ: typ_owned.into_raw(),
        message: msg_owned.into_raw(),
    });
    Box::into_raw(exc)
}

/// Raise an exception (logs to stderr)
#[unsafe(no_mangle)]
pub extern "C" fn exception_raise(exception: *mut Exception) {
    if exception.is_null() { return; }
    let e = unsafe { &*exception };
    let typ = unsafe { CStr::from_ptr(e.typ).to_string_lossy() };
    let msg = unsafe { CStr::from_ptr(e.message).to_string_lossy() };
    eprintln!("Exception raised: {} - {}", typ, msg);
}

/// Check exception type
#[unsafe(no_mangle)]
pub extern "C" fn exception_check(
    exception: *mut Exception,
    typ: *const c_char
) -> bool {
    if exception.is_null() { return false; }
    let e = unsafe { &*exception };
    let exc_typ = unsafe { CStr::from_ptr(e.typ) };
    let chk_typ = unsafe { CStr::from_ptr(typ) };
    exc_typ.to_str().unwrap_or("") == chk_typ.to_str().unwrap_or("")
}

/// Get exception message
#[unsafe(no_mangle)]
pub extern "C" fn exception_get_message(
    exception: *mut Exception
) -> *const c_char {
    if exception.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    unsafe { (*exception).message }
}

/// Get exception type
#[unsafe(no_mangle)]
pub extern "C" fn exception_get_type(
    exception: *mut Exception
) -> *const c_char {
    if exception.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    unsafe { (*exception).typ }
}

/// Free exception object and its strings
#[unsafe(no_mangle)]
pub extern "C" fn exception_free(exception: *mut Exception) {
    if exception.is_null() { return; }
    let e = unsafe { Box::from_raw(exception) };
    unsafe { let _ = CString::from_raw(e.typ); }
    unsafe { let _ = CString::from_raw(e.message); }
}

// -------- Global exception state --------

static mut GLOBAL_EXCEPTION: *mut Exception = ptr::null_mut();

// For AOT compilation, we need to use weak symbols to avoid multiple definitions
// This is because the LLVM module also defines these functions

/// Get current exception
#[cfg(feature = "aot")]
#[no_mangle]
#[link_section = ".text.get_current_exception.weak"]
pub extern "C" fn get_current_exception() -> *mut Exception {
    unsafe { GLOBAL_EXCEPTION }
}

/// Set current exception
#[cfg(feature = "aot")]
#[no_mangle]
#[link_section = ".text.set_current_exception.weak"]
pub extern "C" fn set_current_exception(exc: *mut Exception) {
    unsafe { GLOBAL_EXCEPTION = exc; }
}

/// Clear current exception
#[cfg(feature = "aot")]
#[no_mangle]
#[link_section = ".text.clear_current_exception.weak"]
pub extern "C" fn clear_current_exception() {
    unsafe { GLOBAL_EXCEPTION = ptr::null_mut(); }
}

/// Get current exception (non-AOT version)
#[cfg(not(feature = "aot"))]
#[no_mangle]
pub extern "C" fn get_current_exception() -> *mut Exception {
    unsafe { GLOBAL_EXCEPTION }
}

/// Set current exception (non-AOT version)
#[cfg(not(feature = "aot"))]
#[no_mangle]
pub extern "C" fn set_current_exception(exc: *mut Exception) {
    unsafe { GLOBAL_EXCEPTION = exc; }
}

/// Clear current exception (non-AOT version)
#[cfg(not(feature = "aot"))]
#[no_mangle]
pub extern "C" fn clear_current_exception() {
    unsafe { GLOBAL_EXCEPTION = ptr::null_mut(); }
}

// -------- LLVM module registration --------

/// Register exception operations (new, raise, check, get_message, get_type, free)
pub fn register_exception_functions<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>
) {
    let ptr_t = context.ptr_type(AddressSpace::default());
    // Exception struct type
    let _ = context.struct_type(
        &[ptr_t.into(), ptr_t.into()],
        false
    );
    // exception_new
    module.add_function(
        "exception_new",
        ptr_t.fn_type(&[ptr_t.into(), ptr_t.into()], false),
        None,
    );
    // exception_raise
    module.add_function(
        "exception_raise",
        context.void_type().fn_type(&[ptr_t.into()], false),
        None,
    );
    // exception_check
    module.add_function(
        "exception_check",
        context.bool_type().fn_type(&[ptr_t.into(), ptr_t.into()], false),
        None,
    );
    // exception_get_message
    module.add_function(
        "exception_get_message",
        ptr_t.fn_type(&[ptr_t.into()], false),
        None,
    );
    // exception_get_type
    module.add_function(
        "exception_get_type",
        ptr_t.fn_type(&[ptr_t.into()], false),
        None,
    );
    // exception_free
    module.add_function(
        "exception_free",
        context.void_type().fn_type(&[ptr_t.into()], false),
        None,
    );
}

/// Register exception state functions and global
pub fn register_exception_state<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>
) {
    let ptr_t = context.ptr_type(AddressSpace::default());
    // Global variable
    let global = module.add_global(ptr_t, None, "__current_exception");
    global.set_initializer(&ptr_t.const_null());

    // Always declare the functions as external for AOT compilation
    // This avoids multiple definitions during linking
    if module.get_function("get_current_exception").is_none() {
        module.add_function(
            "get_current_exception",
            ptr_t.fn_type(&[], false),
            Some(inkwell::module::Linkage::External),
        );
    }

    if module.get_function("set_current_exception").is_none() {
        module.add_function(
            "set_current_exception",
            context.void_type().fn_type(&[ptr_t.into()], false),
            Some(inkwell::module::Linkage::External),
        );
    }

    if module.get_function("clear_current_exception").is_none() {
        module.add_function(
            "clear_current_exception",
            context.void_type().fn_type(&[], false),
            Some(inkwell::module::Linkage::External),
        );
    }
}
