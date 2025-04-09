// This file links all the parser test files together

// Include the main parser tests
#[path = "more_tests/parser/parser_tests.rs"]
mod parser_tests;

// Include the integration tests
#[path = "more_tests/parser/integration_tests.rs"]
mod integration_tests;

// Include the error recovery tests
#[path = "more_tests/parser/error_recovery_tests.rs"]
mod error_recovery_tests;

// Include the comprehensive error recovery tests
#[path = "more_tests/parser/error_recovery_comprehensive.rs"]
mod error_recovery_comprehensive;

// Include the simple error tests
#[path = "more_tests/parser/simple_error_test.rs"]
mod simple_error_test;
