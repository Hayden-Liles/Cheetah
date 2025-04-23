#[cfg(test)]
mod boxed_any_tests {
    use cheetah::compiler::runtime::boxed_any::{
        boxed_any_add, boxed_any_as_bool, boxed_any_as_float, boxed_any_as_int, boxed_any_as_string,
        boxed_any_clone, boxed_any_divide, boxed_any_equals, boxed_any_free, boxed_any_from_bool,
        boxed_any_from_float, boxed_any_from_int, boxed_any_from_string, boxed_any_multiply,
        boxed_any_none, boxed_any_not, boxed_any_subtract, boxed_any_to_bool, boxed_any_to_float,
        boxed_any_to_int, boxed_any_to_string, type_tags,
    };
    use std::ffi::{CStr, CString};
    use std::ptr;

    #[test]
    fn test_boxed_any_int() {
        unsafe {
            // Create a BoxedAny from an integer
            let value = boxed_any_from_int(42);
            
            // Check the type tag
            assert_eq!((*value).tag, type_tags::INT);
            
            // Check the value
            assert_eq!((*value).data.int_val, 42);
            
            // Convert to int
            assert_eq!(boxed_any_to_int(value), 42);
            
            // Free the value
            boxed_any_free(value);
        }
    }

    #[test]
    fn test_boxed_any_float() {
        unsafe {
            // Create a BoxedAny from a float
            let value = boxed_any_from_float(3.14);
            
            // Check the type tag
            assert_eq!((*value).tag, type_tags::FLOAT);
            
            // Check the value
            assert!(((*value).data.float_val - 3.14).abs() < 0.0001);
            
            // Convert to float
            assert!((boxed_any_to_float(value) - 3.14).abs() < 0.0001);
            
            // Free the value
            boxed_any_free(value);
        }
    }

    #[test]
    fn test_boxed_any_bool() {
        unsafe {
            // Create a BoxedAny from a boolean
            let value = boxed_any_from_bool(true);
            
            // Check the type tag
            assert_eq!((*value).tag, type_tags::BOOL);
            
            // Check the value
            assert_eq!((*value).data.bool_val, 1);
            
            // Convert to bool
            assert_eq!(boxed_any_to_bool(value), true);
            
            // Free the value
            boxed_any_free(value);
        }
    }

    #[test]
    fn test_boxed_any_string() {
        unsafe {
            // Create a C string
            let c_str = CString::new("Hello, world!").unwrap();
            
            // Create a BoxedAny from a string
            let value = boxed_any_from_string(c_str.as_ptr());
            
            // Check the type tag
            assert_eq!((*value).tag, type_tags::STRING);
            
            // Check the value
            let str_ptr = (*value).data.ptr_val as *const i8;
            let result_str = CStr::from_ptr(str_ptr).to_str().unwrap();
            assert_eq!(result_str, "Hello, world!");
            
            // Free the value
            boxed_any_free(value);
        }
    }

    #[test]
    fn test_boxed_any_none() {
        unsafe {
            // Create a BoxedAny representing None
            let value = boxed_any_none();
            
            // Check the type tag
            assert_eq!((*value).tag, type_tags::NONE);
            
            // Check the value
            assert!((*value).data.ptr_val.is_null());
            
            // Convert to bool
            assert_eq!(boxed_any_to_bool(value), false);
            
            // Free the value
            boxed_any_free(value);
        }
    }

    #[test]
    fn test_boxed_any_clone() {
        unsafe {
            // Create a BoxedAny from an integer
            let value = boxed_any_from_int(42);
            
            // Clone the value
            let cloned = boxed_any_clone(value);
            
            // Check that the clone has the same type and value
            assert_eq!((*cloned).tag, (*value).tag);
            assert_eq!((*cloned).data.int_val, (*value).data.int_val);
            
            // Free both values
            boxed_any_free(value);
            boxed_any_free(cloned);
        }
    }

    #[test]
    fn test_boxed_any_arithmetic() {
        unsafe {
            // Create BoxedAny values
            let a = boxed_any_from_int(10);
            let b = boxed_any_from_int(5);
            
            // Addition
            let sum = boxed_any_add(a, b);
            assert_eq!((*sum).tag, type_tags::INT);
            assert_eq!((*sum).data.int_val, 15);
            
            // Subtraction
            let diff = boxed_any_subtract(a, b);
            assert_eq!((*diff).tag, type_tags::INT);
            assert_eq!((*diff).data.int_val, 5);
            
            // Multiplication
            let prod = boxed_any_multiply(a, b);
            assert_eq!((*prod).tag, type_tags::INT);
            assert_eq!((*prod).data.int_val, 50);
            
            // Division
            let quot = boxed_any_divide(a, b);
            assert_eq!((*quot).tag, type_tags::INT);
            assert_eq!((*quot).data.int_val, 2);
            
            // Free all values
            boxed_any_free(a);
            boxed_any_free(b);
            boxed_any_free(sum);
            boxed_any_free(diff);
            boxed_any_free(prod);
            boxed_any_free(quot);
        }
    }

    #[test]
    fn test_boxed_any_comparison() {
        unsafe {
            // Create BoxedAny values
            let a = boxed_any_from_int(10);
            let b = boxed_any_from_int(10);
            let c = boxed_any_from_int(20);
            
            // Equality
            assert_eq!(boxed_any_equals(a, b), true);
            assert_eq!(boxed_any_equals(a, c), false);
            
            // Free all values
            boxed_any_free(a);
            boxed_any_free(b);
            boxed_any_free(c);
        }
    }

    #[test]
    fn test_boxed_any_logical() {
        unsafe {
            // Create BoxedAny values
            let t = boxed_any_from_bool(true);
            let f = boxed_any_from_bool(false);
            
            // Logical NOT
            let not_t = boxed_any_not(t);
            let not_f = boxed_any_not(f);
            
            assert_eq!(boxed_any_to_bool(not_t), false);
            assert_eq!(boxed_any_to_bool(not_f), true);
            
            // Free all values
            boxed_any_free(t);
            boxed_any_free(f);
            boxed_any_free(not_t);
            boxed_any_free(not_f);
        }
    }

    #[test]
    fn test_boxed_any_type_conversion() {
        unsafe {
            // Create a BoxedAny from an integer
            let int_val = boxed_any_from_int(42);
            
            // Convert to float
            let float_val = boxed_any_as_float(int_val);
            assert_eq!((*float_val).tag, type_tags::FLOAT);
            assert!(((*float_val).data.float_val - 42.0).abs() < 0.0001);
            
            // Convert to bool
            let bool_val = boxed_any_as_bool(int_val);
            assert_eq!((*bool_val).tag, type_tags::BOOL);
            assert_eq!((*bool_val).data.bool_val, 1);
            
            // Convert to string
            let str_val = boxed_any_as_string(int_val);
            assert_eq!((*str_val).tag, type_tags::STRING);
            let str_ptr = (*str_val).data.ptr_val as *const i8;
            let result_str = CStr::from_ptr(str_ptr).to_str().unwrap();
            assert_eq!(result_str, "42");
            
            // Free all values
            boxed_any_free(int_val);
            boxed_any_free(float_val);
            boxed_any_free(bool_val);
            boxed_any_free(str_val);
        }
    }
}
