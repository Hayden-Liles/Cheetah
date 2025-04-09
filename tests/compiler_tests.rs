// This file links all the compiler test files together

// Include the main compiler tests
#[path = "more_tests/compiler/compiler_tests.rs"]
mod compiler_tests;

// Include the additional compiler tests
#[path = "more_tests/compiler/compiler_tests2.rs"]
mod compiler_tests2;

#[path = "more_tests/compiler/compiler_tests3.rs"]
mod compiler_tests3;

#[path = "more_tests/compiler/compiler_tests4.rs"]
mod compiler_tests4;

#[path = "more_tests/compiler/compiler_tests5.rs"]
mod compiler_tests5;

// Include the specialized compiler tests
#[path = "more_tests/compiler/compiler_expr_tests.rs"]
mod compiler_expr_tests;

#[path = "more_tests/compiler/compiler_stmt_tests.rs"]
mod compiler_stmt_tests;

#[path = "more_tests/compiler/compiler_type_tests.rs"]
mod compiler_type_tests;

// Include the integration tests
#[path = "more_tests/compiler/compiler_integration_tests.rs"]
mod compiler_integration_tests;

// Include the comprehensive compiler tests
#[path = "more_tests/compiler/comprehensive_compiler_tests.rs"]
mod comprehensive_compiler_tests;

// Include the AST tests
#[path = "more_tests/compiler/ast_test.rs"]
mod ast_test;

// Include the binary operations tests
#[path = "more_tests/compiler/binary_ops_test.rs"]
mod binary_ops_test;

// Include the advanced binary operations tests
#[path = "more_tests/compiler/binary_ops_advanced_test.rs"]
mod binary_ops_advanced_test;

// Include the binary operations error tests
#[path = "more_tests/compiler/binary_ops_error_test.rs"]
mod binary_ops_error_test;

// Include the string operations tests
#[path = "more_tests/compiler/string_ops_test.rs"]
mod string_ops_test;

// Include the recursive function tests
#[path = "more_tests/compiler/recursive_function_test.rs"]
mod recursive_function_test;

// Include the variable scoping tests
#[path = "more_tests/compiler/variable_scoping_test.rs"]
mod variable_scoping_test;
