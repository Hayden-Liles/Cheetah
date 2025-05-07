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
        // More extensive null checks
        if list_ptr.is_null() {
            eprintln!("[ERROR] list_append_tagged: list_ptr is null");
            return;
        }
        
        let rl = &mut *list_ptr;
        eprintln!("[DEBUG] Appending value {:p} with tag {:?} to list at {:p}", 
                  value, tag, list_ptr);
        eprintln!("[DEBUG] Current list length: {}, capacity: {}", rl.length, rl.capacity);

        // Grow both arrays together
        if rl.length == rl.capacity {
            let new_cap      = if rl.capacity == 0 { 4 } else { rl.capacity * 2 };
            let bytes_ptrs   = new_cap as usize * std::mem::size_of::<*mut c_void>();
            let bytes_tags   = new_cap as usize * std::mem::size_of::<TypeTag>();

            eprintln!("[DEBUG] Growing list capacity from {} to {}", rl.capacity, new_cap);
            
            // Store old pointers for verification
            let old_data = rl.data;
            let old_tags = rl.tags;

            rl.data = if rl.data.is_null() {
                eprintln!("[DEBUG] Allocating new data array");
                malloc(bytes_ptrs)
            } else {
                eprintln!("[DEBUG] Reallocating data array");
                realloc(rl.data as *mut _, bytes_ptrs)
            } as *mut *mut c_void;

            rl.tags = if rl.tags.is_null() {
                eprintln!("[DEBUG] Allocating new tags array");
                malloc(bytes_tags)
            } else {
                eprintln!("[DEBUG] Reallocating tags array");
                realloc(rl.tags as *mut _, bytes_tags)
            } as *mut TypeTag;

            // Verify new allocations
            if rl.data.is_null() || rl.tags.is_null() {
                eprintln!("[ERROR] Memory allocation failed during list growth");
                // Restore old pointers to avoid memory corruption
                if rl.data.is_null() { rl.data = old_data; }
                if rl.tags.is_null() { rl.tags = old_tags; }
                return;
            }

            rl.capacity = new_cap;
        }

        if rl.data.is_null() || rl.tags.is_null() {
            eprintln!("[ERROR] Data or tags array is null");
            return;
        }

        // Add the new element and tag
        *rl.data.add(rl.length as usize) = value;
        *rl.tags.add(rl.length as usize) = tag;
        
        // Debug output for the new element
        eprintln!("[DEBUG] Added element {} at index {}: value={:p}, tag={:?}", 
                  rl.length, rl.length, value, tag);
        
        rl.length += 1;
    }
}

#[no_mangle]
pub extern "C" fn list_debug_print(list_ptr: *mut RawList) {
    unsafe {
        if list_ptr.is_null() {
            eprintln!("[DEBUG] List pointer is null");
            return;
        }
        
        let rl = &*list_ptr;
        eprintln!("[DEBUG] List @ {:p}:", list_ptr);
        eprintln!("  length: {}", rl.length);
        eprintln!("  capacity: {}", rl.capacity);
        eprintln!("  data ptr: {:p}", rl.data);
        eprintln!("  tags ptr: {:p}", rl.tags);
        
        if rl.length > 10000 {
            eprintln!("  [WARN] Unreasonable length value - likely corrupted");
            return;
        }
        
        if !rl.tags.is_null() && !rl.data.is_null() && rl.length > 0 {
            eprintln!("  Elements:");
            for i in 0..std::cmp::min(rl.length, 10) {
                let tag = *rl.tags.add(i as usize);
                let elem_ptr = *rl.data.add(i as usize);
                
                // Describe tag
                let tag_str = match tag {
                    TypeTag::Any => "Any",
                    TypeTag::None_ => "None",
                    TypeTag::Bool => "Bool",
                    TypeTag::Int => "Int",
                    TypeTag::Float => "Float",
                    TypeTag::String => "String",
                    TypeTag::List => "List",
                    TypeTag::Tuple => "Tuple",
                };
                
                eprintln!("    [{}]: tag={} ({}), ptr={:p}", i, tag as u8, tag_str, elem_ptr);
            }
            
            if rl.length > 10 {
                eprintln!("    ... and {} more elements", rl.length - 10);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn list_get_tag(list_ptr: *mut RawList, index: i64) -> TypeTag {
    unsafe {
        // Check for null pointer
        if list_ptr.is_null() {
            eprintln!("[DEBUG] list_get_tag: list_ptr is null");
            return TypeTag::Any;
        }
        
        let rl = &*list_ptr;
        
        // Sanity check for length
        if rl.length < 0 || rl.length > 10000 {
            eprintln!("[DEBUG] list_get_tag: invalid length {}", rl.length);
            return TypeTag::Any;
        }
        
        // Check if tags pointer is null
        if rl.tags.is_null() {
            eprintln!("[DEBUG] list_get_tag: tags pointer is null");
            return TypeTag::Any;
        }
        
        // Check valid index
        if index < 0 || index >= rl.length {
            eprintln!("[DEBUG] list_get_tag: index out of bounds - {} not in [0, {})", index, rl.length);
            return TypeTag::Any;
        }
        
        // Read the tag at this index
        let tag = *rl.tags.add(index as usize);
        
        // Validate the tag value (prevent undefined behavior with corrupted tags)
        match tag {
            TypeTag::Any | TypeTag::None_ | TypeTag::Bool | 
            TypeTag::Int | TypeTag::Float | TypeTag::String |
            TypeTag::List | TypeTag::Tuple => tag,
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