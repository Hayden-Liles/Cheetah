#[cfg(test)]
mod parser_specialized_tests {
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

    mod context_sensitive_parsing_tests {
        use super::*;

        #[test]
        fn test_comprehension_variable_scoping() {
            // Variables in comprehensions have their own scope
            assert_parses("[x for x in range(10)]");
            
            // Nested comprehensions with same variable names
            assert_parses("[x for x in [y for y in [x for x in range(10)]]]");
            
            // Using variables from outer scopes
            assert_parses("x = 10\n[y for y in range(x)]");
            
            // Shadowing variables in comprehensions
            assert_parses("x = 10\n[x for x in range(5)]");
        }

        #[test]
        fn test_lambda_variable_scoping() {
            // Lambda parameters have their own scope
            assert_parses("x = 10\nf = lambda x: x + 1");
            
            // Lambda accessing outer scope
            assert_parses("x = 10\nf = lambda y: x + y");
            
            // Nested lambdas with same parameter names
            assert_parses("f = lambda x: lambda x: x");
            
            // Lambda shadowing variables
            assert_parses("x = 10\nf = lambda x: x * x");
        }

        #[test]
        fn test_function_variable_scoping() {
            // Function parameters have their own scope
            assert_parses("x = 10\ndef f(x):\n    return x + 1");
            
            // Function accessing outer scope
            assert_parses("x = 10\ndef f(y):\n    return x + y");
            
            // Nested functions with same parameter names
            assert_parses("def outer(x):\n    def inner(x):\n        return x * 2\n    return inner(x)");
            
            // Global and nonlocal declarations
            assert_parses("x = 10\ndef f():\n    global x\n    x = 20");
            assert_parses("def outer():\n    x = 10\n    def inner():\n        nonlocal x\n        x = 20\n    inner()\n    return x");
        }

        #[test]
        fn test_except_variable_scoping() {
            // Exception aliases have their own scope
            assert_parses("try:\n    risky()\nexcept Exception as e:\n    handle(e)");
            
            // Exception aliases don't leak outside except blocks
            assert_parses("try:\n    risky()\nexcept Exception as e:\n    handle(e)\nprint('Done')");
        }

        #[test]
        fn test_with_variable_scoping() {
            // With aliases have their own scope
            assert_parses("with open('file.txt') as f:\n    process(f)");
            
            // Multiple with aliases
            assert_parses("with open('input.txt') as input_file, open('output.txt', 'w') as output_file:\n    output_file.write(input_file.read())");
            
            // Nested with statements with same alias names
            assert_parses("with open('outer.txt') as f:\n    with open('inner.txt') as f:\n        process(f)");
        }

        #[test]
        fn test_for_variable_scoping() {
            // For loop variables have their own scope
            assert_parses("for i in range(10):\n    print(i)");
            
            // For loop with tuple unpacking
            assert_parses("for k, v in items.items():\n    print(k, v)");
            
            // Nested for loops with same variable names
            assert_parses("for i in range(10):\n    for i in range(i):\n        print(i)");
        }

        #[test]
        fn test_list_comprehension_context() {
            // List comprehension target should be in store context
            assert_parses("[x for x in range(10)]");
            
            // Tuple unpacking in list comprehension
            assert_parses("[(x, y) for x, y in pairs]");
            
            // Nested unpacking in list comprehension
            assert_parses("[(x, (y, z)) for x, (y, z) in complex_pairs]");
        }

        #[test]
        fn test_assignment_context() {
            // Simple assignments
            assert_parses("x = 1");
            
            // Multiple assignments
            assert_parses("x = y = z = 1");
            
            // Tuple unpacking
            assert_parses("a, b = 1, 2");
            
            // List unpacking
            assert_parses("[a, b] = [1, 2]");
            
            // Complex unpacking
            assert_parses("[(a, b), (c, d)] = [(1, 2), (3, 4)]");
        }

        #[test]
        fn test_load_store_delete_contexts() {
            // Load context
            assert_parses("x");
            assert_parses("print(x)");
            
            // Store context
            assert_parses("x = 1");
            assert_parses("x += 1");
            
            // Delete context
            assert_parses("del x");
            assert_parses("del x[0]");
            assert_parses("del x.attr");
        }

        #[test]
        fn test_break_continue_return_context() {
            // Break and continue in loop context
            assert_parses("for i in range(10):\n    if i > 5:\n        break\n    if i % 2 == 0:\n        continue");
            
            // Break and continue in nested loops
            assert_parses("for i in range(10):\n    for j in range(10):\n        if i * j > 50:\n            break\n        if i + j < 5:\n            continue");
            
            // Return in function context
            assert_parses("def func():\n    return 42");
            
            // Conditional return
            assert_parses("def func(x):\n    if x > 0:\n        return x\n    else:\n        return 0");
        }
    }

    mod complex_comprehensions {
        use super::*;

        #[test]
        fn test_nested_comprehensions() {
            // Nested list comprehensions
            assert_parses("[[x + y for y in range(5)] for x in range(5)]");
            
            // Deeply nested comprehensions
            assert_parses("[[[x + y + z for z in range(5)] for y in range(5)] for x in range(5)]");
            
            // Mixed nested comprehensions
            assert_parses("[{x + y for y in range(5)} for x in range(5)]");
            
            // Comprehension with lambda
            assert_parses("[lambda y: x + y for x in range(5)]");
        }

        #[test]
        fn test_comprehension_conditions() {
            // Simple condition
            assert_parses("[x for x in range(100) if x % 2 == 0]");
            
            // Multiple conditions
            assert_parses("[x for x in range(100) if x % 2 == 0 if x % 3 == 0]");
            
            // Complex conditions
            assert_parses("[x for x in range(100) if all(x % i != 0 for i in range(2, int(x ** 0.5) + 1))]");
        }

        #[test]
        fn test_comprehension_iterations() {
            // Multiple for clauses
            assert_parses("[x + y for x in range(5) for y in range(5)]");
            
            // Multiple for clauses with conditions
            assert_parses("[x + y for x in range(10) if x % 2 == 0 for y in range(10) if y % 2 == 1]");
            
            // Deeply nested for clauses
            assert_parses("[x + y + z for x in range(5) for y in range(5) for z in range(5)]");
        }

        #[test]
        fn test_dict_comprehensions() {
            // Simple dict comprehension
            assert_parses("{x: x*x for x in range(10)}");
            
            // Dict comprehension with condition
            assert_parses("{x: x*x for x in range(100) if x % 10 == 0}");
            
            // Dict comprehension with multiple iterations
            assert_parses("{(x, y): x*y for x in range(5) for y in range(5)}");
            
            // Dict comprehension with tuple unpacking
            assert_parses("{k: v for k, v in zip(keys, values)}");
        }

        #[test]
        fn test_set_comprehensions() {
            // Simple set comprehension
            assert_parses("{x for x in range(10)}");
            
            // Set comprehension with condition
            assert_parses("{x for x in range(100) if x % 10 == 0}");
            
            // Set comprehension with multiple iterations
            assert_parses("{x + y for x in range(5) for y in range(5)}");
            
            // Set comprehension with complex expressions
            assert_parses("{(x, y) for x in range(5) for y in range(5) if x != y}");
        }

        #[test]
        fn test_generator_expressions() {
            // Simple generator expression
            assert_parses("sum(x for x in range(10))");
            
            // Generator with condition
            assert_parses("sum(x for x in range(100) if x % 2 == 0)");
            
            // Generator with multiple iterations
            assert_parses("sum(x + y for x in range(5) for y in range(5))");
            
            // Nested generators
            assert_parses("sum(sum(y for y in range(x)) for x in range(10))");
        }

        #[test]
        fn test_async_comprehensions() {
            // Async for in comprehension
            assert_parses("async def func():\n    return [x async for x in aiter()]");
            
            // Async for with condition
            assert_parses("async def func():\n    return [x async for x in aiter() if pred(x)]");
            
            // Async for with regular for
            assert_parses("async def func():\n    return [x + y async for x in aiter() for y in range(5)]");
            
            // Async with regular and async for
            assert_parses("async def func():\n    return [x + y for x in range(5) async for y in aiter()]");
        }
    }

    mod advanced_expressions {
        use super::*;

        #[test]
        fn test_complex_tuple_unpacking() {
            // Basic tuple unpacking
            assert_parses("a, b = 1, 2");
            
            // Nested tuple unpacking
            assert_parses("(a, (b, c)) = (1, (2, 3))");
            
            // Tuple unpacking with lists
            assert_parses("a, b = [1, 2]");
            
            // Tuple unpacking with starred expressions
            assert_parses("a, *b, c = range(10)");
            
            // Multiple unpackings
            assert_parses("a, b = c, d = 1, 2");
        }

        #[test]
        fn test_starred_expressions() {
            // Starred in assignment
            assert_parses("a, *b, c = range(10)");
            
            // Starred in list literal
            assert_parses("[1, 2, *rest, 3, 4]");
            
            // Starred in tuple literal
            assert_parses("(1, 2, *rest, 3, 4)");
            
            // Starred in function call
            assert_parses("func(1, 2, *args, key=value)");
            
            // Multiple starred expressions
            assert_parses("[*start, *middle, *end]");
        }

        #[test]
        fn test_double_starred_expressions() {
            // Double starred in dict literal
            assert_parses("{**dict1, 'key': 'value', **dict2}");
            
            // Double starred in function call
            assert_parses("func(1, 2, *args, key=value, **kwargs)");
            
            // Multiple double starred expressions
            assert_parses("func(**kw1, **kw2)");
            
            // Mixed starred and double starred
            assert_parses("func(*args1, *args2, **kw1, **kw2)");
        }

        #[test]
        fn test_chained_comparisons() {
            // Simple chained comparison
            assert_parses("a < b < c");
            
            // Chained comparison with different operators
            assert_parses("a < b <= c == d != e >= f > g");
            
            // Chained comparison with is and in
            assert_parses("a is b is not c != d in e not in f");
            
            // Parenthesized parts in chained comparison
            assert_parses("(a + b) < c < (d + e)");
        }

        #[test]
        fn test_conditional_expressions() {
            // Simple ternary
            assert_parses("x if condition else y");
            
            // Nested ternary
            assert_parses("x if a else y if b else z");
            
            // Ternary with complex expressions
            assert_parses("(a + b) if (c < d) else (e * f)");
            
            // Ternary in a larger expression
            assert_parses("result = (a if condition else b) + c");
        }

        #[test]
        fn test_walrus_operator() {
            // Simple walrus
            assert_parses("if (n := len(items)) > 0:\n    print(n)");
            
            // Walrus in a larger expression
            assert_parses("if (a := get_a()) and (b := get_b()):\n    process(a, b)");
            
            // Walrus in comprehension
            assert_parses("[y for x in data if (y := f(x)) > 0]");
            
            // Nested walrus expressions
            assert_parses("if (parent := get_parent()) and (child := parent.get_child()):\n    process(parent, child)");
        }

        #[test]
        fn test_f_strings() {
            // Simple f-string
            assert_parses("f'Hello, {name}!'");
            
            // F-string with expressions
            assert_parses("f'The answer is {2 + 2 * 10}'");
            
            // F-string with format specifiers
            assert_parses("f'Pi is approximately {pi:.2f}'");
            
            // Nested f-strings
            assert_parses("f'Hello, {f\"Mr. {name}\"}!'");
            
            // F-string with conversion flags
            assert_parses("f'Debug: {value!r}, String: {value!s}, ASCII: {value!a}'");
        }

        #[test]
        fn test_complex_string_literals() {
            // Raw string
            assert_parses("r'Raw\\nString'");
            
            // Byte string
            assert_parses("b'Byte String'");
            
            // Raw byte string
            assert_parses("br'Raw\\nByte String'");
            
            // Triple-quoted strings
            assert_parses("'''Triple quoted\nstring'''");
            
            // F-string with triple quotes
            assert_parses("f'''Multi-line\nf-string with {value}'''");
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_structures() {
            // Empty function
            assert_parses("def empty(): pass");
            
            // Empty class
            assert_parses("class Empty: pass");
            
            // Empty conditional blocks
            assert_parses("if condition: pass\nelse: pass");
            
            // Empty loop
            assert_parses("while True: pass");
            
            // Empty try blocks
            assert_parses("try: pass\nexcept: pass\nelse: pass\nfinally: pass");
        }

        #[test]
        fn test_one_element_structures() {
            // One element tuple
            assert_parses("(1,)");
            
            // One element list
            assert_parses("[1]");
            
            // One element set
            assert_parses("{1}");
            
            // One element dict
            assert_parses("{1: 'one'}");
            
            // One-line function
            assert_parses("def one_liner(): return 42");
        }

        #[test]
        fn test_keyword_collisions() {
            // Variable names that look like keywords
            assert_parses("async_result = get_async()");
            assert_parses("await_time = calculate_time()");
            assert_parses("class_name = 'MyClass'");
            assert_parses("if_condition = True");
            
            // Function parameters that look like keywords
            assert_parses("def process(from_date, to_date, in_place=False): pass");
            
            // Keywords in attribute access
            assert_parses("obj.if_attr = True");
            assert_parses("value = obj.class");
        }

        #[test]
        fn test_comment_handling() {
            // Simple comment
            assert_parses("x = 1  # This is a comment");
            
            // Comment only line
            assert_parses("# This is a comment\nx = 1");
            
            // Comment after control flow
            assert_parses("if condition:  # This is a comment\n    pass");
            
            // Comment with unusual characters
            assert_parses("# Comment with special chars: !@#$%^&*()_+");
            
            // Multiple comments
            assert_parses("# First comment\n# Second comment\nx = 1");
        }

        #[test]
        fn test_extreme_whitespace() {
            // No whitespace
            assert_parses("def f(x):return x*x");
            
            // Excessive whitespace
            assert_parses("def    f   (   x   )   :   \n    return    x   *   x");
            
            // Tabs and spaces
            assert_parses("if condition:\n    pass");
            
            // Blank lines
            assert_parses("\n\n\ndef f():\n\n\n    return 42\n\n\n");
            
            // Empty file
            assert_parses("");
        }

        #[test]
        fn test_boundary_numbers() {
            // Zero
            assert_parses("0");
            
            // Maximum safe integer
            assert_parses("9223372036854775807");  // i64::MAX
            
            // Minimum safe integer
            assert_parses("-9223372036854775808");  // i64::MIN
            
            // Floating point precision
            assert_parses("0.1 + 0.2");
            
            // Scientific notation
            assert_parses("1.23e45");
            assert_parses("1.23e-45");
            
            // Different number bases
            assert_parses("0x123ABC");  // Hex
            assert_parses("0o123");     // Octal
            assert_parses("0b101010");  // Binary
        }
    }

    mod specialized_syntax {
        use super::*;

        #[test]
        fn test_match_case_statements() {
            // Simple match-case
            assert_parses(
                "match value:
                    case 1:
                        print('one')
                    case 2:
                        print('two')
                    case _:
                        print('other')"
            );
            
            // Match-case with guards
            assert_parses(
                "match point:
                    case (x, y) if x == y:
                        print('On diagonal')
                    case (0, y):
                        print(f'Y-axis at {y}')
                    case (x, 0):
                        print(f'X-axis at {x}')
                    case _:
                        print('Elsewhere')"
            );
            
            // Match-case with class patterns
            assert_parses(
                "match shape:
                    case Circle(radius=r):
                        print(f'Circle with radius {r}')
                    case Rectangle(width=w, height=h):
                        print(f'Rectangle {w}x{h}')
                    case _:
                        print('Unknown shape')"
            );
            
            // Match-case with alternatives
            assert_parses(
                "match command:
                    case 'quit' | 'exit':
                        quit_app()
                    case 'help' | '?':
                        show_help()
                    case _:
                        print('Unknown command')"
            );
            
            // Complex nested patterns
            assert_parses(
                "match data:
                    case {'name': str(name), 'age': int(age), 'skills': [*skills]}:
                        process_person(name, age, skills)
                    case {'error': str(msg)}:
                        handle_error(msg)
                    case _:
                        handle_unknown_data()"
            );
        }

        #[test]
        fn test_type_annotations() {
            // Simple variable annotations
            assert_parses("x: int = 5");
            assert_parses("y: str");
            
            // Function annotations
            assert_parses("def greet(name: str) -> str:\n    return f'Hello, {name}'");
            
            // Class field annotations
            assert_parses("class Point:\n    x: float\n    y: float\n    \n    def __init__(self, x: float, y: float) -> None:\n        self.x = x\n        self.y = y");
            
            // Complex type annotations
            assert_parses("def process(items: List[Dict[str, Any]]) -> Tuple[int, str]:\n    return (len(items), 'processed')");
            
            // Generic type annotations
            assert_parses("def identity(x: T) -> T:\n    return x");
            
            // Callable annotations
            assert_parses("callback: Callable[[int, str], bool]");
        }

        #[test]
        fn test_async_await() {
            // Async function definition
            assert_parses("async def fetch(url: str) -> str:\n    return await get_data(url)");
            
            // Await expression
            assert_parses("async def process():\n    result = await async_function()");
            
            // Async for loop
            assert_parses("async def process_items():\n    async for item in async_generator():\n        await process_item(item)");
            
            // Async with statement
            assert_parses("async def safe_operation():\n    async with lock:\n        await risky_operation()");
            
            // Async comprehensions
            assert_parses("async def get_results():\n    return [await f(x) async for x in async_gen()]");
        }

        #[test]
        fn test_decorators() {
            // Simple decorator
            assert_parses("@decorator\ndef func(): pass");
            
            // Multiple decorators
            assert_parses("@decorator1\n@decorator2\n@decorator3\ndef func(): pass");
            
            // Decorator with arguments
            assert_parses("@decorator(arg1, arg2, keyword=value)\ndef func(): pass");
            
            // Class decorators
            assert_parses("@singleton\nclass Singleton: pass");
            
            // Method decorators
            assert_parses("class MyClass:\n    @classmethod\n    def class_method(cls): pass\n    \n    @staticmethod\n    def static_method(): pass\n    \n    @property\n    def prop(self): pass");
            
            // Complex decorator expressions
            assert_parses("@decorator1.method().other(param=value)\ndef func(): pass");
        }

        #[test]
        fn test_with_as_statement() {
            // Simple with statement
            assert_parses("with open('file.txt') as f:\n    data = f.read()");
            
            // Multiple context managers
            assert_parses("with open('input.txt') as inf, open('output.txt', 'w') as outf:\n    outf.write(inf.read())");
            
            // Nested with statements
            assert_parses("with context1:\n    with context2:\n        operation()");
            
            // With statement without as clause
            assert_parses("with lock:\n    critical_section()");
            
            // Async with statement
            assert_parses("async def func():\n    async with async_context() as ctx:\n        await operation(ctx)");
        }
    }

    mod regression_tests {
        use super::*;

        #[test]
        fn test_indentation_edge_cases() {
            // Blank lines shouldn't affect indentation
            assert_parses("if condition:\n    x = 1\n\n    y = 2");
            
            // Comments shouldn't affect indentation
            assert_parses("if condition:\n    # Comment\n    x = 1");
            
            // Empty blocks with correct indentation
            assert_parses("if condition:\n    # Just a comment\n    pass");
            
            // Multiple indentation levels
            assert_parses("if a:\n    if b:\n        if c:\n            x = 1\n        y = 2\n    z = 3");
            
            // Dedent multiple levels at once
            assert_parses("if a:\n    if b:\n        if c:\n            x = 1\nz = 3");
        }

        #[test]
        fn test_function_call_edge_cases() {
            // Empty function call
            assert_parses("func()");
            
            // Function call with a single argument
            assert_parses("func(42)");
            
            // Function call with a trailing comma
            assert_parses("func(arg,)");
            
            // Nested function calls
            assert_parses("func1(func2(func3()))");
            
            // Function calls with keyword arguments
            assert_parses("func(1, 2, key1=val1, key2=val2)");
            
            // Function calls with unpacking
            assert_parses("func(1, 2, *args, key=val, **kwargs)");
            
            // Method calls
            assert_parses("obj.method().other_method()");
        }

        #[test]
        fn test_expression_edge_cases() {
            // Empty tuple
            assert_parses("()");
            
            // Tuple with a single element
            assert_parses("(1,)");
            
            // Tuple without parentheses
            assert_parses("a, b = 1, 2");
            
            // Multi-line expressions
            assert_parses("(\n    1,\n    2,\n    3\n)");
            
            // Parenthesized expressions
            assert_parses("(a + b) * (c + d)");
            
            // Nested data structures
            assert_parses("{'key': [1, 2, (3, 4), {5, 6}]}");
        }

        #[test]
        fn test_statement_edge_cases() {
            // Multiple statements on a line
            assert_parses("x = 1; y = 2; z = 3");
            
            // Empty statement (just a semicolon)
            assert_parses(";");
            
            // Statement with trailing semicolon
            assert_parses("x = 1;");
            
            // Import statements with trailing comma
            assert_parses("from module import item1, item2,");
            
            // Multiple assignments with different unpackings
            assert_parses("a, b = c = d, e = 1, 2");
        }

        #[test]
        fn test_syntax_interaction_cases() {
            // Decorators with complex control flow
            assert_parses("@decorator\ndef func():\n    if condition:\n        for item in items:\n            yield item");
            
            // Comprehensions with complex expressions
            assert_parses("results = [(x, y, x+y) for x in range(10) for y in range(10) if x < y]");
            
            // Async functions with context managers and exception handling
            assert_parses("async def process():\n    try:\n        async with lock:\n            await operation()\n    except Exception as e:\n        await handle_error(e)");
            
            // Classes with type annotations, decorators, and complex methods
            assert_parses("@dataclass\nclass Point:\n    x: float\n    y: float\n    \n    @property\n    def distance(self) -> float:\n        return (self.x ** 2 + self.y ** 2) ** 0.5");
        }
    }

    mod potential_cheetah_extensions {

        // These tests are placeholders for potential Cheetah-specific features
        // Uncomment and adapt these tests based on your specific language features

        /*
        #[test]
        fn test_cheetah_function_extensions() {
            // Pure function annotation
            assert_parses("@pure\ndef add(a, b):\n    return a + b");
            
            // Function contracts
            assert_parses("def divide(a, b):\n    require b != 0\n    result = a / b\n    ensure result * b == a\n    return result");
            
            // Default error handling
            assert_parses("def risky() may raise ValueError:\n    return process()");
        }

        #[test]
        fn test_cheetah_type_system() {
            // Union types with |
            assert_parses("def process(value: int | str) -> int | None:\n    pass");
            
            // Type aliases
            assert_parses("type UserId = int\ndef get_user(id: UserId) -> User:\n    pass");
            
            // Refinement types
            assert_parses("def get_positive(x: int where x > 0) -> int where _ > 0:\n    return x");
        }

        #[test]
        fn test_cheetah_pattern_matching() {
            // Enhanced pattern matching
            assert_parses(
                "match value:
                    case int() && < 0:
                        print('Negative integer')
                    case str() && ~r'\\d+':
                        print('String of digits')
                    case _:
                        print('Other')"
            );
        }

        #[test]
        fn test_cheetah_metaprogramming() {
            // Code generation macros
            assert_parses("macro! generate_accessors(Point, x, y, z)");
            
            // Compile-time computation
            assert_parses("#for i in range(10):\n    def func_{i}(): return {i}\n#end");
        }

        #[test]
        fn test_cheetah_concurrency() {
            // Parallel for loop
            assert_parses("parallel for i in range(100):\n    process(i)");
            
            // Actor definitions
            assert_parses("actor Counter:\n    var count = 0\n    message increment():\n        count += 1\n    message get() -> int:\n        return count");
        }
        */
    }
}