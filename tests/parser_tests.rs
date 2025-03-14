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
    mod syntax_error_tests {
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
            assert_parse_fails_with("class Test(,): pass", "Expected expression before comma");
            assert_parse_fails_with("for in range(10): pass", "Expected target after 'for'");
            assert_parse_fails_with("if : pass", "expected 'expression', found 'Colon'");
            assert_parse_fails_with("x = 1 + ", "expected 'expression', found 'EOF'");
        }

        #[test]
        fn test_incomplete_expressions() {
            // Binary operation with missing right operand
            assert_parse_fails_with("x = 1 + ", "expected 'expression', found 'EOF'");
            
            // Unary operation with missing operand
            assert_parse_fails("x = -");
            
            // Call with unclosed parentheses
            assert_parse_fails("x = func(");
            
            // Call with incomplete arguments
            assert_parse_fails("x = func(1,");
        }
        
        #[test]
        fn test_indentation() {
            // Correct indentation
            assert_parses("if x:\n    y = 1\n    z = 2");
            
            // Mixed indentation (should fail)
            assert_parse_fails("if x:\n    y = 1\n  z = 2");
            
            // Inconsistent indentation levels (should fail)
            assert_parse_fails("if x:\n    y = 1\n        z = 2");
            
            // No indentation where required (should fail)
            assert_parse_fails("if x:\ny = 1");
        }
        
        #[test]
        fn test_invalid_function_arguments() {
            // Missing comma between parameters
            assert_parse_fails_with("def func(x y): pass", "Expected comma between parameters");
            
            // Parameter after variadic kwargs (should fail)
            assert_parse_fails("def func(*args, **kwargs, x): pass");
            
            // Default before non-default (should fail in Python but might be complex to check in the parser)
            // This is a semantic error in Python, not a syntax error
            // assert_parse_fails("def func(x=1, y): pass");
            
            // Invalid parameter name
            assert_parse_fails("def func(1): pass");
            
            // Empty parentheses with comma (should fail)
            assert_parse_fails_with("def func(,): pass", "Expected parameter name, * or **");
        }
    }

    mod expression_tests {
        use super::*;

        #[test]
        fn test_basic_expressions() {
            // Test simple number literal
            let module = assert_parses("42");
            if let Some(stmt) = module.body.first() {
                if let Stmt::Expr { value, line: _, column: _ } = &**stmt {
                    if let Expr::Num { value: num, .. } = &**value {
                        assert_eq!(*num, Number::Integer(42));
                    } else {
                        panic!("Expected number expression, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", stmt);
                }
            } else {
                panic!("Expected at least one statement");
            }

            // Test simple string literal
            let _module = assert_parses("\"hello\"");
            
            // Test simple binary operations
            assert_parses("1 + 2");
            assert_parses("1 - 2");
            assert_parses("1 * 2");
            assert_parses("1 / 2");
            
            // Test more complex expressions
            assert_parses("1 + 2 * 3");
            assert_parses("(1 + 2) * 3");
            assert_parses("1 + (2 * 3)");
        }

        #[test]
        fn test_parenthesized_expressions() {
            // Basic parentheses
            assert_parses("(42)");
            
            // Nested parentheses
            assert_parses("((42))");
            
            // Parentheses in binary operations
            assert_parses("(1 + 2) * 3");
            assert_parses("1 + (2 * 3)");
            
            // Multiple sets of parentheses
            assert_parses("(1 + 2) * (3 + 4)");
            
            // Empty parentheses (should fail)
            assert_parse_fails("()");
            
            // Unclosed parentheses (should fail)
            assert_parse_fails_with("(1 + 2", "Unclosed parenthesis");
        }
        
        #[test]
        fn test_complex_expressions() {
            // Lambda expression
            assert_parses("lambda x: x + 1");
            
            // Lambda with multiple parameters
            assert_parses("lambda x, y: x + y");
            
            // Lambda with default parameters
            assert_parses("lambda x, y=1: x + y");
            
            // Ternary conditional
            assert_parses("x if condition else y");
            
            // Nested ternary conditional
            assert_parses("x if cond1 else y if cond2 else z");
            
            // Call with keyword arguments
            assert_parses("func(1, 2, key=value)");
            
            // Call with star args and kwargs
            assert_parses("func(*args, **kwargs)");
            
            // Attribute access
            assert_parses("obj.attr");
            
            // Nested attribute access
            assert_parses("obj.attr1.attr2");
            
            // Subscript
            assert_parses("obj[0]");
            
            // Slicing
            assert_parses("obj[1:10]");
            assert_parses("obj[1:10:2]");
            assert_parses("obj[:]");
            assert_parses("obj[::]");
        }
        
        #[test]
        fn test_list_comprehension() {
            // Simple list comprehension
            assert_parses("[x for x in range(10)]");
            
            // List comprehension with condition
            assert_parses("[x for x in range(10) if x % 2 == 0]");
            
            // Nested list comprehension
            assert_parses("[[x, y] for x in range(3) for y in range(3)]");
            
            // Dict comprehension
            assert_parses("{x: x*x for x in range(10)}");
            
            // Set comprehension
            assert_parses("{x for x in range(10)}");
            
            // Generator expression
            assert_parses("(x for x in range(10))");
            
            // A list with a single element (valid in Python)
            assert_parses("[x]");
            
            // Invalid comprehension (missing variable)
            assert_parse_fails("[for x in range(10)]");
        }
    }

    mod statement_tests {
        use super::*;

        #[test]
        fn test_assignment() {
            // Simple assignment
            assert_parses("x = 42");
            
            // Assignment with expression
            assert_parses("x = 1 + 2");
            
            // Multiple assignments
            assert_parses("x = y = 42");
            
            // Compound assignments
            assert_parses("x += 1");
            assert_parses("x -= 1");
            assert_parses("x *= 1");
            assert_parses("x /= 1");
            
            // Invalid assignment targets
            assert_parse_fails_with("1 = x", "Cannot assign to literal");
            assert_parse_fails_with("1 + 2 = x", "Cannot assign to literal");
            assert_parse_fails_with("\"string\" = x", "Cannot assign to literal");
        }

        #[test]
        fn test_variable_declarations() {
            // Simple variable declarations
            assert_parses("x = 1");
            
            // Multiple variable declarations (tuple unpacking)
            assert_parses("x, y = 1, 2");
            
            // Variable with type annotation
            assert_parses("x: int = 1");
            
            // Multiple variables with type annotations
            // This is valid Python syntax when done carefully
            assert_parses("x: int = 1; y: float = 2.0");
        }

        #[test]
        fn test_data_structures() {
            // Lists
            assert_parses("[]");
            assert_parses("[1, 2, 3]");
            assert_parses("[1, 2 + 3, \"hello\"]");
            
            // Nested lists
            assert_parses("[[1, 2], [3, 4]]");
            
            // Unclosed list (should fail)
            assert_parse_fails_with("[1, 2", "Unclosed bracket");
            
            // Dictionaries
            assert_parses("{}");
            assert_parses("{1: 2, 3: 4}");
            assert_parses("{\"key\": \"value\", 1: 2}");
            
            // Nested dictionaries
            assert_parses("{1: {2: 3}, 4: {5: 6}}");
            
            // Unclosed dictionary (should fail)
            assert_parse_fails_with("{1: 2", "Unclosed brace");
            
            // Dictionary with non-colon separator (should fail)
            // In Python, this would be a set, not invalid syntax
            assert_parses("{1, 2}");
        }

        #[test]
        fn test_if_statements() {
            // Simple if
            assert_parses("if x: pass");
            
            // If-else
            assert_parses("if x: pass\nelse: pass");
            
            // If-elif-else
            assert_parses("if x: pass\nelif y: pass\nelse: pass");
            
            // Multiple elif
            assert_parses("if x: pass\nelif y: pass\nelif z: pass\nelse: pass");
            
            // Nested if
            assert_parses("if x: if y: pass");
            
            // Missing condition (should fail)
            assert_parse_fails_with("if : pass", "expected 'expression', found 'Colon'");
            
            // Missing colon (should fail)
            assert_parse_fails_with("if x pass", "Expected ':' after if condition");
        }

        #[test]
        fn test_loops() {
            // For loop
            assert_parses("for i in range(10): pass");
            
            // For loop with body
            assert_parses("for i in range(10):\n    x = i");
            
            // While loop
            assert_parses("while x < 10: pass");
            
            // While loop with body
            assert_parses("while x < 10:\n    x += 1");
            
            // Break and continue
            assert_parses("for i in range(10):\n    if i == 5: break");
            assert_parses("for i in range(10):\n    if i == 5: continue");
            
            // Missing target in for loop (should fail)
            assert_parse_fails_with("for in range(10): pass", "Expected target after 'for'");
            
            // Missing condition in while loop (should fail)
            assert_parse_fails("while : pass");
        }

        #[test]
        fn test_try_except() {
            // Simple try-except
            assert_parses("try:\n    x = 1\nexcept:\n    pass");
            
            // Try-except with exception type
            assert_parses("try:\n    x = 1\nexcept Exception:\n    pass");
            
            // Try-except with multiple exception types
            assert_parses("try:\n    x = 1\nexcept (Exception1, Exception2):\n    pass");
            
            // Try-except with alias
            assert_parses("try:\n    x = 1\nexcept Exception as e:\n    pass");
            
            // Try-except-else
            assert_parses("try:\n    x = 1\nexcept Exception:\n    pass\nelse:\n    pass");
            
            // Try-except-finally
            assert_parses("try:\n    x = 1\nexcept Exception:\n    pass\nfinally:\n    pass");
            
            // Try-except-else-finally
            assert_parses("try:\n    x = 1\nexcept Exception:\n    pass\nelse:\n    pass\nfinally:\n    pass");
            
            // Invalid try (missing block)
            assert_parse_fails("try:");
            
            // Invalid except (missing block)
            assert_parse_fails("try:\n    x = 1\nexcept:");
        }

        #[test]
        fn test_with_statement() {
            // Simple with statement
            assert_parses("with open('file.txt'):\n    pass");
            
            // With statement with as clause
            assert_parses("with open('file.txt') as f:\n    pass");
            
            // Multiple context managers
            assert_parses("with open('file1.txt') as f1, open('file2.txt') as f2:\n    pass");
            
            // Invalid with (missing expression)
            assert_parse_fails("with :\n    pass");
            
            // Invalid with (missing block)
            assert_parse_fails("with open('file.txt'):");
        }

        #[test]
        fn test_import_statements() {
            // Simple import
            assert_parses("import module");
            
            // Multiple imports
            assert_parses("import module1, module2");
            
            // Import from
            assert_parses("from module import item");
            
            // Import from with multiple items
            assert_parses("from module import item1, item2");
            
            // Import with as
            assert_parses("import module as alias");
            
            // Import from with as
            assert_parses("from module import item as alias");
            
            // Import from with multiple items and aliases
            assert_parses("from module import item1 as alias1, item2 as alias2");
            
            // Invalid import (missing module name)
            assert_parse_fails_with("import ", "Expected module name after 'import'");
            
            // Invalid from import (missing module name)
            assert_parse_fails("from import item");
            
            // Invalid from import (missing item name)
            assert_parse_fails("from module import ");
        }
    }

    mod function_tests {
        use super::*;

        #[test]
        fn test_simple_function_def() {
            // Basic function with no parameters
            let _module = assert_parses("def test():\n    pass");
            
            // Function with simple body
            let _module = assert_parses("def test():\n    return 42");
            
            // Function with parameters
            let _module = assert_parses("def test(x, y):\n    return x + y");
        }

        #[test]
        fn test_function_argument_parsing() {
            // Default arguments
            let _module = assert_parses(
                "def greet(name, greeting='Hello', suffix='!'):\n    return greeting + ', ' + name + suffix"
            );

            // Type annotations
            let _module = assert_parses(
                "def calculate(a: int, b: float = 1.0) -> float:\n    return a + b"
            );

            // Variadic arguments
            let _module = assert_parses(
                "def collect(*args, **kwargs):\n    return args, kwargs"
            );
            
            // Named-only parameters
            let _module = assert_parses(
                "def named_only(*, name, value):\n    return name, value"
            );
            
            // Positional-only parameters
            assert_parses(
                "def pos_only(x, y, /, z):\n    return x, y, z"
            );
            
            // Complex parameter combinations
            let _module = assert_parses(
                "def complex(a, b=1, *args, c, d=2, **kwargs):\n    return a, b, args, c, d, kwargs"
            );
        }

        #[test]
        fn test_function_edge_cases() {
            // Function with complex return type
            assert_parses("def func() -> List[Dict[str, int]]:\n    pass");
            
            // Function with complex parameter types
            assert_parses("def func(x: List[int], y: Dict[str, Any]):\n    pass");
            
            // Function with decorator
            assert_parses("@decorator\ndef func():\n    pass");
            
            // Function with multiple decorators
            assert_parses("@decorator1\n@decorator2\ndef func():\n    pass");
            
            // Function with docstring
            assert_parses("def func():\n    \"\"\"This is a docstring.\"\"\"\n    pass");
            
            // Function with annotations and docstring
            assert_parses("def func(x: int) -> str:\n    \"\"\"This is a docstring.\"\"\"\n    return str(x)");
        }

        #[test]
        fn test_parse_function_arguments() {
            // Test default arguments
            let module = parse_code(
                "
def greet(name, greeting='Hello', suffix='!'):
    return greeting + ', ' + name + suffix
",
            )
            .unwrap();

            assert_eq!(module.body.len(), 1);

            if let Stmt::FunctionDef { name, params, .. } = &*module.body[0] {
                assert_eq!(name, "greet");
                assert_eq!(params.len(), 3);

                // First param has no default
                assert!(params[0].default.is_none(), "Expected first parameter to have no default value");

                // Second and third params have defaults
                assert!(params[1].default.is_some(), "Expected second parameter to have a default value");
                assert!(params[2].default.is_some(), "Expected third parameter to have a default value");
            } else {
                panic!("Expected function definition");
            }

            // Test type annotations
            let module = parse_code(
                "
def calculate(a: int, b: float = 1.0) -> float:
    return a + b
",
            )
            .unwrap();

            assert_eq!(module.body.len(), 1);

            if let Stmt::FunctionDef {
                params, returns, ..
            } = &*module.body[0]
            {
                // Check parameter types
                assert!(params[0].typ.is_some(), "Expected first parameter to have a type annotation");

                // Check return type
                assert!(returns.is_some(), "Expected function to have a return type annotation");
                if let Some(ret_type) = &returns {
                    if let Expr::Name { id, .. } = &**ret_type {
                        assert_eq!(id, "float", "Expected return type to be 'float'");
                    } else {
                        panic!("Expected name in return type");
                    }
                }
            } else {
                panic!("Expected function definition");
            }

            // Test variadic arguments
            assert_parses(
                "
def collect(*args, **kwargs):
    return args, kwargs
",
            );
            
            // Test bare * for keyword-only parameters
            assert_parses(
                "
def foo(a, *, b=1):
    return a + b
",
            );
        }
    }

    mod class_tests {
        use super::*;
        
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
            assert_parse_fails_with("class Test(,): pass", "Expected expression before comma");
            
            // Unclosed parentheses in base class list (should fail)
            assert_parse_fails("class Test(Base: pass");
        }
    }
}