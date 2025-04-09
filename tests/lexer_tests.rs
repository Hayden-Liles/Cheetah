// This file links all the lexer test files together

// Include the main lexer tests
#[path = "more_tests/lexer/lexer_tests.rs"]
mod lexer_tests;

// Include the edge cases tests
#[path = "more_tests/lexer/lexer_edge_cases_tests.rs"]
mod lexer_edge_cases_tests;
