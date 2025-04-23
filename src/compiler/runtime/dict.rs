// dict.rs - Combined dictionary runtime & LLVM registration

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;

use std::ptr;
use std::ffi::c_void;
use super::list::RawList;
use super::value::Value;

/// C-compatible dict struct
#[repr(C)]
pub struct Dict {
    count: i64,
    capacity: i64,
    entries: *mut DictEntry,
}

#[repr(C)]
pub struct DictEntry {
    key: *mut Value,
    value: *mut Value,
    hash: i64,
}

#[repr(C)]
pub struct Tuple {
    length: i64,
    data: *mut *mut Value,
}

unsafe fn tuple_new(length: i64) -> *mut Tuple {
    let tuple = std::alloc::alloc(std::alloc::Layout::new::<Tuple>()) as *mut Tuple;
    (*tuple).length = length;
    let layout = std::alloc::Layout::array::<*mut Value>(length as usize).unwrap();
    (*tuple).data = std::alloc::alloc(layout) as *mut *mut Value;
    std::ptr::write_bytes((*tuple).data as *mut u8, 0, layout.size());
    tuple
}

#[no_mangle]
pub unsafe extern "C" fn dict_new() -> *mut Dict {
    let ptr = std::alloc::alloc(std::alloc::Layout::new::<Dict>()) as *mut Dict;
    (*ptr).count = 0;
    (*ptr).capacity = 0;
    (*ptr).entries = std::ptr::null_mut();
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dict_with_capacity(cap: i64) -> *mut Dict {
    let d = dict_new();
    (*d).capacity = cap;
    let layout = std::alloc::Layout::array::<DictEntry>(cap as usize).unwrap();
    (*d).entries = std::alloc::alloc(layout) as *mut DictEntry;
    std::ptr::write_bytes((*d).entries as *mut u8, 0, layout.size());
    d
}

#[no_mangle]
pub unsafe extern "C" fn dict_set(
    dict: *mut Dict,
    key: *mut Value,
    value: *mut Value,
) {
    if dict.is_null() { return; }
    let d = &mut *dict;

    // If we need to resize
    if d.count >= d.capacity {
        let new_cap = if d.capacity == 0 { 8 } else { d.capacity * 2 };
        let new_layout = std::alloc::Layout::array::<DictEntry>(new_cap as usize).unwrap();
        let new_entries = std::alloc::alloc(new_layout) as *mut DictEntry;
        std::ptr::write_bytes(new_entries as *mut u8, 0, new_layout.size());

        // Copy existing entries
        if !d.entries.is_null() && d.capacity > 0 {
            for i in 0..d.capacity {
                let old_entry = d.entries.add(i as usize);
                if !(*old_entry).key.is_null() {
                    // simple open-addressing or linear probe; for now just shove into next slot
                    let j = i % new_cap;
                    let new_entry = new_entries.add(j as usize);
                    (*new_entry).key = (*old_entry).key;
                    (*new_entry).value = (*old_entry).value;
                    (*new_entry).hash = (*old_entry).hash;
                }
            }

            // Free old entries
            let old_layout = std::alloc::Layout::array::<DictEntry>(d.capacity as usize).unwrap();
            std::alloc::dealloc(d.entries as *mut u8, old_layout);
        }

        d.entries = new_entries;
        d.capacity = new_cap;
    }

    // simple open‑addressing or linear probe; for now just shove into next slot
    let i = d.count as usize % d.capacity as usize;
    let entry = d.entries.add(i);
    (*entry).key = key;
    (*entry).value = value;
    (*entry).hash = 0;  // optional
    d.count += 1;
}

#[no_mangle]
pub unsafe extern "C" fn dict_get(
    dict: *mut Dict,
    key: *mut Value,
) -> *mut Value {
    if dict.is_null() { return ptr::null_mut(); }
    let d = &*dict;

    for i in 0..d.capacity {
        let entry = d.entries.add(i as usize);
        if !(*entry).key.is_null() && (*entry).key == key {
            return (*entry).value;
        }
    }

    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn dict_contains(
    dict: *mut Dict,
    key: *mut Value,
) -> bool {
    if dict.is_null() { return false; }
    let d = &*dict;

    for i in 0..d.capacity {
        let entry = d.entries.add(i as usize);
        if !(*entry).key.is_null() && (*entry).key == key {
            return true;
        }
    }

    false
}

#[no_mangle]
pub unsafe extern "C" fn dict_len(dict: *mut Dict) -> i64 {
    if dict.is_null() { return 0; }
    (*dict).count
}

#[no_mangle]
pub unsafe extern "C" fn dict_free(dict: *mut Dict) {
    if dict.is_null() { return; }
    let d = &mut *dict;

    if !d.entries.is_null() && d.capacity > 0 {
        let layout = std::alloc::Layout::array::<DictEntry>(d.capacity as usize).unwrap();
        std::alloc::dealloc(d.entries as *mut u8, layout);
    }

    let layout = std::alloc::Layout::new::<Dict>();
    std::alloc::dealloc(dict as *mut u8, layout);
}

#[no_mangle]
pub unsafe extern "C" fn dict_keys(dict: *mut Dict) -> *mut RawList {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;

    // Use the list_with_capacity function from list.rs
    let keys_list = super::list::list_with_capacity(count);
    let mut added = 0;

    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            // The key is already a Value pointer
            *(*keys_list).data.add(added as usize) = (*entry).key;
            added += 1;
        }
    }

    (*keys_list).length = added;
    keys_list
}

#[no_mangle]
pub unsafe extern "C" fn dict_values(dict: *mut Dict) -> *mut RawList {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;

    // Use the list_with_capacity function from list.rs
    let values_list = super::list::list_with_capacity(count);
    let mut added = 0;

    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            // The value is already a Value pointer
            *(*values_list).data.add(added as usize) = (*entry).value;
            added += 1;
        }
    }

    (*values_list).length = added;
    values_list
}

#[no_mangle]
pub unsafe extern "C" fn dict_items(dict: *mut Dict) -> *mut RawList {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;

    // Use the list_with_capacity function from list.rs
    let items_list = super::list::list_with_capacity(count);
    let mut added = 0;

    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            let tpl = tuple_new(2);
            *(*tpl).data.add(0) = (*entry).key;
            *(*tpl).data.add(1) = (*entry).value;
            // Create a Value with tag Tuple
            let tuple_value = super::value::value_alloc(super::value::ValueTag::Tuple, tpl as *mut c_void);
            *(*items_list).data.add(added as usize) = tuple_value;
            added += 1;
        }
    }

    (*items_list).length = added;
    items_list
}

/// Register dictionary functions in the LLVM module
pub fn register_dict_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ], false);
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false);

    module.add_function(
        "dict_new",
        context.ptr_type(AddressSpace::default()).fn_type(&[], false),
        None,
    );
    module.add_function(
        "dict_with_capacity",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.i64_type().into()], false),
        None,
    );
    module.add_function(
        "dict_get",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_set",
        context.void_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_contains",
        context.i8_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_remove",
        context.i8_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_clear",
        context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "dict_len",
        context.i64_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "dict_free",
        context.void_type().fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "dict_merge",
        context.ptr_type(AddressSpace::default()).fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_update",
        context.void_type().fn_type(&[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
        ], false),
        None,
    );
    module.add_function(
        "dict_keys",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "dict_values",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
    module.add_function(
        "dict_items",
        context.ptr_type(AddressSpace::default()).fn_type(&[context.ptr_type(AddressSpace::default()).into()], false),
        None,
    );
}

pub fn get_dict_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}

pub fn get_dict_entry_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    )
}

pub fn get_dict_element_ptr_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context.ptr_type(AddressSpace::default()).as_basic_type_enum()
}

/// Register dictionary runtime functions for JIT execution
pub fn register_dict_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("dict_new") {
        engine.add_global_mapping(&f, dict_new as usize);
    }
    if let Some(f) = module.get_function("dict_with_capacity") {
        engine.add_global_mapping(&f, dict_with_capacity as usize);
    }
    if let Some(f) = module.get_function("dict_set") {
        engine.add_global_mapping(&f, dict_set as usize);
    }
    if let Some(f) = module.get_function("dict_get") {
        engine.add_global_mapping(&f, dict_get as usize);
    }
    if let Some(f) = module.get_function("dict_contains") {
        engine.add_global_mapping(&f, dict_contains as usize);
    }
    if let Some(f) = module.get_function("dict_len") {
        engine.add_global_mapping(&f, dict_len as usize);
    }
    if let Some(f) = module.get_function("dict_free") {
        engine.add_global_mapping(&f, dict_free as usize);
    }
    if let Some(f) = module.get_function("dict_keys") {
        engine.add_global_mapping(&f, dict_keys as usize);
    }
    if let Some(f) = module.get_function("dict_values") {
        engine.add_global_mapping(&f, dict_values as usize);
    }
    if let Some(f) = module.get_function("dict_items") {
        engine.add_global_mapping(&f, dict_items as usize);
    }
    Ok(())
}