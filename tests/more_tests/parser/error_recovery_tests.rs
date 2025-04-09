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

#[test]
fn test_multiple_error_reporting() {
    // Multiple syntax errors in a single file
    let source = r#"
def func(x y): # Missing comma
    retrun x + y # Typo in return

for in range(10): # Missing target
    print(i)
    "#;

    let result = parse_code(source);
    assert!(result.is_err(), "Parsing should fail");

    // With our improved error recovery, we should now get multiple errors
    if let Err(errors) = result {
        // We should have at least 2 errors (one for each major syntax issue)
        assert!(errors.len() >= 2, "Expected at least 2 errors, got {}", errors.len());

        // Check that the first error is about the missing comma
        let first_error = &errors[0];
        match first_error {
            ParseError::UnexpectedToken { line, .. } |
            ParseError::InvalidSyntax { line, .. } |
            ParseError::EOF { line, .. } => {
                assert_eq!(*line, 2, "First error should be on line 2 (the function definition)");
            }
        }

        // Check that we have an error for the 'for in range' line
        let has_for_error = errors.iter().any(|e| {
            match e {
                ParseError::UnexpectedToken { line, .. } |
                ParseError::InvalidSyntax { line, .. } |
                ParseError::EOF { line, .. } => *line == 5,
            }
        });

        assert!(has_for_error, "Should have an error for the 'for in range' line");
    }
}
