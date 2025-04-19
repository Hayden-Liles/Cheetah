// dict_methods.rs - Runtime support for dictionary methods

use std::ptr;

// Dictionary struct definition (must match the one in dict_ops.rs)
#[repr(C)]
pub struct Dict {
    count: i64,
    capacity: i64,
    entries: *mut DictEntry,
}

// Dictionary entry struct definition (must match the one in dict_ops.rs)
#[repr(C)]
pub struct DictEntry {
    key: *mut CVoid,
    value: *mut CVoid,
    hash: i64,
}

// List struct definition (must match the one in list_ops.rs)
#[repr(C)]
pub struct List {
    length: i64,
    capacity: i64,
    data: *mut *mut CVoid,
}

// Tuple struct definition
#[repr(C)]
pub struct Tuple {
    length: i64,
    data: *mut *mut CVoid,
}

// Void pointer type
type CVoid = std::ffi::c_void;

/// Create a new list with the given capacity
unsafe fn list_with_capacity(capacity: i64) -> *mut List {
    unsafe {
        let list = std::alloc::alloc(std::alloc::Layout::new::<List>()) as *mut List;
        (*list).length = 0;
        (*list).capacity = capacity;
        let layout = std::alloc::Layout::array::<*mut CVoid>(capacity as usize).unwrap();
        (*list).data = std::alloc::alloc(layout) as *mut *mut CVoid;
        std::ptr::write_bytes((*list).data as *mut u8, 0, layout.size());
        list
    }
}

/// Create a new tuple with the given length
unsafe fn tuple_new(length: i64) -> *mut Tuple {
    unsafe {
        let tuple = std::alloc::alloc(std::alloc::Layout::new::<Tuple>()) as *mut Tuple;
        (*tuple).length = length;
        let layout = std::alloc::Layout::array::<*mut CVoid>(length as usize).unwrap();
        (*tuple).data = std::alloc::alloc(layout) as *mut *mut CVoid;
        std::ptr::write_bytes((*tuple).data as *mut u8, 0, layout.size());
        tuple
    }
}

/// Get the keys of a dictionary as a list
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_keys(dict: *mut Dict) -> *mut List {
    unsafe {
        if dict.is_null() {
            return ptr::null_mut();
        }

        let count = (*dict).count;
        let entries = (*dict).entries;

        let keys_list = list_with_capacity(count);

        let mut added = 0;
        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                (*(*keys_list).data.add(added as usize)) = (*entry).key;
                added += 1;
            }
        }

        (*keys_list).length = added;

        keys_list
    }
}

/// Get the values of a dictionary as a list
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_values(dict: *mut Dict) -> *mut List {
    unsafe {
        if dict.is_null() {
            return ptr::null_mut();
        }

        let count = (*dict).count;
        let entries = (*dict).entries;

        let values_list = list_with_capacity(count);

        let mut added = 0;
        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                (*(*values_list).data.add(added as usize)) = (*entry).value;
                added += 1;
            }
        }

        (*values_list).length = added;

        values_list
    }
}

/// Get the items (key-value pairs) of a dictionary as a list of tuples
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dict_items(dict: *mut Dict) -> *mut List {
    unsafe {
        if dict.is_null() {
            return ptr::null_mut();
        }

        let count = (*dict).count;
        let entries = (*dict).entries;

        let items_list = list_with_capacity(count);

        let mut added = 0;
        for i in 0..(*dict).capacity {
            let entry = entries.add(i as usize);
            if !(*entry).key.is_null() {
                let tuple = tuple_new(2);

                (*(*tuple).data.add(0)) = (*entry).key;
                (*(*tuple).data.add(1)) = (*entry).value;

                (*(*items_list).data.add(added as usize)) = tuple as *mut CVoid;
                added += 1;
            }
        }

        (*items_list).length = added;

        items_list
    }
}
