// list.rs - Combined list runtime & LLVM registration

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use libc::{calloc, free, malloc, realloc, c_char};
use std::ffi::c_void;
use std::ptr;

use crate::compiler::runtime::string::free_string;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeTag {
    Any   = 0,
    None_ = 1,
    Bool  = 2,
    Int   = 3,
    Float = 4,
    String = 5,
    List  = 6,
    Tuple = 7,
}

/// C-compatible raw list struct
#[repr(C)]
pub struct RawList {
    pub length:   i64,
    pub capacity: i64,
    pub data:     *mut *mut c_void,
    pub tags:     *mut TypeTag,
    pub bulk_storage: *mut c_void, // For tracking bulk allocations
}

#[no_mangle]
pub extern "C" fn list_new() -> *mut RawList {
    let ptr = unsafe { malloc(std::mem::size_of::<RawList>()) } as *mut RawList;
    if ptr.is_null() { return ptr; }
    unsafe {
        (*ptr).length      = 0;
        (*ptr).capacity    = 0;
        (*ptr).data        = ptr::null_mut();
        (*ptr).tags        = ptr::null_mut();
        (*ptr).bulk_storage = ptr::null_mut();
    }
    // Explicitly avoid printing the pointer address
    ptr
}

#[no_mangle]
pub extern "C" fn list_with_capacity(cap: i64) -> *mut RawList {
    unsafe {
        let rl = list_new();
        if rl.is_null() { return rl; }

        (*rl).capacity = cap;
        (*rl).data = calloc(cap as usize,
                            std::mem::size_of::<*mut c_void>())
                     as *mut *mut c_void;

        (*rl).tags = calloc(cap as usize,
                            std::mem::size_of::<TypeTag>())
                     as *mut TypeTag;
        rl
    }
}

/// Create a list of consecutive integers from start (inclusive) to end (exclusive)
/// This is a specialized function for efficiently creating range lists
/// Uses a single bulk allocation for all integers to improve memory efficiency
#[no_mangle]
pub extern "C" fn list_from_range(start: i64, end: i64) -> *mut RawList {
    unsafe {
        // Calculate size
        let size = if end > start { end - start } else { 0 };

        // Create list with exact capacity
        let rl = list_with_capacity(size);
        if rl.is_null() { return rl; }

        // Allocate a single block for all integers
        let bulk_size = size as usize * std::mem::size_of::<i64>();
        let bulk_data = malloc(bulk_size) as *mut i64;
        if bulk_data.is_null() {
            // If bulk allocation fails, fall back to individual allocations
            for i in 0..size {
                let value = start + i;
                let int_ptr = malloc(std::mem::size_of::<i64>()) as *mut i64;
                *int_ptr = value;

                // Store pointer and tag
                *(*rl).data.add(i as usize) = int_ptr as *mut c_void;
                *(*rl).tags.add(i as usize) = TypeTag::Int;
            }
        } else {
            // Store the bulk allocation in the list for later cleanup
            (*rl).bulk_storage = bulk_data as *mut c_void;

            // Fill the bulk data and set up pointers
            for i in 0..size {
                let value = start + i;
                *bulk_data.add(i as usize) = value;

                // Store pointer to the element in the bulk array
                *(*rl).data.add(i as usize) = bulk_data.add(i as usize) as *mut c_void;
                *(*rl).tags.add(i as usize) = TypeTag::Int;
            }

            // Removed debug print
        }

        // Set final length
        (*rl).length = size;

        rl
    }
}

#[no_mangle]
pub extern "C" fn list_append(list_ptr: *mut RawList, value: *mut c_void) {
    list_append_tagged(list_ptr, value, TypeTag::Any);
}

#[no_mangle]
pub extern "C" fn list_append_tagged(list_ptr: *mut RawList,
                                     value: *mut c_void,
                                     tag:   TypeTag)
{
    unsafe {
        let rl = &mut *list_ptr;

        // Grow both arrays together
        if rl.length == rl.capacity {
            let new_cap      = if rl.capacity == 0 { 4 } else { rl.capacity * 2 };
            let bytes_ptrs   = new_cap as usize * std::mem::size_of::<*mut c_void>();
            let bytes_tags   = new_cap as usize * std::mem::size_of::<TypeTag>();

            rl.data = if rl.data.is_null() {
                malloc(bytes_ptrs)
            } else {
                realloc(rl.data as *mut _, bytes_ptrs)
            } as *mut *mut c_void;

            rl.tags = if rl.tags.is_null() {
                malloc(bytes_tags)
            } else {
                realloc(rl.tags as *mut _, bytes_tags)
            } as *mut TypeTag;

            rl.capacity = new_cap;
        }

        *rl.data.add(rl.length as usize) = value;
        *rl.tags.add(rl.length as usize) = tag;    // store tag in lock‑step
        rl.length += 1;
    }
}

#[no_mangle]
pub extern "C" fn list_get_tag(list_ptr: *mut RawList, index: i64) -> TypeTag {
    unsafe {
        let rl = &*list_ptr;
        if index < 0 || index >= rl.length {
            TypeTag::Any
        } else {
            *rl.tags.add(index as usize)
        }
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
        for i in 0..ra.length { list_append(out, list_get(a, i)); }
        for i in 0..rb.length { list_append(out, list_get(b, i)); }
        out
    }
}

#[no_mangle]
pub extern "C" fn list_repeat(src: *mut RawList, times: i64) -> *mut RawList {
    unsafe {
        let rs = &*src;
        let out = list_with_capacity(rs.length * times);
        for _ in 0..times {
            for i in 0..rs.length { list_append(out, list_get(src, i)); }
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
        if list_ptr.is_null() { return; }

        // Removed debug print

        let rl = &mut *list_ptr;

        // Check if we have a bulk storage allocation
        if !rl.bulk_storage.is_null() {
            // Removed debug print
            free(rl.bulk_storage);
            // When using bulk storage, individual elements don't need to be freed
            // as they're part of the bulk allocation
        } else {
            // Free each element based on its tag type
            if !rl.data.is_null() && !rl.tags.is_null() {
                for i in 0..rl.length {
                    let elem_ptr = *rl.data.add(i as usize);
                    let tag = *rl.tags.add(i as usize);

                    // Only free elements that should be owned by this list
                    // String, List, and Tuple types need to be freed
                    match tag {
                        TypeTag::String => {
                            if !elem_ptr.is_null() {
                                // Removed debug print
                                free_string(elem_ptr as *mut c_char);
                            }
                        },
                        TypeTag::List => {
                            if !elem_ptr.is_null() {
                                // Removed debug print
                                list_free(elem_ptr as *mut RawList);
                            }
                        },
                        TypeTag::Tuple => {
                            // Free tuple memory if it was dynamically allocated
                            if !elem_ptr.is_null() {
                                // Removed debug print
                                free(elem_ptr);
                            }
                        },
                        _ => {
                            // Other types still need their memory freed
                            if !elem_ptr.is_null() {
                                // Removed debug print
                                free(elem_ptr);
                            }
                        }
                    }
                }
            }
        }

        // Free the data and tags arrays
        if !rl.data.is_null() {
            free(rl.data as *mut _);
        }
        if !rl.tags.is_null() {
            free(rl.tags as *mut _);
        }

        // Finally free the list structure itself
        free(list_ptr as *mut _);
    }
}

#[no_mangle]
pub extern "C" fn list_len(list_ptr: *mut RawList) -> i64 {
    unsafe {
        if list_ptr.is_null() { 0 }
        else { (&*list_ptr).length }
    }
}

/// Register list operation functions in the LLVM module
pub fn register_list_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let _list_struct_type = context.struct_type(
        &[
            context.i64_type().into(),          // length
            context.i64_type().into(),          // capacity
            context.ptr_type(AddressSpace::default()).into(), // data **
            context.ptr_type(AddressSpace::default()).into(), // tags **
            context.ptr_type(AddressSpace::default()).into(), // bulk_storage *
        ],
        false);

    module.add_function(
        "list_new",
        context.ptr_type(AddressSpace::default()).fn_type(&[], false),
        None,
    );
    module.add_function(
        "list_with_capacity",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into()], false),
        None,
    );
    module.add_function(
        "list_from_range",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.i64_type().into(),
            context.i64_type().into()
        ], false),
        None,
    );
    module.add_function(
        "list_append",
        context.void_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "list_append_tagged",
        context.void_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i8_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "list_get",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "list_get_tag",
        context.i8_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "list_set",
        context.void_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "list_concat",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "list_repeat",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "list_slice",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(), context.i64_type().into(), context.i64_type().into(),
        ], false),
        None,
    );
    module.add_function(
        "list_free",
        context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "list_len",
        context.i64_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
}

pub fn get_list_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    // If we've already created it, just return the handle
    if let Some(st) = context.get_struct_type("RawList") {
        return st;
    }

    // Otherwise create an *opaque* named struct and set its body once
    let st = context.opaque_struct_type("RawList");
    st.set_body(
        &[
            context.i64_type().into(),                    // length
            context.i64_type().into(),                    // capacity
            context.ptr_type(AddressSpace::default()).into(), // data **
            context.ptr_type(AddressSpace::default()).into(), // tags **
            context.ptr_type(AddressSpace::default()).into(), // bulk_storage *
        ],
        false,
    );
    st
}

pub fn get_list_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context.ptr_type(AddressSpace::default()).as_basic_type_enum()
}

/// Register list runtime mappings for the JIT engine
pub fn register_list_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("list_new") { engine.add_global_mapping(&f, list_new as usize); }
    if let Some(f) = module.get_function("list_with_capacity") { engine.add_global_mapping(&f, list_with_capacity as usize); }
    if let Some(f) = module.get_function("list_from_range") { engine.add_global_mapping(&f, list_from_range as usize); }
    if let Some(f) = module.get_function("list_append") { engine.add_global_mapping(&f, list_append as usize); }
    if let Some(f) = module.get_function("list_append_tagged") { engine.add_global_mapping(&f, list_append_tagged as usize); }
    if let Some(f) = module.get_function("list_get") { engine.add_global_mapping(&f, list_get as usize); }
    if let Some(f) = module.get_function("list_get_tag") { engine.add_global_mapping(&f, list_get_tag as usize); }
    if let Some(f) = module.get_function("list_set") { engine.add_global_mapping(&f, list_set as usize); }
    if let Some(f) = module.get_function("list_concat") { engine.add_global_mapping(&f, list_concat as usize); }
    if let Some(f) = module.get_function("list_repeat") { engine.add_global_mapping(&f, list_repeat as usize); }
    if let Some(f) = module.get_function("list_slice") { engine.add_global_mapping(&f, list_slice as usize); }
    if let Some(f) = module.get_function("list_free") { engine.add_global_mapping(&f, list_free as usize); }
    if let Some(f) = module.get_function("list_len") { engine.add_global_mapping(&f, list_len as usize); }
    Ok(())
}