use std::ffi::c_void;
use std::ptr;
use libc::{malloc, calloc, realloc, free};
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

/// C-compatible raw list struct
#[repr(C)]
pub struct RawList {
    pub length: i64,
    pub capacity: i64,
    pub data: *mut *mut c_void,
}

#[no_mangle]
pub extern "C" fn list_new() -> *mut RawList {
    // Allocate the RawList struct
    let ptr = unsafe { malloc(std::mem::size_of::<RawList>()) } as *mut RawList;
    if ptr.is_null() {
        return ptr;
    }
    unsafe {
        (*ptr).length = 0;
        (*ptr).capacity = 0;
        (*ptr).data = ptr::null_mut();
    }
    ptr
}

#[no_mangle]
pub extern "C" fn list_with_capacity(cap: i64) -> *mut RawList {
    unsafe {
        let rl = list_new();
        if rl.is_null() {
            return rl;
        }
        (*rl).capacity = cap;
        (*rl).data = calloc(cap as usize, std::mem::size_of::<*mut c_void>()) as *mut *mut c_void;
        rl
    }
}

#[no_mangle]
pub extern "C" fn list_append(list_ptr: *mut RawList, value: *mut c_void) {
    unsafe {
        let rl = &mut *list_ptr;
        if rl.length == rl.capacity {
            // grow capacity
            let new_cap = if rl.capacity == 0 { 4 } else { rl.capacity * 2 };
            let size = (new_cap as usize) * std::mem::size_of::<*mut c_void>();
            let new_data = if rl.data.is_null() {
                malloc(size)
            } else {
                realloc(rl.data as *mut _, size)
            } as *mut *mut c_void;
            rl.data = new_data;
            rl.capacity = new_cap;
        }
        // store the element
        *rl.data.add(rl.length as usize) = value;
        rl.length += 1;
    }
}

#[no_mangle]
pub extern "C" fn list_get(list_ptr: *mut RawList, index: i64) -> *mut c_void {
    unsafe {
        let rl = &*list_ptr;
        if index < 0 || index >= rl.length {
            ptr::null_mut()
        } else {
            *rl.data.add(index as usize)
        }
    }
}

#[no_mangle]
pub extern "C" fn list_set(list_ptr: *mut RawList, index: i64, value: *mut c_void) {
    unsafe {
        let rl = &mut *list_ptr;
        if index >= 0 && index < rl.length {
            *rl.data.add(index as usize) = value;
        }
    }
}

#[no_mangle]
pub extern "C" fn list_concat(a: *mut RawList, b: *mut RawList) -> *mut RawList {
    unsafe {
        let ra = &*a;
        let rb = &*b;
        let out = list_with_capacity(ra.length + rb.length);
        for i in 0..ra.length {
            list_append(out, list_get(a, i));
        }
        for i in 0..rb.length {
            list_append(out, list_get(b, i));
        }
        out
    }
}

#[no_mangle]
pub extern "C" fn list_repeat(src: *mut RawList, times: i64) -> *mut RawList {
    unsafe {
        let rs = &*src;
        let out = list_with_capacity(rs.length * times);
        for _ in 0..times {
            for i in 0..rs.length {
                list_append(out, list_get(src, i));
            }
        }
        out
    }
}

#[no_mangle]
pub extern "C" fn list_slice(src: *mut RawList, start: i64, stop: i64, step: i64) -> *mut RawList {
    let out = list_new();
    let mut i = start;
    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        list_append(out, list_get(src, i));
        i += step;
    }
    out
}

#[no_mangle]
pub extern "C" fn list_free(list_ptr: *mut RawList) {
    unsafe {
        if list_ptr.is_null() {
            return;
        }
        let rl = &mut *list_ptr;
        if !rl.data.is_null() {
            free(rl.data as *mut _);
        }
        free(list_ptr as *mut _);
    }
}

#[no_mangle]
pub extern "C" fn list_len(list_ptr: *mut RawList) -> i64 {
    unsafe {
        if list_ptr.is_null() {
            return 0;
        }
        let rl = &*list_ptr;
        rl.length
    }
}

/// Register list runtime functions with the JIT execution engine
pub fn register_list_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>
) -> Result<(), String> {
    // Map list_new function
    if let Some(function) = module.get_function("list_new") {
        engine.add_global_mapping(&function, list_new as usize);
    }

    // Map list_with_capacity function
    if let Some(function) = module.get_function("list_with_capacity") {
        engine.add_global_mapping(&function, list_with_capacity as usize);
    }

    // Map list_append function
    if let Some(function) = module.get_function("list_append") {
        engine.add_global_mapping(&function, list_append as usize);
    }

    // Map list_get function
    if let Some(function) = module.get_function("list_get") {
        engine.add_global_mapping(&function, list_get as usize);
    }

    // Map list_set function
    if let Some(function) = module.get_function("list_set") {
        engine.add_global_mapping(&function, list_set as usize);
    }

    // Map list_concat function
    if let Some(function) = module.get_function("list_concat") {
        engine.add_global_mapping(&function, list_concat as usize);
    }

    // Map list_repeat function
    if let Some(function) = module.get_function("list_repeat") {
        engine.add_global_mapping(&function, list_repeat as usize);
    }

    // Map list_slice function
    if let Some(function) = module.get_function("list_slice") {
        engine.add_global_mapping(&function, list_slice as usize);
    }

    // Map list_free function
    if let Some(function) = module.get_function("list_free") {
        engine.add_global_mapping(&function, list_free as usize);
    }

    // Map list_len function
    if let Some(function) = module.get_function("list_len") {
        engine.add_global_mapping(&function, list_len as usize);
    }

    Ok(())
}
