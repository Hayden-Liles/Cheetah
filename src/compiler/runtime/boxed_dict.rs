// boxed_dict.rs - Dictionary implementation using BoxedAny values

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::StructType;
use inkwell::AddressSpace;
use inkwell::execution_engine::ExecutionEngine;

use libc::{calloc, free, malloc};
use std::ffi::c_void;
use std::ptr;

use super::boxed_any::{BoxedAny, type_tags};

/// C-compatible dictionary struct using BoxedAny values
#[repr(C)]
pub struct BoxedDict {
    pub count: i64,
    pub capacity: i64,
    pub entries: *mut BoxedDictEntry,
}

/// C-compatible dictionary entry using BoxedAny values
#[repr(C)]
pub struct BoxedDictEntry {
    pub key: *mut BoxedAny,
    pub value: *mut BoxedAny,
    pub hash: i64,
}

/// Create a new empty dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_new() -> *mut BoxedDict {
    let dict = unsafe { malloc(std::mem::size_of::<BoxedDict>()) as *mut BoxedDict };
    if dict.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        (*dict).count = 0;
        (*dict).capacity = 0;
        (*dict).entries = ptr::null_mut();
    }

    dict
}

/// Create a dictionary with the specified capacity
#[no_mangle]
pub extern "C" fn boxed_dict_with_capacity(capacity: i64) -> *mut BoxedDict {
    let dict = boxed_dict_new();
    if dict.is_null() {
        return ptr::null_mut();
    }

    // Calculate size for debugging purposes
    let _size = (capacity as usize) * std::mem::size_of::<BoxedDictEntry>();
    let entries = unsafe { calloc(capacity as usize, std::mem::size_of::<BoxedDictEntry>()) as *mut BoxedDictEntry };

    if entries.is_null() {
        unsafe { free(dict as *mut c_void); }
        return ptr::null_mut();
    }

    unsafe {
        (*dict).capacity = capacity;
        (*dict).entries = entries;
    }

    dict
}

/// Free a dictionary and all its entries
#[no_mangle]
pub extern "C" fn boxed_dict_free(dict: *mut BoxedDict) {
    if dict.is_null() {
        return;
    }

    unsafe {
        let entries = (*dict).entries;
        if !entries.is_null() {
            // Free all keys and values
            for i in 0..(*dict).capacity {
                let entry = entries.add(i as usize);
                if !(*entry).key.is_null() {
                    super::boxed_any::boxed_any_free((*entry).key);

                    if !(*entry).value.is_null() {
                        super::boxed_any::boxed_any_free((*entry).value);
                    }
                }
            }

            // Free the entries array
            free(entries as *mut c_void);
        }

        // Free the dictionary itself
        free(dict as *mut c_void);
    }
}

/// Get the number of entries in a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_len(dict: *mut BoxedDict) -> i64 {
    if dict.is_null() {
        return 0;
    }

    unsafe { (*dict).count }
}

/// Compute a hash for a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_dict_hash(key: *mut BoxedAny) -> i64 {
    if key.is_null() {
        return 0;
    }

    unsafe {
        match (*key).tag {
            type_tags::INT => (*key).data.int_val,
            type_tags::FLOAT => (*key).data.float_val as i64,
            type_tags::BOOL => if (*key).data.bool_val != 0 { 1 } else { 0 },
            type_tags::STRING => {
                let ptr = (*key).data.ptr_val as *const i8;
                if ptr.is_null() {
                    0
                } else {
                    let mut hash: i64 = 0;
                    let mut i = 0;
                    loop {
                        let c = *ptr.add(i);
                        if c == 0 {
                            break;
                        }
                        hash = hash.wrapping_mul(31).wrapping_add(c as i64);
                        i += 1;
                    }
                    hash
                }
            },
            _ => key as i64, // Use pointer address for other types
        }
    }
}

/// Find an entry in a dictionary
fn find_entry(dict: *mut BoxedDict, key: *mut BoxedAny) -> Option<*mut BoxedDictEntry> {
    if dict.is_null() || key.is_null() {
        return None;
    }

    unsafe {
        let hash = boxed_dict_hash(key);
        let capacity = (*dict).capacity;

        if capacity == 0 {
            return None;
        }

        let entries = (*dict).entries;
        let mut index = (hash % capacity) as usize;
        let start_index = index;

        loop {
            let entry = entries.add(index);

            if (*entry).key.is_null() {
                // Empty slot
                return Some(entry);
            }

            if (*entry).hash == hash && super::boxed_any::boxed_any_equals((*entry).key, key) {
                // Found matching key
                return Some(entry);
            }

            // Linear probing
            index = (index + 1) % capacity as usize;

            // If we've gone all the way around, stop
            if index == start_index {
                return None;
            }
        }
    }
}

/// Resize a dictionary when it gets too full
fn resize_dict(dict: *mut BoxedDict) {
    unsafe {
        let old_capacity = (*dict).capacity;
        let new_capacity = if old_capacity == 0 { 8 } else { old_capacity * 2 };

        let new_dict = boxed_dict_with_capacity(new_capacity);
        if new_dict.is_null() {
            return;
        }

        let old_entries = (*dict).entries;

        // Rehash all entries
        for i in 0..old_capacity {
            let entry = old_entries.add(i as usize);
            if !(*entry).key.is_null() {
                if let Some(new_entry) = find_entry(new_dict, (*entry).key) {
                    (*new_entry).key = (*entry).key;
                    (*new_entry).value = (*entry).value;
                    (*new_entry).hash = (*entry).hash;
                    (*new_dict).count += 1;
                }
            }
        }

        // Free old entries array but not the keys/values
        free(old_entries as *mut c_void);

        // Update the original dict
        (*dict).capacity = new_capacity;
        (*dict).entries = (*new_dict).entries;

        // Free the new dict struct but not its entries
        (*new_dict).entries = ptr::null_mut();
        free(new_dict as *mut c_void);
    }
}

/// Get a value from a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_get(dict: *mut BoxedDict, key: *mut BoxedAny) -> *mut BoxedAny {
    if dict.is_null() || key.is_null() {
        return super::boxed_any::boxed_any_none();
    }

    if let Some(entry) = find_entry(dict, key) {
        unsafe {
            if !(*entry).key.is_null() && super::boxed_any::boxed_any_equals((*entry).key, key) {
                return (*entry).value;
            }
        }
    }

    super::boxed_any::boxed_any_none()
}

/// Set a value in a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_set(dict: *mut BoxedDict, key: *mut BoxedAny, value: *mut BoxedAny) {
    if dict.is_null() || key.is_null() {
        return;
    }

    unsafe {
        // Resize if needed
        if (*dict).count >= (*dict).capacity * 3 / 4 {
            resize_dict(dict);
        }

        if (*dict).capacity == 0 {
            resize_dict(dict);
        }

        if let Some(entry) = find_entry(dict, key) {
            let is_new = (*entry).key.is_null();

            if is_new {
                (*entry).key = key;
                (*entry).hash = boxed_dict_hash(key);
                (*dict).count += 1;
            } else {
                // Free the old value
                if !(*entry).value.is_null() {
                    super::boxed_any::boxed_any_free((*entry).value);
                }
            }

            (*entry).value = value;
        }
    }
}

/// Check if a key exists in a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_contains(dict: *mut BoxedDict, key: *mut BoxedAny) -> bool {
    if dict.is_null() || key.is_null() {
        return false;
    }

    if let Some(entry) = find_entry(dict, key) {
        unsafe {
            return !(*entry).key.is_null() && super::boxed_any::boxed_any_equals((*entry).key, key);
        }
    }

    false
}

/// Remove a key from a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_remove(dict: *mut BoxedDict, key: *mut BoxedAny) -> bool {
    if dict.is_null() || key.is_null() {
        return false;
    }

    if let Some(entry) = find_entry(dict, key) {
        unsafe {
            if !(*entry).key.is_null() && super::boxed_any::boxed_any_equals((*entry).key, key) {
                // Free the key and value
                super::boxed_any::boxed_any_free((*entry).key);
                if !(*entry).value.is_null() {
                    super::boxed_any::boxed_any_free((*entry).value);
                }

                // Mark the entry as empty
                (*entry).key = ptr::null_mut();
                (*entry).value = ptr::null_mut();
                (*entry).hash = 0;

                (*dict).count -= 1;
                return true;
            }
        }
    }

    false
}

/// Clear all entries from a dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_clear(dict: *mut BoxedDict) {
    if dict.is_null() {
        return;
    }

    unsafe {
        let entries = (*dict).entries;
        if !entries.is_null() {
            for i in 0..(*dict).capacity {
                let entry = entries.add(i as usize);
                if !(*entry).key.is_null() {
                    // Free the key and value
                    super::boxed_any::boxed_any_free((*entry).key);
                    if !(*entry).value.is_null() {
                        super::boxed_any::boxed_any_free((*entry).value);
                    }

                    // Mark the entry as empty
                    (*entry).key = ptr::null_mut();
                    (*entry).value = ptr::null_mut();
                    (*entry).hash = 0;
                }
            }
        }

        (*dict).count = 0;
    }
}

/// Create a BoxedAny from a BoxedDict
#[no_mangle]
pub extern "C" fn boxed_any_from_dict(dict: *mut BoxedDict) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::DICT;
        (*boxed).data.ptr_val = dict as *mut c_void;
    }
    boxed
}

/// Get the BoxedDict from a BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_dict(value: *const BoxedAny) -> *mut BoxedDict {
    if value.is_null() {
        return boxed_dict_new();
    }

    unsafe {
        if (*value).tag == type_tags::DICT {
            (*value).data.ptr_val as *mut BoxedDict
        } else {
            // If it's not a dict, create a new empty dict
            boxed_dict_new()
        }
    }
}

/// Get all keys from a dictionary as a BoxedList
#[no_mangle]
pub extern "C" fn boxed_dict_keys(dict: *mut BoxedDict) -> *mut super::boxed_list::BoxedList {
    if dict.is_null() {
        return super::boxed_list::boxed_list_new();
    }

    unsafe {
        let count = (*dict).count;
        let entries = (*dict).entries;
        let keys_list = super::boxed_list::boxed_list_with_capacity(count);

        if keys_list.is_null() {
            return super::boxed_list::boxed_list_new();
        }

        let mut _added = 0;
        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                // Clone the key before adding it to the list
                let key_clone = super::boxed_any::boxed_any_clone((*entry).key);
                super::boxed_list::boxed_list_append(keys_list, key_clone);
                _added += 1;
            }
        }

        keys_list
    }
}

/// Get all values from a dictionary as a BoxedList
#[no_mangle]
pub extern "C" fn boxed_dict_values(dict: *mut BoxedDict) -> *mut super::boxed_list::BoxedList {
    if dict.is_null() {
        return super::boxed_list::boxed_list_new();
    }

    unsafe {
        let count = (*dict).count;
        let entries = (*dict).entries;
        let values_list = super::boxed_list::boxed_list_with_capacity(count);

        if values_list.is_null() {
            return super::boxed_list::boxed_list_new();
        }

        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                // Clone the value before adding it to the list
                let value_clone = super::boxed_any::boxed_any_clone((*entry).value);
                super::boxed_list::boxed_list_append(values_list, value_clone);
            }
        }

        values_list
    }
}

/// Get all key-value pairs from a dictionary as a BoxedList of tuples
#[no_mangle]
pub extern "C" fn boxed_dict_items(dict: *mut BoxedDict) -> *mut super::boxed_list::BoxedList {
    if dict.is_null() {
        return super::boxed_list::boxed_list_new();
    }

    unsafe {
        let count = (*dict).count;
        let entries = (*dict).entries;
        let items_list = super::boxed_list::boxed_list_with_capacity(count);

        if items_list.is_null() {
            return super::boxed_list::boxed_list_new();
        }

        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                // Create a tuple with key and value clones
                let tuple_list = super::boxed_list::boxed_list_with_capacity(2);

                let key_clone = super::boxed_any::boxed_any_clone((*entry).key);
                let value_clone = super::boxed_any::boxed_any_clone((*entry).value);

                super::boxed_list::boxed_list_append(tuple_list, key_clone);
                super::boxed_list::boxed_list_append(tuple_list, value_clone);

                // Convert the list to a BoxedAny and add it to the items list
                let tuple_any = super::boxed_list::boxed_any_from_list(tuple_list);
                super::boxed_list::boxed_list_append(items_list, tuple_any);
            }
        }

        items_list
    }
}

/// Merge two dictionaries into a new dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_merge(dict1: *mut BoxedDict, dict2: *mut BoxedDict) -> *mut BoxedDict {
    if dict1.is_null() {
        if dict2.is_null() {
            return boxed_dict_new();
        } else {
            // Clone dict2
            let result = boxed_dict_with_capacity(unsafe { (*dict2).capacity });
            boxed_dict_update(result, dict2);
            return result;
        }
    } else if dict2.is_null() {
        // Clone dict1
        let result = boxed_dict_with_capacity(unsafe { (*dict1).capacity });
        boxed_dict_update(result, dict1);
        return result;
    }

    unsafe {
        let capacity1 = (*dict1).capacity;
        let capacity2 = (*dict2).capacity;
        // We could use these counts for optimization in the future
        let _count1 = (*dict1).count;
        let _count2 = (*dict2).count;

        // Create a new dictionary with enough capacity for both
        let result = boxed_dict_with_capacity(capacity1 + capacity2);

        // Copy entries from dict1
        let entries1 = (*dict1).entries;
        for i in 0..capacity1 {
            let entry = entries1.add(i as usize);
            if !(*entry).key.is_null() {
                let key_clone = super::boxed_any::boxed_any_clone((*entry).key);
                let value_clone = super::boxed_any::boxed_any_clone((*entry).value);
                boxed_dict_set(result, key_clone, value_clone);
            }
        }

        // Copy entries from dict2 (overwriting any duplicates from dict1)
        let entries2 = (*dict2).entries;
        for i in 0..capacity2 {
            let entry = entries2.add(i as usize);
            if !(*entry).key.is_null() {
                let key_clone = super::boxed_any::boxed_any_clone((*entry).key);
                let value_clone = super::boxed_any::boxed_any_clone((*entry).value);
                boxed_dict_set(result, key_clone, value_clone);
            }
        }

        result
    }
}

/// Update a dictionary with entries from another dictionary
#[no_mangle]
pub extern "C" fn boxed_dict_update(target: *mut BoxedDict, source: *mut BoxedDict) {
    if target.is_null() || source.is_null() {
        return;
    }

    unsafe {
        let capacity = (*source).capacity;
        let entries = (*source).entries;

        for i in 0..capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                let key_clone = super::boxed_any::boxed_any_clone((*entry).key);
                let value_clone = super::boxed_any::boxed_any_clone((*entry).value);
                boxed_dict_set(target, key_clone, value_clone);
            }
        }
    }
}

/// Register BoxedDict functions in the LLVM module
pub fn register_boxed_dict_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    let void_type = context.void_type();
    let i64_type = context.i64_type();
    // We might need this type in the future
    let _i8_type = context.i8_type();
    let bool_type = context.bool_type();
    let boxed_any_ptr_type = context.ptr_type(AddressSpace::default());
    let boxed_dict_ptr_type = context.ptr_type(AddressSpace::default());
    let boxed_list_ptr_type = context.ptr_type(AddressSpace::default());

    // Dict creation and management functions
    module.add_function(
        "boxed_dict_new",
        boxed_dict_ptr_type.fn_type(&[], false),
        None,
    );

    module.add_function(
        "boxed_dict_with_capacity",
        boxed_dict_ptr_type.fn_type(&[i64_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_free",
        void_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_len",
        i64_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    // Dict operations
    module.add_function(
        "boxed_dict_get",
        boxed_any_ptr_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_dict_set",
        void_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_any_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_dict_contains",
        bool_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_dict_remove",
        bool_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_any_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_dict_clear",
        void_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_keys",
        boxed_list_ptr_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_values",
        boxed_list_ptr_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_items",
        boxed_list_ptr_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_dict_merge",
        boxed_dict_ptr_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_dict_ptr_type.into(),
        ], false),
        None,
    );

    module.add_function(
        "boxed_dict_update",
        void_type.fn_type(&[
            boxed_dict_ptr_type.into(),
            boxed_dict_ptr_type.into(),
        ], false),
        None,
    );

    // BoxedAny conversion functions
    module.add_function(
        "boxed_any_from_dict",
        boxed_any_ptr_type.fn_type(&[boxed_dict_ptr_type.into()], false),
        None,
    );

    module.add_function(
        "boxed_any_as_dict",
        boxed_dict_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false),
        None,
    );
}

/// Register BoxedDict runtime mappings for the JIT engine
pub fn register_boxed_dict_runtime_functions(
    engine: &ExecutionEngine<'_>,
    module: &Module<'_>,
) -> Result<(), String> {
    // Creation and management functions
    if let Some(f) = module.get_function("boxed_dict_new") {
        engine.add_global_mapping(&f, boxed_dict_new as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_with_capacity") {
        engine.add_global_mapping(&f, boxed_dict_with_capacity as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_free") {
        engine.add_global_mapping(&f, boxed_dict_free as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_len") {
        engine.add_global_mapping(&f, boxed_dict_len as usize);
    }

    // Dictionary operations
    if let Some(f) = module.get_function("boxed_dict_get") {
        engine.add_global_mapping(&f, boxed_dict_get as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_set") {
        engine.add_global_mapping(&f, boxed_dict_set as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_contains") {
        engine.add_global_mapping(&f, boxed_dict_contains as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_remove") {
        engine.add_global_mapping(&f, boxed_dict_remove as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_clear") {
        engine.add_global_mapping(&f, boxed_dict_clear as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_keys") {
        engine.add_global_mapping(&f, boxed_dict_keys as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_values") {
        engine.add_global_mapping(&f, boxed_dict_values as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_items") {
        engine.add_global_mapping(&f, boxed_dict_items as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_merge") {
        engine.add_global_mapping(&f, boxed_dict_merge as usize);
    }

    if let Some(f) = module.get_function("boxed_dict_update") {
        engine.add_global_mapping(&f, boxed_dict_update as usize);
    }

    // BoxedAny conversion functions
    if let Some(f) = module.get_function("boxed_any_from_dict") {
        engine.add_global_mapping(&f, boxed_any_from_dict as usize);
    }

    if let Some(f) = module.get_function("boxed_any_as_dict") {
        engine.add_global_mapping(&f, boxed_any_as_dict as usize);
    }

    Ok(())
}

pub fn get_boxed_dict_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.i64_type().into(),
            context.i64_type().into(),
            context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    )
}

pub fn get_boxed_dict_entry_struct_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            context.ptr_type(AddressSpace::default()).into(),
            context.ptr_type(AddressSpace::default()).into(),
            context.i64_type().into(),
        ],
        false,
    )
}
