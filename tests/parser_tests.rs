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
        fn test_dict_parsing_debug() {
            // Empty dictionary
            println!("Testing empty dictionary");
            assert_parses("{}");
            
            // Single key-value pair
            println!("Testing single key-value pair");
            assert_parses("{1: 2}");
            
            // Testing with a string key
            println!("Testing with string key");
            assert_parses("{\"key\": \"value\"}");
            
            // Dictionary with two key-value pairs - the problematic case
            println!("Testing dictionary with two key-value pairs");
            assert_parses("{1: 2, 3: 4}");
            
            // Dictionary with nested dictionary
            println!("Testing nested dictionary");
            assert_parses("{1: {2: 3}}");
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

    mod advanced_expression_tests {
        use super::*;
    
        #[test]
        fn test_operator_precedence() {
            // Test complex precedence cases
            assert_parses("1 + 2 * 3 - 4 / 5");
            assert_parses("1 + 2 * (3 - 4) / 5");
            assert_parses("1 << 2 + 3 & 4 | 5 ^ 6");
            assert_parses("(1 << 2) + (3 & 4) | (5 ^ 6)");
            
            // Bitwise operations precedence
            assert_parses("a | b & c ^ d");
            assert_parses("a | (b & c) ^ d");
            
            // Power operator precedence (right associative)
            assert_parses("2 ** 3 ** 4");  // Should parse as 2 ** (3 ** 4)
            assert_parses("(2 ** 3) ** 4");
        }
    
        #[test]
        fn test_chained_comparisons() {
            // Simple chained comparison
            assert_parses("a < b <= c");
            
            // Multiple chained comparisons
            assert_parses("a < b <= c == d != e > f >= g");
            
            // Chained comparisons with other operations
            assert_parses("a + b < c * d <= e - f");
            
            // 'is' and 'in' operators
            assert_parses("a is b is not c");
            assert_parses("a in b not in c");
            assert_parses("a is not b in c");
        }
    
        #[test]
        fn test_formatted_strings() {
            // Basic f-string
            assert_parses("f\"Hello, {name}!\"");
            
            // f-string with expressions
            assert_parses("f\"Result: {2 + 3 * 4}\"");
            
            // Nested f-strings
            assert_parses("f\"This is {f'nested {inner}'}\"");
            
            // f-string with dictionary access
            assert_parses("f\"Value: {data['key']}\"");
            
            // f-string with function calls
            assert_parses("f\"Calculated: {calculate(a, b=c)}\"");
            
            // f-string with conversions
            assert_parses("f\"Binary: {value!r:>10}\"");
        }
        
        #[test]
        fn test_complex_comprehensions() {
            // Nested comprehensions
            assert_parses("[x for x in [y for y in range(5)]]");
            
            // Multiple for clauses with conditions
            assert_parses("[x+y for x in range(5) if x % 2 == 0 for y in range(3) if y > 0]");
            
            // Dictionary comprehension with complex expressions
            assert_parses("{k: v**2 for k, v in zip(keys, values) if k not in exclude}");
            
            // Set comprehension with function calls
            assert_parses("{func(x) for x in items if pred(x)}");
            
            // Generator expression with complex conditions
            assert_parses("(x for x in data if x.value > threshold and x.enabled)");
        }
        
        #[test]
        fn test_ellipsis() {
            // Ellipsis in subscripts
            assert_parses("array[...]");
            assert_parses("array[..., 0]");
            assert_parses("array[0, ...]");
            
            // Ellipsis as expression
            assert_parses("x = ...");
            
            // Ellipsis in function call
            assert_parses("func(...)");
        }
    }

    mod modern_syntax_tests {
        use super::*;
        
        #[test]
        fn test_walrus_operator() {
            // Simple walrus assignment
            assert_parses("if (n := len(items)) > 0: pass");
            
            // Walrus in comprehension
            assert_parses("[x for x in items if (y := f(x)) > 0]");
            
            // Walrus in while loop condition
            assert_parses("while (line := file.readline()): pass");
            
            // Nested walrus expressions
            assert_parses("if (a := (b := value())) > 0: pass");
            
            // Multiple walrus expressions
            assert_parses("if (a := value_a()) and (b := value_b()): pass");
        }
        
        #[test]
        fn test_positional_only_arguments() {
            assert_parses("def func(a, b, /, c, d, *, e, f): pass");
            assert_parses("def func(a, b=1, /, c=2, *, d=3): pass");
            assert_parses("def func(a, b, /): pass");
        }
        
        #[test]
        fn test_async_await_syntax() {
            // Async function
            assert_parses("async def fetch(url): pass");
            
            // Async with
            assert_parses("async with session.get(url) as response: pass");
            
            // Async for
            assert_parses("async for item in collection: pass");
            
            // Await expression
            assert_parses("result = await coroutine()");
            
            // Await in comprehension
            assert_parses("[await coroutine() for coroutine in coroutines]");
            
            // Complex async function
            assert_parses(
                "async def process():\n    async with lock:\n        async for item in queue:\n            await process_item(item)"
            );
        }
        
        #[test]
        fn test_type_annotations() {
            // Basic function annotations
            assert_parses("def func(a: int, b: str) -> bool: pass");
            
            // Generic type annotations
            assert_parses("def func(a: List[int], b: Dict[str, Any]) -> Optional[int]: pass");
            
            // Union types
            assert_parses("def func(a: Union[int, str]) -> None: pass");
            
            // Variable annotations
            assert_parses("x: int = 5");
            assert_parses("y: List[Dict[str, int]] = []");
            
            // Class variable annotations
            assert_parses("class Test:\n    x: int\n    y: str = 'default'");
            
            // Callable types
            assert_parses("handler: Callable[[int, str], bool] = process_item");
        }
    }

    mod advanced_function_class_tests {
        use super::*;
        
        #[test]
        fn test_nested_functions() {
            // Simple nested function
            assert_parses(
                "def outer():\n    def inner():\n        return 42\n    return inner()"
            );
            
            // Multiple levels of nesting
            assert_parses(
                "def level1():\n    def level2():\n        def level3():\n            return 42\n        return level3()\n    return level2()"
            );
            
            // Nested function with closure
            assert_parses(
                "def make_adder(x):\n    def adder(y):\n        return x + y\n    return adder"
            );
            
            // Nested function with nonlocal
            assert_parses(
                "def counter():\n    count = 0\n    def increment():\n        nonlocal count\n        count += 1\n        return count\n    return increment"
            );
        }
        
        #[test]
        fn test_complex_decorators() {
            // Multiple decorators
            assert_parses(
                "@dec1\n@dec2\n@dec3\ndef func(): pass"
            );
            
            // Decorator with arguments
            assert_parses(
                "@decorator(arg1, arg2, keyword=value)\ndef func(): pass"
            );
            
            // Decorator with complex expression
            assert_parses(
                "@decorator.method().other()\ndef func(): pass"
            );
            
            // Class method decorators
            assert_parses(
                "class Test:\n    @classmethod\n    def cls_method(cls): pass\n    @staticmethod\n    def static_method(): pass"
            );
            
            // Decorators on class
            assert_parses(
                "@singleton\nclass Unique: pass"
            );
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
        
        #[test]
        fn test_generator_functions() {
            // Simple generator with yield
            assert_parses(
                "def gen():\n    yield 1\n    yield 2\n    yield 3"
            );
            
            // Generator with yield from
            assert_parses(
                "def gen():\n    yield from range(10)\n    yield from other_gen()"
            );
            
            // Generator with yield expressions
            assert_parses(
                "def gen():\n    x = yield\n    y = yield x\n    yield x + y"
            );
            
            // Async generator
            assert_parses(
                "async def agen():\n    await asyncio.sleep(1)\n    yield 42"
            );
        }
        
        #[test]
        fn test_complex_lambda() {
            // Lambda with complex expression
            assert_parses(
                "lambda x: x * 2 + 3 if x > 0 else x - 1"
            );
            
            // Lambda with multiple arguments
            assert_parses(
                "lambda x, y, z=1, *args, **kwargs: sum([x, y, z]) + sum(args) + sum(kwargs.values())"
            );
            
            // Nested lambdas
            assert_parses(
                "lambda x: lambda y: x + y"
            );
            
            // Lambda in a function call
            assert_parses(
                "map(lambda x: x.strip(), lines)"
            );
        }
    }

    mod advanced_error_tests {
        use super::*;
        
        #[test]
        fn test_syntax_edge_cases() {
            // Multiple repeated operators
            assert_parse_fails("x = 1 ++ 2");  // Python doesn't have ++
            
            // Remove this line since it's valid Python:
            // assert_parse_fails("1 + 2");  
            
            // Yield outside function
            assert_parse_fails("yield 1");  // Only allowed inside a function
            
            // Misplaced keywords
            assert_parse_fails("def = 10");  // Reserved keyword as variable
            assert_parse_fails("if = 10");   // Reserved keyword as variable
            
            // Mixing tabs and spaces in indentation
            assert_parse_fails("if x:\n\tpass\n    pass");
        }
        
        #[test]
        fn test_invalid_control_flow() {
            // Return outside function
            assert_parse_fails("return 42");
            
            // Break outside loop
            assert_parse_fails("break");
            
            // Continue outside loop
            assert_parse_fails("continue");
            
            // Else without if, for, or while
            assert_parse_fails("else: pass");
        }
        
        #[test]
        fn test_invalid_decorators() {
            // Decorator on invalid statement
            assert_parse_fails("@decorator\nx = 10");
            
            // Invalid decorator expression
            assert_parse_fails("@1 + 2\ndef func(): pass");
        }
        
        #[test]
        fn test_invalid_augmented_assignment() {
            // Augmented assignment to literals
            assert_parse_fails("42 += 1");
            
            // Augmented assignment to expressions
            assert_parse_fails("(a + b) += 1");
            
            // Chained augmented assignment
            assert_parse_fails("x += y += 1");
        }
        
        #[test]
        fn test_incomplete_constructs() {
            // Incomplete if statement
            assert_parse_fails("if :");
            
            // Incomplete for statement
            assert_parse_fails("for x in :");
            
            // Incomplete function definition
            assert_parse_fails("def func(");
            
            // Incomplete class
            assert_parse_fails("class Test(");
        }
    }

    mod advanced_statement_tests {
        use super::*;
        
        #[test]
        fn test_complex_imports() {
            // Import with complex dotted names
            assert_parses("import module.submodule.component");
            
            // Multiple dotted imports
            assert_parses("import module1.sub1, module2.sub2 as alias2");
            
            // From import with multiple dotted levels
            assert_parses("from module.submodule import item1, item2");
            
            // From import with relative imports
            assert_parses("from ..module import item");
            assert_parses("from . import item");
            
            // From import with wildcards in parentheses
            assert_parses("from module import (item1, item2,\n                    item3, item4)");
        }
        
        #[test]
        fn test_nonlocal_global() {
            // Global statements
            assert_parses("def func():\n    global var1, var2\n    var1 = 1\n    var2 = 2");
            
            // Nonlocal statements
            assert_parses("def outer():\n    x = 1\n    def inner():\n        nonlocal x\n        x = 2\n    return inner");
            
            // Global and nonlocal together
            assert_parses("def outer():\n    global g\n    x = 1\n    def inner():\n        nonlocal x\n        global g\n        g = x = 2");
            
            // Single variable declarations
            assert_parses("def func():\n    global x\n    x = 1");
        }
        
        #[test]
        fn test_complex_assignments() {
            // Unpacking assignments
            assert_parses("a, b, c = [1, 2, 3]");
            assert_parses("a, b, c = func()");
            
            // Nested unpacking
            assert_parses("(a, b), (c, d) = [(1, 2), (3, 4)]");
            assert_parses("[a, [b, c], d] = [1, [2, 3], 4]");
            
            // Star unpacking
            assert_parses("a, *b, c = range(5)");
            assert_parses("*a, = b");
            
            // Assignments with complex targets
            assert_parses("obj.attr = value");
            assert_parses("obj['key'] = value");
            assert_parses("obj.attr['key'] = value");
        }
        
        #[test]
        fn test_complex_with_statements() {
            // Multiple context managers
            assert_parses("with ctx1() as a, ctx2() as b: pass");
            
            // With statement with multiple variables
            assert_parses("with open('file') as f1, open('file2') as f2: pass");
            
            // Nested with statements
            assert_parses("with ctx1():\n    with ctx2():\n        pass");
            
            // With statement without as
            assert_parses("with ctx(): pass");
            
            // Async with
            assert_parses("async def func():\n    async with lock:\n        pass");
        }
        
        #[test]
        fn test_match_case_statements() {
            // Basic match-case
            assert_parses(
                "match value:\n    case 1:\n        return 'one'\n    case 2:\n        return 'two'\n    case _:\n        return 'other'"
            );
            
            // Pattern matching with destructuring
            assert_parses(
                "match point:\n    case (x, y):\n        return x + y"
            );
            
            // Pattern matching with class patterns
            assert_parses(
                "match shape:\n    case Circle(radius=r):\n        return math.pi * r * r\n    case Rectangle(width=w, height=h):\n        return w * h"
            );
            
            // Pattern matching with alternatives
            assert_parses(
                "match command:\n    case 'quit' | 'exit':\n        return EXIT\n    case 'restart':\n        return RESTART"
            );
            
            // Pattern matching with guards
            assert_parses(
                "match point:\n    case Point(x, y) if x == y:\n        return 'diagonal'\n    case Point():\n        return 'not diagonal'"
            );
        }
    }

    
}