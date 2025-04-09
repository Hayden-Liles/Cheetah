// This file links all the type checker test files together

// Include the main typechecker tests
#[path = "more_tests/typechecker/typechecker_tests.rs"]
mod typechecker_tests;

// Include the comprehensive typechecker tests
#[path = "more_tests/typechecker/typechecker_comprehensive.rs"]
mod typechecker_comprehensive;

// Include the binary operations tests
#[path = "more_tests/typechecker/typechecker_binary_ops.rs"]
mod typechecker_binary_ops;

// Include the functions and control flow tests
#[path = "more_tests/typechecker/typechecker_functions_control.rs"]
mod typechecker_functions_control;

// Include the type annotations tests
#[path = "more_tests/typechecker/typechecker_annotations.rs"]
mod typechecker_annotations;
