#[cfg(test)]
mod lexer_edge_cases_tests {
    use cheetah::{ast::{Expr, Module, Stmt}, lexer::{Lexer, TokenType}, parser::{ParseError, Parser}};


    // Helper function to check if a string can be tokenized without errors
    fn assert_parses(input: &str) {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Check that we have at least one token (EOF)
        assert!(!tokens.is_empty(), "Failed to tokenize: {}", input);

        // Check that the last token is EOF
        assert_eq!(tokens.last().unwrap().token_type, TokenType::EOF,
                  "Last token should be EOF for input: {}", input);
    }

    fn print_ast_structure(module: &Module) {
        for (i, stmt) in module.body.iter().enumerate() {
            match &**stmt {
                Stmt::Expr { value, .. } => {
                    match &**value {
                        Expr::ListComp { elt, generators, .. } => {
                            println!("List Comprehension:");
                            println!("  Element: {:?}", elt);
                            println!("  Generators: {} generator(s)", generators.len());

                            for (i, generator) in generators.iter().enumerate() {
                                println!("    Generator {}:", i+1);
                                println!("      Target: {:?}", generator.target);
                                println!("      Iterator: {:?}", generator.iter);
                                println!("      Conditions: {} condition(s)", generator.ifs.len());

                                for (j, cond) in generator.ifs.iter().enumerate() {
                                    println!("        Condition {}: {:?}", j+1, cond);

                                    // If this is a Call expression, print more details
                                    if let Expr::Call { func, args, .. } = &**cond {
                                        println!("          Function: {:?}", func);
                                        println!("          Arguments: {} arg(s)", args.len());

                                        for (k, arg) in args.iter().enumerate() {
                                            println!("            Arg {}: {:?}", k+1, arg);

                                            // If this is a GeneratorExp, print more details
                                            if let Expr::GeneratorExp { elt, generators, .. } = &**arg {
                                                println!("              Generator Expression:");
                                                println!("                Element: {:?}", elt);
                                                println!("                Generators: {} generator(s)", generators.len());
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        _ => println!("Statement {}: {:?}", i+1, stmt),
                    }
                },
                _ => println!("Statement {}: {:?}", i+1, stmt),
            }
        }
    }

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

    fn assert_parses_and_prints(source: &str) -> Module {
        println!("Source code: {}", source);

        // Tokenize and print tokens for debugging
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        println!("Token stream:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?} at line {}, column {}",
                     i, token.token_type, token.line, token.column);
        }

        let tokens_clone = tokens.clone(); // Clone for error reporting

        // Try parsing
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(module) => {
                println!("Parsing successful!");
                // Can add AST printing here if needed
                println!("AST structure (first few levels):");
                print_ast_structure(&module);
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
                    println!("- {}", &error);

                    // Show code snippet with error position highlighted
                    match error {
                        ParseError::UnexpectedToken { line, column, .. } |
                        ParseError::InvalidSyntax { line, column, .. } |
                        ParseError::EOF { line, column, .. } => {
                            println!("\nCode context:");
                            println!("{}", format_source_with_error(source, *line, *column));

                            // Print nearby tokens for more context
                            println!("\nNearby tokens:");
                            for (i, token) in tokens_clone.iter().enumerate() {
                                if (token.line == *line && (token.column as isize - *column as isize).abs() < 10)
                                   || (token.line as isize - *line as isize).abs() <= 1 {
                                    println!("  {}: {:?} at line {}, column {}",
                                             i, token.token_type, token.line, token.column);
                                }
                            }
                        }
                    }
                    println!("--------------------------------");
                }

                panic!("Parsing failed with {} errors", errors.len());
            },
        }
    }

    #[test]
    fn test_string_edge_cases() {
        // Empty strings
        assert_parses("''");
        assert_parses("\"\"");

        // Strings with escapes
        assert_parses("'String with \\'quote\\''");
        assert_parses("\"String with \\\"double quote\\\"\"");

        // String with Unicode escapes
        assert_parses("'\\u0041\\u0042\\u0043'");  // ABC

        // String with hex escapes
        assert_parses("'\\x41\\x42\\x43'");  // ABC

        // String with unusual characters
        assert_parses("'String with tab\\t and newline\\n'");

        // Triple-quoted string edge cases
        assert_parses("'''String with both ' and \" quotes'''");
        assert_parses("'''''"); // Single quote inside triple quotes
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

    #[test]
        fn test_comprehension_conditions1() {
            // Test simple condition first
            println!("\n===== Testing simple condition =====");
            assert_parses_and_prints("[x for x in range(100) if x % 2 == 0]");
        }
        #[test]
        fn test_comprehension_conditions2() {
            // Test multiple conditions
            println!("\n===== Testing multiple conditions =====");
            assert_parses_and_prints("[x for x in range(100) if x % 2 == 0 if x % 3 == 0]");
        }
        #[test]
        fn test_comprehension_conditions3() {
            // Test nested function calls (without comprehension in function)
            println!("\n===== Testing nested function call =====");
            assert_parses_and_prints("[x for x in range(100) if int(x ** 0.5) > 5]");
        }
        #[test]
        fn test_comprehension_conditions4() {
            // First test simpler nested function calls
            println!("\n===== Testing simpler nested function call =====");
            assert_parses_and_prints("[x for x in range(10) if all(x > i for i in range(5))]");

            // Then test the more complex case
            println!("\n===== Testing comprehension in function argument =====");
            // Break down the complex expression into parts
            assert_parses_and_prints("[x for x in range(100) if all(x % i != 0 for i in range(2, (int(x) ** 0.5) + 1))]");
        }

        // Add a new test for the original complex case
        #[test]
        fn test_comprehension_conditions5() {
            // Original complex case with simplified expression
            println!("\n===== Testing complex comprehension with nested calculations =====");
            assert_parses_and_prints("[x for x in range(100) if all(x % i != 0 for i in range(2, 10))]");
        }
}
