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

// Include the global and nonlocal statement tests
#[path = "more_tests/compiler/global_nonlocal_test.rs"]
mod global_nonlocal_test;

// Include the closure tests
#[path = "more_tests/compiler/closure_test.rs"]
mod closure_test;

// Include the nonlocal debug tests
#[path = "more_tests/compiler/nonlocal_debug_test.rs"]
mod nonlocal_debug_test;

// Include the simple nonlocal tests
#[path = "more_tests/compiler/simple_nonlocal_test.rs"]
mod simple_nonlocal_test;

// Include the tuple tests
#[path = "more_tests/compiler/tuple_test.rs"]
mod tuple_test;

// Include the comprehensive tuple tests
#[path = "more_tests/compiler/comprehensive_tuple_test.rs"]
mod comprehensive_tuple_test;

// Include the tuple subscript tests
#[path = "more_tests/compiler/tuple_subscript_test.rs"]
mod tuple_subscript_test;

// Include the tuple type inference tests
#[path = "more_tests/compiler/tuple_type_inference_test.rs"]
mod tuple_type_inference_test;

// Include the for loop tests
#[path = "more_tests/compiler/for_loop_test.rs"]
mod for_loop_test;

// Include the comprehensive for loop tests
#[path = "more_tests/compiler/comprehensive_for_loop_test.rs"]
mod comprehensive_for_loop_test;

// Include the range tests
#[path = "more_tests/compiler/range_test.rs"]
mod range_test;

// Include the list operations tests
#[path = "more_tests/compiler/list_operations_test.rs"]
mod list_operations_test;

// Include the slice operations tests
#[path = "more_tests/compiler/slice_operations_test.rs"]
mod slice_operations_test;

// Include the string slice operations tests
#[path = "more_tests/compiler/string_slice_operations_test.rs"]
mod string_slice_operations_test;

// Include the len function tests
#[path = "more_tests/compiler/len_function_test.rs"]
mod len_function_test;

// Include the exception handling tests
#[path = "more_tests/compiler/exception_test.rs"]
mod exception_test;

// Include the exception validation tests
#[path = "more_tests/compiler/exception_validation_test.rs"]
mod exception_validation_test;

// Include the list comprehension tests
#[path = "more_tests/compiler/list_comprehension_test.rs"]
mod list_comprehension_test;

// Include the advanced list comprehension tests
#[path = "more_tests/compiler/advanced_list_comprehension_test.rs"]
mod advanced_list_comprehension_test;

// Include the dictionary operations tests
#[path = "more_tests/compiler/dict_operations_test.rs"]
mod dict_operations_test;

// Include the comprehensive dictionary tests
#[path = "more_tests/compiler/comprehensive_dict_test.rs"]
mod comprehensive_dict_test;

// Include the advanced dictionary operations tests
#[path = "more_tests/compiler/advanced_dict_operations_test.rs"]
mod advanced_dict_operations_test;

// Include the nested dictionary tests
#[path = "more_tests/compiler/nested_dict_test.rs"]
mod nested_dict_test;

// Include the dictionary methods tests
#[path = "more_tests/compiler/dict_methods_test.rs"]
mod dict_methods_test;

// Include the dictionary comprehension tests
#[path = "more_tests/compiler/dict_comprehension_test.rs"]
mod dict_comprehension_test;

// Include the dictionary membership tests
#[path = "more_tests/compiler/dict_membership_test.rs"]
mod dict_membership_test;

// Include the dictionary function integration tests
#[path = "more_tests/compiler/dict_function_integration_test.rs"]
mod dict_function_integration_test;

// Include the dictionary function simple tests
#[path = "more_tests/compiler/dict_function_simple_test.rs"]
mod dict_function_simple_test;

// Include the dictionary function minimal tests
#[path = "more_tests/compiler/dict_function_minimal_test.rs"]
mod dict_function_minimal_test;

// Include the dictionary function return tests
#[path = "more_tests/compiler/dict_function_return_test.rs"]
mod dict_function_return_test;

// Include the print function tests
#[path = "more_tests/compiler/print_function_test.rs"]
mod print_function_test;
