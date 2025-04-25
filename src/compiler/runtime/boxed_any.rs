// boxed_any.rs - Implementation of the BoxedAny type for Cheetah's single blessed value layout
//
// This file implements a tagged union representation for all Cheetah values.
// BoxedAny is a struct that can represent any value in the Cheetah language,
// with a type tag to indicate what kind of value it contains.

use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use libc::{free, malloc};

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

/// Type tags for BoxedAny values
pub mod type_tags {
    pub const INT: i32 = 1;
    pub const FLOAT: i32 = 2;
    pub const BOOL: i32 = 3;
    pub const NONE: i32 = 4;
    pub const STRING: i32 = 5;
    pub const BYTES: i32 = 6;
    pub const LIST: i32 = 7;
    pub const TUPLE: i32 = 8;
    pub const DICT: i32 = 9;
    pub const SET: i32 = 10;
    pub const FUNCTION: i32 = 11;
    pub const CLASS: i32 = 12;
    pub const BIGINT: i32 = 13;
}

/// A tagged union representing any Cheetah value
#[repr(C)]
pub struct BoxedAny {
    /// Type tag indicating what kind of value this is
    pub tag: i32,
    /// The actual value data
    pub data: ValueData,
}

/// Union of all possible value types
#[repr(C)]
pub union ValueData {
    /// Integer value
    pub int_val: i64,
    /// Float value
    pub float_val: f64,
    /// Boolean value (stored as i8 for alignment)
    pub bool_val: i8,
    /// Pointer to heap-allocated data (for strings, lists, etc.)
    pub ptr_val: *mut c_void,
}

/// Create a BoxedAny from an integer
#[no_mangle]
pub extern "C" fn boxed_any_from_int(value: i64) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::INT;
        (*boxed).data.int_val = value;
    }
    boxed
}

/// Create a BoxedAny from a float
#[no_mangle]
pub extern "C" fn boxed_any_from_float(value: f64) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::FLOAT;
        (*boxed).data.float_val = value;
    }
    boxed
}

/// Create a BoxedAny from a boolean
#[no_mangle]
pub extern "C" fn boxed_any_from_bool(value: bool) -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::BOOL;
        (*boxed).data.bool_val = if value { 1 } else { 0 };
    }
    boxed
}

/// Create a BoxedAny representing None
#[no_mangle]
pub extern "C" fn boxed_any_none() -> *mut BoxedAny {
    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };
    unsafe {
        (*boxed).tag = type_tags::NONE;
        (*boxed).data.ptr_val = ptr::null_mut();
    }
    boxed
}

/// Create a BoxedAny from a C string
#[no_mangle]
pub extern "C" fn boxed_any_from_string(value: *const c_char) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_none();
    }

    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };

    unsafe {
        let c_str = CStr::from_ptr(value);
        let len = c_str.to_bytes().len();

        // Allocate memory for the string (including null terminator)
        let str_ptr = malloc(len + 1) as *mut c_char;

        // Copy the string content
        ptr::copy_nonoverlapping(value, str_ptr, len + 1);

        (*boxed).tag = type_tags::STRING;
        (*boxed).data.ptr_val = str_ptr as *mut c_void;
    }

    boxed
}

/// Free a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_free(value: *mut BoxedAny) {
    if value.is_null() {
        return;
    }

    unsafe {
        // Free any heap-allocated data based on the tag
        match (*value).tag {
            type_tags::STRING | type_tags::BYTES => {
                if !(*value).data.ptr_val.is_null() {
                    free((*value).data.ptr_val);
                }
            },
            type_tags::LIST => {
                // Free the list structure
                if !(*value).data.ptr_val.is_null() {
                    let list_ptr = (*value).data.ptr_val as *mut super::boxed_list::BoxedList;
                    super::boxed_list::boxed_list_free(list_ptr);
                }
            },
            type_tags::TUPLE => {
                // Free the tuple structure
                if !(*value).data.ptr_val.is_null() {
                    let tuple_ptr = (*value).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                    super::boxed_tuple::boxed_tuple_free(tuple_ptr);
                }
            },
            type_tags::DICT => {
                // Free the dictionary structure
                if !(*value).data.ptr_val.is_null() {
                    let dict_ptr = (*value).data.ptr_val as *mut super::boxed_dict::BoxedDict;
                    super::boxed_dict::boxed_dict_free(dict_ptr);
                }
            },
            type_tags::BIGINT => {
                // Free the big integer structure
                if !(*value).data.ptr_val.is_null() {
                    let bigint_ptr = (*value).data.ptr_val as *mut super::boxed_bigint::BigIntRaw;
                    super::boxed_bigint::bigint_free(bigint_ptr);
                }
            },
            _ => {
                // Other types don't have heap-allocated data
            }
        }

        // Free the BoxedAny itself
        free(value as *mut c_void);
    }
}

/// Get the type tag of a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_get_type(value: *const BoxedAny) -> i32 {
    if value.is_null() {
        return type_tags::NONE;
    }

    unsafe { (*value).tag }
}

/// Convert a BoxedAny to a string representation
#[no_mangle]
pub extern "C" fn boxed_any_to_string(value: *const BoxedAny) -> *mut c_char {
    if value.is_null() {
        return CString::new("None").unwrap().into_raw();
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => {
                let s = format!("{}", (*value).data.int_val);
                CString::new(s).unwrap().into_raw()
            },
            type_tags::FLOAT => {
                let s = format!("{}", (*value).data.float_val);
                CString::new(s).unwrap().into_raw()
            },
            type_tags::BOOL => {
                let s = if (*value).data.bool_val != 0 { "True" } else { "False" };
                CString::new(s).unwrap().into_raw()
            },
            type_tags::NONE => {
                CString::new("None").unwrap().into_raw()
            },
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    CString::new("").unwrap().into_raw()
                } else {
                    let c_str = CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    let s = c_str.to_str().unwrap_or("").to_string();
                    CString::new(s).unwrap().into_raw()
                }
            },
            type_tags::BIGINT => {
                if (*value).data.ptr_val.is_null() {
                    CString::new("0").unwrap().into_raw()
                } else {
                    let bigint_ptr = (*value).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                    let str_ptr = super::boxed_bigint::bigint_to_string(bigint_ptr);
                    let c_str = CStr::from_ptr(str_ptr);
                    let s = c_str.to_str().unwrap_or("0").to_string();

                    // Free the temporary string
                    free(str_ptr as *mut c_void);

                    CString::new(s).unwrap().into_raw()
                }
            },
            _ => {
                let s = format!("<object at {:p}>", value);
                CString::new(s).unwrap().into_raw()
            }
        }
    }
}

/// Clone a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_clone(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => {
                boxed_any_from_int((*value).data.int_val)
            },
            type_tags::FLOAT => {
                boxed_any_from_float((*value).data.float_val)
            },
            type_tags::BOOL => {
                boxed_any_from_bool((*value).data.bool_val != 0)
            },
            type_tags::NONE => {
                boxed_any_none()
            },
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    boxed_any_from_string(ptr::null())
                } else {
                    // Create a proper deep copy of the string
                    let c_str = std::ffi::CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    let len = c_str.to_bytes().len();

                    // Allocate memory for the string (including null terminator)
                    let str_ptr = malloc(len + 1) as *mut c_char;

                    // Copy the string content
                    ptr::copy_nonoverlapping((*value).data.ptr_val as *const c_char, str_ptr, len + 1);

                    // Create a new BoxedAny with the copied string
                    let boxed = malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny;
                    (*boxed).tag = type_tags::STRING;
                    (*boxed).data.ptr_val = str_ptr as *mut c_void;

                    boxed
                }
            },
            type_tags::LIST => {
                // Deep clone a list
                let list_ptr = (*value).data.ptr_val as *mut super::boxed_list::BoxedList;
                let length = super::boxed_list::boxed_list_len(list_ptr);
                let new_list = super::boxed_list::boxed_list_with_capacity(length);

                // Clone each element
                for i in 0..length {
                    let item = super::boxed_list::boxed_list_get(list_ptr, i);
                    if !item.is_null() {
                        let item_clone = boxed_any_clone(item);
                        super::boxed_list::boxed_list_append(new_list, item_clone);
                    } else {
                        super::boxed_list::boxed_list_append(new_list, boxed_any_none());
                    }
                }

                super::boxed_list::boxed_any_from_list(new_list)
            },
            type_tags::TUPLE => {
                // Deep clone a tuple
                let tuple_ptr = (*value).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                let new_tuple = super::boxed_tuple::boxed_tuple_clone(tuple_ptr);
                super::boxed_tuple::boxed_any_from_tuple(new_tuple)
            },
            type_tags::DICT => {
                // Deep clone a dictionary
                let dict_ptr = (*value).data.ptr_val as *mut super::boxed_dict::BoxedDict;
                let keys = super::boxed_dict::boxed_dict_keys(dict_ptr);
                let length = super::boxed_list::boxed_list_len(keys);
                let new_dict = super::boxed_dict::boxed_dict_with_capacity(length);

                // Clone each key-value pair
                for i in 0..length {
                    let key = super::boxed_list::boxed_list_get(keys, i);
                    if !key.is_null() {
                        let val = super::boxed_dict::boxed_dict_get(dict_ptr, key);
                        let key_clone = boxed_any_clone(key);
                        let val_clone = if !val.is_null() { boxed_any_clone(val) } else { boxed_any_none() };
                        super::boxed_dict::boxed_dict_set(new_dict, key_clone, val_clone);
                    }
                }

                // Free the keys list
                super::boxed_list::boxed_list_free(keys);

                super::boxed_dict::boxed_any_from_dict(new_dict)
            },
            type_tags::BIGINT => {
                // Deep clone a big integer
                let bigint_ptr = (*value).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let new_bigint = super::boxed_bigint::bigint_clone(bigint_ptr);
                super::boxed_bigint::boxed_any_from_bigint(new_bigint)
            },
            _ => {
                // For other types, create a shallow copy
                let boxed = malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny;
                ptr::copy_nonoverlapping(value, boxed, 1);
                boxed
            }
        }
    }
}

// Basic arithmetic operations

/// Add two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_add(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let x = (*a).data.int_val;
                let y = (*b).data.int_val;

                // Check for overflow
                match x.checked_add(y) {
                    Some(result) => boxed_any_from_int(result),
                    None => {
                        // Overflow occurred, promote to BigInt
                        let big_a = super::boxed_bigint::bigint_from_i64(x);
                        let big_b = super::boxed_bigint::bigint_from_i64(y);
                        let big_result = super::boxed_bigint::bigint_add(big_a, big_b);

                        // Free the temporary BigInts
                        super::boxed_bigint::bigint_free(big_a);
                        super::boxed_bigint::bigint_free(big_b);

                        super::boxed_bigint::boxed_any_from_bigint(big_result)
                    }
                }
            },
            (type_tags::BIGINT, type_tags::BIGINT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_add(big_a, big_b);
                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::BIGINT, type_tags::INT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = super::boxed_bigint::bigint_from_i64((*b).data.int_val);
                let big_result = super::boxed_bigint::bigint_add(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_b);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::INT, type_tags::BIGINT) => {
                let big_a = super::boxed_bigint::bigint_from_i64((*a).data.int_val);
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_add(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_a);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                let result = (*a).data.float_val + (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                let result = (*a).data.int_val as f64 + (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                let result = (*a).data.float_val + (*b).data.int_val as f64;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::BIGINT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let b_str = super::boxed_bigint::bigint_to_string(big_b);
                let b_float = std::ffi::CStr::from_ptr(b_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                libc::free(b_str as *mut c_void);

                let result = (*a).data.float_val + b_float;
                boxed_any_from_float(result)
            },
            (type_tags::BIGINT, type_tags::FLOAT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let a_str = super::boxed_bigint::bigint_to_string(big_a);
                let a_float = std::ffi::CStr::from_ptr(a_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                libc::free(a_str as *mut c_void);

                let result = a_float + (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::STRING, type_tags::STRING) => {
                // String concatenation
                if (*a).data.ptr_val.is_null() && (*b).data.ptr_val.is_null() {
                    // Both strings are empty
                    return boxed_any_from_string(CString::new("").unwrap().as_ptr());
                }

                if (*a).data.ptr_val.is_null() {
                    // First string is empty, return a copy of the second
                    return boxed_any_from_string((*b).data.ptr_val as *const c_char);
                }

                if (*b).data.ptr_val.is_null() {
                    // Second string is empty, return a copy of the first
                    return boxed_any_from_string((*a).data.ptr_val as *const c_char);
                }

                // Both strings have content, concatenate them
                let str_a = CStr::from_ptr((*a).data.ptr_val as *const c_char);
                let str_b = CStr::from_ptr((*b).data.ptr_val as *const c_char);

                let a_bytes = str_a.to_bytes();
                let b_bytes = str_b.to_bytes();

                // Allocate memory for the concatenated string (including null terminator)
                let total_len = a_bytes.len() + b_bytes.len();
                let result_ptr = malloc(total_len + 1) as *mut c_char;

                // Copy the first string
                ptr::copy_nonoverlapping(a_bytes.as_ptr(), result_ptr as *mut u8, a_bytes.len());

                // Copy the second string
                ptr::copy_nonoverlapping(b_bytes.as_ptr(), (result_ptr as *mut u8).add(a_bytes.len()), b_bytes.len());

                // Add null terminator
                *((result_ptr as *mut u8).add(total_len)) = 0;

                // Create a new BoxedAny with the concatenated string
                let boxed = malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny;
                (*boxed).tag = type_tags::STRING;
                (*boxed).data.ptr_val = result_ptr as *mut c_void;

                boxed
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Subtract two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_subtract(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let x = (*a).data.int_val;
                let y = (*b).data.int_val;

                // Check for overflow
                match x.checked_sub(y) {
                    Some(result) => boxed_any_from_int(result),
                    None => {
                        // Overflow occurred, promote to BigInt
                        let big_a = super::boxed_bigint::bigint_from_i64(x);
                        let big_b = super::boxed_bigint::bigint_from_i64(y);
                        let big_result = super::boxed_bigint::bigint_subtract(big_a, big_b);

                        // Free the temporary BigInts
                        super::boxed_bigint::bigint_free(big_a);
                        super::boxed_bigint::bigint_free(big_b);

                        super::boxed_bigint::boxed_any_from_bigint(big_result)
                    }
                }
            },
            (type_tags::BIGINT, type_tags::BIGINT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_subtract(big_a, big_b);
                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::BIGINT, type_tags::INT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = super::boxed_bigint::bigint_from_i64((*b).data.int_val);
                let big_result = super::boxed_bigint::bigint_subtract(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_b);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::INT, type_tags::BIGINT) => {
                let big_a = super::boxed_bigint::bigint_from_i64((*a).data.int_val);
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_subtract(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_a);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                let result = (*a).data.float_val - (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                let result = (*a).data.int_val as f64 - (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                let result = (*a).data.float_val - (*b).data.int_val as f64;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::BIGINT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let b_str = super::boxed_bigint::bigint_to_string(big_b);
                let b_float = std::ffi::CStr::from_ptr(b_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(b_str as *mut c_void);

                let result = (*a).data.float_val - b_float;
                boxed_any_from_float(result)
            },
            (type_tags::BIGINT, type_tags::FLOAT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let a_str = super::boxed_bigint::bigint_to_string(big_a);
                let a_float = std::ffi::CStr::from_ptr(a_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(a_str as *mut c_void);

                let result = a_float - (*b).data.float_val;
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Multiply two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_multiply(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let x = (*a).data.int_val;
                let y = (*b).data.int_val;

                // Check for overflow
                match x.checked_mul(y) {
                    Some(result) => boxed_any_from_int(result),
                    None => {
                        // Overflow occurred, promote to BigInt
                        let big_a = super::boxed_bigint::bigint_from_i64(x);
                        let big_b = super::boxed_bigint::bigint_from_i64(y);
                        let big_result = super::boxed_bigint::bigint_multiply(big_a, big_b);

                        // Free the temporary BigInts
                        super::boxed_bigint::bigint_free(big_a);
                        super::boxed_bigint::bigint_free(big_b);

                        super::boxed_bigint::boxed_any_from_bigint(big_result)
                    }
                }
            },
            (type_tags::BIGINT, type_tags::BIGINT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_multiply(big_a, big_b);
                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::BIGINT, type_tags::INT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = super::boxed_bigint::bigint_from_i64((*b).data.int_val);
                let big_result = super::boxed_bigint::bigint_multiply(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_b);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::INT, type_tags::BIGINT) => {
                let big_a = super::boxed_bigint::bigint_from_i64((*a).data.int_val);
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_multiply(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_a);

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                let result = (*a).data.float_val * (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                let result = (*a).data.int_val as f64 * (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                let result = (*a).data.float_val * (*b).data.int_val as f64;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::BIGINT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let b_str = super::boxed_bigint::bigint_to_string(big_b);
                let b_float = std::ffi::CStr::from_ptr(b_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(b_str as *mut c_void);

                let result = (*a).data.float_val * b_float;
                boxed_any_from_float(result)
            },
            (type_tags::BIGINT, type_tags::FLOAT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let a_str = super::boxed_bigint::bigint_to_string(big_a);
                let a_float = std::ffi::CStr::from_ptr(a_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(a_str as *mut c_void);

                let result = a_float * (*b).data.float_val;
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Divide two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_divide(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Division by zero
                    return boxed_any_none();
                }

                // Check if the division results in a whole number
                if (*a).data.int_val % (*b).data.int_val == 0 {
                    let result = (*a).data.int_val / (*b).data.int_val;
                    boxed_any_from_int(result)
                } else {
                    // Convert to float for non-integer division
                    let result = (*a).data.int_val as f64 / (*b).data.int_val as f64;
                    boxed_any_from_float(result)
                }
            },
            (type_tags::BIGINT, type_tags::BIGINT) => {
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_divide(big_a, big_b);

                if big_result.is_null() {
                    // Division by zero or error
                    return boxed_any_none();
                }

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::BIGINT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Division by zero
                    return boxed_any_none();
                }

                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_b = super::boxed_bigint::bigint_from_i64((*b).data.int_val);
                let big_result = super::boxed_bigint::bigint_divide(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_b);

                if big_result.is_null() {
                    // Error in division
                    return boxed_any_none();
                }

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::INT, type_tags::BIGINT) => {
                let big_a = super::boxed_bigint::bigint_from_i64((*a).data.int_val);
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let big_result = super::boxed_bigint::bigint_divide(big_a, big_b);

                // Free the temporary BigInt
                super::boxed_bigint::bigint_free(big_a);

                if big_result.is_null() {
                    // Division by zero or error
                    return boxed_any_none();
                }

                super::boxed_bigint::boxed_any_from_bigint(big_result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = (*a).data.float_val / (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = (*a).data.int_val as f64 / (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = (*a).data.float_val / (*b).data.int_val as f64;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::BIGINT) => {
                // Convert BigInt to float (may lose precision for very large values)
                let big_b = (*b).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let b_str = super::boxed_bigint::bigint_to_string(big_b);
                let b_float = std::ffi::CStr::from_ptr(b_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(b_str as *mut c_void);

                if b_float == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }

                let result = (*a).data.float_val / b_float;
                boxed_any_from_float(result)
            },
            (type_tags::BIGINT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }

                // Convert BigInt to float (may lose precision for very large values)
                let big_a = (*a).data.ptr_val as *const super::boxed_bigint::BigIntRaw;
                let a_str = super::boxed_bigint::bigint_to_string(big_a);
                let a_float = std::ffi::CStr::from_ptr(a_str)
                    .to_str()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);

                // Free the temporary string
                free(a_str as *mut c_void);

                let result = a_float / (*b).data.float_val;
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

// Comparison operations

/// Compare two BoxedAny values for equality
#[no_mangle]
pub extern "C" fn boxed_any_equals(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }

    unsafe {
        if (*a).tag != (*b).tag {
            // Special case for numeric types
            match ((*a).tag, (*b).tag) {
                (type_tags::INT, type_tags::FLOAT) => {
                    return (*a).data.int_val as f64 == (*b).data.float_val;
                },
                (type_tags::FLOAT, type_tags::INT) => {
                    return (*a).data.float_val == (*b).data.int_val as f64;
                },
                _ => {
                    return false;
                },
            }
        }

        match (*a).tag {
            type_tags::INT => {
                (*a).data.int_val == (*b).data.int_val
            },
            type_tags::FLOAT => {
                (*a).data.float_val == (*b).data.float_val
            },
            type_tags::BOOL => {
                (*a).data.bool_val == (*b).data.bool_val
            },
            type_tags::NONE => {
                true // None equals None
            },
            type_tags::STRING => {
                if (*a).data.ptr_val.is_null() && (*b).data.ptr_val.is_null() {
                    true
                } else if (*a).data.ptr_val.is_null() || (*b).data.ptr_val.is_null() {
                    false
                } else {
                    let str_a = CStr::from_ptr((*a).data.ptr_val as *const c_char);
                    let str_b = CStr::from_ptr((*b).data.ptr_val as *const c_char);
                    let a_bytes = str_a.to_bytes();
                    let b_bytes = str_b.to_bytes();
                    a_bytes == b_bytes
                }
            },
            _ => {
                // For other types, compare pointer values
                (*a).data.ptr_val == (*b).data.ptr_val
            }
        }
    }
}

/// Convert a BoxedAny to a boolean value
#[no_mangle]
pub extern "C" fn boxed_any_to_bool(value: *const BoxedAny) -> bool {
    if value.is_null() {
        return false;
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => (*value).data.int_val != 0,
            type_tags::FLOAT => (*value).data.float_val != 0.0,
            type_tags::BOOL => (*value).data.bool_val != 0,
            type_tags::NONE => false,
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    false
                } else {
                    let c_str = CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    !c_str.to_bytes().is_empty()
                }
            },
            _ => true, // Other objects are truthy by default
        }
    }
}

/// Logical AND operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_and(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    // Short-circuit evaluation: if a is falsy, return a
    if !boxed_any_to_bool(a) {
        return boxed_any_clone(a);
    }

    // Otherwise, return b
    boxed_any_clone(b)
}

/// Logical OR operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_or(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    // Short-circuit evaluation: if a is truthy, return a
    if boxed_any_to_bool(a) {
        return boxed_any_clone(a);
    }

    // Otherwise, return b
    boxed_any_clone(b)
}

/// Logical NOT operation on a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_not(value: *const BoxedAny) -> *mut BoxedAny {
    let result = !boxed_any_to_bool(value);
    boxed_any_from_bool(result)
}

/// Convert a BoxedAny to an integer value
#[no_mangle]
pub extern "C" fn boxed_any_to_int(value: *const BoxedAny) -> i64 {
    if value.is_null() {
        return 0;
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => (*value).data.int_val,
            type_tags::FLOAT => (*value).data.float_val as i64,
            type_tags::BOOL => if (*value).data.bool_val != 0 { 1 } else { 0 },
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    0
                } else {
                    let c_str = CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    if let Ok(s) = c_str.to_str() {
                        s.parse::<i64>().unwrap_or(0)
                    } else {
                        0
                    }
                }
            },
            _ => 0,
        }
    }
}

/// Convert a BoxedAny to a float value
#[no_mangle]
pub extern "C" fn boxed_any_to_float(value: *const BoxedAny) -> f64 {
    if value.is_null() {
        return 0.0;
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => (*value).data.int_val as f64,
            type_tags::FLOAT => (*value).data.float_val,
            type_tags::BOOL => if (*value).data.bool_val != 0 { 1.0 } else { 0.0 },
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    0.0
                } else {
                    let c_str = CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    if let Ok(s) = c_str.to_str() {
                        s.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
            },
            _ => 0.0,
        }
    }
}

// Type conversion functions

/// Convert a BoxedAny to an integer BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_int(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_from_int(0);
    }

    boxed_any_from_int(boxed_any_to_int(value))
}

/// Convert a BoxedAny to a float BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_float(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_from_float(0.0);
    }

    boxed_any_from_float(boxed_any_to_float(value))
}

/// Convert a BoxedAny to a boolean BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_bool(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_from_bool(false);
    }

    boxed_any_from_bool(boxed_any_to_bool(value))
}

/// Convert a BoxedAny to a string BoxedAny
#[no_mangle]
pub extern "C" fn boxed_any_as_string(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_from_string(CString::new("None").unwrap().as_ptr());
    }

    let c_str = boxed_any_to_string(value);
    let result = boxed_any_from_string(c_str);

    // Free the temporary C string
    unsafe {
        let _ = CString::from_raw(c_str);
    }

    result
}

/// Register BoxedAny functions in the module
pub fn register_boxed_any_functions<'ctx>(context: &'ctx Context, module: &mut Module<'ctx>) {
    // Define external functions for BoxedAny operations
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();
    let char_ptr_type = context.ptr_type(AddressSpace::default());
    let void_ptr_type = context.ptr_type(AddressSpace::default());

    // Define BoxedAny pointer type
    let boxed_any_ptr_type = void_ptr_type;

    // Creation functions
    let boxed_any_from_int_type = boxed_any_ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("boxed_any_from_int", boxed_any_from_int_type, None);

    let boxed_any_from_float_type = boxed_any_ptr_type.fn_type(&[f64_type.into()], false);
    module.add_function("boxed_any_from_float", boxed_any_from_float_type, None);

    let boxed_any_from_bool_type = boxed_any_ptr_type.fn_type(&[bool_type.into()], false);
    module.add_function("boxed_any_from_bool", boxed_any_from_bool_type, None);

    let boxed_any_none_type = boxed_any_ptr_type.fn_type(&[], false);
    module.add_function("boxed_any_none", boxed_any_none_type, None);

    let boxed_any_from_string_type = boxed_any_ptr_type.fn_type(&[char_ptr_type.into()], false);
    module.add_function("boxed_any_from_string", boxed_any_from_string_type, None);

    // Memory management functions
    let boxed_any_free_type = void_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_free", boxed_any_free_type, None);

    let boxed_any_clone_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_clone", boxed_any_clone_type, None);

    // Type information functions
    let boxed_any_get_type_type = i32_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_get_type", boxed_any_get_type_type, None);

    // Conversion functions
    let boxed_any_to_string_type = char_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_to_string", boxed_any_to_string_type, None);

    let boxed_any_to_bool_type = bool_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_to_bool", boxed_any_to_bool_type, None);

    let boxed_any_to_int_type = i64_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_to_int", boxed_any_to_int_type, None);

    let boxed_any_to_float_type = f64_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_to_float", boxed_any_to_float_type, None);

    // Type conversion functions
    let boxed_any_as_int_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_as_int", boxed_any_as_int_type, None);

    let boxed_any_as_float_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_as_float", boxed_any_as_float_type, None);

    let boxed_any_as_bool_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_as_bool", boxed_any_as_bool_type, None);

    let boxed_any_as_string_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_as_string", boxed_any_as_string_type, None);

    // Arithmetic operations
    let binary_op_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()], false);

    module.add_function("boxed_any_add", binary_op_type, None);
    module.add_function("boxed_any_subtract", binary_op_type, None);
    module.add_function("boxed_any_multiply", binary_op_type, None);
    module.add_function("boxed_any_divide", binary_op_type, None);

    // Comparison operations
    let equals_type = bool_type.fn_type(&[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_equals", equals_type, None);

    // Logical operations
    module.add_function("boxed_any_and", binary_op_type, None);
    module.add_function("boxed_any_or", binary_op_type, None);

    let unary_op_type = boxed_any_ptr_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_not", unary_op_type, None);

    // Length function
    let len_type = i64_type.fn_type(&[boxed_any_ptr_type.into()], false);
    module.add_function("boxed_any_len", len_type, None);
}

/// Get the length of a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_len(value: *const BoxedAny) -> i64 {
    if value.is_null() {
        return 0;
    }

    unsafe {
        match (*value).tag {
            type_tags::STRING => {
                if (*value).data.ptr_val.is_null() {
                    0
                } else {
                    let c_str = CStr::from_ptr((*value).data.ptr_val as *const c_char);
                    c_str.to_bytes().len() as i64
                }
            },
            type_tags::LIST => {
                if (*value).data.ptr_val.is_null() {
                    0
                } else {
                    // Call the boxed_list_len function
                    super::boxed_list::boxed_list_len((*value).data.ptr_val as *mut super::boxed_list::BoxedList)
                }
            },
            type_tags::DICT => {
                if (*value).data.ptr_val.is_null() {
                    0
                } else {
                    // Call the boxed_dict_len function
                    super::boxed_dict::boxed_dict_len((*value).data.ptr_val as *mut super::boxed_dict::BoxedDict)
                }
            },
            _ => 0, // Other types don't have a length
        }
    }
}
