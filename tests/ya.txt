#[cfg(test)]
mod comprehensive_tests {
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

    // Helper function to parse code and return a Module or error
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

    // Helper to verify parsing succeeds
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
                    println!("- {}", ErrorFormatter(error));
                }
                
                panic!("Parsing failed with {} errors", errors.len());
            },
        }
    }

    // Helper to verify parsing fails
    fn assert_parse_fails(source: &str) {
        match parse_code(source) {
            Ok(_) => {
                panic!("Expected parsing to fail, but it succeeded: {}", source);
            },
            Err(_) => (), // Pass if it fails
        }
    }

    // Helper to verify parsing fails with a specific error message
    fn assert_parse_fails_with(source: &str, expected_error_substr: &str) {
        match parse_code(source) {
            Ok(_) => {
                panic!("Expected parsing to fail with '{}', but it succeeded: {}", 
                       expected_error_substr, source);
            },
            Err(errors) => {
                let error_message = format!("{}", ErrorFormatter(&errors[0]));
                if !error_message.contains(expected_error_substr) {
                    panic!("Expected error containing '{}', got: '{}'\nCode: {}", 
                           expected_error_substr, error_message, source);
                }
            },
        }
    }

    // Tests for Python 3.10+ features
    mod python_310_tests {
        use super::*;
        
        #[test]
        fn test_pattern_matching_basic() {
            // Basic match cases
            assert_parses(
                "match value:
                    case 1:
                        print('one')
                    case 2:
                        print('two')
                    case _:
                        print('other')"
            );
        }
        
        #[test]
        fn test_pattern_matching_complex() {
            // More complex patterns
            assert_parses(
                "match point:
                    case (0, 0):
                        print('Origin')
                    case (0, y):
                        print(f'Y={y}')
                    case (x, 0):
                        print(f'X={x}')
                    case (x, y):
                        print(f'X={x}, Y={y}')
                    case _:
                        print('Not a point')"
            );
        }
        
        #[test]
        fn test_pattern_matching_class_patterns() {
            // Class patterns in match
            assert_parses(
                "match shape:
                    case Circle(radius=r):
                        print(f'Circle with radius {r}')
                    case Rectangle(width=w, height=h):
                        print(f'Rectangle with width {w} and height {h}')
                    case _:
                        print('Unknown shape')"
            );
        }
        
        #[test]
        fn test_pattern_matching_guards() {
            // Pattern matching with guards
            assert_parses(
                "match value:
                    case x if x < 0:
                        print('Negative')
                    case x if x > 0:
                        print('Positive')
                    case 0:
                        print('Zero')"
            );
        }
        
        #[test]
        fn test_pattern_matching_or_patterns() {
            // Or patterns in match
            assert_parses(
                "match command:
                    case 'quit' | 'exit':
                        print('Exiting')
                    case 'help' | '?':
                        print('Help')
                    case _:
                        print('Unknown command')"
            );
        }
        
        #[test]
        fn test_pattern_matching_nested() {
            // Nested patterns
            assert_parses(
                "match data:
                    case {'name': name, 'age': age, 'address': {'city': city, 'country': country}}:
                        print(f'{name} from {city}, {country}')
                    case {'name': name, 'age': age}:
                        print(f'{name}, {age} years old')
                    case _:
                        print('Invalid data')"
            );
        }
    }
    
    // Tests for Python 3.9 features
    mod python_39_tests {
        use super::*;
        
        #[test]
        fn test_dictionary_union() {
            // Dictionary union with |
            assert_parses("combined = dict1 | dict2");
            
            // Dictionary union with |=
            assert_parses("dict1 |= dict2");
            
            // Complex dictionary unions
            assert_parses("result = {1: 'a', 2: 'b'} | {2: 'c', 3: 'd'} | {4: 'e'}");
            
            // Dictionary comprehension with union
            assert_parses("result = {k: v for k, v in dict1.items()} | {k: v for k, v in dict2.items()}");
        }
        
        #[test]
        fn test_type_annotation_simplifications() {
            // Built-in generic type annotations
            assert_parses("def func(items: list[int]) -> dict[str, list[int]]: pass");
            
            // Type annotations with multiple arguments
            assert_parses("def func(x: tuple[int, str, bool]) -> None: pass");
            
            // Variable annotations
            assert_parses("names: list[str] = []");
            assert_parses("values: dict[str, tuple[int, float]] = {}");
        }
    }
    
    // Tests for Python 3.8 features
    mod python_38_tests {
        use super::*;
        
        #[test]
        fn test_positional_only_parameters() {
            // Basic positional-only parameters
            assert_parses("def func(a, b, /, c, d): pass");
            
            // Positional-only with default values
            assert_parses("def func(a, b=0, /, c=0, d=0): pass");
            
            // Positional-only with varargs and kwargs
            assert_parses("def func(a, b, /, c, *args, d=0, **kwargs): pass");
            
            // Only positional-only params
            assert_parses("def func(a, b, /): pass");
            
            // All parameter types
            assert_parses("def func(a, b=0, /, c=0, *args, d, e=0, **kwargs): pass");
        }
        
        #[test]
        fn test_assignment_expressions() {
            // Basic walrus operator
            assert_parses("if (n := len(items)) > 0: print(n)");
            
            // Walrus in list comprehensions
            assert_parses("values = [y for x in data if (y := f(x)) > 0]");
            
            // Walrus in while loop
            assert_parses("while (line := input()) != 'quit': print(line)");
            
            // Multiple walrus operators
            assert_parses("if (a := x()) and (b := y()): print(a, b)");
            
            // Nested walrus operators
            assert_parses("print((a := 1), (b := a + 1), (c := b + 1))");
            
            // Walrus in dictionary comprehension
            assert_parses("result = {k: (v := f(k)) for k in keys if v > 0}");
        }
        
        #[test]
        fn test_f_string_self_documenting() {
            // Self-documenting expressions in f-strings
            assert_parses("print(f'{x=}')");
            
            // Multiple self-documenting expressions
            assert_parses("print(f'{x=}, {y=}, {z=}')");
            
            // Self-documenting expressions with format specs
            assert_parses("print(f'{x=:5d}, {y=:.2f}')");
            
            // Complex expressions
            assert_parses("print(f'{x + y=}, {func(x, y)=}')");
        }
    }
    
    // Tests for Python 3.7 features
    mod python_37_tests {
        use super::*;
        
        #[test]
        fn test_postponed_evaluation_annotations() {
            // from __future__ import annotations
            assert_parses("from __future__ import annotations\n\ndef func(x: 'SomeType') -> 'ReturnType': pass");
            
            // Complex forward references
            assert_parses(
                "from __future__ import annotations\n\nclass Tree:\n    def __init__(self, left: 'Tree', right: 'Tree'): pass"
            );
        }
        
        #[test]
        fn test_dataclasses() {
            // Basic dataclass
            assert_parses(
                "@dataclass\nclass Point:\n    x: int\n    y: int = 0"
            );
            
            // Dataclass with methods
            assert_parses(
                "@dataclass\nclass Point:\n    x: int\n    y: int = 0\n    \n    def distance(self) -> float:\n        return (self.x**2 + self.y**2) ** 0.5"
            );
        }
    }
    
    // Tests for Python 3.6 features
    mod python_36_tests {
        use super::*;
        
        #[test]
        fn test_f_strings() {
            // Simple f-string
            assert_parses("name = 'world'\nprint(f'Hello, {name}!')");
            
            // F-string with expressions
            assert_parses("print(f'Result: {2 + 3 * 4}')");
            
            // Nested f-strings
            assert_parses("inner = 'world'\nprint(f'Hello, {f\"{inner}\"} again!')");
            
            // F-string with dictionary access
            assert_parses("data = {'name': 'Alice'}\nprint(f'Hello, {data[\"name\"]}!')");
            
            // F-string with format specs
            assert_parses("value = 42\nprint(f'Value: {value:08d}')");
            
            // F-string with conversions
            assert_parses("obj = Object()\nprint(f'Debug: {obj!r}, String: {obj!s}')");
        }
        
        #[test]
        fn test_variable_annotations() {
            // Simple annotations
            assert_parses("x: int = 1");
            
            // Annotations without initialization
            assert_parses("x: int");
            
            // Complex annotations
            assert_parses("x: List[Dict[str, Tuple[int, float]]]");
            
            // Class attribute annotations
            assert_parses("class C:\n    x: int\n    y: str = 'hello'");
            
            // Multiple annotations
            assert_parses("x: int = 1; y: float = 2.0; z: str");
        }
        
        #[test]
        fn test_async_generators() {
            // Async generators
            assert_parses(
                "async def gen():\n    for i in range(10):\n        await asyncio.sleep(0.1)\n        yield i"
            );
            
            // Async comprehensions
            assert_parses(
                "async def func():\n    return [x async for x in async_iter()]"
            );
            
            // Async with
            assert_parses(
                "async def func():\n    async with resource() as r:\n        await process(r)"
            );
        }
    }
    
    // Tests for specific parser functions
    mod parser_function_tests {
        use super::*;
        
        #[test]
        fn test_parse_expression() {
            // Simple expressions
            assert_parses("42");
            assert_parses("1 + 2 * 3");
            
            // Complex expressions
            assert_parses("a or b and c");
            assert_parses("not a or not b");
            assert_parses("a if b else c");
            assert_parses("(a + b) * (c - d)");
            
            // Calls, attributes, subscripts
            assert_parses("func(a, b, c=d)");
            assert_parses("obj.attr.method()");
            assert_parses("arr[0][1][2]");
            
            // Lambda expressions
            assert_parses("lambda x, y=1: x + y");
            assert_parses("lambda *args, **kwargs: sum(args) + sum(kwargs.values())");
        }
        
        #[test]
        fn test_parse_statement() {
            // Assignment statements
            assert_parses("x = 1");
            assert_parses("x, y = 1, 2");
            assert_parses("x += 1");
            
            // Import statements
            assert_parses("import module");
            assert_parses("from module import item");
            
            // Control flow statements
            assert_parses("if x: pass");
            assert_parses("for i in range(10): pass");
            assert_parses("while True: pass");
            
            // Function and class definitions
            assert_parses("def func(): pass");
            assert_parses("class Test: pass");
            
            // Try-except statements
            assert_parses("try: pass\nexcept Exception: pass");
            
            // With statements
            assert_parses("with open('file') as f: pass");
        }
        
        #[test]
        fn test_parse_class_arguments() {
            // Empty class
            assert_parses("class Test: pass");
            
            // Single base class
            assert_parses("class Test(Base): pass");
            
            // Multiple base classes
            assert_parses("class Test(Base1, Base2): pass");
            
            // Keyword arguments
            assert_parses("class Test(metaclass=Meta): pass");
            
            // Base and keyword arguments
            assert_parses("class Test(Base, metaclass=Meta): pass");
            
            // Function call as base
            assert_parses("class Test(get_base()): pass");
            
            // Star args (not valid in Python but should parse)
            assert_parses("class Test(*bases): pass");
            
            // Star kwargs
            assert_parses("class Test(**kwargs): pass");
            
            // Multiple kwargs
            assert_parses("class Test(metaclass=Meta, **kwargs): pass");
            
            // Complex class arguments
            assert_parses("class Test(Base1, Base2, *bases, metaclass=Meta, **kwargs): pass");
        }
    }
    
    // Tests for parser error handling
    mod error_handling_tests {
        use super::*;
        
        #[test]
        fn test_syntax_error_messages() {
            // Missing closing parenthesis
            assert_parse_fails_with("func(1, 2", "Unclosed parenthesis");
            
            // Missing colon in if statement
            assert_parse_fails_with("if condition print('yes')", "Expected ':' after if condition");
            
            // Invalid assignment target
            assert_parse_fails_with("1 + 2 = 3", "Cannot assign to");
            
            // Parameter after **kwargs
            assert_parse_fails_with("def func(**kwargs, x): pass", "Parameter after **kwargs");
            
            // Yield outside function
            assert_parse_fails_with("yield 5", "outside of function");
            
            // Break outside loop
            assert_parse_fails_with("break", "outside loop");
            
            // Continue outside loop
            assert_parse_fails_with("continue", "outside loop");
        }
        
        #[test]
        fn test_indentation_errors() {
            // Inconsistent indentation
            assert_parse_fails("if x:\n  pass\n    pass");
            
            // Dedent errors
            assert_parse_fails("if x:\n    pass\n  else:\n    pass");
            
            // Missing indent after colon
            assert_parse_fails("if x:\npass");
        }
    }
    
    // Tests for edge cases and stress tests
    mod edge_case_tests {
        use super::*;
        
        #[test]
        fn test_deeply_nested_expressions() {
            // Deeply nested expressions
            assert_parses("(((((((1 + 2) * 3) / 4) ** 5) % 6) << 7) | 8)");
            
            // Deeply nested function calls
            assert_parses("func1(func2(func3(func4(func5(func6(func7(func8(1, 2), 3), 4), 5), 6), 7), 8), 9)");
            
            // Deeply nested list comprehensions
            assert_parses("[x for x in [y for y in [z for z in [a for a in range(10)]]]]");
        }
        
        #[test]
        fn test_long_identifiers() {
            // Very long identifiers
            let long_name = "a".repeat(1000);
            assert_parses(&format!("{} = 42", long_name));
            
            // Long function names
            assert_parses(&format!("def {}(): pass", long_name));
            
            // Long class names
            assert_parses(&format!("class {}(): pass", long_name));
        }
        
        #[test]
        fn test_unicode_identifiers() {
            // Unicode variable names
            assert_parses("π = 3.14159");
            
            // Unicode function names
            assert_parses("def résumé(): pass");
            
            // Unicode class names
            assert_parses("class Привет: pass");
            
            // Mix of Unicode and ASCII
            assert_parses("def func_привет(α, β=2): return α + β");
        }
        
        #[test]
        fn test_extreme_whitespace() {
            // Lots of whitespace
            assert_parses("x    =     42      ");
            
            // Multiple blank lines
            assert_parses("\n\n\n\nx = 42\n\n\n\n");
            
            // Whitespace in expressions
            assert_parses("( 1 + 2 ) * ( 3 + 4 )");
        }
    }
    
    // Tests for specific Python syntax features
    mod python_syntax_tests {
        use super::*;
        
        #[test]
        fn test_decorators() {
            // Single decorator
            assert_parses("@decorator\ndef func(): pass");
            
            // Multiple decorators
            assert_parses("@decorator1\n@decorator2\n@decorator3\ndef func(): pass");
            
            // Decorated class
            assert_parses("@singleton\nclass Singleton: pass");
            
            // Decorator with arguments
            assert_parses("@decorator(arg1, arg2, kwarg=value)\ndef func(): pass");
            
            // Complex decorators
            assert_parses("@decorator.method().attribute\ndef func(): pass");
        }
        
        #[test]
        fn test_generator_expressions() {
            // Simple generator expressions
            assert_parses("(x for x in range(10))");
            
            // Generator with condition
            assert_parses("(x for x in range(10) if x % 2 == 0)");
            
            // Multiple for clauses
            assert_parses("(x+y for x in range(5) for y in range(5))");
            
            // Multiple conditions
            assert_parses("(x for x in range(10) if x % 2 == 0 if x % 3 == 0)");
            
            // Generator with multiple for clauses and conditions
            assert_parses("(x+y for x in range(5) if x > 0 for y in range(5) if y > 0)");
        }
        
        #[test]
        fn test_complex_for_statements() {
            // For with else
            assert_parses("for i in range(10):\n    pass\nelse:\n    pass");
            
            // For with break in the body
            assert_parses("for i in range(10):\n    if i == 5: break");
            
            // For with continue
            assert_parses("for i in range(10):\n    if i % 2 == 0: continue\n    print(i)");
            
            // Nested for loops
            assert_parses("for i in range(10):\n    for j in range(10):\n        print(i, j)");
            
            // For with complex iterator expression
            assert_parses("for x, (y, z) in zip(a, [(1, 2), (3, 4)]):\n    print(x, y, z)");
        }
        
        #[test]
        fn test_function_annotations() {
            // Function with parameter and return annotations
            assert_parses("def func(x: int, y: float) -> str:\n    return str(x + y)");
            
            // Default values with annotations
            assert_parses("def func(x: int = 0, y: float = 0.0) -> str:\n    return str(x + y)");
            
            // Complex type annotations
            assert_parses("def func(x: List[Dict[str, Tuple[int, float]]]) -> Optional[str]:\n    pass");
            
            // Forward references in quotes
            assert_parses("def func(x: 'Node', y: 'Tree') -> 'Result':\n    pass");
            
            // Self-referential annotations
            assert_parses("def func(x: 'List[Node]') -> 'Node':\n    pass");
        }
        
        #[test]
        fn test_context_managers() {
            // Basic with statement
            assert_parses("with open('file.txt') as f:\n    data = f.read()");
            
            // Multiple context managers
            assert_parses("with open('in.txt') as inf, open('out.txt', 'w') as outf:\n    outf.write(inf.read())");
            
            // Nested with statements
            assert_parses(
                "with open('file1.txt') as f1:\n    with open('file2.txt') as f2:\n        data = f1.read() + f2.read()"
            );
            
            // Context manager without as clause
            assert_parses("with mutex:\n    critical_section()");
            
            // Tuple target in with
            assert_parses("with conn() as (cur, commit):\n    pass");
        }
    }
    
    // Tests for arithmetic and logical operations
    mod operation_tests {
        use super::*;
        
        #[test]
        fn test_arithmetic_operations() {
            // Basic operations
            assert_parses("a + b");
            assert_parses("a - b");
            assert_parses("a * b");
            assert_parses("a / b");
            assert_parses("a // b");  // Floor division
            assert_parses("a % b");   // Modulo
            assert_parses("a ** b");  // Power
            
            // Unary operations
            assert_parses("+a");
            assert_parses("-b");
            
            // Combined operations
            assert_parses("a + b * c");
            assert_parses("(a + b) * c");
            assert_parses("a ** b ** c");  // Right-associative
            assert_parses("(a ** b) ** c");
        }
        
        #[test]
        fn test_bitwise_operations() {
            // Bitwise operations
            assert_parses("a & b");  // AND
            assert_parses("a | b");  // OR
            assert_parses("a ^ b");  // XOR
            assert_parses("~a");     // NOT
            assert_parses("a << b"); // Left shift
            assert_parses("a >> b"); // Right shift
            
            // Combined bitwise operations
            assert_parses("a | b & c");
            assert_parses("a | (b & c)");
            assert_parses("(a | b) & c");
            assert_parses("a & b ^ c | d");
        }
        
        #[test]
        fn test_logical_operations() {
            // Logical operations
            assert_parses("a and b");
            assert_parses("a or b");
            assert_parses("not a");
            
            // Combined logical operations
            assert_parses("a and b or c");
            assert_parses("a and (b or c)");
            assert_parses("not a and not b");
            assert_parses("not (a and b)");
            
            // Logical operations with comparisons
            assert_parses("a < b and c > d");
            assert_parses("a == b or c != d");
        }
        
        #[test]
        fn test_comparison_operations() {
            // Comparison operations
            assert_parses("a == b");
            assert_parses("a != b");
            assert_parses("a < b");
            assert_parses("a <= b");
            assert_parses("a > b");
            assert_parses("a >= b");
            
            // Identity and membership tests
            assert_parses("a is b");
            assert_parses("a is not b");
            assert_parses("a in b");
            assert_parses("a not in b");
            
            // Chained comparisons
            assert_parses("a < b < c");
            assert_parses("a <= b <= c");
            assert_parses("a > b > c");
            assert_parses("a >= b >= c");
            assert_parses("a == b == c");
            assert_parses("a < b > c != d");
        }
    }
}