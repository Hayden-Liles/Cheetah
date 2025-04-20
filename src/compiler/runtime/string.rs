// string.rs - Combined string runtime & LLVM registration

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

#[no_mangle]
pub extern "C" fn int_to_string(value: i64) -> *mut c_char {
    let s = format!("{}", value);
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn float_to_string(value: f64) -> *mut c_char {
    let s = format!("{}", value);
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn bool_to_string(value: i64) -> *mut c_char {
    let s = if value != 0 { "True" } else { "False" }.to_string();
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn string_to_int(value: *const c_char) -> i64 {
    let s = unsafe { CStr::from_ptr(value).to_str().unwrap_or("") };
    s.parse().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn string_to_float(value: *const c_char) -> f64 {
    let s = unsafe { CStr::from_ptr(value).to_str().unwrap_or("") };
    s.parse().unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn string_to_bool(value: *const c_char) -> bool {
    match unsafe { CStr::from_ptr(value).to_str().unwrap_or("") }.to_lowercase().as_str() {
        "true" | "1" => true,
        _ => false,
    }
}

#[no_mangle]
pub extern "C" fn string_get_char(value: *const c_char, index: i64) -> i64 {
    let s = unsafe { CStr::from_ptr(value).to_str().unwrap_or("") };
    if index < 0 || index >= s.len() as i64 { return 0 }
    s.chars().nth(index as usize).map(|c| c as i64).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn char_to_string(value: i64) -> *mut c_char {
    let c = std::char::from_u32(value as u32).unwrap_or('\0');
    CString::new(c.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn string_slice(
    value: *const c_char,
    start: i64,
    stop: i64,
    step: i64,
) -> *mut c_char {
    let s = unsafe { CStr::from_ptr(value).to_str().unwrap_or("") };
    if s.is_empty() || step == 0 {
        return CString::new("").unwrap().into_raw();
    }
    let len = s.len() as i64;
    let start = start.clamp(0, len);
    let stop = stop.clamp(0, len);
    let mut res = String::new();
    if step > 0 {
        let mut i = start;
        while i < stop {
            if let Some(c) = s.chars().nth(i as usize) { res.push(c); }
            i += step;
        }
    } else {
        let mut i = start;
        while i > stop {
            if let Some(c) = s.chars().nth(i as usize) { res.push(c); }
            i += step;
        }
    }
    CString::new(res).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn string_len(value: *const c_char) -> i64 {
    unsafe { CStr::from_ptr(value).to_str().unwrap_or("").len() as i64 }
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { let _ = CString::from_raw(ptr); }
    }
}

#[no_mangle]
pub extern "C" fn string_concat(s1: *const c_char, s2: *const c_char) -> *mut c_char {
    let s1 = unsafe { CStr::from_ptr(s1).to_str().unwrap_or("") };
    let s2 = unsafe { CStr::from_ptr(s2).to_str().unwrap_or("") };
    CString::new(format!("{}{}", s1, s2)).unwrap().into_raw()
}

/// Register string functions in the LLVM module
pub fn register_string_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    module.add_function(
        "string_get_char",
        context.i64_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "char_to_string",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into()], false),
        None,
    );
    module.add_function(
        "string_slice",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(), context.i64_type().into(), context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "string_len",
        context.i64_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "string_concat",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "free_string",
        context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
}
