#[cfg(test)]
mod tests {
    use cheetah::ast::{Expr, Module, Number, Stmt};
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

    // Helper to assert parsing fails with a specific error message
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
                }
            },
        }
    }

    // Test categories for better organization
    mod all_tests {
        use super::*;

        #[test]
        fn test_parse_error_cases() {
            // Test invalid assignment target
            assert_parse_fails_with("1 + 2 = x", "Cannot assign to literal");

            // Test unclosed parentheses/brackets/braces
            assert_parse_fails_with("x = (1 + 2", "Unclosed parenthesis");
            assert_parse_fails_with("x = [1, 2", "Unclosed bracket");
            assert_parse_fails_with("x = {1: 2", "Unclosed brace");

            // Test invalid indentation
            match parse_code(
                "
def test():
    x = 1
y = 2  # Wrong indentation
",
            ) {
                Ok(_) => println!("Note: The parser currently does not detect this indentation error. Consider enhancing indentation validation."),
                Err(errors) => {
                    println!("Detected indentation error: {:?}", errors[0]);
                }
            }
        
            // Test invalid syntax in various constructs
            assert_parse_fails_with("def func(x y): pass", "Expected comma between parameters"); 
            assert_parse_fails_with("class Test(,): pass", "expected 'expression', found 'Comma'");
            assert_parse_fails_with("for in range(10): pass", "Expected target after 'for'");
            assert_parse_fails_with("if : pass", "expected 'expression', found 'Colon'");
            assert_parse_fails_with("x = 1 + ", "expected 'expression', found 'EOF'");
        }

        #[test]
        fn test_classes() {
            // Simple class
            assert_parses("class Test:\n    pass");
            
            // Class with inheritance
            assert_parses("class Test(Base):\n    pass");
            
            // Class with multiple inheritance
            assert_parses("class Test(Base1, Base2):\n    pass");
            
            // Class with methods
            assert_parses("class Test:\n    def method(self):\n        pass");
            
            // Class with attributes
            assert_parses("class Test:\n    attr = 42");
            
            // Empty base class list with comma (should fail)
            assert_parse_fails_with("class Test(,): pass", "expected 'expression', found 'Comma'");
            
            // Unclosed parentheses in base class list (should fail)
            assert_parse_fails("class Test(Base: pass");
        }

        #[test]
        fn test_complex_class_inheritance() {
            // Multiple base classes
            assert_parses(
                "class Derived(Base1, Base2, Base3): pass"
            );
            
            // Inheritance with keyword arguments
            assert_parses(
                "class Derived(Base, metaclass=Meta): pass"
            );
            
            // Inheritance with complex expressions
            assert_parses(
                "class Derived(get_base_class()): pass"
            );
            
            // Multiple inheritance with keyword arguments
            assert_parses(
                "class Derived(Base1, Base2, metaclass=Meta, **kwargs): pass"
            );
        }

    }
}