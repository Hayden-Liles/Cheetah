#[allow(dead_code)]
#[cfg(test)]
mod parser_specialized_tests {
    use cheetah::ast::Module;
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
        
    }
}