use cheetah::ast::Module;
use cheetah::lexer::Lexer;
use cheetah::parser::{ParseError, parse};

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

    parse(tokens)
}

// Helper function to check if errors contain a specific line number
fn has_error_on_line(errors: &[ParseError], line: usize) -> bool {
    errors.iter().any(|e| match e {
        ParseError::UnexpectedToken { line: l, .. } |
        ParseError::InvalidSyntax { line: l, .. } |
        ParseError::EOF { line: l, .. } => *l == line,
    })
}

#[test]
fn test_missing_comma_in_function_params() {
    let source = r#"
def func(x y z):
    return x + y + z

print("This should be parsed")
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Error should be on line 2 (the function definition with missing comma)
        assert!(has_error_on_line(&errors, 2), "Should have an error on line 2");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);
        }
    }
}

#[test]
fn test_multiple_syntax_errors() {
    let source = r#"
def func(x, y:  # Missing closing parenthesis
    return x + y

if True  # Missing colon
    print("True")
else:
    print("False")

print("Final statement")
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);

            // Print the line number for each error
            match error {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => {
                    println!("  Line: {}", line);
                }
            }
        }
    }
}

#[test]
fn test_unclosed_delimiters() {
    let source = r#"
x = [1, 2, 3  # Missing closing bracket
y = 10
z = 20
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);

            // Print the line number for each error
            match error {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => {
                    println!("  Line: {}", line);
                }
            }
        }
    }
}

#[test]
fn test_incomplete_expressions() {
    let source = r#"
x = 1 +  # Incomplete expression
y = 2 * 3
z = 4 /
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);

            // Print the line number for each error
            match error {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => {
                    println!("  Line: {}", line);
                }
            }
        }
    }
}

#[test]
fn test_invalid_indentation() {
    let source = r#"
def test():
    x = 1
  y = 2  # Wrong indentation
    z = 3
"#;

    let result = parse_code(source);

    // The parser might not specifically check for indentation errors,
    // but it should at least detect a syntax error
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);
        }
    }
}

#[test]
fn test_complex_error_scenario() {
    let source = r#"
class Test:
    def __init__(self, name age):  # Missing comma
        self.name = name
        self.age = age

    def greet(self:  # Missing closing parenthesis
        return f"Hello, {self.name}!"

test = Test("John", 30)
print(test.greet())
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);

            // Print the line number for each error
            match error {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => {
                    println!("  Line: {}", line);
                }
            }
        }
    }
}

#[test]
fn test_error_in_nested_structures() {
    let source = r#"
def outer():
    def inner(x y):  # Missing comma
        return x + y

    return inner(5, 10)

result = outer()
print(result)
"#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    if let Err(errors) = result {
        // Should have at least one error
        assert!(!errors.is_empty(), "Should have at least one error");

        // Print all errors for debugging
        println!("Found {} errors:", errors.len());
        for (i, error) in errors.iter().enumerate() {
            println!("Error {}: {}", i+1, error);

            // Print the line number for each error
            match error {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => {
                    println!("  Line: {}", line);
                }
            }
        }
    }
}
