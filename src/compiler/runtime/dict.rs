// dict.rs - Combined dictionary runtime & LLVM registration

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::AddressSpace;

use std::ptr;
use std::ffi::c_void;

/// C-compatible dict struct
#[repr(C)]
pub struct Dict {
    count: i64,
    capacity: i64,
    entries: *mut DictEntry,
}

#[repr(C)]
pub struct DictEntry {
    key: *mut c_void,
    value: *mut c_void,
    hash: i64,
}

#[repr(C)]
pub struct List {
    length: i64,
    capacity: i64,
    data: *mut *mut c_void,
}

#[repr(C)]
pub struct Tuple {
    length: i64,
    data: *mut *mut c_void,
}

/// Create a new list with given capacity (used by dict methods)
unsafe fn list_with_capacity(capacity: i64) -> *mut List {
    let list = std::alloc::alloc(std::alloc::Layout::new::<List>()) as *mut List;
    (*list).length = 0;
    (*list).capacity = capacity;
    let layout = std::alloc::Layout::array::<*mut c_void>(capacity as usize).unwrap();
    (*list).data = std::alloc::alloc(layout) as *mut *mut c_void;
    std::ptr::write_bytes((*list).data as *mut u8, 0, layout.size());
    list
}

unsafe fn tuple_new(length: i64) -> *mut Tuple {
    let tuple = std::alloc::alloc(std::alloc::Layout::new::<Tuple>()) as *mut Tuple;
    (*tuple).length = length;
    let layout = std::alloc::Layout::array::<*mut c_void>(length as usize).unwrap();
    (*tuple).data = std::alloc::alloc(layout) as *mut *mut c_void;
    std::ptr::write_bytes((*tuple).data as *mut u8, 0, layout.size());
    tuple
}

#[no_mangle]
pub unsafe extern "C" fn dict_keys(dict: *mut Dict) -> *mut List {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;
    let keys_list = list_with_capacity(count);
    let mut added = 0;
    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            *(*keys_list).data.add(added as usize) = (*entry).key;
            added += 1;
        }
    }
    (*keys_list).length = added;
    keys_list
}

#[no_mangle]
pub unsafe extern "C" fn dict_values(dict: *mut Dict) -> *mut List {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;
    let values_list = list_with_capacity(count);
    let mut added = 0;
    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            *(*values_list).data.add(added as usize) = (*entry).value;
            added += 1;
        }
    }
    (*values_list).length = added;
    values_list
}

#[no_mangle]
pub unsafe extern "C" fn dict_items(dict: *mut Dict) -> *mut List {
    if dict.is_null() { return ptr::null_mut(); }
    let count = (*dict).count;
    let entries = (*dict).entries;
    let items_list = list_with_capacity(count);
    let mut added = 0;
    for i in 0..(*dict).capacity {
        let entry = entries.add(i as usize);
        if !(*entry).key.is_null() {
            let tpl = tuple_new(2);
            *(*tpl).data.add(0) = (*entry).key;
            *(*tpl).data.add(1) = (*entry).value;
            *(*items_list).data.add(added as usize) = tpl as *mut c_void;
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