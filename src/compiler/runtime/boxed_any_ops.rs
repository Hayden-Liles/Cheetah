// boxed_any_ops.rs - Additional operations for BoxedAny values

// No external imports needed for these operations

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
            _ => {
                // Other types don't support item assignment
                // We could print an error message here, but for now we'll just silently fail
            }
        }
    }
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

    // Register function to set an item in a container
    module.add_function("boxed_any_set_item", set_item_fn_type, None);
}
