// boxed_any_ops.rs - Additional operations for BoxedAny values

use super::boxed_any::{BoxedAny, type_tags, boxed_any_from_int, boxed_any_from_float, boxed_any_from_bool, boxed_any_none};

/// Floor division operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_floor_div(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
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
                let result = (*a).data.int_val / (*b).data.int_val;
                boxed_any_from_int(result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = ((*a).data.float_val / (*b).data.float_val).floor();
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = (((*a).data.int_val as f64) / (*b).data.float_val).floor();
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Division by zero
                    return boxed_any_none();
                }
                let result = ((*a).data.float_val / ((*b).data.int_val as f64)).floor();
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Modulo operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_modulo(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Modulo by zero
                    return boxed_any_none();
                }
                let result = (*a).data.int_val % (*b).data.int_val;
                boxed_any_from_int(result)
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Modulo by zero
                    return boxed_any_none();
                }
                let result = (*a).data.float_val % (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                if (*b).data.float_val == 0.0 {
                    // Modulo by zero
                    return boxed_any_none();
                }
                let result = ((*a).data.int_val as f64) % (*b).data.float_val;
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                if (*b).data.int_val == 0 {
                    // Modulo by zero
                    return boxed_any_none();
                }
                let result = (*a).data.float_val % ((*b).data.int_val as f64);
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Power operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_power(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                // For integer exponentiation, we need to handle special cases
                if (*b).data.int_val < 0 {
                    // Negative exponent with integer base results in float
                    let base = (*a).data.int_val as f64;
                    let exp = (*b).data.int_val as f64;
                    let result = base.powf(exp);
                    boxed_any_from_float(result)
                } else {
                    // Positive exponent can be computed with integer power
                    let mut result = 1i64;
                    let mut base = (*a).data.int_val;
                    let mut exp = (*b).data.int_val;

                    while exp > 0 {
                        if exp & 1 == 1 {
                            result *= base;
                        }
                        base *= base;
                        exp >>= 1;
                    }

                    boxed_any_from_int(result)
                }
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                let result = (*a).data.float_val.powf((*b).data.float_val);
                boxed_any_from_float(result)
            },
            (type_tags::INT, type_tags::FLOAT) => {
                let result = ((*a).data.int_val as f64).powf((*b).data.float_val);
                boxed_any_from_float(result)
            },
            (type_tags::FLOAT, type_tags::INT) => {
                let result = (*a).data.float_val.powi((*b).data.int_val as i32);
                boxed_any_from_float(result)
            },
            _ => {
                // Type error
                boxed_any_none()
            }
        }
    }
}

/// Bitwise OR operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_bit_or(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let result = (*a).data.int_val | (*b).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - bitwise operations only work on integers
                boxed_any_none()
            }
        }
    }
}

/// Bitwise AND operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_bit_and(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let result = (*a).data.int_val & (*b).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - bitwise operations only work on integers
                boxed_any_none()
            }
        }
    }
}

/// Bitwise XOR operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_bit_xor(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                let result = (*a).data.int_val ^ (*b).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - bitwise operations only work on integers
                boxed_any_none()
            }
        }
    }
}

/// Left shift operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_lshift(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                if (*b).data.int_val < 0 {
                    // Negative shift is an error
                    return boxed_any_none();
                }
                let result = (*a).data.int_val << (*b).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - shift operations only work on integers
                boxed_any_none()
            }
        }
    }
}

/// Right shift operation on two BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_rshift(a: *const BoxedAny, b: *const BoxedAny) -> *mut BoxedAny {
    if a.is_null() || b.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                if (*b).data.int_val < 0 {
                    // Negative shift is an error
                    return boxed_any_none();
                }
                let result = (*a).data.int_val >> (*b).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - shift operations only work on integers
                boxed_any_none()
            }
        }
    }
}

/// Less than comparison for BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_less_than(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                (*a).data.int_val < (*b).data.int_val
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                (*a).data.float_val < (*b).data.float_val
            },
            (type_tags::INT, type_tags::FLOAT) => {
                ((*a).data.int_val as f64) < (*b).data.float_val
            },
            (type_tags::FLOAT, type_tags::INT) => {
                (*a).data.float_val < ((*b).data.int_val as f64)
            },
            (type_tags::STRING, type_tags::STRING) => {
                if (*a).data.ptr_val.is_null() {
                    return !(*b).data.ptr_val.is_null(); // null < non-null
                }
                if (*b).data.ptr_val.is_null() {
                    return false; // non-null !< null
                }
                let str_a = std::ffi::CStr::from_ptr((*a).data.ptr_val as *const std::os::raw::c_char);
                let str_b = std::ffi::CStr::from_ptr((*b).data.ptr_val as *const std::os::raw::c_char);
                str_a.to_bytes() < str_b.to_bytes()
            },
            _ => {
                // Type error or incomparable types
                false
            }
        }
    }
}

/// Less than or equal comparison for BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_less_than_or_equal(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null(); // null <= null is true
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                (*a).data.int_val <= (*b).data.int_val
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                (*a).data.float_val <= (*b).data.float_val
            },
            (type_tags::INT, type_tags::FLOAT) => {
                ((*a).data.int_val as f64) <= (*b).data.float_val
            },
            (type_tags::FLOAT, type_tags::INT) => {
                (*a).data.float_val <= ((*b).data.int_val as f64)
            },
            (type_tags::STRING, type_tags::STRING) => {
                if (*a).data.ptr_val.is_null() {
                    return true; // null <= anything
                }
                if (*b).data.ptr_val.is_null() {
                    return false; // non-null !<= null
                }
                let str_a = std::ffi::CStr::from_ptr((*a).data.ptr_val as *const std::os::raw::c_char);
                let str_b = std::ffi::CStr::from_ptr((*b).data.ptr_val as *const std::os::raw::c_char);
                str_a.to_bytes() <= str_b.to_bytes()
            },
            _ => {
                // Type error or incomparable types
                false
            }
        }
    }
}

/// Greater than comparison for BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_greater_than(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                (*a).data.int_val > (*b).data.int_val
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                (*a).data.float_val > (*b).data.float_val
            },
            (type_tags::INT, type_tags::FLOAT) => {
                ((*a).data.int_val as f64) > (*b).data.float_val
            },
            (type_tags::FLOAT, type_tags::INT) => {
                (*a).data.float_val > ((*b).data.int_val as f64)
            },
            (type_tags::STRING, type_tags::STRING) => {
                if (*a).data.ptr_val.is_null() {
                    return false; // null !> anything
                }
                if (*b).data.ptr_val.is_null() {
                    return true; // non-null > null
                }
                let str_a = std::ffi::CStr::from_ptr((*a).data.ptr_val as *const std::os::raw::c_char);
                let str_b = std::ffi::CStr::from_ptr((*b).data.ptr_val as *const std::os::raw::c_char);
                str_a.to_bytes() > str_b.to_bytes()
            },
            _ => {
                // Type error or incomparable types
                false
            }
        }
    }
}

/// Greater than or equal comparison for BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_greater_than_or_equal(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null(); // null >= null is true
    }

    unsafe {
        match ((*a).tag, (*b).tag) {
            (type_tags::INT, type_tags::INT) => {
                (*a).data.int_val >= (*b).data.int_val
            },
            (type_tags::FLOAT, type_tags::FLOAT) => {
                (*a).data.float_val >= (*b).data.float_val
            },
            (type_tags::INT, type_tags::FLOAT) => {
                ((*a).data.int_val as f64) >= (*b).data.float_val
            },
            (type_tags::FLOAT, type_tags::INT) => {
                (*a).data.float_val >= ((*b).data.int_val as f64)
            },
            (type_tags::STRING, type_tags::STRING) => {
                if (*b).data.ptr_val.is_null() {
                    return true; // anything >= null
                }
                if (*a).data.ptr_val.is_null() {
                    return false; // null !>= non-null
                }
                let str_a = std::ffi::CStr::from_ptr((*a).data.ptr_val as *const std::os::raw::c_char);
                let str_b = std::ffi::CStr::from_ptr((*b).data.ptr_val as *const std::os::raw::c_char);
                str_a.to_bytes() >= str_b.to_bytes()
            },
            _ => {
                // Type error or incomparable types
                false
            }
        }
    }
}

/// Not equal comparison for BoxedAny values
#[no_mangle]
pub extern "C" fn boxed_any_not_equals(a: *const BoxedAny, b: *const BoxedAny) -> bool {
    !super::boxed_any::boxed_any_equals(a, b)
}

/// Create a BoxedAny boolean from a comparison result
#[no_mangle]
pub extern "C" fn boxed_any_from_comparison(result: bool) -> *mut BoxedAny {
    boxed_any_from_bool(result)
}

/// Negate a BoxedAny value (unary minus)
#[no_mangle]
pub extern "C" fn boxed_any_negate(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => {
                let result = -(*value).data.int_val;
                boxed_any_from_int(result)
            },
            type_tags::FLOAT => {
                let result = -(*value).data.float_val;
                boxed_any_from_float(result)
            },
            _ => {
                // Type error - can only negate numeric types
                boxed_any_none()
            }
        }
    }
}

/// Bitwise NOT operation on a BoxedAny value (unary ~)
#[no_mangle]
pub extern "C" fn boxed_any_bitwise_not(value: *const BoxedAny) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_none();
    }

    unsafe {
        match (*value).tag {
            type_tags::INT => {
                let result = !(*value).data.int_val;
                boxed_any_from_int(result)
            },
            _ => {
                // Type error - can only perform bitwise NOT on integers
                boxed_any_none()
            }
        }
    }
}

/// Call a method on a BoxedAny value
#[no_mangle]
pub extern "C" fn boxed_any_call_method(obj: *mut BoxedAny, method_name: *const std::os::raw::c_char, args: *mut *mut BoxedAny, num_args: i32) -> *mut BoxedAny {
    if obj.is_null() || method_name.is_null() {
        return super::boxed_any::boxed_any_none();
    }

    let method_name_str = unsafe {
        let c_str = std::ffi::CStr::from_ptr(method_name);
        c_str.to_str().unwrap_or("")
    };

    unsafe {
        match (*obj).tag {
            type_tags::LIST => {
                let list_ptr = (*obj).data.ptr_val as *mut super::boxed_list::BoxedList;

                match method_name_str {
                    "append" => {
                        if num_args != 1 || args.is_null() {
                            return super::boxed_any::boxed_any_none();
                        }

                        let arg = *args;
                        if !arg.is_null() {
                            // Clone the value to avoid double-free issues
                            let value_clone = super::boxed_any::boxed_any_clone(arg);
                            super::boxed_list::boxed_list_append(list_ptr, value_clone);
                        } else {
                            // Append None if the argument is null
                            super::boxed_list::boxed_list_append(list_ptr, super::boxed_any::boxed_any_none());
                        }

                        // Return None as the result of append
                        return super::boxed_any::boxed_any_none();
                    },
                    "len" => {
                        if num_args != 0 {
                            return super::boxed_any::boxed_any_none();
                        }

                        let length = super::boxed_list::boxed_list_len(list_ptr);
                        return super::boxed_any::boxed_any_from_int(length as i64);
                    },
                    _ => {
                        // Unknown method
                        return super::boxed_any::boxed_any_none();
                    }
                }
            },
            type_tags::DICT => {
                let dict_ptr = (*obj).data.ptr_val as *mut super::boxed_dict::BoxedDict;
            
                match method_name_str {
                    // ----------------------------------------------------------
                    //                  read-only helpers
                    // ----------------------------------------------------------
                    "keys" => {
                        if num_args != 0 { return super::boxed_any::boxed_any_none(); }
            
                        let keys = super::boxed_dict::boxed_dict_keys(dict_ptr);
                        return super::boxed_list::boxed_any_from_list(keys);
                    }
            
                    "values" => {
                        if num_args != 0 { return super::boxed_any::boxed_any_none(); }
            
                        let vals = super::boxed_dict::boxed_dict_values(dict_ptr);
                        return super::boxed_list::boxed_any_from_list(vals);
                    }
            
                    "items" => {
                        if num_args != 0 { return super::boxed_any::boxed_any_none(); }
            
                        let items = super::boxed_dict::boxed_dict_items(dict_ptr);
                        return super::boxed_list::boxed_any_from_list(items);
                    }
            
                    // ----------------------------------------------------------
                    //                        get
                    // ----------------------------------------------------------
                    "get" => {
                        // Signature mimics Python: get(key[, default])
                        if num_args < 1 || num_args > 2 || args.is_null() {
                            return super::boxed_any::boxed_any_none();
                        }
            
                        let key      = *args;
                        let value    = super::boxed_dict::boxed_dict_get(dict_ptr, key);
                        let found    = !value.is_null()
                                    && (*value).tag != super::boxed_any::type_tags::NONE;
            
                        if found {
                            // Key exists → return a clone so caller owns it
                            return super::boxed_any::boxed_any_clone(value);
                        }
            
                        // Key missing: return default if provided, else None
                        if num_args == 2 {
                            let default_value = *args.add(1);
                            return super::boxed_any::boxed_any_clone(default_value);
                        }
            
                        return super::boxed_any::boxed_any_none();
                    }
            
                    // ----------------------------------------------------------
                    _ => {
                        // Unknown method name – return None (could raise later)
                        return super::boxed_any::boxed_any_none();
                    }
                }
            },
            type_tags::STRING => {
                let str_ptr = (*obj).data.ptr_val as *const std::os::raw::c_char;

                match method_name_str {
                    "strip" => {
                        if num_args > 1 {
                            return super::boxed_any::boxed_any_none();
                        }

                        if str_ptr.is_null() {
                            return super::boxed_any::boxed_any_from_string(std::ptr::null());
                        }

                        let c_str = std::ffi::CStr::from_ptr(str_ptr);
                        let str_bytes = c_str.to_bytes();

                        // Trim whitespace from the beginning
                        let start = str_bytes.iter()
                            .position(|&b| !b.is_ascii_whitespace())
                            .unwrap_or(str_bytes.len());

                        // Trim whitespace from the end
                        let end = str_bytes.iter()
                            .rposition(|&b| !b.is_ascii_whitespace())
                            .map(|i| i + 1)
                            .unwrap_or(start);

                        // Create a slice with the trimmed content
                        let trimmed = &str_bytes[start..end];

                        // Create a new string
                        let result = String::from_utf8_lossy(trimmed).to_string();
                        let c_result = std::ffi::CString::new(result).unwrap();
                        return super::boxed_any::boxed_any_from_string(c_result.as_ptr());
                    },
                    _ => {
                        // Unknown method
                        return super::boxed_any::boxed_any_none();
                    }
                }
            },
            _ => {
                // Type doesn't support method calls
                return super::boxed_any::boxed_any_none();
            }
        }
    }
}

/// Get an item from a container (list, dict, etc.)
#[no_mangle]
pub extern "C" fn boxed_any_get_item(container: *const BoxedAny, key: *const BoxedAny) -> *mut BoxedAny {
    if container.is_null() || key.is_null() {
        return super::boxed_any::boxed_any_none();
    }

    unsafe {
        match (*container).tag {
            type_tags::LIST => {
                // For lists, we expect the key to be an integer
                if (*key).tag == type_tags::INT {
                    let list_ptr = (*container).data.ptr_val as *mut super::boxed_list::BoxedList;
                    let index = (*key).data.int_val;

                    // Get the item from the list
                    let item = super::boxed_list::boxed_list_get(list_ptr, index);
                    if item.is_null() {
                        return super::boxed_any::boxed_any_none();
                    }
                    return super::boxed_any::boxed_any_clone(item);
                }
            },
            type_tags::DICT => {
                // For dictionaries, the key can be any type
                let dict_ptr = (*container).data.ptr_val as *mut super::boxed_dict::BoxedDict;

                // Get the item from the dictionary
                let item = super::boxed_dict::boxed_dict_get(dict_ptr, key as *mut BoxedAny);
                if !item.is_null() {
                    return super::boxed_any::boxed_any_clone(item);
                }
            },
            type_tags::TUPLE => {
                // For tuples, we expect the key to be an integer
                if (*key).tag == type_tags::INT {
                    let tuple_ptr = (*container).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                    let index = (*key).data.int_val;

                    // Get the item from the tuple
                    let item = super::boxed_tuple::boxed_tuple_get(tuple_ptr, index);
                    if !item.is_null() {
                        return super::boxed_any::boxed_any_clone(item);
                    }
                }
            },
            _ => {
                // Other types don't support item access
                // We could print an error message here, but for now we'll just return None
            }
        }
        super::boxed_any::boxed_any_none()
    }
}

/// Get a slice of a container (list, string, etc.)
#[no_mangle]
pub extern "C" fn boxed_any_slice(container: *const BoxedAny, start: *const BoxedAny, end: *const BoxedAny, step: *const BoxedAny) -> *mut BoxedAny {
    if container.is_null() {
        return super::boxed_any::boxed_any_none();
    }

    // Default values for start, end, and step
    let start_val = if start.is_null() { 0 } else { super::boxed_any::boxed_any_to_int(start) };
    let end_val = if end.is_null() { i64::MAX } else { super::boxed_any::boxed_any_to_int(end) };
    let step_val = if step.is_null() { 1 } else { super::boxed_any::boxed_any_to_int(step) };

    // Step cannot be zero
    if step_val == 0 {
        return super::boxed_any::boxed_any_none();
    }

    unsafe {
        match (*container).tag {
            type_tags::LIST => {
                let list_ptr = (*container).data.ptr_val as *mut super::boxed_list::BoxedList;
                let list_len = super::boxed_list::boxed_list_len(list_ptr);

                // Adjust negative indices
                let adjusted_start = if start_val < 0 { list_len + start_val } else { start_val };
                let adjusted_end = if end_val < 0 { list_len + end_val } else { end_val };

                // Clamp indices to valid range
                let clamped_start = adjusted_start.clamp(0, list_len);
                let clamped_end = adjusted_end.clamp(0, list_len);

                // Create a new list for the slice
                let result_list = super::boxed_list::boxed_list_new();

                // Copy elements to the new list
                let mut i = clamped_start;
                while (step_val > 0 && i < clamped_end) || (step_val < 0 && i > clamped_end) {
                    let item = super::boxed_list::boxed_list_get(list_ptr, i);
                    if !item.is_null() {
                        let item_clone = super::boxed_any::boxed_any_clone(item);
                        super::boxed_list::boxed_list_append(result_list, item_clone);
                    }
                    i += step_val;
                }

                // Create a BoxedAny from the list
                let boxed_result = super::boxed_list::boxed_any_from_list(result_list);
                return boxed_result;
            },
            type_tags::STRING => {
                let str_ptr = (*container).data.ptr_val as *const std::os::raw::c_char;
                if str_ptr.is_null() {
                    return super::boxed_any::boxed_any_from_string(std::ptr::null());
                }

                let c_str = std::ffi::CStr::from_ptr(str_ptr);
                let str_len = c_str.to_bytes().len() as i64;

                // Adjust negative indices
                let adjusted_start = if start_val < 0 { str_len + start_val } else { start_val };
                let adjusted_end = if end_val < 0 { str_len + end_val } else { end_val };

                // Clamp indices to valid range
                let clamped_start = adjusted_start.clamp(0, str_len);
                let clamped_end = adjusted_end.clamp(0, str_len);

                // Get the string slice
                let bytes = c_str.to_bytes();
                let mut result = String::new();

                // Copy characters to the new string
                let mut i = clamped_start;
                while (step_val > 0 && i < clamped_end) || (step_val < 0 && i > clamped_end) {
                    if i >= 0 && i < str_len {
                        result.push(bytes[i as usize] as char);
                    }
                    i += step_val;
                }

                // Create a BoxedAny from the string
                let c_result = std::ffi::CString::new(result).unwrap();
                let boxed_result = super::boxed_any::boxed_any_from_string(c_result.as_ptr());
                return boxed_result;
            },
            _ => {
                // Other types don't support slicing
                return super::boxed_any::boxed_any_none();
            }
        }
    }
}

/// Set an item in a container (list, dict, etc.)
#[no_mangle]
pub extern "C" fn boxed_any_set_item(container: *mut BoxedAny, key: *mut BoxedAny, value: *mut BoxedAny) {
    if container.is_null() || key.is_null() || value.is_null() {
        return;
    }

    unsafe {
        match (*container).tag {
            type_tags::LIST => {
                // For lists, we expect the key to be an integer
                if (*key).tag == type_tags::INT {
                    let list_ptr = (*container).data.ptr_val as *mut super::boxed_list::BoxedList;
                    let index = (*key).data.int_val;

                    // Clone the value to avoid double-free issues
                    let value_clone = super::boxed_any::boxed_any_clone(value);

                    // Set the item in the list
                    super::boxed_list::boxed_list_set(list_ptr, index, value_clone);
                }
            },
            type_tags::DICT => {
                // For dictionaries, the key can be any type
                let dict_ptr = (*container).data.ptr_val as *mut super::boxed_dict::BoxedDict;

                // Clone the key and value to avoid double-free issues
                let key_clone = super::boxed_any::boxed_any_clone(key);
                let value_clone = super::boxed_any::boxed_any_clone(value);

                // Set the item in the dictionary
                super::boxed_dict::boxed_dict_set(dict_ptr, key_clone, value_clone);
            },
            type_tags::TUPLE => {
                // For tuples, we expect the key to be an integer
                if (*key).tag == type_tags::INT {
                    let tuple_ptr = (*container).data.ptr_val as *mut super::boxed_tuple::BoxedTuple;
                    let index = (*key).data.int_val;

                    // Clone the value to avoid double-free issues
                    let value_clone = super::boxed_any::boxed_any_clone(value);

                    // Set the item in the tuple
                    super::boxed_tuple::boxed_tuple_set(tuple_ptr, index, value_clone);
                }
            },
            _ => {
                // Other types don't support item assignment
                // We could print an error message here, but for now we'll just silently fail
            }
        }
    }
}

/// Register BoxedAny operations for JIT execution
pub fn register_boxed_any_ops_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    // Binary operations
    if let Some(f) = module.get_function("boxed_any_add") {
        engine.add_global_mapping(&f, super::boxed_any::boxed_any_add as usize);
    }

    if let Some(f) = module.get_function("boxed_any_subtract") {
        engine.add_global_mapping(&f, super::boxed_any::boxed_any_subtract as usize);
    }

    if let Some(f) = module.get_function("boxed_any_multiply") {
        engine.add_global_mapping(&f, super::boxed_any::boxed_any_multiply as usize);
    }

    if let Some(f) = module.get_function("boxed_any_divide") {
        engine.add_global_mapping(&f, super::boxed_any::boxed_any_divide as usize);
    }

    if let Some(f) = module.get_function("boxed_any_floor_div") {
        engine.add_global_mapping(&f, boxed_any_floor_div as usize);
    }

    if let Some(f) = module.get_function("boxed_any_modulo") {
        engine.add_global_mapping(&f, boxed_any_modulo as usize);
    }

    if let Some(f) = module.get_function("boxed_any_power") {
        engine.add_global_mapping(&f, boxed_any_power as usize);
    }

    // Bitwise operations
    if let Some(f) = module.get_function("boxed_any_bit_or") {
        engine.add_global_mapping(&f, boxed_any_bit_or as usize);
    }

    if let Some(f) = module.get_function("boxed_any_bit_and") {
        engine.add_global_mapping(&f, boxed_any_bit_and as usize);
    }

    if let Some(f) = module.get_function("boxed_any_bit_xor") {
        engine.add_global_mapping(&f, boxed_any_bit_xor as usize);
    }

    if let Some(f) = module.get_function("boxed_any_lshift") {
        engine.add_global_mapping(&f, boxed_any_lshift as usize);
    }

    if let Some(f) = module.get_function("boxed_any_rshift") {
        engine.add_global_mapping(&f, boxed_any_rshift as usize);
    }

    if let Some(f) = module.get_function("boxed_any_bitwise_not") {
        engine.add_global_mapping(&f, boxed_any_bitwise_not as usize);
    }

    // Comparison operations
    if let Some(f) = module.get_function("boxed_any_equals") {
        engine.add_global_mapping(&f, super::boxed_any::boxed_any_equals as usize);
    }

    if let Some(f) = module.get_function("boxed_any_not_equals") {
        engine.add_global_mapping(&f, boxed_any_not_equals as usize);
    }

    if let Some(f) = module.get_function("boxed_any_less_than") {
        engine.add_global_mapping(&f, boxed_any_less_than as usize);
    }

    if let Some(f) = module.get_function("boxed_any_less_than_or_equal") {
        engine.add_global_mapping(&f, boxed_any_less_than_or_equal as usize);
    }

    if let Some(f) = module.get_function("boxed_any_greater_than") {
        engine.add_global_mapping(&f, boxed_any_greater_than as usize);
    }

    if let Some(f) = module.get_function("boxed_any_greater_than_or_equal") {
        engine.add_global_mapping(&f, boxed_any_greater_than_or_equal as usize);
    }

    if let Some(f) = module.get_function("boxed_any_from_comparison") {
        engine.add_global_mapping(&f, boxed_any_from_comparison as usize);
    }

    // Unary operations
    if let Some(f) = module.get_function("boxed_any_negate") {
        engine.add_global_mapping(&f, boxed_any_negate as usize);
    }

    // Container operations
    if let Some(f) = module.get_function("boxed_any_get_item") {
        engine.add_global_mapping(&f, boxed_any_get_item as usize);
    }

    if let Some(f) = module.get_function("boxed_any_set_item") {
        engine.add_global_mapping(&f, boxed_any_set_item as usize);
    }

    if let Some(f) = module.get_function("boxed_any_slice") {
        engine.add_global_mapping(&f, boxed_any_slice as usize);
    }

    if let Some(f) = module.get_function("boxed_any_call_method") {
        engine.add_global_mapping(&f, boxed_any_call_method as usize);
    }

    Ok(())
}

/// Register all BoxedAny operation functions in the module
pub fn register_boxed_any_ops_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    // Get the BoxedAny pointer type
    let boxed_any_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();
    let void_type = context.void_type();

    // Function type for binary operations: (BoxedAny*, BoxedAny*) -> BoxedAny*
    let binary_op_fn_type = boxed_any_ptr_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
        false,
    );

    // Function type for comparison operations: (BoxedAny*, BoxedAny*) -> bool
    let comparison_fn_type = bool_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
        false,
    );

    // Function type for creating a BoxedAny from a comparison result: (bool) -> BoxedAny*
    let from_comparison_fn_type = boxed_any_ptr_type.fn_type(
        &[bool_type.into()],
        false,
    );

    // Function type for setting an item: (BoxedAny*, BoxedAny*, BoxedAny*) -> void
    let set_item_fn_type = void_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
        false,
    );

    // Function type for getting an item: (BoxedAny*, BoxedAny*) -> BoxedAny*
    let get_item_fn_type = boxed_any_ptr_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
        false,
    );

    // Function type for slicing: (BoxedAny*, BoxedAny*, BoxedAny*, BoxedAny*) -> BoxedAny*
    let slice_fn_type = boxed_any_ptr_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into(), boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
        false,
    );

    // Register floor division function
    module.add_function("boxed_any_floor_div", binary_op_fn_type, None);

    // Register modulo function
    module.add_function("boxed_any_modulo", binary_op_fn_type, None);

    // Register power function
    module.add_function("boxed_any_power", binary_op_fn_type, None);

    // Register bitwise operations
    module.add_function("boxed_any_bit_or", binary_op_fn_type, None);
    module.add_function("boxed_any_bit_and", binary_op_fn_type, None);
    module.add_function("boxed_any_bit_xor", binary_op_fn_type, None);

    // Register shift operations
    module.add_function("boxed_any_lshift", binary_op_fn_type, None);
    module.add_function("boxed_any_rshift", binary_op_fn_type, None);

    // Register comparison operations
    module.add_function("boxed_any_less_than", comparison_fn_type, None);
    module.add_function("boxed_any_less_than_or_equal", comparison_fn_type, None);
    module.add_function("boxed_any_greater_than", comparison_fn_type, None);
    module.add_function("boxed_any_greater_than_or_equal", comparison_fn_type, None);
    module.add_function("boxed_any_not_equals", comparison_fn_type, None);

    // Register function to create a BoxedAny from a comparison result
    module.add_function("boxed_any_from_comparison", from_comparison_fn_type, None);

    // Function type for unary operations: (BoxedAny*) -> BoxedAny*
    let unary_op_fn_type = boxed_any_ptr_type.fn_type(
        &[boxed_any_ptr_type.into()],
        false,
    );

    // Register negate function
    module.add_function("boxed_any_negate", unary_op_fn_type, None);

    // Register bitwise NOT function
    module.add_function("boxed_any_bitwise_not", unary_op_fn_type, None);

    // Register function to get an item from a container
    module.add_function("boxed_any_get_item", get_item_fn_type, None);

    // Register function to set an item in a container
    module.add_function("boxed_any_set_item", set_item_fn_type, None);

    // Register function to slice a container
    module.add_function("boxed_any_slice", slice_fn_type, None);

    // Function type for method calls: (BoxedAny*, char*, BoxedAny**, i32) -> BoxedAny*
    let method_call_fn_type = boxed_any_ptr_type.fn_type(
        &[
            boxed_any_ptr_type.into(),
            context.ptr_type(inkwell::AddressSpace::default()).into(),
            context.ptr_type(inkwell::AddressSpace::default()).into(),
            context.i32_type().into(),
        ],
        false,
    );

    // Register method call function
    module.add_function("boxed_any_call_method", method_call_fn_type, None);
}
