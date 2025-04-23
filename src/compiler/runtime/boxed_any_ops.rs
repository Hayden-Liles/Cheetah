// boxed_any_ops.rs - Additional operations for BoxedAny values

// No external imports needed for these operations

use super::boxed_any::{BoxedAny, type_tags, boxed_any_from_int, boxed_any_from_float, boxed_any_none};

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

/// Register all BoxedAny operation functions in the module
pub fn register_boxed_any_ops_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    // Get the BoxedAny pointer type
    let boxed_any_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // Function type for binary operations: (BoxedAny*, BoxedAny*) -> BoxedAny*
    let binary_op_fn_type = boxed_any_ptr_type.fn_type(
        &[boxed_any_ptr_type.into(), boxed_any_ptr_type.into()],
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
}
