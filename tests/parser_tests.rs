#[cfg(test)]
mod tests {
    use cheetah::ast::{Expr, Module, Number, Stmt};
    use cheetah::lexer::{Lexer, Token, TokenType};
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

    // Format the source code for error display with highlighted line
    fn format_source_with_error(source: &str, line: usize, column: usize) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut result = String::new();
        
        // Determine line numbers to display (context around error)
        let start_line = if line > 2 { line - 2 } else { 0 };
        let end_line = std::cmp::min(line + 2, lines.len());
        
        // Calculate padding for line numbers
        let line_num_width = end_line.to_string().len();
        
        for (i, &line_str) in lines.iter().enumerate().skip(start_line).take(end_line - start_line) {
            let line_num = i + 1;
            
            // Add line number and source code
            let prefix = format!("{:>width$} | ", line_num, width = line_num_width);
            result.push_str(&prefix);
            result.push_str(line_str);
            result.push('\n');
            
            // Add error indicator if this is the error line
            if line_num == line {
                let mut indicator = " ".repeat(prefix.len());
                indicator.push_str(&" ".repeat(column.saturating_sub(1)));
                indicator.push('^');
                result.push_str(&indicator);
                result.push('\n');
            }
        }
        
        result
    }

    // Helper function to print token stream for debugging
    fn print_token_stream(source: &str) {
        println!("\n=== TOKEN STREAM ANALYSIS ===");
        println!("Source code: {}", source);
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        if !lexer.get_errors().is_empty() {
            println!("LEXER ERRORS:");
            for error in lexer.get_errors() {
                println!("- Line {}, Column {}: {}", error.line, error.column, error.message);
            }
            return;
        }
        
        println!("TOKENS:");
        let mut token_pos = 0;
        for token in &tokens {
            println!("{}: {:?} at line {}, column {} (\"{}\")", 
                token_pos, 
                token.token_type, 
                token.line, 
                token.column,
                token.lexeme
            );
            token_pos += 1;
        }
        println!("=== END TOKEN STREAM ===\n");
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

    // Helper function to assert parsing succeeds and print the AST for debugging
    fn assert_parses(source: &str) -> Module {
        println!("\n>>> TESTING PARSER WITH: '{}'", source);
        print_token_stream(source);
        
        match parse_code(source) {
            Ok(module) => {
                println!("✓ PARSING SUCCEEDED");
                module
            },
            Err(errors) => {
                println!("\n================================");
                println!("PARSING FAILED FOR CODE SNIPPET:");
                println!("================================");
                println!("{}", source);
                println!("\nERRORS:");
                
                for error in &errors {
                    // Print detailed error information
                    println!("- {}", ErrorFormatter(error));
                    
                    // Show code snippet with error position highlighted
                    match error {
                        ParseError::UnexpectedToken { line, column, .. } |
                        ParseError::InvalidSyntax { line, column, .. } |
                        ParseError::EOF { line, column, .. } => {
                            println!("\nCode context:");
                            println!("{}", format_source_with_error(source, *line, *column));
                        }
                    }
                    println!("--------------------------------");
                }
                
                panic!("Parsing failed with {} errors", errors.len());
            },
        }
    }

    // Helper to assert parsing fails with any error
    fn assert_parse_fails(source: &str) {
        println!("\n>>> TESTING PARSER FAILURE WITH: '{}'", source);
        print_token_stream(source);
        
        match parse_code(source) {
            Ok(_) => {
                println!("\n=============================");
                println!("EXPECTED FAILURE BUT SUCCEEDED:");
                println!("=============================");
                println!("{}", source);
                panic!("Expected parsing to fail, but it succeeded");
            },
            Err(_) => {
                println!("✓ PARSING FAILED AS EXPECTED");
            },
        }
    }

    // Helper to assert parsing fails with a specific error message
    fn assert_parse_fails_with(source: &str, expected_error_substr: &str) {
        println!("\n>>> TESTING PARSER FAILURE WITH: '{}'", source);
        println!(">>> EXPECTED ERROR: '{}'", expected_error_substr);
        print_token_stream(source);
        
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
                    
                    // Show code snippet with error position highlighted
                    match &errors[0] {
                        ParseError::UnexpectedToken { line, column, .. } |
                        ParseError::InvalidSyntax { line, column, .. } |
                        ParseError::EOF { line, column, .. } => {
                            println!("\nCode context:");
                            println!("{}", format_source_with_error(source, *line, *column));
                        }
                    }
                    
                    panic!("Error message doesn't match expected substring");
                } else {
                    println!("✓ PARSING FAILED WITH EXPECTED ERROR MESSAGE");
                }
            },
        }
    }

    // Test categories for better organization
    mod lambda_tests {
        use super::*;

        #[test]
        fn test_simple_lambda() {
            // Start with the simplest lambda
            assert_parses("lambda: None");
            
            // Single parameter
            assert_parses("lambda x: x");
        }
        
        #[test]
        fn test_lambda_with_multiple_params() {
            // Multiple parameters
            assert_parses("lambda x, y: x + y");
            
            // With default value
            assert_parses("lambda x, y=10: x + y");
        }
        
        #[test]
        fn test_lambda_with_varargs() {
            // Test varargs (*args) in isolation
            assert_parses("lambda *args: sum(args)");
        }
        
        #[test]
        fn test_lambda_with_kwargs() {
            // Test kwargs (**kwargs) in isolation
            assert_parses("lambda **kwargs: sum(kwargs.values())");
        }
        
        #[test]
        fn test_lambda_mixed_args() {
            // Mix of regular and varargs
            assert_parses("lambda x, *args: x + sum(args)");
            
            // Mix of regular and kwargs
            assert_parses("lambda x, **kwargs: x + sum(kwargs.values())");
            
            // Mix of regular, varargs and kwargs
            assert_parses("lambda x, *args, **kwargs: x + sum(args) + sum(kwargs.values())");
        }
        
        #[test]
        fn test_lambda_with_default_and_special_args() {
            // Regular params with default values
            assert_parses("lambda x, y=10, z=20: x + y + z");
            
            // Regular params with defaults and varargs
            assert_parses("lambda x, y=10, *args: x + y + sum(args)");
            
            // Regular params with defaults and kwargs
            assert_parses("lambda x, y=10, **kwargs: x + y + sum(kwargs.values())");
            
            // The complete mix
            assert_parses("lambda x, y=10, z=20, *args, **kwargs: x + y + z + sum(args) + sum(kwargs.values())");
        }
        
        #[test]
        fn test_complex_lambda_expressions() {
            // Lambda with complex expression
            assert_parses("lambda x: x * 2 + 3 if x > 0 else x - 1");
            
            // Nested lambdas
            assert_parses("lambda x: lambda y: x + y");
            
            // Lambda in a function call
            assert_parses("map(lambda x: x.strip(), lines)");
            
            // The most complex test case from the original test
            assert_parses("lambda x, y, z=1, *args, **kwargs: sum([x, y, z]) + sum(args) + sum(kwargs.values())");
        }
    }
}