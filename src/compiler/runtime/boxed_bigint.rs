// boxed_bigint.rs - Implementation of BigInt for Cheetah's BoxedAny
//
// This file implements a big integer type using the num-bigint crate.
// BigIntRaw is a struct that represents a multi-precision integer.

use std::ffi::{c_void, CString, c_char};
use std::ptr;
use libc::{free, malloc};
use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::{Zero, ToPrimitive};

use inkwell::context::Context;
use inkwell::module::Module;

use super::boxed_any::{BoxedAny, type_tags, boxed_any_none};

// Define the raw BigInt struct for FFI
#[repr(C)]
pub struct BigIntRaw {
    // Pointer to the BigInt instance
    ptr: *mut c_void,
}

// Create a new BigInt from a string
#[no_mangle]
pub extern "C" fn bigint_from_string(s: *const c_char) -> *mut BigIntRaw {
    if s.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s);
        let str_slice = c_str.to_str().unwrap_or("0");

        // Parse the string into a BigInt
        let bigint = match BigInt::from_str(str_slice) {
            Ok(n) => n,
            Err(_) => BigInt::zero(),
        };

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(bigint);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Create a new BigInt from an i64
#[no_mangle]
pub extern "C" fn bigint_from_i64(value: i64) -> *mut BigIntRaw {
    unsafe {
        // Create a BigInt from the i64 value
        let bigint = BigInt::from(value);

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(bigint);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Convert a BigInt to a string
#[no_mangle]
pub extern "C" fn bigint_to_string(bigint: *const BigIntRaw) -> *mut c_char {
    if bigint.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let bigint_ptr = (*bigint).ptr as *const BigInt;
        let bigint_ref = &*bigint_ptr;

        // Convert the BigInt to a string
        let s = bigint_ref.to_string();

        // Convert the Rust string to a C string
        match CString::new(s) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

// Free a BigInt
#[no_mangle]
pub extern "C" fn bigint_free(bigint: *mut BigIntRaw) {
    if !bigint.is_null() {
        unsafe {
            // Get the BigInt pointer
            let bigint_ptr = (*bigint).ptr as *mut BigInt;

            // Convert the raw pointer back to a Box and drop it
            if !bigint_ptr.is_null() {
                let _ = Box::from_raw(bigint_ptr);
            }

            // Free the BigIntRaw struct
            free(bigint as *mut c_void);
        }
    }
}

// Clone a BigInt
#[no_mangle]
pub extern "C" fn bigint_clone(bigint: *const BigIntRaw) -> *mut BigIntRaw {
    if bigint.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let bigint_ptr = (*bigint).ptr as *const BigInt;
        let bigint_ref = &*bigint_ptr;

        // Clone the BigInt
        let cloned = bigint_ref.clone();

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(cloned);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Add two BigInts
#[no_mangle]
pub extern "C" fn bigint_add(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let a_ptr = (*a).ptr as *const BigInt;
        let b_ptr = (*b).ptr as *const BigInt;

        let a_ref = &*a_ptr;
        let b_ref = &*b_ptr;

        // Add the BigInts
        let result = a_ref + b_ref;

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(result);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Subtract two BigInts
#[no_mangle]
pub extern "C" fn bigint_subtract(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let a_ptr = (*a).ptr as *const BigInt;
        let b_ptr = (*b).ptr as *const BigInt;

        let a_ref = &*a_ptr;
        let b_ref = &*b_ptr;

        // Subtract the BigInts
        let result = a_ref - b_ref;

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(result);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Multiply two BigInts
#[no_mangle]
pub extern "C" fn bigint_multiply(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let a_ptr = (*a).ptr as *const BigInt;
        let b_ptr = (*b).ptr as *const BigInt;

        let a_ref = &*a_ptr;
        let b_ref = &*b_ptr;

        // Multiply the BigInts
        let result = a_ref * b_ref;

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(result);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Divide two BigInts
#[no_mangle]
pub extern "C" fn bigint_divide(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let a_ptr = (*a).ptr as *const BigInt;
        let b_ptr = (*b).ptr as *const BigInt;

        let a_ref = &*a_ptr;
        let b_ref = &*b_ptr;

        // Check for division by zero
        if b_ref.is_zero() {
            // Return zero for division by zero
            return bigint_from_i64(0);
        }

        // Divide the BigInts
        let result = a_ref / b_ref;

        // Allocate memory for the BigIntRaw struct
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;

        // Create a Box<BigInt> and convert it to a raw pointer
        let boxed = Box::new(result);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Compare two BigInts
#[no_mangle]
pub extern "C" fn bigint_cmp(a: *const BigIntRaw, b: *const BigIntRaw) -> i32 {
    if a.is_null() || b.is_null() {
        return 0;
    }

    unsafe {
        let a_ptr = (*a).ptr as *const BigInt;
        let b_ptr = (*b).ptr as *const BigInt;

        let a_ref = &*a_ptr;
        let b_ref = &*b_ptr;

        // Compare the BigInts
        match a_ref.cmp(b_ref) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

// Convert a BigInt to an i64
#[no_mangle]
pub extern "C" fn bigint_to_i64(bigint: *const BigIntRaw) -> i64 {
    if bigint.is_null() {
        return 0;
    }

    unsafe {
        let bigint_ptr = (*bigint).ptr as *const BigInt;
        let bigint_ref = &*bigint_ptr;

        // Convert the BigInt to an i64
        bigint_ref.to_i64().unwrap_or(0)
    }
}

// Check if a BigInt fits in an i64
#[no_mangle]
pub extern "C" fn bigint_fits_i64(bigint: *const BigIntRaw) -> bool {
    if bigint.is_null() {
        return true;
    }

    unsafe {
        let bigint_ptr = (*bigint).ptr as *const BigInt;
        let bigint_ref = &*bigint_ptr;

        // Check if the BigInt fits in an i64
        bigint_ref.to_i64().is_some()
    }
}

// Create a BoxedAny from a BigInt
#[no_mangle]
pub extern "C" fn boxed_any_from_bigint(bigint: *mut BigIntRaw) -> *mut BoxedAny {
    if bigint.is_null() {
        return boxed_any_none();
    }

    unsafe {
        // Allocate memory for the BoxedAny struct
        let boxed = malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny;

        // Set the tag and data
        (*boxed).tag = type_tags::BIGINT;
        (*boxed).data.ptr_val = bigint as *mut c_void;

        boxed
    }
}

// Get the BigInt from a BoxedAny
#[no_mangle]
pub extern "C" fn bigint_from_boxed_any(boxed: *const BoxedAny) -> *mut BigIntRaw {
    if boxed.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        if (*boxed).tag != type_tags::BIGINT {
            return ptr::null_mut();
        }

        (*boxed).data.ptr_val as *mut BigIntRaw
    }
}

// Check if a value is a BigInt
#[no_mangle]
pub extern "C" fn is_bigint(boxed: *const BoxedAny) -> bool {
    if boxed.is_null() {
        return false;
    }

    unsafe {
        (*boxed).tag == type_tags::BIGINT
    }
}

// Calculate the nth Fibonacci number using BigInt
#[no_mangle]
pub extern "C" fn bigint_fib(n: u64) -> *mut BigIntRaw {
    if n <= 1 {
        return bigint_from_i64(n as i64);
    }

    unsafe {
        // Create BigInts for F(0) and F(1)
        let mut a = BigInt::from(0u64);
        let mut b = BigInt::from(1u64);

        // Calculate F(n) iteratively
        for _ in 2..=n {
            let temp = a.clone();
            a = b.clone();
            b = temp + b;
        }

        // Create a BigIntRaw from the result
        let raw = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;
        let boxed = Box::new(b);
        (*raw).ptr = Box::into_raw(boxed) as *mut c_void;

        raw
    }
}

// Register BigInt functions with LLVM
#[no_mangle]
pub extern "C" fn register_boxed_bigint_functions<'ctx>(
    _context: &'ctx Context,
    _module: &mut Module<'ctx>
) {
    // This function is a no-op in the new implementation
    // All the functions are already registered via #[no_mangle]
}
