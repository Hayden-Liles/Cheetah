// boxed_tuple.rs - Tuple implementation using BoxedAny values
//
// This file implements a tuple type using BoxedAny values.
// A tuple is essentially a fixed-size list of BoxedAny values.

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use libc::{calloc, free, malloc};
use std::ffi::c_void;
use std::ptr;

use super::boxed_any::{BoxedAny, type_tags};

/// C-compatible tuple struct using BoxedAny values
#[repr(C)]
pub struct BoxedTuple {
    pub length: i64,
    pub data: *mut *mut BoxedAny,
}

/// Create a new empty tuple
#[no_mangle]
pub extern "C" fn boxed_tuple_new() -> *mut BoxedTuple {
    boxed_tuple_new_with_length(0)
}

/// Create a new tuple with the specified length
#[no_mangle]
pub extern "C" fn boxed_tuple_new_with_length(length: i64) -> *mut BoxedTuple {
    let tuple = unsafe { malloc(std::mem::size_of::<BoxedTuple>()) as *mut BoxedTuple };
    if tuple.is_null() {
        return ptr::null_mut();
    }

    let data = unsafe { calloc(length as usize, std::mem::size_of::<*mut BoxedAny>()) as *mut *mut BoxedAny };
    if data.is_null() && length > 0 {
        unsafe { free(tuple as *mut c_void); }
        return ptr::null_mut();
    }

    unsafe {
        (*tuple).length = length;
        (*tuple).data = data;
    }

    tuple
}

/// Append a value to a tuple (reallocates the tuple)
#[no_mangle]
pub extern "C" fn boxed_tuple_append(tuple: *mut BoxedTuple, value: *mut BoxedAny) {
    if tuple.is_null() {
        return;
    }

    unsafe {
        let old_length = (*tuple).length;
        let new_length = old_length + 1;

        // Reallocate the data array
        let new_data = calloc(new_length as usize, std::mem::size_of::<*mut BoxedAny>()) as *mut *mut BoxedAny;
        if new_data.is_null() {
            return;
        }

        // Copy existing data
        if !(*tuple).data.is_null() {
            for i in 0..old_length {
                *new_data.add(i as usize) = *(*tuple).data.add(i as usize);
            }
            free((*tuple).data as *mut c_void);
        }

        // Add the new value
        *new_data.add(old_length as usize) = value;

        // Update the tuple
        (*tuple).data = new_data;
        (*tuple).length = new_length;
    }
}

/// Get an item from a tuple at the specified index
#[no_mangle]
pub extern "C" fn boxed_tuple_get(tuple: *mut BoxedTuple, index: i64) -> *mut BoxedAny {
    if tuple.is_null() {
        return super::boxed_any::boxed_any_none();
    }

    unsafe {
        // Handle negative indices (Python-style)
        let adjusted_index = if index < 0 {
            (*tuple).length + index
        } else {
            index
        };

        if adjusted_index < 0 || adjusted_index >= (*tuple).length {
            return super::boxed_any::boxed_any_none();
        }

        let value = *(*tuple).data.add(adjusted_index as usize);
        if value.is_null() {
            super::boxed_any::boxed_any_none()
        } else {
            value
        }
    }
}

/// Set an item in a tuple at the specified index
#[no_mangle]
pub extern "C" fn boxed_tuple_set(tuple: *mut BoxedTuple, index: i64, value: *mut BoxedAny) {
    if tuple.is_null() {
        return;
    }

    unsafe {
        // Handle negative indices (Python-style)
        let adjusted_index = if index < 0 {
            (*tuple).length + index
        } else {
            index
        };

        if adjusted_index < 0 || adjusted_index >= (*tuple).length {
            return;
        }

        // Free the old value if it exists
        let old_value = *(*tuple).data.add(adjusted_index as usize);
        if !old_value.is_null() {
            super::boxed_any::boxed_any_free(old_value);
        }

        // Set the new value
        *(*tuple).data.add(adjusted_index as usize) = value;
    }
}

/// Get the length of a tuple
#[no_mangle]
pub extern "C" fn boxed_tuple_len(tuple: *mut BoxedTuple) -> i64 {
    if tuple.is_null() {
        return 0;
    }

    unsafe { (*tuple).length }
}

/// Free a tuple and all its items
#[no_mangle]
pub extern "C" fn boxed_tuple_free(tuple: *mut BoxedTuple) {
    if tuple.is_null() {
        return;
    }

    unsafe {
        // Free all items in the tuple
        for i in 0..(*tuple).length {
            let value = *(*tuple).data.add(i as usize);
            if !value.is_null() {
                super::boxed_any::boxed_any_free(value);
            }
        }

        // Free the data array
        if !(*tuple).data.is_null() {
            free((*tuple).data as *mut c_void);
        }

        // Free the tuple itself
        free(tuple as *mut c_void);
    }
}

/// Create a BoxedAny from a BoxedTuple
#[no_mangle]
pub extern "C" fn boxed_any_from_tuple(tuple: *mut BoxedTuple) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::TUPLE;
        (*boxed).data.ptr_val = tuple as *mut c_void;
    }
    boxed
}

/// Get the BoxedTuple from a BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_tuple(value: *const BoxedAny) -> *mut BoxedTuple {
    if value.is_null() {
        return boxed_tuple_new();
    }

    unsafe {
        if (*value).tag == type_tags::TUPLE {
            (*value).data.ptr_val as *mut BoxedTuple
        } else if (*value).tag == type_tags::LIST {
            // Convert a list to a tuple
            let list = (*value).data.ptr_val as *mut super::boxed_list::BoxedList;
            let length = super::boxed_list::boxed_list_len(list);
            let tuple = boxed_tuple_new_with_length(length);

            for i in 0..length {
                let item = super::boxed_list::boxed_list_get(list, i);
                let item_clone = super::boxed_any::boxed_any_clone(item);
                boxed_tuple_set(tuple, i, item_clone);
            }

            tuple
        } else {
            // If it's not a tuple or list, create a new empty tuple
            boxed_tuple_new()
        }
    }
}

/// Clone a tuple
#[no_mangle]
pub extern "C" fn boxed_tuple_clone(tuple: *mut BoxedTuple) -> *mut BoxedTuple {
    if tuple.is_null() {
        return boxed_tuple_new();
    }

    unsafe {
        let length = (*tuple).length;
        let new_tuple = boxed_tuple_new_with_length(length);

        for i in 0..length {
            let value = *(*tuple).data.add(i as usize);
            if !value.is_null() {
                let cloned = super::boxed_any::boxed_any_clone(value);
                boxed_tuple_set(new_tuple, i, cloned);
            }
        }

        new_tuple
    }
}

/// Convert a tuple to a list
#[no_mangle]
pub extern "C" fn boxed_tuple_to_list(tuple: *mut BoxedTuple) -> *mut super::boxed_list::BoxedList {
    if tuple.is_null() {
        return super::boxed_list::boxed_list_new();
    }

    unsafe {
        let length = (*tuple).length;
        let list = super::boxed_list::boxed_list_with_capacity(length);

        for i in 0..length {
            let value = *(*tuple).data.add(i as usize);
            if !value.is_null() {
                let cloned = super::boxed_any::boxed_any_clone(value);
                super::boxed_list::boxed_list_append(list, cloned);
            } else {
                super::boxed_list::boxed_list_append(list, super::boxed_any::boxed_any_none());
            }
        }

        list
    }
}

/// Register BoxedTuple functions in the LLVM module
pub fn register_boxed_tuple_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let void_type = context.void_type();
    let i64_type = context.i64_type();
    let boxed_any_ptr_type = context.ptr_type(AddressSpace::default());
    let boxed_tuple_ptr_type = context.ptr_type(AddressSpace::default());
    let boxed_list_ptr_type = context.ptr_type(AddressSpace::default());

    // Tuple creation and management functions
    module.add_function(
        "boxed_tuple_new",
        boxed_tuple_ptr_type.fn_type(&[], false),
        None,
    );

    module.add_function(
        "boxed_tuple_new_with_length",
        boxed_tuple_ptr_type.fn_type(&[i64_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_tuple_append",
        void_type.fn_type(&[
            boxed_tuple_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_tuple_get",
        boxed_any_ptr_type.fn_type(&[
            boxed_tuple_ptr_type.into(),
            i64_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_tuple_set",
        void_type.fn_type(&[
            boxed_tuple_ptr_type.into(),
            i64_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_tuple_len",
        i64_type.fn_type(&[boxed_tuple_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_tuple_free",
        void_type.fn_type(&[boxed_tuple_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_tuple_clone",
        boxed_tuple_ptr_type.fn_type(&[boxed_tuple_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_tuple_to_list",
        boxed_list_ptr_type.fn_type(&[boxed_tuple_ptr_type.into()], false),
        None,
    );

    // BoxedAny conversion functions
    module.add_function(
        "boxed_any_from_tuple",
        boxed_any_ptr_type.fn_type(&[boxed_tuple_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_any_as_tuple",
        boxed_tuple_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false),
        None,
    );
}

/// Register BoxedTuple runtime mappings for the JIT engine
pub fn register_boxed_tuple_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("boxed_tuple_new") {
        engine.add_global_mapping(&f, boxed_tuple_new as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_new_with_length") {
        engine.add_global_mapping(&f, boxed_tuple_new_with_length as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_append") {
        engine.add_global_mapping(&f, boxed_tuple_append as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_get") {
        engine.add_global_mapping(&f, boxed_tuple_get as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_set") {
        engine.add_global_mapping(&f, boxed_tuple_set as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_len") {
        engine.add_global_mapping(&f, boxed_tuple_len as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_free") {
        engine.add_global_mapping(&f, boxed_tuple_free as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_clone") {
        engine.add_global_mapping(&f, boxed_tuple_clone as usize);
    }

    if let Some(f) = module.get_function("boxed_tuple_to_list") {
        engine.add_global_mapping(&f, boxed_tuple_to_list as usize);
    }

    if let Some(f) = module.get_function("boxed_any_from_tuple") {
        engine.add_global_mapping(&f, boxed_any_from_tuple as usize);
    }

    if let Some(f) = module.get_function("boxed_any_as_tuple") {
        engine.add_global_mapping(&f, boxed_any_as_tuple as usize);
    }

    Ok(())
}

pub fn get_boxed_tuple_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}
