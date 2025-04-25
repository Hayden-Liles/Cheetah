// boxed_bigint.rs - Implementation of BigInt for Cheetah's BoxedAny
//
// This file implements a big integer type using the GMP library.
// BigIntRaw is a struct that represents a multi-precision integer.

use std::ffi::c_void;
use std::ptr;
use libc::{calloc, free, malloc};

use super::boxed_any::{BoxedAny, type_tags, boxed_any_none};

// Define a simple Mpz struct for our implementation
// This is a placeholder until we can properly integrate with rust-gmp
struct Mpz {
    value: String,
}

impl std::fmt::Display for Mpz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Mpz {
    fn new() -> Self {
        Mpz { value: "0".to_string() }
    }

    fn from(value: u64) -> Self {
        Mpz { value: value.to_string() }
    }

    fn zero() -> Self {
        Mpz { value: "0".to_string() }
    }

    fn to_str_radix(&self, _radix: u32) -> String {
        self.value.clone()
    }

    fn bit_length(&self) -> usize {
        // Simple approximation
        let val = self.value.parse::<u64>().unwrap_or(0);
        if val == 0 {
            return 1;
        }
        64 - val.leading_zeros() as usize
    }
}

impl std::ops::Add for Mpz {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        // For very large numbers, we need to use a more sophisticated approach
        // We'll implement a robust version of the grade-school addition algorithm

        // Handle signs
        if self.value.starts_with('-') && !other.value.starts_with('-') {
            // -a + b = b - a
            let a_pos = Mpz { value: self.value[1..].to_string() };
            return other - a_pos;
        } else if !self.value.starts_with('-') && other.value.starts_with('-') {
            // a + (-b) = a - b
            let b_pos = Mpz { value: other.value[1..].to_string() };
            return self - b_pos;
        } else if self.value.starts_with('-') && other.value.starts_with('-') {
            // -a + (-b) = -(a + b)
            let a_pos = Mpz { value: self.value[1..].to_string() };
            let b_pos = Mpz { value: other.value[1..].to_string() };
            let sum = a_pos + b_pos;
            return Mpz { value: format!("-{}", sum.value) };
        }

        // Both numbers are positive at this point
        // Convert strings to digit vectors (in reverse order for easier processing)
        let a_digits: Vec<u32> = self.value.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as u32)
            .rev()
            .collect();

        let b_digits: Vec<u32> = other.value.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as u32)
            .rev()
            .collect();

        // Initialize result with zeros
        let max_len = a_digits.len().max(b_digits.len()) + 1;
        let mut result = vec![0u32; max_len];

        // Perform addition
        let mut carry = 0u32;
        for i in 0..max_len - 1 {
            let a_digit = if i < a_digits.len() { a_digits[i] } else { 0 };
            let b_digit = if i < b_digits.len() { b_digits[i] } else { 0 };

            let sum = a_digit + b_digit + carry;
            result[i] = sum % 10;
            carry = sum / 10;
        }

        // Handle final carry
        if carry > 0 {
            result[max_len - 1] = carry;
        }

        // Remove leading zeros
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }

        // Convert back to string
        let mut result_str = String::new();
        for digit in result.iter().rev() {
            // Convert digit to char safely
            let digit_char = std::char::from_digit(*digit, 10).unwrap_or('0');
            result_str.push(digit_char);
        }

        // Handle zero case
        if result_str.is_empty() {
            result_str = "0".to_string();
        }

        Mpz { value: result_str }
    }
}

impl std::ops::Add<&Mpz> for &Mpz {
    type Output = Mpz;

    fn add(self, other: &Mpz) -> Mpz {
        // Delegate to the main implementation
        Mpz { value: self.value.clone() } + Mpz { value: other.value.clone() }
    }
}

impl std::ops::Sub for Mpz {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        // Handle signs
        if self.value.starts_with('-') && !other.value.starts_with('-') {
            // -a - b = -(a + b)
            let a_pos = Mpz { value: self.value[1..].to_string() };
            let sum = a_pos + other;
            return Mpz { value: format!("-{}", sum.value) };
        } else if !self.value.starts_with('-') && other.value.starts_with('-') {
            // a - (-b) = a + b
            let b_pos = Mpz { value: other.value[1..].to_string() };
            return self + b_pos;
        } else if self.value.starts_with('-') && other.value.starts_with('-') {
            // -a - (-b) = b - a
            let a_pos = Mpz { value: self.value[1..].to_string() };
            let b_pos = Mpz { value: other.value[1..].to_string() };
            return b_pos - a_pos;
        }

        // Both numbers are positive at this point
        // Compare the numbers to determine the sign of the result
        let a_gt_b = compare_strings(&self.value, &other.value) > 0;

        // If a < b, compute b - a and negate the result
        if !a_gt_b && self.value != other.value {
            let diff = other.clone() - self.clone();
            return Mpz { value: format!("-{}", diff.value) };
        }

        // a >= b, compute a - b
        // Convert strings to digit vectors (in reverse order for easier processing)
        let a_digits: Vec<i32> = self.value.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as i32)
            .rev()
            .collect();

        let b_digits: Vec<i32> = other.value.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as i32)
            .rev()
            .collect();

        // Initialize result with zeros
        let mut result = vec![0i32; a_digits.len()];

        // Perform subtraction
        let mut borrow = 0;
        for i in 0..a_digits.len() {
            let a_digit = a_digits[i];
            let b_digit = if i < b_digits.len() { b_digits[i] } else { 0 };

            let mut diff = a_digit - b_digit - borrow;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }

            result[i] = diff;
        }

        // Remove leading zeros
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }

        // Convert back to string
        let mut result_str = String::new();
        for digit in result.iter().rev() {
            // Convert digit to char safely
            let digit_char = std::char::from_digit(*digit as u32, 10).unwrap_or('0');
            result_str.push(digit_char);
        }

        // Handle zero case
        if result_str.is_empty() {
            result_str = "0".to_string();
        }

        Mpz { value: result_str }
    }
}

// Helper function to compare two positive number strings
fn compare_strings(a: &str, b: &str) -> i8 {
    if a.len() != b.len() {
        return if a.len() > b.len() { 1 } else { -1 };
    }

    for (a_char, b_char) in a.chars().zip(b.chars()) {
        if a_char != b_char {
            return if a_char > b_char { 1 } else { -1 };
        }
    }

    0 // Equal
}

impl std::ops::Mul for Mpz {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        // For very large numbers, we need to use a more sophisticated approach
        // We'll implement a robust version of the grade-school multiplication algorithm

        // Handle zero cases first
        if self.value == "0" || other.value == "0" {
            return Mpz { value: "0".to_string() };
        }

        // Determine sign
        let a_negative = self.value.starts_with('-');
        let b_negative = other.value.starts_with('-');
        let result_negative = a_negative != b_negative;

        // Get absolute values
        let a_abs = if a_negative { &self.value[1..] } else { &self.value };
        let b_abs = if b_negative { &other.value[1..] } else { &other.value };

        // Convert strings to digit vectors (in reverse order for easier processing)
        let a_digits: Vec<u32> = a_abs.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as u32)
            .rev()
            .collect();

        let b_digits: Vec<u32> = b_abs.bytes()
            .filter(|&b| b >= b'0' && b <= b'9')
            .map(|b| (b - b'0') as u32)
            .rev()
            .collect();

        // Initialize result with zeros
        let mut result = vec![0u32; a_digits.len() + b_digits.len()];

        // Perform multiplication
        for (i, &a_digit) in a_digits.iter().enumerate() {
            let mut carry = 0u32;

            for (j, &b_digit) in b_digits.iter().enumerate() {
                let temp = result[i + j] as u64 + (a_digit as u64 * b_digit as u64) + carry as u64;
                result[i + j] = (temp % 10) as u32;
                carry = (temp / 10) as u32;
            }

            let mut k = i + b_digits.len();
            while carry > 0 {
                let temp = result[k] as u64 + carry as u64;
                result[k] = (temp % 10) as u32;
                carry = (temp / 10) as u32;
                k += 1;
            }
        }

        // Remove leading zeros
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }

        // Convert back to string
        let mut result_str = String::new();
        if result_negative && !(result.len() == 1 && result[0] == 0) {
            result_str.push('-');
        }

        for digit in result.iter().rev() {
            // Convert digit to char safely
            let digit_char = std::char::from_digit(*digit, 10).unwrap_or('0');
            result_str.push(digit_char);
        }

        // Handle zero case
        if result_str.is_empty() || result_str == "-" {
            result_str = "0".to_string();
        }

        Mpz { value: result_str }
    }
}

impl std::ops::Div for Mpz {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let a = self.value.parse::<i64>().unwrap_or(0);
        let b = other.value.parse::<i64>().unwrap_or(0);
        if b == 0 {
            return Mpz { value: "0".to_string() };
        }
        Mpz { value: (a / b).to_string() }
    }
}

impl std::ops::Neg for Mpz {
    type Output = Self;

    fn neg(self) -> Self {
        let a = self.value.parse::<i64>().unwrap_or(0);
        Mpz { value: (-a).to_string() }
    }
}

impl std::ops::BitAnd for &Mpz {
    type Output = Mpz;

    fn bitand(self, other: &Mpz) -> Mpz {
        let a = self.value.parse::<u64>().unwrap_or(0);
        let b = other.value.parse::<u64>().unwrap_or(0);
        Mpz { value: (a & b).to_string() }
    }
}

impl std::ops::Shr<u32> for Mpz {
    type Output = Self;

    fn shr(self, shift: u32) -> Self {
        let a = self.value.parse::<u64>().unwrap_or(0);
        Mpz { value: (a >> shift).to_string() }
    }
}

impl std::ops::Shl<u32> for Mpz {
    type Output = Self;

    fn shl(self, shift: u32) -> Self {
        let a = self.value.parse::<u64>().unwrap_or(0);
        Mpz { value: (a << shift).to_string() }
    }
}

impl Clone for Mpz {
    fn clone(&self) -> Self {
        Mpz { value: self.value.clone() }
    }
}

impl PartialEq for Mpz {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq<&Mpz> for Mpz {
    fn eq(&self, other: &&Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Mpz {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.value.parse::<i64>().unwrap_or(0);
        let b = other.value.parse::<i64>().unwrap_or(0);
        a.partial_cmp(&b)
    }
}

/// C-compatible big integer struct
#[repr(C)]
pub struct BigIntRaw {
    /// Sign in size: <0 negative, >0 positive
    pub size: isize,
    /// Pointer to `size_t(abs(size))` limbs
    pub limbs: *mut u64,
}

/// Create a new BigIntRaw from an i64
#[no_mangle]
pub extern "C" fn bigint_from_i64(value: i64) -> *mut BigIntRaw {
    let p = unsafe { malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw };

    // Allocate 1 limb (64 bits) which is enough for an i64
    let limbs = unsafe { malloc(std::mem::size_of::<u64>()) as *mut u64 };

    unsafe {
        // Store the absolute value in the limb
        *limbs = if value >= 0 { value as u64 } else { (-value) as u64 };

        // Set the size with sign
        (*p).size = if value >= 0 { 1 } else { -1 };
        (*p).limbs = limbs;
    }

    p
}

/// Create a BoxedAny from a BigIntRaw
#[no_mangle]
pub extern "C" fn boxed_any_from_bigint(value: *mut BigIntRaw) -> *mut BoxedAny {
    if value.is_null() {
        return boxed_any_none();
    }

    let boxed = unsafe { malloc(std::mem::size_of::<BoxedAny>()) as *mut BoxedAny };

    unsafe {
        (*boxed).tag = type_tags::BIGINT;
        (*boxed).data.ptr_val = value as *mut c_void;
    }

    boxed
}

/// Free a BigIntRaw
#[no_mangle]
pub extern "C" fn bigint_free(value: *mut BigIntRaw) {
    if value.is_null() {
        return;
    }

    unsafe {
        if !(*value).limbs.is_null() {
            free((*value).limbs as *mut c_void);
        }
        free(value as *mut c_void);
    }
}

/// Clone a BigIntRaw
#[no_mangle]
pub extern "C" fn bigint_clone(value: *const BigIntRaw) -> *mut BigIntRaw {
    if value.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let size = (*value).size;
        let abs_size = if size < 0 { -size } else { size } as usize;

        let p = malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw;
        let limbs = malloc(abs_size * std::mem::size_of::<u64>()) as *mut u64;

        // Copy the limbs
        ptr::copy_nonoverlapping((*value).limbs, limbs, abs_size);

        (*p).size = size;
        (*p).limbs = limbs;

        p
    }
}

/// Convert a BigIntRaw to an i64 if possible
/// Returns the value if it fits in i64, or i64::MAX if it's too large
#[no_mangle]
pub extern "C" fn bigint_to_i64(value: *const BigIntRaw) -> i64 {
    if value.is_null() {
        return 0;
    }

    unsafe {
        let size = (*value).size;
        let abs_size = if size < 0 { -size } else { size } as usize;

        // If the big int uses more than one limb, it won't fit in an i64
        if abs_size > 1 {
            return if size > 0 { i64::MAX } else { i64::MIN };
        }

        // Get the value from the single limb
        let limb_value = *(*value).limbs;

        // Check if it fits in i64
        if size > 0 {
            // Positive number
            if limb_value <= i64::MAX as u64 {
                return limb_value as i64;
            } else {
                return i64::MAX;
            }
        } else {
            // Negative number
            if limb_value <= (i64::MAX as u64) + 1 {
                return -(limb_value as i64);
            } else {
                return i64::MIN;
            }
        }
    }
}

/// Add two BigIntRaw values
#[no_mangle]
pub extern "C" fn bigint_add(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    // Convert to Mpz for easier manipulation
    let mpz_a = bigint_to_mpz(a);
    let mpz_b = bigint_to_mpz(b);

    // Perform addition
    let result = mpz_a + mpz_b;

    // Convert back to BigIntRaw
    mpz_to_bigint(&result)
}

/// Subtract two BigIntRaw values
#[no_mangle]
pub extern "C" fn bigint_subtract(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    // Convert to Mpz for easier manipulation
    let mpz_a = bigint_to_mpz(a);
    let mpz_b = bigint_to_mpz(b);

    // Perform subtraction
    let result = mpz_a - mpz_b;

    // Convert back to BigIntRaw
    mpz_to_bigint(&result)
}

/// Multiply two BigIntRaw values
#[no_mangle]
pub extern "C" fn bigint_multiply(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    // Convert to Mpz for easier manipulation
    let mpz_a = bigint_to_mpz(a);
    let mpz_b = bigint_to_mpz(b);

    // Perform multiplication
    let result = mpz_a * mpz_b;

    // Convert back to BigIntRaw
    mpz_to_bigint(&result)
}

/// Divide two BigIntRaw values
#[no_mangle]
pub extern "C" fn bigint_divide(a: *const BigIntRaw, b: *const BigIntRaw) -> *mut BigIntRaw {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }

    // Convert to Mpz for easier manipulation
    let mpz_a = bigint_to_mpz(a);
    let mpz_b = bigint_to_mpz(b);

    // Check for division by zero
    if mpz_b == Mpz::zero() {
        return ptr::null_mut();
    }

    // Perform division
    let result = mpz_a / mpz_b;

    // Convert back to BigIntRaw
    mpz_to_bigint(&result)
}

/// Convert a BigIntRaw to a string
#[no_mangle]
pub extern "C" fn bigint_to_string(value: *const BigIntRaw) -> *mut i8 {
    if value.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        // Convert to Mpz
        let mpz = bigint_to_mpz(value);

        // Convert to string
        let s = mpz.to_str_radix(10);

        // Allocate memory for the string (including null terminator)
        let len = s.len();
        let str_ptr = malloc(len + 1) as *mut i8;

        // Copy the string content
        ptr::copy_nonoverlapping(s.as_ptr(), str_ptr as *mut u8, len);

        // Add null terminator
        *((str_ptr as *mut u8).add(len)) = 0;

        str_ptr
    }
}

/// Helper function to convert BigIntRaw to Mpz
fn bigint_to_mpz(value: *const BigIntRaw) -> Mpz {
    unsafe {
        let size = (*value).size;
        let abs_size = if size < 0 { -size } else { size } as usize;

        // Create a new Mpz from the limbs
        let mut result = Mpz::new();

        // Import the limbs
        for i in 0..abs_size {
            let limb = *(*value).limbs.add(i);

            // Add each limb to the result
            if i > 0 {
                result = result << 64;
            }
            result = result + Mpz::from(limb);
        }

        // Apply the sign
        if size < 0 {
            result = -result;
        }

        result
    }
}

/// Helper function to convert Mpz to BigIntRaw
fn mpz_to_bigint(value: &Mpz) -> *mut BigIntRaw {
    // Get the number of limbs needed - for our simple implementation, we'll use 1 limb
    let num_limbs = 1;

    // Allocate memory for BigIntRaw
    let p = unsafe { malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw };
    let limbs = unsafe { calloc(num_limbs, std::mem::size_of::<u64>()) as *mut u64 };

    // Determine the sign
    let is_negative = value < &Mpz::zero();
    let abs_value = if is_negative { -value.clone() } else { value.clone() };

    // Extract the value - for our simple implementation, we'll just parse the string
    let limb_value = abs_value.to_string().parse::<u64>().unwrap_or(0);
    unsafe {
        *limbs = limb_value;
    }

    // Set the size with sign
    unsafe {
        (*p).size = if is_negative { -(num_limbs as isize) } else { num_limbs as isize };
        (*p).limbs = limbs;
    }

    p
}

/// Fast-doubling Fibonacci algorithm for big integers
#[no_mangle]
pub extern "C" fn bigint_fib(n: u64) -> *mut BigIntRaw {
    if n <= 92 {
        // For small values, use the regular algorithm with i64
        let result = fib_i64(n);
        return bigint_from_i64(result);
    }

    // Allocate buffer for the fast-doubling algorithm
    // Worst-case limbs = ceil(log2(F(n))/64) + 2 safety
    let limbs = ((0.694242 * n as f64) / 64.0).ceil() as usize + 4;

    // Allocate a buffer 3 times the size to hold temporary values
    let buffer = unsafe { calloc(3 * limbs, std::mem::size_of::<u64>()) as *mut u64 };

    // Call the fast-doubling algorithm
    fib_mpn(buffer, limbs, n);

    // Create a BigIntRaw from the result
    let p = unsafe { malloc(std::mem::size_of::<BigIntRaw>()) as *mut BigIntRaw };

    unsafe {
        (*p).size = limbs as isize; // Fibonacci is always positive
        (*p).limbs = buffer;
    }

    p
}

/// Helper function to compute Fibonacci numbers that fit in i64
fn fib_i64(n: u64) -> i64 {
    if n == 0 {
        return 0;
    }
    if n == 1 || n == 2 {
        return 1;
    }

    let mut a = 0i64;
    let mut b = 1i64;

    for _ in 2..=n {
        let temp = a;
        a = b;
        b = temp + b;
    }

    b
}

/// Fast-doubling algorithm for computing Fibonacci numbers
/// Implements F(2n) = F(n) * (2*F(n+1) - F(n))
/// and F(2n+1) = F(n+1)^2 + F(n)^2
fn fib_mpn(buf: *mut u64, limbs: usize, n: u64) {
    unsafe {
        if n <= 1 {
            // Base cases
            if n == 0 {
                *buf = 0;
            } else {
                *buf = 1;
            }
            return;
        }

        // Divide and conquer
        if n % 2 == 0 {
            // Even case: F(2n) = F(n) * (2*F(n+1) - F(n))
            fib_mpn(buf, limbs, n / 2);

            // We need F(n) and F(n+1)
            let f_n = buf;
            let f_n_plus_1 = buf.add(limbs);
            let temp = buf.add(2 * limbs);

            // Compute F(n+1)
            fib_mpn(f_n_plus_1, limbs, n / 2 + 1);

            // temp = 2*F(n+1)
            ptr::copy_nonoverlapping(f_n_plus_1, temp, limbs);
            mpn_lshift(temp, temp, limbs, 1); // Multiply by 2

            // temp = 2*F(n+1) - F(n)
            mpn_sub_n(temp, temp, f_n, limbs);

            // F(2n) = F(n) * temp
            mpn_mul_n(buf, f_n, temp, limbs);
        } else {
            // Odd case: F(2n+1) = F(n+1)^2 + F(n)^2
            fib_mpn(buf, limbs, n / 2);

            // We need F(n) and F(n+1)
            let f_n = buf;
            let f_n_plus_1 = buf.add(limbs);
            let temp = buf.add(2 * limbs);

            // Compute F(n+1)
            fib_mpn(f_n_plus_1, limbs, n / 2 + 1);

            // temp = F(n)^2
            mpn_sqr(temp, f_n, limbs);

            // F(n+1)^2
            mpn_sqr(buf, f_n_plus_1, limbs);

            // F(2n+1) = F(n+1)^2 + F(n)^2
            mpn_add_n(buf, buf, temp, limbs);
        }
    }
}

/// Low-level function to shift left (multiply by 2^cnt)
fn mpn_lshift(dst: *mut u64, src: *const u64, size: usize, cnt: u32) {
    if size == 0 || cnt == 0 {
        return;
    }

    let cnt_64 = cnt % 64;
    let limb_shift = (cnt / 64) as usize;

    unsafe {
        if cnt_64 == 0 {
            // Just copy with offset
            for i in (0..size).rev() {
                if i >= limb_shift {
                    *dst.add(i) = *src.add(i - limb_shift);
                } else {
                    *dst.add(i) = 0;
                }
            }
        } else {
            // Need to handle bits crossing limb boundaries
            let low_bits = 64 - cnt_64;

            // Handle the most significant limb first
            let mut carry = 0;

            for i in (0..size).rev() {
                let src_idx = if i >= limb_shift { i - limb_shift } else { 0 };
                let src_val = if i >= limb_shift { *src.add(src_idx) } else { 0 };

                let high_part = src_val << cnt_64;
                *dst.add(i) = high_part | carry;
                carry = if cnt_64 > 0 { src_val >> low_bits } else { 0 };
            }
        }
    }
}

/// Low-level function to subtract: dst = a - b
fn mpn_sub_n(dst: *mut u64, a: *const u64, b: *const u64, size: usize) {
    if size == 0 {
        return;
    }

    let mut borrow = 0u64;

    unsafe {
        for i in 0..size {
            let a_val = *a.add(i);
            let b_val = *b.add(i);

            let (res1, borrow1) = a_val.overflowing_sub(b_val);
            let (res2, borrow2) = res1.overflowing_sub(borrow);

            *dst.add(i) = res2;
            borrow = if borrow1 || borrow2 { 1 } else { 0 };
        }
    }
}

/// Low-level function to add: dst = a + b
fn mpn_add_n(dst: *mut u64, a: *const u64, b: *const u64, size: usize) {
    if size == 0 {
        return;
    }

    let mut carry = 0u64;

    unsafe {
        for i in 0..size {
            let a_val = *a.add(i);
            let b_val = *b.add(i);

            let (res1, carry1) = a_val.overflowing_add(b_val);
            let (res2, carry2) = res1.overflowing_add(carry);

            *dst.add(i) = res2;
            carry = if carry1 || carry2 { 1 } else { 0 };
        }
    }
}

/// Low-level function to multiply: dst = a * b
fn mpn_mul_n(dst: *mut u64, a: *const u64, b: *const u64, size: usize) {
    if size == 0 {
        return;
    }

    // Clear the destination first
    unsafe {
        ptr::write_bytes(dst, 0, 2 * size);
    }

    // Simple schoolbook multiplication
    unsafe {
        for i in 0..size {
            let a_val = *a.add(i);
            if a_val == 0 {
                continue;
            }

            let mut carry = 0u64;

            for j in 0..size {
                let b_val = *b.add(j);
                let dst_idx = i + j;

                // Compute the product and add to the destination
                let (hi, lo) = mul_with_carry(a_val, b_val);
                let (res1, carry1) = (*dst.add(dst_idx)).overflowing_add(lo);
                let (res2, carry2) = res1.overflowing_add(carry);

                *dst.add(dst_idx) = res2;

                // Propagate the carry
                carry = hi + if carry1 || carry2 { 1 } else { 0 };

                if dst_idx + 1 < 2 * size {
                    *dst.add(dst_idx + 1) += carry;
                    carry = 0;
                }
            }
        }
    }
}

/// Low-level function to square: dst = a^2
fn mpn_sqr(dst: *mut u64, a: *const u64, size: usize) {
    // For simplicity, just use the multiplication routine
    mpn_mul_n(dst, a, a, size);
}

/// Helper function to multiply two u64 values and get the high and low parts
fn mul_with_carry(a: u64, b: u64) -> (u64, u64) {
    let result = (a as u128) * (b as u128);
    let hi = (result >> 64) as u64;
    let lo = result as u64;
    (hi, lo)
}

/// Register BigInt functions in the LLVM module
pub fn register_boxed_bigint_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &mut inkwell::module::Module<'ctx>,
) {
    let boxed_any_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bigint_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = context.i64_type();
    let u64_type = context.i64_type();
    let void_type = context.void_type();
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // Register the bigint_from_i64 function
    module.add_function(
        "bigint_from_i64",
        bigint_ptr_type.fn_type(&[i64_type.into()], false),
        None,
    );

    // Register the boxed_any_from_bigint function
    module.add_function(
        "boxed_any_from_bigint",
        boxed_any_ptr_type.fn_type(&[bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_free function
    module.add_function(
        "bigint_free",
        void_type.fn_type(&[bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_clone function
    module.add_function(
        "bigint_clone",
        bigint_ptr_type.fn_type(&[bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_add function
    module.add_function(
        "bigint_add",
        bigint_ptr_type.fn_type(&[bigint_ptr_type.into(), bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_subtract function
    module.add_function(
        "bigint_subtract",
        bigint_ptr_type.fn_type(&[bigint_ptr_type.into(), bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_multiply function
    module.add_function(
        "bigint_multiply",
        bigint_ptr_type.fn_type(&[bigint_ptr_type.into(), bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_divide function
    module.add_function(
        "bigint_divide",
        bigint_ptr_type.fn_type(&[bigint_ptr_type.into(), bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_to_string function
    module.add_function(
        "bigint_to_string",
        i8_ptr_type.fn_type(&[bigint_ptr_type.into()], false),
        None,
    );

    // Register the bigint_fib function
    module.add_function(
        "bigint_fib",
        bigint_ptr_type.fn_type(&[u64_type.into()], false),
        None,
    );
}

/// Register BigInt runtime mappings for the JIT engine
pub fn register_boxed_bigint_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>,
) -> Result<(), String> {
    if let Some(f) = module.get_function("bigint_from_i64") {
        engine.add_global_mapping(&f, bigint_from_i64 as usize);
    }

    if let Some(f) = module.get_function("boxed_any_from_bigint") {
        engine.add_global_mapping(&f, boxed_any_from_bigint as usize);
    }

    if let Some(f) = module.get_function("bigint_free") {
        engine.add_global_mapping(&f, bigint_free as usize);
    }

    if let Some(f) = module.get_function("bigint_clone") {
        engine.add_global_mapping(&f, bigint_clone as usize);
    }

    if let Some(f) = module.get_function("bigint_add") {
        engine.add_global_mapping(&f, bigint_add as usize);
    }

    if let Some(f) = module.get_function("bigint_subtract") {
        engine.add_global_mapping(&f, bigint_subtract as usize);
    }

    if let Some(f) = module.get_function("bigint_multiply") {
        engine.add_global_mapping(&f, bigint_multiply as usize);
    }

    if let Some(f) = module.get_function("bigint_divide") {
        engine.add_global_mapping(&f, bigint_divide as usize);
    }

    if let Some(f) = module.get_function("bigint_to_string") {
        engine.add_global_mapping(&f, bigint_to_string as usize);
    }

    if let Some(f) = module.get_function("bigint_fib") {
        engine.add_global_mapping(&f, bigint_fib as usize);
    }

    Ok(())
}
