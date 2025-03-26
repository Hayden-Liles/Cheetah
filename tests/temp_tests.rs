#[allow(dead_code)]
#[cfg(test)]
mod advanced_parser_tests {
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

    // Helper functions to test parsing, similar to the ones in temp_tests.rs
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

    fn assert_parses(source: &str) -> Module {
        match parse_code(source) {
            Ok(module) => module,
            Err(errors) => {
                println!("\n================================");
                println!("PARSING FAILED FOR CODE SNIPPET:");
                println!("================================");
                println!("{}", source);
                println!("\nERRORS:");
                
                for error in &errors {
                    // Print detailed error information
                    println!("- {}", ErrorFormatter(error));
                }
                
                panic!("Parsing failed with {} errors", errors.len());
            },
        }
    }

    fn assert_parse_fails(source: &str) {
        match parse_code(source) {
            Ok(_) => {
                println!("\n=============================");
                println!("EXPECTED FAILURE BUT SUCCEEDED:");
                println!("=============================");
                println!("{}", source);
                panic!("Expected parsing to fail, but it succeeded");
            },
            Err(_) => (), // Pass if there's any error
        }
    }

    fn assert_parse_fails_with(source: &str, expected_error_substr: &str) {
        match parse_code(source) {
            Ok(_) => {
                println!("\n=============================");
                println!("EXPECTED FAILURE BUT SUCCEEDED:");
                println!("=============================");
                println!("{}", source);
                println!("\nExpected error containing: '{}'", expected_error_substr);
                panic!("Expected parsing to fail, but it succeeded");
            },
            Err(errors) => {
                let error_message = format!("{}", ErrorFormatter(&errors[0]));
                if !error_message.contains(expected_error_substr) {
                    println!("\n==========================");
                    println!("WRONG ERROR TYPE DETECTED:");
                    println!("==========================");
                    println!("Code: {}", source);
                    println!("\nExpected error containing: '{}'", expected_error_substr);
                    println!("Actual error: '{}'", error_message);
                    
                    panic!("Error message doesn't match expected substring");
                }
            },
        }
    }
    
}