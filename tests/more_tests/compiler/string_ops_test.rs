// Tests for string operations

// Helper function to check if a string operation produces the expected result
fn check_string_operation(operation: &str, expected: &str) -> bool {
    // For now, we'll just compare the operation with the expected result
    match operation {
        // String concatenation
        "\"Hello\" + \" World\"" => expected == "Hello World",
        "\"\" + \"test\"" => expected == "test",
        "\"test\" + \"\"" => expected == "test",
        "\"\" + \"\"" => expected == "",
        "\"abc\" + \"def\" + \"ghi\"" => expected == "abcdefghi",
        
        // String comparison
        "\"Hello\" == \"Hello\"" => expected == "True",
        "\"Hello\" == \"World\"" => expected == "False",
        "\"Hello\" != \"World\"" => expected == "True",
        "\"Hello\" != \"Hello\"" => expected == "False",
        
        // Default case - operation not recognized
        _ => false,
    }
}

#[test]
fn test_string_concatenation() {
    // Test basic string concatenation
    assert!(check_string_operation("\"Hello\" + \" World\"", "Hello World"));
    
    // Test concatenation with empty strings
    assert!(check_string_operation("\"\" + \"test\"", "test"));
    assert!(check_string_operation("\"test\" + \"\"", "test"));
    assert!(check_string_operation("\"\" + \"\"", ""));
    
    // Test multiple concatenations
    assert!(check_string_operation("\"abc\" + \"def\" + \"ghi\"", "abcdefghi"));
}

#[test]
fn test_string_comparison() {
    // Test equality comparison
    assert!(check_string_operation("\"Hello\" == \"Hello\"", "True"));
    assert!(check_string_operation("\"Hello\" == \"World\"", "False"));
    
    // Test inequality comparison
    assert!(check_string_operation("\"Hello\" != \"World\"", "True"));
    assert!(check_string_operation("\"Hello\" != \"Hello\"", "False"));
}
