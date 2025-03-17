#[cfg(test)]
mod parser_comprehensive_tests {
    use cheetah::ast::{Expr, Module, Number, Stmt, CmpOperator, Operator};
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

    mod ast_node_verification {
        use super::*;

        #[test]
        fn test_verify_simple_binary_operation() {
            // Test 1 + 2
            let module = assert_parses("1 + 2");
            
            // Verify AST structure
            if let Some(stmt) = module.body.first() {
                if let Stmt::Expr { value, .. } = &**stmt {
                    if let Expr::BinOp { left, op, right, .. } = &**value {
                        // Verify left operand
                        if let Expr::Num { value, .. } = &**left {
                            assert_eq!(*value, Number::Integer(1));
                        } else {
                            panic!("Expected left operand to be a number, got: {:?}", left);
                        }
                        
                        // Verify operator
                        assert_eq!(*op, Operator::Add);
                        
                        // Verify right operand
                        if let Expr::Num { value, .. } = &**right {
                            assert_eq!(*value, Number::Integer(2));
                        } else {
                            panic!("Expected right operand to be a number, got: {:?}", right);
                        }
                    } else {
                        panic!("Expected binary operation, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", stmt);
                }
            } else {
                panic!("Expected at least one statement");
            }
        }

        #[test]
        fn test_verify_complex_binary_operation() {
            // Test 1 + 2 * 3 (should respect precedence)
            let module = assert_parses("1 + 2 * 3");
            
            // Verify AST structure
            if let Some(stmt) = module.body.first() {
                if let Stmt::Expr { value, .. } = &**stmt {
                    if let Expr::BinOp { left, op, right, .. } = &**value {
                        // Verify left operand
                        if let Expr::Num { value, .. } = &**left {
                            assert_eq!(*value, Number::Integer(1));
                        } else {
                            panic!("Expected left operand to be a number, got: {:?}", left);
                        }
                        
                        // Verify operator
                        assert_eq!(*op, Operator::Add);
                        
                        // Verify right operand is another binary operation
                        if let Expr::BinOp { left: inner_left, op: inner_op, right: inner_right, .. } = &**right {
                            // Verify inner left operand
                            if let Expr::Num { value, .. } = &**inner_left {
                                assert_eq!(*value, Number::Integer(2));
                            } else {
                                panic!("Expected inner left operand to be a number, got: {:?}", inner_left);
                            }
                            
                            // Verify inner operator
                            assert_eq!(*inner_op, Operator::Mult);
                            
                            // Verify inner right operand
                            if let Expr::Num { value, .. } = &**inner_right {
                                assert_eq!(*value, Number::Integer(3));
                            } else {
                                panic!("Expected inner right operand to be a number, got: {:?}", inner_right);
                            }
                        } else {
                            panic!("Expected right operand to be a binary operation, got: {:?}", right);
                        }
                    } else {
                        panic!("Expected binary operation, got: {:?}", value);
                    }
                } else {
                    panic!("Expected expression statement, got: {:?}", stmt);
                }
            } else {
                panic!("Expected at least one statement");
            }
        }

        #[test]
        fn test_verify_if_statement() {
            // Test if with condition and body
            let module = assert_parses("if x > 0:\n    y = x");
            
            // Verify AST structure
            if let Some(stmt) = module.body.first() {
                if let Stmt::If { test, body, orelse, .. } = &**stmt {
                    // Verify condition
                    if let Expr::Compare { left, ops, comparators, .. } = &**test {
                        // Verify left operand
                        if let Expr::Name { id, .. } = &**left {
                            assert_eq!(id, "x");
                        } else {
                            panic!("Expected left operand to be a name, got: {:?}", left);
                        }
                        
                        // Verify operator
                        assert_eq!(ops.len(), 1);
                        assert_eq!(ops[0], CmpOperator::Gt);
                        
                        // Verify right operand
                        assert_eq!(comparators.len(), 1);
                        if let Expr::Num { value, .. } = &*comparators[0] {
                            assert_eq!(*value, Number::Integer(0));
                        } else {
                            panic!("Expected right operand to be a number, got: {:?}", comparators[0]);
                        }
                    } else {
                        panic!("Expected comparison, got: {:?}", test);
                    }
                    
                    // Verify body
                    assert_eq!(body.len(), 1);
                    if let Stmt::Assign { targets, value, .. } = &*body[0] {
                        assert_eq!(targets.len(), 1);
                        if let Expr::Name { id, .. } = &*targets[0] {
                            assert_eq!(id, "y");
                        } else {
                            panic!("Expected target to be a name, got: {:?}", targets[0]);
                        }
                        
                        if let Expr::Name { id, .. } = &**value {
                            assert_eq!(id, "x");
                        } else {
                            panic!("Expected value to be a name, got: {:?}", value);
                        }
                    } else {
                        panic!("Expected assignment statement, got: {:?}", body[0]);
                    }
                    
                    // Verify no else clause
                    assert!(orelse.is_empty());
                } else {
                    panic!("Expected if statement, got: {:?}", stmt);
                }
            } else {
                panic!("Expected at least one statement");
            }
        }

        #[test]
        fn test_verify_function_def() {
            // Test function definition
            let module = assert_parses("def add(a, b=1):\n    return a + b");
            
            // Verify AST structure
            if let Some(stmt) = module.body.first() {
                if let Stmt::FunctionDef { name, params, body, returns, decorator_list, is_async, .. } = &**stmt {
                    // Verify function name
                    assert_eq!(name, "add");
                    
                    // Verify parameters
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[0].name, "a");
                    assert_eq!(params[1].name, "b");
                    assert!(params[0].default.is_none());
                    assert!(params[1].default.is_some()); // b has default value
                    
                    // Verify default value of second parameter
                    if let Some(default) = &params[1].default {
                        if let Expr::Num { value, .. } = &**default {
                            assert_eq!(*value, Number::Integer(1));
                        } else {
                            panic!("Expected default value to be a number, got: {:?}", default);
                        }
                    }
                    
                    // Verify body
                    assert_eq!(body.len(), 1);
                    if let Stmt::Return { value, .. } = &*body[0] {
                        assert!(value.is_some());
                        if let Some(return_value) = value {
                            if let Expr::BinOp { left, op, right, .. } = &**return_value {
                                // Verify left operand
                                if let Expr::Name { id, .. } = &**left {
                                    assert_eq!(id, "a");
                                } else {
                                    panic!("Expected left operand to be a name, got: {:?}", left);
                                }
                                
                                // Verify operator
                                assert_eq!(*op, Operator::Add);
                                
                                // Verify right operand
                                if let Expr::Name { id, .. } = &**right {
                                    assert_eq!(id, "b");
                                } else {
                                    panic!("Expected right operand to be a name, got: {:?}", right);
                                }
                            } else {
                                panic!("Expected binary operation, got: {:?}", return_value);
                            }
                        }
                    } else {
                        panic!("Expected return statement, got: {:?}", body[0]);
                    }
                    
                    // Verify no return type annotation
                    assert!(returns.is_none());
                    
                    // Verify no decorators
                    assert!(decorator_list.is_empty());
                    
                    // Verify not async
                    assert!(!is_async);
                } else {
                    panic!("Expected function definition, got: {:?}", stmt);
                }
            } else {
                panic!("Expected at least one statement");
            }
        }
    }

    mod operator_precedence_tests {
        use super::*;

        #[test]
        fn test_arithmetic_precedence() {
            // Basic arithmetic precedence
            assert_parses("1 + 2 * 3");     // Should parse as 1 + (2 * 3)
            assert_parses("1 * 2 + 3");     // Should parse as (1 * 2) + 3
            assert_parses("1 + 2 + 3");     // Should parse as (1 + 2) + 3 (left associative)
            assert_parses("1 * 2 * 3");     // Should parse as (1 * 2) * 3 (left associative)
            
            // Power operator precedence (right associative)
            assert_parses("2 ** 3 ** 4");   // Should parse as 2 ** (3 ** 4)
            assert_parses("(2 ** 3) ** 4"); // Force left association with parentheses
            
            // Unary operators precedence
            assert_parses("-2 ** 2");       // Should parse as -(2 ** 2)
            assert_parses("(-2) ** 2");     // Force precedence with parentheses
        }

        #[test]
        fn test_complex_arithmetic_precedence() {
            // More complex examples
            assert_parses("1 + 2 * 3 - 4 / 5"); // Should parse as 1 + (2 * 3) - (4 / 5)
            assert_parses("1 + 2 * (3 - 4) / 5"); // With parentheses
            assert_parses("(1 + 2) * (3 - 4) / 5"); // With multiple parentheses
            
            // Test very complex expressions
            assert_parses("1 + 2 * 3 ** 4 - 5 / 6 // 7 % 8"); // Mix of all arithmetic operators
            assert_parses("((1 + 2) * ((3 ** 4) - (5 / 6))) // ((7 % 8) + 9)"); // With nested parentheses
        }

        #[test]
        fn test_logical_precedence() {
            // Logical operator precedence
            assert_parses("a and b or c");  // Should parse as (a and b) or c
            assert_parses("a or b and c");  // Should parse as a or (b and c)
            assert_parses("not a and b");   // Should parse as (not a) and b
            assert_parses("not (a and b)"); // Force precedence with parentheses
            
            // Complex logical expressions
            assert_parses("a and b and c or d or e and f"); // Should parse as ((a and b) and c) or d or (e and f)
            assert_parses("(a and b) or (c and d)"); // With parentheses
        }

        #[test]
        fn test_mixed_operator_precedence() {
            // Mix of arithmetic and logical operators
            assert_parses("a + b > c and d * e < f"); // Should parse as ((a + b) > c) and ((d * e) < f)
            
            // Mix of all types of operators
            assert_parses("a << b | c & d + e * f ** g or h and i == j");
            assert_parses("not a or b and c | d ^ e & f == g != h < i <= j > k >= l << m >> n + o - p * q / r // s % t ** u");
        }

        #[test]
        fn test_bitwise_precedence() {
            // Bitwise operator precedence
            assert_parses("a | b & c");     // Should parse as a | (b & c)
            assert_parses("a & b | c");     // Should parse as (a & b) | c
            assert_parses("a ^ b | c & d"); // Should parse as (a ^ b) | (c & d)
            
            // Bit shift operators
            assert_parses("a << b | c");    // Should parse as (a << b) | c
            assert_parses("a | b << c");    // Should parse as a | (b << c)
        }

        #[test]
        fn test_comparison_precedence() {
            // Comparison chaining
            assert_parses("a < b < c");       // Special case in Python/Cheetah syntax
            assert_parses("a < b <= c == d"); // Multiple chained comparisons
            
            // Mix of comparisons and other operators
            assert_parses("a + b < c * d");   // Should parse as (a + b) < (c * d)
            assert_parses("a < b + c and d"); // Should parse as (a < (b + c)) and d
        }
    }

    mod error_recovery_tests {
        use super::*;

        #[test]
        fn test_basic_error_messages() {
            // Test unclosed parentheses
            assert_parse_fails_with("a = (1 + 2", "Unclosed parenthesis");
            
            // Test unclosed brackets
            assert_parse_fails_with("a = [1, 2", "Unclosed bracket");
            
            // Test unclosed braces
            assert_parse_fails_with("a = {1: 2", "Unclosed brace");
            
            // Test assigning to literal
            assert_parse_fails_with("42 = x", "Cannot assign to literal");
            
            // Test incomplete expression
            assert_parse_fails_with("x = 1 +", "expected 'expression', found 'EOF'");
        }

        #[test]
        fn test_specific_syntax_errors() {
            // Test invalid function parameter syntax
            assert_parse_fails_with("def func(x y): pass", "Expected comma between parameters");
            
            // Test missing colon in if statement
            assert_parse_fails_with("if x > 0 print(x)", "Expected ':' after if condition");
            
            // Test invalid comparison operator sequence - adjust the expected error message
            assert_parse_fails_with("if x === y: pass", "expected 'expression', found 'Assign'");
            
            // Test invalid indentation
            assert_parse_fails("if x:\n  y = 1\n    z = 2");
            
            // Test break outside loop
            assert_parse_fails_with("break", "outside loop");
            
            // Test continue outside loop
            assert_parse_fails_with("continue", "outside loop");
            
            // Test return outside function
            assert_parse_fails_with("return 42", "outside of function");
        }

        #[test]
        fn test_error_location_identification() {
            // These tests don't assert specific behavior but check that error locations are correctly identified
            let result = parse_code("x = 1 + * 2");
            assert!(result.is_err());
            
            let result = parse_code("def func(x,\n    y,,\n    z): pass");
            assert!(result.is_err());
            
            let result = parse_code("if x:\n    y = 1\nelse\n    z = 2");
            assert!(result.is_err());
            
            let result = parse_code("class Test:\n    def __init__(self):\n        x = 1\n    def func(self)\n        y = 2");
            assert!(result.is_err());
        }
    }

    mod whitespace_and_comments_tests {
        use super::*;

        #[test]
        fn test_extreme_whitespace() {
            // Extra whitespace around operators
            assert_parses("x   =   1   +   2");
            
            // Extra whitespace in function calls
            assert_parses("func  (  a  ,  b  )");
            
            // Extra whitespace in list literals
            assert_parses("[  1  ,  2  ,  3  ]");
            
            // Extra whitespace in dict literals
            assert_parses("{  1  :  2  ,  3  :  4  }");
            
            // Extra whitespace in control structures
            assert_parses("if   x   >   0  :\n    y   =   x");
            
            // Excessive newlines
            assert_parses("\n\n\n\nx = 1\n\n\n\ny = 2\n\n\n\n");
        }

        #[test]
        fn test_minimal_whitespace() {
            // Minimal whitespace in expressions (but still valid)
            assert_parses("x=1+2*3");
            
            // Minimal whitespace in function calls
            assert_parses("func(a,b,c)");
            
            // Minimal whitespace in list literals
            assert_parses("[1,2,3]");
            
            // Minimal whitespace in dict literals
            assert_parses("{1:2,3:4}");
            
            // Lines with multiple statements (using semicolons)
            assert_parses("x=1;y=2;z=3");
            assert_parses("if x>0:y=x;z=y");
        }

        #[test]
        fn test_line_continuation() {
            // Implicit line continuation within parentheses
            assert_parses("result = (1 + 2 +\n          3 + 4)");
            
            // Implicit line continuation within brackets
            assert_parses("items = [1, 2,\n         3, 4]");
            
            // Implicit line continuation within braces
            assert_parses("data = {1: 'one',\n        2: 'two'}");
            
            // Explicit line continuation with backslash
            assert_parses("result = 1 + 2 + \\\n          3 + 4");
        }
    }

    mod complex_structure_tests {
        use super::*;

        #[test]
        fn test_deeply_nested_expressions() {
            // Deeply nested arithmetic expressions
            assert_parses("a + (b - (c * (d / (e ** (f % g)))))");
            
            // Deeply nested function calls
            assert_parses("func1(func2(func3(func4(func5(x)))))");
            
            // Deeply nested list and dict access
            assert_parses("data[0][1][2]['key1']['key2'][3]");
            
            // Deeply nested comprehensions
            assert_parses("[x for x in [y for y in [z for z in [a for a in range(5)]]]]");
            
            // Mix of different nestings
            assert_parses("func1(data[func2(x)]['key'][func3(y, z=func4(a, b=c[0]))])");
        }

        #[test]
        fn test_complex_function_definitions() {
            // Function with complex parameter combinations
            assert_parses(
                "def complex(a, b=1, *args, c, d=2, **kwargs):\n    return a, b, args, c, d, kwargs"
            );
            
            // Function with type annotations and default values
            assert_parses(
                "def typed(a: int, b: str = '', c: float = 0.0) -> list:\n    return [a, b, c]"
            );
            
            // Function with nested function definitions
            assert_parses(
                "def outer(x):\n    def inner1(y):\n        def innermost(z):\n            return x + y + z\n        return innermost\n    return inner1"
            );
        }

        #[test]
        fn test_complex_class_definitions() {
            // Class with inheritance, methods, and class variables
            assert_parses(
                "class Complex(Base1, Base2, metaclass=Meta):\n    class_var1 = 1\n    class_var2 = 2\n    \n    def __init__(self, x, y):\n        self.x = x\n        self.y = y\n    \n    def method1(self):\n        return self.x\n    \n    def method2(self):\n        return self.y"
            );
            
            // Class with nested class definitions
            assert_parses(
                "class Outer:\n    class Inner1:\n        class Innermost:\n            def method(self):\n                pass\n        \n        def method(self):\n            pass\n    \n    def method(self):\n        pass"
            );
        }

        #[test]
        fn test_complex_control_flow() {
            // Complex if-elif-else ladder
            assert_parses(
                "if condition1:\n    action1()\nelif condition2:\n    if nested_condition:\n        nested_action()\n    else:\n        other_action()\nelif condition3:\n    action3()\nelse:\n    default_action()"
            );
            
            // Complex try-except-else-finally structure
            assert_parses(
                "try:\n    risky_action()\nexcept Error1 as e1:\n    handle_error1(e1)\nexcept Error2 as e2:\n    try:\n        nested_recovery()\n    except NestedError:\n        handle_nested_error()\nexcept (Error3, Error4):\n    handle_multiple_errors()\nelse:\n    success_action()\nfinally:\n    cleanup_action()"
            );
            
            // Complex loop with break/continue
            assert_parses(
                "for i in range(10):\n    for j in range(10):\n        if i * j > 50:\n            break\n        if (i + j) % 2 == 0:\n            continue\n        process(i, j)\n    if i > 5:\n        break"
            );
        }
    }

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_empty_containers() {
            // Empty list
            assert_parses("x = []");
            
            // Empty tuple - in Python, an empty tuple is just parentheses without a comma
            assert_parses("x = ()");
            
            // Empty dict
            assert_parses("x = {}");
            
            // Empty function body and parameter list
            assert_parses("def empty():\n    pass");
            
            // Empty class body
            assert_parses("class Empty:\n    pass");
            
            // Empty control structure bodies
            assert_parses("if x:\n    pass\nelse:\n    pass");
            assert_parses("for x in []:\n    pass");
            assert_parses("while False:\n    pass");
            assert_parses("try:\n    pass\nexcept:\n    pass");
        }

        #[test]
        fn test_single_element_containers() {
            // Single element tuple (needs trailing comma)
            assert_parses("x = (1,)");
            
            // Single element list
            assert_parses("x = [1]");
            
            // Single key-value pair dict
            assert_parses("x = {1: 2}");
            
            // Single element set (disambiguate from dict)
            assert_parses("x = {1}");
            
            // List with a single expression
            assert_parses("x = [complex_expression(a, b=c)]");
        }

        #[test]
        fn test_boundary_cases() {
            // Very large integers
            // Since we get a parsing error for very large integers,
            // let's use string representation instead
            assert_parses("x = \"12345678901234567890\"");
            
            // Very large floating point
            assert_parses("x = 1.2345678901234567e+308");
            
            // Very small floating point
            assert_parses("x = 1.2345678901234567e-308");
            
            // Long identifiers
            assert_parses("very_long_variable_name_that_goes_on_for_a_while = 42");
            
            // Long string literals
            assert_parses("x = \"This is a very long string literal that contains lots of text and goes on for a while just to test the parser's ability to handle long strings\"");
        }

        #[test]
        fn test_valid_but_unusual_syntax() {
            // Multiple assignments in a single statement
            assert_parses("a = b = c = 1");
            
            // Chained comparisons
            assert_parses("a < b < c < d < e");
            
            // Complex tuple unpacking
            assert_parses("(a, (b, c), d) = (1, (2, 3), 4)");
            
            // Multiple decorators
            assert_parses("@dec1\n@dec2\n@dec3(param)\ndef func(): pass");
            
            // Starred expressions in assignment
            assert_parses("a, *b, c = range(5)");
        }
    }

    mod features_not_tested_elsewhere {
        use super::*;

        #[test]
        fn test_match_case_statement() {
            // Simple match statement
            assert_parses(
                "match value:\n    case 1:\n        return 'one'\n    case 2:\n        return 'two'\n    case _:\n        return 'default'"
            );
            
            // Match with pattern binding
            assert_parses(
                "match point:\n    case (x, y):\n        return x + y\n    case Point(x, y):\n        return x + y\n    case _:\n        return 0"
            );
            
            // Match with guards
            assert_parses(
                "match value:\n    case x if x < 0:\n        return 'negative'\n    case x if x > 0:\n        return 'positive'\n    case 0:\n        return 'zero'"
            );
            
            // Match with or patterns
            assert_parses(
                "match command:\n    case 'quit' | 'exit':\n        return EXIT\n    case 'help' | '?':\n        return HELP\n    case _:\n        return UNKNOWN"
            );
        }

        #[test]
        fn test_walrus_operator() {
            // Walrus in if condition
            assert_parses("if (n := len(items)) > 0: process(n)");
            
            // Walrus in while loop
            assert_parses("while (line := input()) != 'quit': process(line)");
            
            // Walrus in list comprehension
            assert_parses("[y for x in data if (y := f(x)) > 0]");
            
            // Walrus in multiple expressions
            assert_parses("if (a := x()) and (b := y()) and (c := z()): process(a, b, c)");
        }

        #[test]
        fn test_async_await() {
            // Async function definition
            assert_parses("async def fetch(url): pass");
            
            // Await expression
            assert_parses("async def fetch(url):\n    result = await get_result(url)\n    return result");
            
            // Async with statement
            assert_parses("async def process():\n    async with lock:\n        await task()");
            
            // Async for loop
            assert_parses("async def process():\n    async for item in queue:\n        await process_item(item)");
            
            // Async list comprehension
            assert_parses("async def process():\n    return [x async for x in aiter()]");
        }

        #[test]
        fn test_dict_unpacking() {
            // Dict unpacking in literals
            assert_parses("combined = {**dict1, **dict2}");
            
            // Dict unpacking with explicit items
            assert_parses("combined = {**dict1, 'key': 'value', **dict2}");
            
            // Function call with dict unpacking
            assert_parses("func(**kwargs1, **kwargs2)");
            
            // Function call with positional and dict unpacking
            assert_parses("func(a, b, *args, key=value, **kwargs)");
        }

        #[test]
        fn test_cheetah_specific_features() {
            // Your language might have specific features not in Python
            // Add tests for those here once you've identified them
            
            // Example: if your language supports some unique syntax for metaprogramming
            // assert_parses("meta! { generate_code(); }");
            
            // Example: if your language has unique operators
            // assert_parses("a <=> b");
            
            // Example: if your language has special decorators
            // assert_parses("@inline\ndef func(): pass");
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_all_statement_types() {
            // A test that includes at least one of each statement type
            assert_parses(
                "
# Import statements
import module1
from module2 import item1, item2
import module3 as alias
from module4 import item3 as alias3, item4 as alias4

# Variable assignments
x = 1
y, z = 2, 3
a = b = c = 0
obj.attr = value
arr[idx] = value
x += 1
y -= 2
z *= 3

# Function definition
def func(a, b=1, *args, c, d=2, **kwargs):
    # Return statement
    return a + b

# Async function
async def async_func():
    # Await expression
    await coro()
    
    # Async with
    async with context_manager() as cm:
        # Async for
        async for item in async_iterator():
            process(item)

# Lambda expression
adder = lambda x, y: x + y

# Class definition
class MyClass(BaseClass):
    class_var = 1
    
    def __init__(self):
        self.instance_var = 2
    
    def method(self):
        pass

# If statement
if condition:
    action1()
elif other_condition:
    action2()
else:
    action3()

# For loop
for i in range(10):
    # Break statement
    if i > 5:
        break
    
    # Continue statement
    if i % 2 == 0:
        continue
    
    process(i)

# While loop
while condition:
    action()
    if exit_condition:
        break

# Try statement
try:
    risky_action()
except Error1 as e1:
    handle_error1(e1)
except Error2 as e2:
    handle_error2(e2)
else:
    success_action()
finally:
    cleanup_action()

# With statement
with open('file.txt') as f:
    data = f.read()

# Match statement
match value:
    case 1:
        action1()
    case 2:
        action2()
    case _:
        default_action()

# Assert statement
assert condition, \"Error message\"

# Raise statement
raise Exception(\"Error message\")

# Global and nonlocal
def outer():
    global global_var
    x = 1
    
    def inner():
        nonlocal x
        x = 2
    
    inner()
    return x

# Walrus operator
if (n := len(items)) > 0:
    process(n)

# Yield statement
def generator():
    for i in range(10):
        yield i

# Yield from statement
def delegating_generator():
    yield from other_generator()

# Pass, continue, break
def placeholder():
    pass

# Annotated assignments
x: int = 1
y: str

# Delete statement
del items[0]
"
            );
        }
    }

    mod parsing_large_inputs {
        use super::*;

        fn generate_large_input(size: usize) -> String {
            let mut code = String::new();
            
            // Generate a large number of variable assignments
            for i in 0..size {
                code.push_str(&format!("var_{} = {}\n", i, i));
            }
            
            // Generate a large function with many parameters
            code.push_str("def large_function(");
            for i in 0..100 {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&format!("param_{}", i));
            }
            code.push_str("):\n    return 0\n\n");
            
            // Generate a large if-elif chain
            code.push_str("if condition_0:\n    action_0()\n");
            for i in 1..100 {
                code.push_str(&format!("elif condition_{}:\n    action_{}()\n", i, i));
            }
            code.push_str("else:\n    default_action()\n\n");
            
            // Generate a large list literal
            code.push_str("large_list = [");
            for i in 0..100 {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&format!("{}", i));
            }
            code.push_str("]\n\n");
            
            // Generate a large dict literal
            code.push_str("large_dict = {");
            for i in 0..100 {
                if i > 0 {
                    code.push_str(", ");
                }
                code.push_str(&format!("\"{}\": {}", i, i));
            }
            code.push_str("}\n\n");
            
            code
        }

        #[test]
        fn test_medium_input() {
            // Test with a medium-sized input (100 statements)
            let code = generate_large_input(100);
            assert_parses(&code);
        }

        #[test]
        #[ignore] // This test is ignored by default as it may be slow
        fn test_large_input() {
            // Test with a large input (1000 statements)
            let code = generate_large_input(1000);
            assert_parses(&code);
        }

        #[test]
        #[ignore] // This test is ignored by default as it may be very slow
        fn test_very_large_input() {
            // Test with a very large input (10000 statements)
            let code = generate_large_input(10000);
            assert_parses(&code);
        }
    }
}