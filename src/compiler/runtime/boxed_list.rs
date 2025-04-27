// boxed_list.rs - List implementation using BoxedAny values

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use libc::{calloc, free, malloc, realloc};
use std::ffi::c_void;
use std::ptr;

use super::boxed_any::{BoxedAny, type_tags};

/// C-compatible list struct using BoxedAny values
#[repr(C)]
pub struct BoxedList {
    pub length: i64,
    pub capacity: i64,
    pub data: *mut *mut BoxedAny,
}

#[no_mangle]
pub extern "C" fn boxed_list_new() -> *mut BoxedList {
    let ptr = unsafe { malloc(std::mem::size_of::<BoxedList>()) as *mut BoxedList };
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
pub extern "C" fn boxed_list_with_capacity(cap: i64) -> *mut BoxedList {
    unsafe {
        let list = boxed_list_new();
        if list.is_null() { return list; }
        (*list).capacity = cap;
        (*list).data = calloc(cap as usize, std::mem::size_of::<*mut BoxedAny>()) as *mut *mut BoxedAny;
        list
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_append(list_ptr: *mut BoxedList, value: *mut BoxedAny) {
    unsafe {
        let list = &mut *list_ptr;
        if list.length == list.capacity {
            let new_cap = if list.capacity == 0 { 4 } else { list.capacity * 2 };
            let size = (new_cap as usize) * std::mem::size_of::<*mut BoxedAny>();
            let new_data = if list.data.is_null() {
                malloc(size)
            } else {
                realloc(list.data as *mut _, size)
            } as *mut *mut BoxedAny;
            list.data = new_data;
            list.capacity = new_cap;
        }
        *list.data.add(list.length as usize) = value;
        list.length += 1;
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_get(list_ptr: *mut BoxedList, index: i64) -> *mut BoxedAny {
    unsafe {
        let list = &*list_ptr;
        if index < 0 || index >= list.length {
            ptr::null_mut()
        } else {
            *list.data.add(index as usize)
        }
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_set(list_ptr: *mut BoxedList, index: i64, value: *mut BoxedAny) {
    unsafe {
        let list = &mut *list_ptr;
        if index >= 0 && index < list.length {
            // Free the old value if it exists
            let old_value = *list.data.add(index as usize);
            if !old_value.is_null() {
                super::boxed_any::boxed_any_free(old_value);
            }

            // Set the new value
            *list.data.add(index as usize) = value;
        }
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_concat(a: *mut BoxedList, b: *mut BoxedList) -> *mut BoxedList {
    unsafe {
        let list_a = &*a;
        let list_b = &*b;
        let out = boxed_list_with_capacity(list_a.length + list_b.length);

        // Clone and append values from list A
        for i in 0..list_a.length {
            let value = boxed_list_get(a, i);
            if !value.is_null() {
                let cloned = super::boxed_any::boxed_any_clone(value);
                boxed_list_append(out, cloned);
            } else {
                boxed_list_append(out, super::boxed_any::boxed_any_none());
            }
        }

        // Clone and append values from list B
        for i in 0..list_b.length {
            let value = boxed_list_get(b, i);
            if !value.is_null() {
                let cloned = super::boxed_any::boxed_any_clone(value);
                boxed_list_append(out, cloned);
            } else {
                boxed_list_append(out, super::boxed_any::boxed_any_none());
            }
        }

        out
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_repeat(src: *mut BoxedList, times: i64) -> *mut BoxedList {
    unsafe {
        let list_src = &*src;
        let out = boxed_list_with_capacity(list_src.length * times);

        for _ in 0..times {
            for i in 0..list_src.length {
                let value = boxed_list_get(src, i);
                if !value.is_null() {
                    let cloned = super::boxed_any::boxed_any_clone(value);
                    boxed_list_append(out, cloned);
                } else {
                    boxed_list_append(out, super::boxed_any::boxed_any_none());
                }
            }
        }

        out
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_slice(src: *mut BoxedList, start: i64, stop: i64, step: i64) -> *mut BoxedList {
    let out = boxed_list_new();
    let mut i = start;

    while (step > 0 && i < stop) || (step < 0 && i > stop) {
        let value = boxed_list_get(src, i);
        if !value.is_null() {
            let cloned = super::boxed_any::boxed_any_clone(value);
            boxed_list_append(out, cloned);
        } else {
            boxed_list_append(out, super::boxed_any::boxed_any_none());
        }
        i += step;
    }

    out
}

#[no_mangle]
pub extern "C" fn boxed_list_free(list_ptr: *mut BoxedList) {
    unsafe {
        if list_ptr.is_null() { return; }

        let list = &mut *list_ptr;

        // Free all the BoxedAny values in the list
        if !list.data.is_null() {
            for i in 0..list.length {
                let value = *list.data.add(i as usize);
                if !value.is_null() {
                    super::boxed_any::boxed_any_free(value);
                }
            }

            // Free the data array
            free(list.data as *mut _);
        }

        // Free the list itself
        free(list_ptr as *mut _);
    }
}

#[no_mangle]
pub extern "C" fn boxed_list_len(list_ptr: *mut BoxedList) -> i64 {
    unsafe {
        if list_ptr.is_null() { 0 }
        else { (&*list_ptr).length }
    }
}

/// Create a BoxedAny from a BoxedList
#[no_mangle]
pub extern "C" fn boxed_any_from_list(list_ptr: *mut BoxedList) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::LIST;
        (*boxed).data.ptr_val = list_ptr as *mut c_void;
    }
    boxed
}

/// Get the BoxedList from a BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_list(value: *const BoxedAny) -> *mut BoxedList {
    if value.is_null() {
        return boxed_list_new();
    }

    unsafe {
        if (*value).tag == type_tags::LIST {
            (*value).data.ptr_val as *mut BoxedList
        } else {
            // If it's not a list, create a new empty list
            boxed_list_new()
        }
    }
}

/// Register BoxedList functions in the LLVM module
pub fn register_boxed_list_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let boxed_any_ptr_type = context.ptr_type(AddressSpace::default());
    let boxed_list_ptr_type = context.ptr_type(AddressSpace::default());

    // List creation and management functions
    module.add_function(
        "boxed_list_new",
        boxed_list_ptr_type.fn_type(&[], false),
        None,
    );

    module.add_function(
        "boxed_list_with_capacity",
        boxed_list_ptr_type.fn_type(&[i64_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_list_append",
        void_type.fn_type(&[
            boxed_list_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_get",
        boxed_any_ptr_type.fn_type(&[
            boxed_list_ptr_type.into(),
            i64_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_set",
        void_type.fn_type(&[
            boxed_list_ptr_type.into(),
            i64_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_concat",
        boxed_list_ptr_type.fn_type(&[
            boxed_list_ptr_type.into(),
            boxed_list_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_repeat",
        boxed_list_ptr_type.fn_type(&[
            boxed_list_ptr_type.into(),
            i64_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_slice",
        boxed_list_ptr_type.fn_type(&[
            boxed_list_ptr_type.into(),
            i64_type.into(), i64_type.into(), i64_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_list_free",
        void_type.fn_type(&[boxed_list_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_list_len",
        i64_type.fn_type(&[boxed_list_ptr_type.into()], false),
        None,
    );

    // BoxedAny conversion functions
    module.add_function(
        "boxed_any_from_list",
        boxed_any_ptr_type.fn_type(&[boxed_list_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_any_as_list",
        boxed_list_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false),
        None,
    );
}

/// Register BoxedList runtime mappings for the JIT engine
pub fn register_boxed_list_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("boxed_list_new") {
        engine.add_global_mapping(&f, boxed_list_new as usize);
    }

    if let Some(f) = module.get_function("boxed_list_with_capacity") {
        engine.add_global_mapping(&f, boxed_list_with_capacity as usize);
    }

    if let Some(f) = module.get_function("boxed_list_append") {
        engine.add_global_mapping(&f, boxed_list_append as usize);
    }

    if let Some(f) = module.get_function("boxed_list_get") {
        engine.add_global_mapping(&f, boxed_list_get as usize);
    }

    if let Some(f) = module.get_function("boxed_list_set") {
        engine.add_global_mapping(&f, boxed_list_set as usize);
    }

    if let Some(f) = module.get_function("boxed_list_concat") {
        engine.add_global_mapping(&f, boxed_list_concat as usize);
    }

    if let Some(f) = module.get_function("boxed_list_repeat") {
        engine.add_global_mapping(&f, boxed_list_repeat as usize);
    }

    if let Some(f) = module.get_function("boxed_list_slice") {
        engine.add_global_mapping(&f, boxed_list_slice as usize);
    }

    if let Some(f) = module.get_function("boxed_list_free") {
        engine.add_global_mapping(&f, boxed_list_free as usize);
    }

    if let Some(f) = module.get_function("boxed_list_len") {
        engine.add_global_mapping(&f, boxed_list_len as usize);
    }

    if let Some(f) = module.get_function("boxed_any_from_list") {
        engine.add_global_mapping(&f, boxed_any_from_list as usize);
    }

    if let Some(f) = module.get_function("boxed_any_as_list") {
        engine.add_global_mapping(&f, boxed_any_as_list as usize);
    }

    Ok(())
}

pub fn get_boxed_list_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}
