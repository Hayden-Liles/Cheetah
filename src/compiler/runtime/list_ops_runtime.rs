// list_ops_runtime.rs - Runtime implementation of list operations

use std::ptr;
use std::ffi::c_void;

// List struct definition (must match the one in list_ops.rs)
#[repr(C)]
pub struct List {
    length: i64,
    capacity: i64,
    data: *mut *mut c_void,
}

// Simple global array to store range values
static mut RANGE_SIZE: i64 = 0;

/// Create a new empty list
#[unsafe(no_mangle)]
pub extern "C" fn list_new() -> *mut List {

    // Allocate memory for the list
    let list = Box::new(List {
        length: 0,
        capacity: 10, // Default initial capacity
        data: ptr::null_mut(),
    });

    // Convert to raw pointer
    Box::into_raw(list)
}

/// Create a new list with the given capacity
#[unsafe(no_mangle)]
pub extern "C" fn list_with_capacity(capacity: i64) -> *mut List {

    // Ensure the capacity is at least 1
    let actual_capacity = if capacity < 1 { 1 } else { capacity };

    // Allocate memory for the data array
    let data = unsafe {
        let layout = std::alloc::Layout::array::<*mut c_void>(actual_capacity as usize).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout) as *mut *mut c_void;

        if ptr.is_null() {
            // Memory allocation failed
            eprintln!("Failed to allocate memory for list data");
            return ptr::null_mut();
        }

        ptr
    };

    // Create the list
    let list = Box::new(List {
        length: 0,
        capacity: actual_capacity,
        data,
    });

    // Convert to raw pointer
    Box::into_raw(list)
}

/// Get an item from a list
#[unsafe(no_mangle)]
pub extern "C" fn list_get(list_ptr: *mut List, index: i64) -> *mut c_void {

    // Check if this is a range object - use a safer approach
    if list_ptr as usize <= 1000000 && list_ptr as usize > 0 && (list_ptr as usize & 0xFFF) == 0 {
        unsafe {
            // This is a range object, the pointer value is the range size
            let range_size = list_ptr as i64;

            // Check if the index is valid
            if index < 0 || index >= range_size {
                eprintln!("IndexError: range index {} out of range (size: {})", index, range_size);
                return ptr::null_mut();
            }

            // Create a new integer on the heap
            let value_ptr = Box::into_raw(Box::new(index));
            println!("Range: returning index {} as value", index);
            return value_ptr as *mut c_void;
        }
    }

    // Regular list handling
    if list_ptr.is_null() {
        eprintln!("Invalid list pointer in list_get");
        return ptr::null_mut();
    }

    unsafe {
        // Handle negative indices (Python-like behavior)
        let actual_index = if index < 0 {
            (*list_ptr).length + index
        } else {
            index
        };

        // Check if the index is valid
        if actual_index < 0 || actual_index >= (*list_ptr).length {
            // Index out of bounds
            eprintln!("IndexError: list index {} out of range (length: {})", index, (*list_ptr).length);
            return ptr::null_mut();
        }

        // Check if the data pointer is valid
        if (*list_ptr).data.is_null() {
            eprintln!("Invalid list data pointer in list_get");
            return ptr::null_mut();
        }

        // Get the item at the specified index
        let item_ptr = *(*list_ptr).data.add(actual_index as usize);

        // If the item is null, create a default value (0)
        if item_ptr.is_null() {
            let value_ptr = Box::into_raw(Box::new(0i64));
            return value_ptr as *mut c_void;
        }

        // Return a copy of the integer value
        let value = *(item_ptr as *const i64);
        let new_ptr = Box::into_raw(Box::new(value));
        new_ptr as *mut c_void
    }
}

/// Set an item in a list
#[unsafe(no_mangle)]
pub extern "C" fn list_set(list_ptr: *mut List, index: i64, value: *mut c_void) {

    if list_ptr.is_null() {
        eprintln!("Invalid list pointer in list_set");
        return;
    }

    unsafe {
        // Handle negative indices (Python-like behavior)
        let actual_index = if index < 0 {
            (*list_ptr).length + index
        } else {
            index
        };

        // Check if the index is valid
        if actual_index < 0 || actual_index >= (*list_ptr).length {
            // Index out of bounds
            eprintln!("IndexError: list index {} out of range (length: {})", index, (*list_ptr).length);
            return;
        }

        // Check if the data pointer is valid
        if (*list_ptr).data.is_null() {
            eprintln!("Invalid list data pointer in list_set");
            return;
        }

        // Free the old value if it exists
        let old_ptr = *(*list_ptr).data.add(actual_index as usize);
        if !old_ptr.is_null() {
            let _ = Box::from_raw(old_ptr as *mut i64);
        }

        // Create a copy of the value
        let new_value: i64;
        if value.is_null() {
            new_value = 0;
        } else {
            new_value = *(value as *const i64);
        }

        let new_ptr = Box::into_raw(Box::new(new_value));

        // Store the new value in the list
        *(*list_ptr).data.add(actual_index as usize) = new_ptr as *mut c_void;
    }
}

/// Append an item to a list
#[unsafe(no_mangle)]
pub extern "C" fn list_append(list_ptr: *mut List, value: *mut c_void) {

    if list_ptr.is_null() {
        eprintln!("Invalid list pointer in list_append");
        return;
    }

    unsafe {
        // Check if we need to allocate or resize the data array
        if (*list_ptr).data.is_null() {
            // Allocate memory for the data array
            let capacity = (*list_ptr).capacity;
            let layout = std::alloc::Layout::array::<*mut c_void>(capacity as usize).unwrap();
            (*list_ptr).data = std::alloc::alloc_zeroed(layout) as *mut *mut c_void;

            if (*list_ptr).data.is_null() {
                eprintln!("Failed to allocate memory for list data");
                return;
            }
        } else if (*list_ptr).length >= (*list_ptr).capacity {
            // Resize the data array
            let old_capacity = (*list_ptr).capacity;
            let new_capacity = old_capacity * 2;

            // Allocate a new data array
            let layout = std::alloc::Layout::array::<*mut c_void>(new_capacity as usize).unwrap();
            let new_data = std::alloc::alloc_zeroed(layout) as *mut *mut c_void;

            if new_data.is_null() {
                eprintln!("Failed to resize list data array");
                return;
            }

            // Copy the old data to the new array
            for i in 0..(*list_ptr).length as usize {
                *new_data.add(i) = *(*list_ptr).data.add(i);
            }

            // Free the old data array
            let old_layout = std::alloc::Layout::array::<*mut c_void>(old_capacity as usize).unwrap();
            std::alloc::dealloc((*list_ptr).data as *mut u8, old_layout);

            // Update the list
            (*list_ptr).data = new_data;
            (*list_ptr).capacity = new_capacity;
        }

        // Create a copy of the value
        let new_value: i64;
        if value.is_null() {
            new_value = 0;
        } else {
            new_value = *(value as *const i64);
        }

        let new_ptr = Box::into_raw(Box::new(new_value));

        // Add the value to the list
        *(*list_ptr).data.add((*list_ptr).length as usize) = new_ptr as *mut c_void;
        (*list_ptr).length += 1;

    }
}

/// Get the length of a list
#[unsafe(no_mangle)]
pub extern "C" fn list_len(list_ptr: *mut List) -> i64 {

    // Check if this is a range object - use a safer approach
    if list_ptr as usize <= 1000000 && list_ptr as usize > 0 && (list_ptr as usize & 0xFFF) == 0 {
        // This is a range object, the pointer value is the range size
        unsafe {
            let range_size = list_ptr as i64;
            println!("Range: returning size {}", range_size);
            return range_size;
        }
    }

    if list_ptr.is_null() {
        eprintln!("Invalid list pointer in list_len");
        return 0;
    }

    unsafe { (*list_ptr).length }
}

/// Free a list's memory
#[unsafe(no_mangle)]
pub extern "C" fn list_free(list_ptr: *mut List) {

    if list_ptr.is_null() {
        return;
    }

    unsafe {
        // Free each item in the list
        if !(*list_ptr).data.is_null() {
            for i in 0..(*list_ptr).length as usize {
                let item_ptr = *(*list_ptr).data.add(i);
                if !item_ptr.is_null() {
                    let _ = Box::from_raw(item_ptr as *mut i64);
                }
            }

            // Free the data array
            let layout = std::alloc::Layout::array::<*mut c_void>((*list_ptr).capacity as usize).unwrap();
            std::alloc::dealloc((*list_ptr).data as *mut u8, layout);
        }

        // Free the list itself
        let _ = Box::from_raw(list_ptr);
    }
}

/// Create a slice of a list
#[unsafe(no_mangle)]
pub extern "C" fn list_slice(list_ptr: *mut List, start: i64, stop: i64, step: i64) -> *mut List {

    // Handle invalid step
    if step == 0 {
        eprintln!("ValueError: slice step cannot be zero");
        return list_new();
    }

    // Handle range objects - use a safer approach
    if list_ptr as usize <= 1000000 && list_ptr as usize > 0 && (list_ptr as usize & 0xFFF) == 0 {
        // This is a range object, calculate the slice
        let range_size = list_ptr as i64;
        let (norm_start, norm_stop) = normalize_indices(start, stop, range_size);

        // Calculate the size of the slice
        let slice_size = calculate_slice_size(norm_start, norm_stop, step);

        // Create a new list for the slice
        let result_list = list_with_capacity(slice_size);
        if result_list.is_null() {
            return ptr::null_mut();
        }

        // Fill the slice
        let mut count = 0;
        if step > 0 {
            let mut i = norm_start;
            while i < norm_stop {
                // Create a value for the index
                let value_ptr = Box::into_raw(Box::new(i)) as *mut c_void;

                // Add it to the list
                unsafe {
                    *(*result_list).data.add(count) = value_ptr;
                    (*result_list).length += 1;
                }

                i += step;
                count += 1;
            }
        } else {
            // step < 0
            let mut i = norm_start;
            while i > norm_stop {
                // Create a value for the index
                let value_ptr = Box::into_raw(Box::new(i)) as *mut c_void;

                // Add it to the list
                unsafe {
                    *(*result_list).data.add(count) = value_ptr;
                    (*result_list).length += 1;
                }

                i += step;
                count += 1;
            }
        }

        return result_list;
    }

    // Regular list handling
    if list_ptr.is_null() {
        eprintln!("Invalid list pointer in list_slice");
        return ptr::null_mut();
    }

    unsafe {
        let list_len = (*list_ptr).length;
        let (norm_start, norm_stop) = normalize_indices(start, stop, list_len);

        // Calculate the size of the slice
        let slice_size = calculate_slice_size(norm_start, norm_stop, step);

        // Create a new list for the slice
        let result_list = list_with_capacity(slice_size);
        if result_list.is_null() {
            return ptr::null_mut();
        }

        // Check if the data pointer is valid
        if (*list_ptr).data.is_null() {
            eprintln!("Invalid list data pointer in list_slice");
            return result_list;
        }

        // Fill the slice
        let mut count = 0;
        if step > 0 {
            let mut i = norm_start;
            while i < norm_stop {
                let value_ptr = *(*list_ptr).data.add(i as usize);

                if !value_ptr.is_null() {
                    // Copy the value
                    let value = *(value_ptr as *const i64);
                    let new_ptr = Box::into_raw(Box::new(value)) as *mut c_void;

                    // Store it in the result list
                    *(*result_list).data.add(count) = new_ptr;
                    (*result_list).length += 1;
                }

                i += step;
                count += 1;
            }
        } else {
            // step < 0
            let mut i = norm_start;
            while i > norm_stop {
                let value_ptr = *(*list_ptr).data.add(i as usize);

                if !value_ptr.is_null() {
                    // Copy the value
                    let value = *(value_ptr as *const i64);
                    let new_ptr = Box::into_raw(Box::new(value)) as *mut c_void;

                    // Store it in the result list
                    *(*result_list).data.add(count) = new_ptr;
                    (*result_list).length += 1;
                }

                i += step;
                count += 1;
            }
        }

        result_list
    }
}

// Helper functions

/// Normalize indices for slicing
fn normalize_indices(start: i64, stop: i64, len: i64) -> (i64, i64) {
    let norm_start = if start < 0 {
        (start + len).max(0)
    } else {
        start.min(len)
    };

    let norm_stop = if stop < 0 {
        (stop + len).max(0)
    } else {
        stop.min(len)
    };

    (norm_start, norm_stop)
}

/// Calculate the size of a slice
fn calculate_slice_size(start: i64, stop: i64, step: i64) -> i64 {
    if (step > 0 && start >= stop) || (step < 0 && start <= stop) {
        return 0;
    }

    if step > 0 {
        return (stop - start + step - 1) / step;
    } else {
        return (start - stop - step - 1) / (-step);
    }
}

/// Concatenate two lists
#[unsafe(no_mangle)]
pub extern "C" fn list_concat(list1_ptr: *mut List, list2_ptr: *mut List) -> *mut List {

    if list1_ptr.is_null() || list2_ptr.is_null() {
        eprintln!("Invalid list pointers in list_concat");
        return ptr::null_mut();
    }

    unsafe {
        let list1_len = (*list1_ptr).length;
        let list2_len = (*list2_ptr).length;

        // Create a new list with enough capacity
        let result_list = list_with_capacity(list1_len + list2_len);
        if result_list.is_null() {
            return ptr::null_mut();
        }

        // Check if the data pointers are valid
        if !(*list1_ptr).data.is_null() {
            // Copy elements from the first list
            for i in 0..list1_len as usize {
                let value_ptr = *(*list1_ptr).data.add(i);

                if !value_ptr.is_null() {
                    // Copy the value
                    let value = *(value_ptr as *const i64);
                    let new_ptr = Box::into_raw(Box::new(value)) as *mut c_void;

                    // Store it in the result list
                    *(*result_list).data.add((*result_list).length as usize) = new_ptr;
                    (*result_list).length += 1;
                }
            }
        }

        if !(*list2_ptr).data.is_null() {
            // Copy elements from the second list
            for i in 0..list2_len as usize {
                let value_ptr = *(*list2_ptr).data.add(i);

                if !value_ptr.is_null() {
                    // Copy the value
                    let value = *(value_ptr as *const i64);
                    let new_ptr = Box::into_raw(Box::new(value)) as *mut c_void;

                    // Store it in the result list
                    *(*result_list).data.add((*result_list).length as usize) = new_ptr;
                    (*result_list).length += 1;
                }
            }
        }

        result_list
    }
}

/// Repeat a list n times
#[unsafe(no_mangle)]
pub extern "C" fn list_repeat(list_ptr: *mut List, count: i64) -> *mut List {

    if list_ptr.is_null() || count <= 0 {
        // Return an empty list for invalid inputs
        return list_new();
    }

    unsafe {
        let list_len = (*list_ptr).length;

        // Create a new list with enough capacity
        let result_list = list_with_capacity(list_len * count);
        if result_list.is_null() {
            return ptr::null_mut();
        }

        // Check if the data pointer is valid
        if (*list_ptr).data.is_null() {
            // Nothing to repeat
            return result_list;
        }

        // Repeat the list count times
        for _ in 0..count {
            for i in 0..list_len as usize {
                let value_ptr = *(*list_ptr).data.add(i);

                if !value_ptr.is_null() {
                    // Copy the value
                    let value = *(value_ptr as *const i64);
                    let new_ptr = Box::into_raw(Box::new(value)) as *mut c_void;

                    // Store it in the result list
                    *(*result_list).data.add((*result_list).length as usize) = new_ptr;
                    (*result_list).length += 1;
                }
            }
        }

        result_list
    }
}