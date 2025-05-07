// list.rs - Combined list runtime & LLVM registration

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use libc::{calloc, free, malloc, realloc};
use std::ffi::c_void;
use std::ptr;

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
}

#[no_mangle]
pub extern "C" fn list_new() -> *mut RawList {
    let ptr = unsafe { malloc(std::mem::size_of::<RawList>()) } as *mut RawList;
    if ptr.is_null() { return ptr; }
    unsafe {
        (*ptr).length   = 0;
        (*ptr).capacity = 0;
        (*ptr).data     = ptr::null_mut();
        (*ptr).tags     = ptr::null_mut();
    }
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
        println!("DEBUG: list_append_tagged called with tag: {:?} ({})", tag, tag as u8);

        if list_ptr.is_null() {
            println!("DEBUG: ERROR - list_ptr is null!");
            return;
        }

        let rl = &mut *list_ptr;
        println!("DEBUG: Current list length: {}, capacity: {}", rl.length, rl.capacity);

        // Grow both arrays together
        if rl.length == rl.capacity {
            let new_cap      = if rl.capacity == 0 { 4 } else { rl.capacity * 2 };
            let bytes_ptrs   = new_cap as usize * std::mem::size_of::<*mut c_void>();
            let bytes_tags   = new_cap as usize * std::mem::size_of::<TypeTag>();

            println!("DEBUG: Growing list capacity from {} to {}", rl.capacity, new_cap);

            rl.data = if rl.data.is_null() {
                println!("DEBUG: Allocating new data array");
                malloc(bytes_ptrs)
            } else {
                println!("DEBUG: Reallocating data array");
                realloc(rl.data as *mut _, bytes_ptrs)
            } as *mut *mut c_void;

            rl.tags = if rl.tags.is_null() {
                println!("DEBUG: Allocating new tags array");
                malloc(bytes_tags)
            } else {
                println!("DEBUG: Reallocating tags array");
                realloc(rl.tags as *mut _, bytes_tags)
            } as *mut TypeTag;

            rl.capacity = new_cap;
        }

        if rl.data.is_null() {
            println!("DEBUG: ERROR - data pointer is null after allocation!");
            return;
        }

        if rl.tags.is_null() {
            println!("DEBUG: ERROR - tags pointer is null after allocation!");
            return;
        }

        println!("DEBUG: Storing value at index {}", rl.length);
        *rl.data.add(rl.length as usize) = value;

        println!("DEBUG: Storing tag {:?} ({}) at index {}", tag, tag as u8, rl.length);
        *rl.tags.add(rl.length as usize) = tag;    // store tag in lock‑step

        rl.length += 1;
        println!("DEBUG: List length increased to {}", rl.length);
    }
}

#[no_mangle]
pub extern "C" fn list_get_tag(list_ptr: *mut RawList, index: i64) -> TypeTag {
    unsafe {
        println!("DEBUG: list_get_tag called with index {}", index);
        let rl = &*list_ptr;
        println!("DEBUG: list length: {}, capacity: {}", rl.length, rl.capacity);

        if index < 0 || index >= rl.length {
            println!("DEBUG: Index out of bounds, returning TypeTag::Any");
            TypeTag::Any
        } else {
            if rl.tags.is_null() {
                println!("DEBUG: Tags pointer is null, returning TypeTag::Any");
                TypeTag::Any
            } else {
                let tag = *rl.tags.add(index as usize);
                println!("DEBUG: Retrieved tag: {:?} ({})", tag, tag as u8);
                tag
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn list_get(list_ptr: *mut RawList, index: i64) -> *mut c_void {
    unsafe {
        println!("DEBUG: list_get called with index {}", index);

        if list_ptr.is_null() {
            println!("DEBUG: ERROR - list_ptr is null!");
            return ptr::null_mut();
        }

        let rl = &*list_ptr;
        println!("DEBUG: List length: {}, capacity: {}", rl.length, rl.capacity);

        if index < 0 || index >= rl.length {
            println!("DEBUG: Index out of bounds, returning null");
            ptr::null_mut()
        } else {
            if rl.data.is_null() {
                println!("DEBUG: ERROR - data pointer is null!");
                return ptr::null_mut();
            }

            let value_ptr = *rl.data.add(index as usize);
            println!("DEBUG: Retrieved value pointer: {:?} at index {}", value_ptr, index);
            value_ptr
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
        let rl = &mut *list_ptr;
        if !rl.data.is_null() { free(rl.data as *mut _); }
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
    if let Some(f) = module.get_function("list_append") { engine.add_global_mapping(&f, list_append as usize); }
    if let Some(f) = module.get_function("list_append_tagged") { engine.add_global_mapping(&f, list_append_tagged as usize); }
    if let Some(f) = module.get_function("list_get") { engine.add_global_mapping(&f, list_get as usize); }
    if let Some(f) = module.get_function("list_set") { engine.add_global_mapping(&f, list_set as usize); }
    if let Some(f) = module.get_function("list_concat") { engine.add_global_mapping(&f, list_concat as usize); }
    if let Some(f) = module.get_function("list_repeat") { engine.add_global_mapping(&f, list_repeat as usize); }
    if let Some(f) = module.get_function("list_slice") { engine.add_global_mapping(&f, list_slice as usize); }
    if let Some(f) = module.get_function("list_free") { engine.add_global_mapping(&f, list_free as usize); }
    if let Some(f) = module.get_function("list_len") { engine.add_global_mapping(&f, list_len as usize); }
    Ok(())
}