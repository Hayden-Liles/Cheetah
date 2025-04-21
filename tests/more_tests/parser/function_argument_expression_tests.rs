use cheetah::ast::Module;
use cheetah::lexer::Lexer;
use cheetah::parser::{ParseError, Parser};

fn parse_code(source: &str) -> Result<Module, Vec<ParseError>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    if !lexer.get_errors().is_empty() {
        let parse_errors: Vec<ParseError> = lexer
            .get_errors()
            .iter()
            .map(|e| ParseError::InvalidSyntax {
                message: e.message.clone(),
                line: e.line,
                column: e.column,
                suggestion: None,
            })
            .collect();
        return Err(parse_errors);
    }

    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn assert_parses(source: &str) {
    let result = parse_code(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

/// Tests for expressions in function arguments
/// These tests focus on the specific issues found in print_test_comprehensive.ch
#[test]
fn test_function_call_with_binary_op_in_args() {
    // These should now parse successfully with the fixed parser
    assert_parses(r#"print("Sum:", a + b)"#);
    assert_parses(r#"print("Product:", a * b)"#);
}

#[test]
fn test_function_call_with_comparison_in_args() {
    // These should now parse successfully with the fixed parser
    assert_parses(r#"print("Comparison:", a < b)"#);
    assert_parses(r#"print("Equality:", a == b)"#);
}

#[test]
fn test_function_call_with_string_concat_in_args() {
    // This should now parse successfully with the fixed parser
    assert_parses(r#"print("Concat:", a + " " + b)"#);
}

#[test]
fn test_function_call_with_multiple_expression_args() {
    // This should now parse successfully with the fixed parser
    assert_parses(r#"print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])"#);
}

#[test]
fn test_nested_function_calls_with_expressions() {
    // These should now parse successfully with the fixed parser
    assert_parses(r#"outer_func(inner_func(a + b))"#);
    assert_parses(r#"calculate(sum(a, b) + product(c, d))"#);
}

#[test]
fn test_function_call_with_complex_expressions() {
    // This should now parse successfully with the fixed parser
    assert_parses(r#"print("Complex:", (a + b) * c / (d - e))"#);
}

/// Tests for the specific examples from print_test_comprehensive.ch
#[test]
fn test_print_test_comprehensive_examples() {
    // These should now parse successfully with the fixed parser

    // Line 95: print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])
    assert_parses(r#"print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])"#);

    // Line 101: print("Concatenated:", first + " " + last)
    assert_parses(r#"print("Concatenated:", first + " " + last)"#);

    // Line 107: print("Comparisons:", x < y, x > y, x == y, x != y)
    assert_parses(r#"print("Comparisons:", x < y, x > y, x == y, x != y)"#);
}
