// This file links all the test files together for easier test execution

// Include the main test files
#[path = "compiler_tests.rs"]
mod compiler_tests;

#[path = "lexer_tests.rs"]
mod lexer_tests;

#[path = "parser_tests.rs"]
mod parser_tests;

#[path = "type_checker_tests.rs"]
mod type_checker_tests;

// This test ensures that all the test modules are properly linked
#[test]
fn test_all_modules_linked() {
    // This test doesn't do anything, it just ensures that all the modules are linked
    assert!(true);
}
