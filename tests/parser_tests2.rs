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
    mod tests {
        use super::*;
        
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
    }
}