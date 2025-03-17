#[cfg(test)]
mod parser_recovery_tests {
    use cheetah::ast::Module;
    use cheetah::lexer::Lexer;
    use cheetah::parser::{ParseError, Parser};
    use std::fmt;

    // Custom formatter for error types
    struct ErrorFormatter<'a>(pub &'a ParseError);

    impl<'a> fmt::Display for ErrorFormatter<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                ParseError::UnexpectedToken { expected, found, line, column } => {
                    write!(f, "Unexpected token at line {}, column {}: expected '{}', found '{:?}'", 
                           line, column, expected, found)
                },
                ParseError::InvalidSyntax { message, line, column } => {
                    write!(f, "Invalid syntax at line {}, column {}: {}", 
                           line, column, message)
                },
                ParseError::EOF { expected, line, column } => {
                    write!(f, "Unexpected EOF at line {}, column {}: expected '{}'", 
                           line, column, expected)
                },
            }
        }
    }

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
                })
                .collect();
            return Err(parse_errors);
        }

        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // Helper to check that parsing fails and returns useful error information
    fn check_error_quality(source: &str) -> String {
        match parse_code(source) {
            Ok(_) => {
                panic!("Expected parsing to fail, but it succeeded: {}", source);
            },
            Err(errors) => {
                assert!(!errors.is_empty(), "Expected at least one error to be reported");
                let error_message = format!("{}", ErrorFormatter(&errors[0]));
                
                // Verify that error message contains useful information
                assert!(!error_message.is_empty(), "Error message should not be empty");
                assert!(error_message.contains("line"), "Error message should contain line number");
                assert!(error_message.contains("column"), "Error message should contain column number");
                
                error_message
            },
        }
    }

    #[test]
    fn test_unclosed_delimiters() {
        // Unclosed parentheses
        let error = check_error_quality("func(1, 2");
        assert!(error.contains("Unclosed parenthesis") || error.contains("expected ')'"), 
               "Error message should mention unclosed parenthesis: {}", error);
        
        // Unclosed brackets
        let error = check_error_quality("items = [1, 2, 3");
        assert!(error.contains("Unclosed bracket") || error.contains("expected ']'"), 
               "Error message should mention unclosed bracket: {}", error);
        
        // Unclosed braces
        let error = check_error_quality("data = {1: 'one', 2: 'two'");
        assert!(error.contains("Unclosed brace") || error.contains("expected '}'"), 
               "Error message should mention unclosed brace: {}", error);
        
        // Unclosed string
        // Note: This is usually caught by the lexer, not the parser
        // let error = check_error_quality("message = 'Hello, world");
        
        // Nested unclosed delimiters
        let error = check_error_quality("func(1, [2, 3, {4: 5");
        assert!(error.contains("Unclosed") || error.contains("expected"), 
               "Error message should mention unclosed delimiter: {}", error);
    }

    #[test]
    fn test_mismatched_delimiters() {
        // Closing with wrong delimiter
        let error = check_error_quality("func(1, 2]");
        assert!(error.contains("expected ')'") || error.contains("Unexpected token"), 
               "Error message should identify mismatched delimiter: {}", error);
        
        // Closing too many times
        let error = check_error_quality("func(1, 2))");
        assert!(error.contains("Unexpected token") || error.contains("unexpected ')'"), 
               "Error message should identify extra delimiter: {}", error);
        
        // Mismatched in complex expression
        let error = check_error_quality("x = (1 + 2] * 3");
        assert!(error.contains("expected ')'") || error.contains("Unexpected token"), 
               "Error message should identify mismatched delimiter: {}", error);
    }

    #[test]
    fn test_invalid_expressions() {
        // Binary operator without right operand
        let error = check_error_quality("x = 1 +");
        assert!(error.contains("expected 'expression'") || error.contains("Incomplete expression"), 
               "Error message should indicate incomplete expression: {}", error);
        
        // Consecutive operators
        let error = check_error_quality("x = 1 + + 2");
        assert!(error.contains("Invalid syntax") || error.contains("Unexpected token"), 
               "Error message should indicate invalid operator sequence: {}", error);
        
        // Invalid assignment target
        let error = check_error_quality("1 + 2 = 3");
        assert!(error.contains("Cannot assign to") || error.contains("Invalid assignment target"), 
               "Error message should indicate invalid assignment target: {}", error);
        
        // Invalid expressions in different contexts
        let error = check_error_quality("def func(a = 1 +): pass");
        assert!(!error.is_empty(), "Should report an error for invalid default parameter expression");
        
        let error = check_error_quality("if 1 + * 2: pass");
        assert!(!error.is_empty(), "Should report an error for invalid condition expression");
    }

    #[test]
    fn test_invalid_statements() {
        // Return outside function
        let error = check_error_quality("return 42");
        assert!(error.contains("outside of function") || error.contains("Return statement outside"), 
               "Error message should indicate return outside function: {}", error);
        
        // Break outside loop
        let error = check_error_quality("break");
        assert!(error.contains("outside loop") || error.contains("Break statement outside"), 
               "Error message should indicate break outside loop: {}", error);
        
        // Continue outside loop
        let error = check_error_quality("continue");
        assert!(error.contains("outside loop") || error.contains("Continue statement outside"), 
               "Error message should indicate continue outside loop: {}", error);
        
        // Yield outside function
        let error = check_error_quality("yield 42");
        assert!(error.contains("outside of function") || error.contains("Yield statement outside"), 
               "Error message should indicate yield outside function: {}", error);
    }

    #[test]
    fn test_indentation_errors() {
        // Inconsistent indentation
        let error = check_error_quality("if condition:\n  x = 1\n    y = 2");
        assert!(!error.is_empty(), "Should report an error for inconsistent indentation");
        
        // Missing indentation
        let error = check_error_quality("if condition:\nx = 1");
        assert!(!error.is_empty(), "Should report an error for missing indentation");
        
        // Unexpected indentation
        let error = check_error_quality("x = 1\n    y = 2");
        assert!(!error.is_empty(), "Should report an error for unexpected indentation");
    }

    #[test]
    fn test_invalid_function_definitions() {
        // Missing parameter name
        let error = check_error_quality("def func(, ): pass");
        assert!(error.contains("Expected parameter name") || error.contains("expected 'expression'"), 
               "Error message should indicate missing parameter: {}", error);
        
        // Missing function name
        let error = check_error_quality("def (x, y): pass");
        assert!(error.contains("function name") || error.contains("expected identifier"), 
               "Error message should indicate missing function name: {}", error);
        
        // Parameter after *args
        let _error = check_error_quality("def func(*args, x): pass");
        // This is actually valid Python syntax (for keyword-only parameters)
        
        // Parameter after **kwargs
        let error = check_error_quality("def func(**kwargs, x): pass");
        assert!(error.contains("Parameter after **kwargs") || error.contains("Invalid syntax"), 
               "Error message should indicate invalid parameter order: {}", error);
        
        // Invalid parameter syntax
        let error = check_error_quality("def func(x y): pass");
        assert!(error.contains("Expected comma between parameters") || error.contains("expected ','"), 
               "Error message should indicate missing comma: {}", error);
    }

    #[test]
    fn test_invalid_class_definitions() {
        // Missing class name
        let error = check_error_quality("class : pass");
        assert!(error.contains("class name") || error.contains("expected identifier"), 
               "Error message should indicate missing class name: {}", error);
        
        // Invalid base class syntax
        let error = check_error_quality("class Test(Base1 Base2): pass");
        assert!(error.contains("Expected comma between") || error.contains("expected ','"), 
               "Error message should indicate missing comma: {}", error);
        
        // Empty parentheses with comma
        let error = check_error_quality("class Test(,): pass");
        assert!(error.contains("expected 'expression'") || error.contains("Invalid syntax"), 
               "Error message should indicate invalid base class list: {}", error);
    }

    #[test]
    fn test_invalid_control_flow() {
        // If without condition
        let error = check_error_quality("if: pass");
        assert!(error.contains("expected 'expression'") || error.contains("Expected condition"), 
               "Error message should indicate missing condition: {}", error);
        
        // If without colon
        let error = check_error_quality("if condition pass");
        assert!(error.contains("Expected ':'") || error.contains("expected ':'"), 
               "Error message should indicate missing colon: {}", error);
        
        // For without target
        let error = check_error_quality("for in items: pass");
        assert!(error.contains("Expected target") || error.contains("expected 'expression'"), 
               "Error message should indicate missing target: {}", error);
        
        // For without iterator
        let error = check_error_quality("for x in: pass");
        assert!(error.contains("expected 'expression'") || error.contains("Expected iterator"), 
               "Error message should indicate missing iterator: {}", error);
        
        // While without condition
        let error = check_error_quality("while: pass");
        assert!(error.contains("expected 'expression'") || error.contains("Expected condition"), 
               "Error message should indicate missing condition: {}", error);
    }

    #[test]
    fn test_multiline_errors() {
        // Syntax error spanning multiple lines
        let error = check_error_quality("def func():\n    return (1 +\n          \n          )");
        assert!(!error.is_empty(), "Should report an error for invalid return expression");
        
        // Unclosed delimiter across multiple lines
        let error = check_error_quality("items = [\n    1,\n    2,\n    3\n    ");
        assert!(!error.is_empty(), "Should report an error for unclosed bracket across lines");
        
        // Invalid function call across multiple lines
        let error = check_error_quality("result = func(\n    arg1,\n    arg2,\n    ,\n    arg4\n)");
        assert!(!error.is_empty(), "Should report an error for invalid function call syntax");
    }

    #[test]
    fn test_error_positioning() {
        // Parse some invalid code and check that error positions are reasonable
        let code = "x = 1 + * 2";
        match parse_code(code) {
            Err(errors) => {
                let error = &errors[0];
                match error {
                    ParseError::UnexpectedToken { line, column, .. } |
                    ParseError::InvalidSyntax { line, column, .. } |
                    ParseError::EOF { line, column, .. } => {
                        // The error should be around the '*' character, which is at position 8
                        assert_eq!(*line, 1, "Error should be on line 1");
                        assert!((*column >= 7 && *column <= 9), 
                               "Error should be near the '*' character (column 8), but was at column {}", column);
                    }
                }
            },
            Ok(_) => panic!("Expected parsing to fail"),
        }
        
        let code = "def func(x, ):\n    return x";
        match parse_code(code) {
            Err(errors) => {
                let error = &errors[0];
                match error {
                    ParseError::UnexpectedToken { line, column, .. } |
                    ParseError::InvalidSyntax { line, column, .. } |
                    ParseError::EOF { line, column, .. } => {
                        // The error should be around the trailing comma
                        assert_eq!(*line, 1, "Error should be on line 1");
                        assert!((*column >= 11 && *column <= 12), 
                               "Error should be near the trailing comma, but was at column {}", column);
                    }
                }
            },
            Ok(_) => panic!("Expected parsing to fail"),
        }
    }

    #[test]
    fn test_cascading_errors() {
        // A single syntax error should not cause a cascade of errors
        let code = "def func():\n    if x = 1: # Error: assignment in condition\n        return x\n    else:\n        return 0";
        match parse_code(code) {
            Err(errors) => {
                // There should be just one error reported
                assert_eq!(errors.len(), 1, "Only one error should be reported");
                // The error should be on line 2
                match &errors[0] {
                    ParseError::UnexpectedToken { line, .. } |
                    ParseError::InvalidSyntax { line, .. } |
                    ParseError::EOF { line, .. } => {
                        assert_eq!(*line, 2, "Error should be on line 2");
                    }
                }
            },
            Ok(_) => panic!("Expected parsing to fail"),
        }
    }
}