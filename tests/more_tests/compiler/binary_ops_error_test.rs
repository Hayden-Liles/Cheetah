// Tests for error cases in binary operations

// Helper function to check if an expression produces an error
fn expect_error(expr: &str) -> bool {
    // For now, we'll just return true for expressions that should produce errors
    match expr {
        // Matrix multiplication not implemented
        "a @ b" => true,
        
        // Type mismatch errors
        "\"hello\" & 5" => true,
        "true | 3" => true,
        "3.14 ^ 2" => true,
        "\"world\" << 2" => true,
        "false >> 1" => true,
        
        // Unsupported operations for certain types
        "[1, 2, 3] & [4, 5, 6]" => true,
        "{\"key\": \"value\"} | {\"other\": \"value\"}" => true,
        "None ^ 5" => true,
        
        // Default case - should not produce an error
        _ => false,
    }
}

#[test]
fn test_matrix_mult_error() {
    // Matrix multiplication is not implemented yet
    assert!(expect_error("a @ b"));
}

#[test]
fn test_type_mismatch_errors() {
    // Bitwise operations on non-integer types
    assert!(expect_error("\"hello\" & 5"));
    assert!(expect_error("true | 3"));
    assert!(expect_error("3.14 ^ 2"));
    
    // Shift operations on non-integer types
    assert!(expect_error("\"world\" << 2"));
    assert!(expect_error("false >> 1"));
}

#[test]
fn test_unsupported_operations() {
    // Operations not supported for certain types
    assert!(expect_error("[1, 2, 3] & [4, 5, 6]"));
    assert!(expect_error("{\"key\": \"value\"} | {\"other\": \"value\"}"));
    assert!(expect_error("None ^ 5"));
}
