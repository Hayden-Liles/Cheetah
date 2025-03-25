#[allow(dead_code)]
#[cfg(test)]
mod parser_specialized_tests {
    use cheetah::ast::{Expr, Module, Stmt};
    use cheetah::formatter::CodeFormatter;
    use cheetah::lexer::Lexer;
    use cheetah::parser::{ParseError, Parser};
    use std::fmt;
    use cheetah::visitor::Visitor;
    use cheetah::symtable::SymbolTableBuilder;

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

    // Helper function to parse and print the AST or detailed error
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
                    println!("- {}", ErrorFormatter(error));
                    
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

    // Print a simplified version of the AST structure
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

    fn parse_and_format(source: &str, indent_size: usize) -> Result<String, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        println!("Tokens: {:?}", tokens);
        if !lexer.get_errors().is_empty() {
            return Err(format!("Lexer errors: {:?}", lexer.get_errors()));
        }
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(module) => {
                println!("AST: {:?}", module.body);
                let mut formatter = CodeFormatter::new(indent_size);
                formatter.visit_module(&module);
                Ok(formatter.get_output().to_string())
            },
            Err(errors) => {
                println!("Parse errors: {:?}", errors);
                Err(format!("Parser errors: {:?}", errors))
            },
        }
    }

    // Helper function to parse and build symbol table
    fn parse_and_analyze(source: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        if !lexer.get_errors().is_empty() {
            return Err(format!("Lexer errors: {:?}", lexer.get_errors()));
        }
        
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(module) => {
                let mut symbol_table = SymbolTableBuilder::new();
                symbol_table.visit_module(&module);
                
                Ok(())
            },
            Err(errors) => Err(format!("Parser errors: {:?}", errors)),
        }
    }

    mod tests {
        use super::*;

        #[test]
        fn test_comprehension_conditions() {
            // Test simple condition first
            println!("\n===== Testing simple condition =====");
            assert_parses_and_prints("[x for x in range(100) if x % 2 == 0]");
            
            // Test multiple conditions
            println!("\n===== Testing multiple conditions =====");
            assert_parses_and_prints("[x for x in range(100) if x % 2 == 0 if x % 3 == 0]");
            
            // Test nested function calls (without comprehension in function)
            println!("\n===== Testing nested function call =====");
            assert_parses_and_prints("[x for x in range(100) if int(x ** 0.5) > 5]");
            
            // Test comprehension inside function call (the problematic case)
            println!("\n===== Testing comprehension in function argument =====");
            assert_parses_and_prints("[x for x in range(100) if all(x % i != 0 for i in range(2, int(x ** 0.5) + 1))]");
        }
    }
}