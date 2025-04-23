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

fn assert_parse_fails(source: &str) {
    let result = parse_code(source);
    assert!(result.is_err(), "Expected parsing to fail, but it succeeded");
}

#[test]
fn test_print_with_basic_expressions() {
    // Test print with simple expressions
    assert_parses(r#"print(42)"#);
    assert_parses(r#"print("Hello")"#);
    assert_parses(r#"print(True)"#);
    assert_parses(r#"print(None)"#);
    assert_parses(r#"print([])"#);
    assert_parses(r#"print({})"#);
}

#[test]
fn test_print_with_binary_operations() {
    // Test print with binary operations
    assert_parses(r#"print(1 + 2)"#);
    assert_parses(r#"print(3 - 4)"#);
    assert_parses(r#"print(5 * 6)"#);
    assert_parses(r#"print(7 / 8)"#);
    assert_parses(r#"print(9 // 10)"#);
    assert_parses(r#"print(11 % 12)"#);
    assert_parses(r#"print(13 ** 14)"#);
    assert_parses(r#"print(15 << 16)"#);
    assert_parses(r#"print(17 >> 18)"#);
    assert_parses(r#"print(19 | 20)"#);
    assert_parses(r#"print(21 & 22)"#);
    assert_parses(r#"print(23 ^ 24)"#);
}

#[test]
fn test_print_with_comparison_operations() {
    // Test print with comparison operations
    assert_parses(r#"print(1 == 2)"#);
    assert_parses(r#"print(3 != 4)"#);
    assert_parses(r#"print(5 < 6)"#);
    assert_parses(r#"print(7 <= 8)"#);
    assert_parses(r#"print(9 > 10)"#);
    assert_parses(r#"print(11 >= 12)"#);
    assert_parses(r#"print(13 is 14)"#);
    assert_parses(r#"print(15 is not 16)"#);
    assert_parses(r#"print(17 in [17, 18])"#);
    assert_parses(r#"print(19 not in [20, 21])"#);
}

#[test]
fn test_print_with_string_concatenation() {
    // Test print with string concatenation
    assert_parses(r#"print("Hello" + " World")"#);
    assert_parses(r#"print("a" + "b" + "c")"#);
    assert_parses(r#"print("prefix_" + variable)"#);
    assert_parses(r#"print(variable + "_suffix")"#);
}

// Tests that reproduce the exact errors from the print_test_comprehensive.ch file
#[test]
fn test_print_test_comprehensive_success() {
    // These tests are now expected to succeed with the improved parser implementation
    // They match the examples from print_test_comprehensive.ch

    // Line 95: print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])
    assert_parses(r#"print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])"#);

    // Line 101: print("Concatenated:", first + " " + last)
    assert_parses(r#"print("Concatenated:", first + " " + last)"#);

    // Line 107: print("Comparisons:", x < y, x > y, x == y, x != y)
    assert_parses(r#"print("Comparisons:", x < y, x > y, x == y, x != y)"#);
}

#[test]
fn test_print_with_f_strings() {
    // Test print with f-strings
    assert_parses(r#"print(f"Value: {value}")"#);
    assert_parses(r#"print(f"Sum: {a + b}")"#);
    assert_parses(r#"print(f"Complex: {(a + b) * c}")"#);
    assert_parses(r#"print(f"Multiple: {a}, {b}, {a + b}")"#);
}
