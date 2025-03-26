// Make all modules public so they can be imported in tests
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod symtable;
pub mod visitor;
pub mod formatter;

// Import the Visitor trait so it's in scope
use crate::visitor::Visitor;

/// Parse the given Python-like source code into an AST
pub fn parse(source: &str) -> Result<ast::Module, Vec<parser::ParseError>> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    
    // Check for lexer errors
    if !lexer.get_errors().is_empty() {
        // Convert lexer errors to parser errors
        let errors = lexer.get_errors().iter().map(|e| {
            parser::ParseError::invalid_syntax(&e.message, e.line, e.column)
        }).collect();
        
        return Err(errors);
    }
    
    // Parse tokens into AST using the new parser interface
    parser::parse(tokens)
}

/// Format the given AST back to Python-like source code
pub fn format_ast(module: &ast::Module, indent_size: usize) -> String {
    let mut formatter = formatter::CodeFormatter::new(indent_size);
    formatter.visit_module(module);
    formatter.get_output().to_string()
}

/// Build a symbol table from the given AST
pub fn build_symbol_table(module: &ast::Module) -> symtable::SymbolTableBuilder {
    let mut builder = symtable::SymbolTableBuilder::new();
    builder.visit_module(module);
    builder
}

/// Parse Python-like source code and print the AST structure
pub fn print_ast(source: &str) -> Result<(), String> {
    match parse(source) {
        Ok(module) => {
            let mut printer = visitor::AstPrinter::new();
            let output = printer.visit_module(&module);
            println!("{}", output);
            Ok(())
        },
        Err(errors) => {
            for error in errors {
                println!("Error: {}", error.get_message());
            }
            Err("Parse errors occurred".to_string())
        },
    }
}

/// Parse Python-like source code, format it, and return the formatted code
pub fn format_code(source: &str, indent_size: usize) -> Result<String, String> {
    match parse(source) {
        Ok(module) => Ok(format_ast(&module, indent_size)),
        Err(errors) => {
            let error_messages = errors.iter()
                .map(|e| e.get_message())
                .collect::<Vec<String>>()
                .join("\n");
            Err(error_messages)
        },
    }
}

/// Parse Python-like source code and analyze it with the symbol table
pub fn analyze_code(source: &str) -> Result<(), String> {
    match parse(source) {
        Ok(module) => {
            let symbol_table = build_symbol_table(&module);
            symbol_table.print_symbol_table();
            
            let undefined = symbol_table.get_undefined_names();
            if !undefined.is_empty() {
                println!("\nUndefined names:");
                for name in undefined {
                    println!("  {}", name);
                }
            }
            
            Ok(())
        },
        Err(errors) => {
            for error in errors {
                println!("Error: {}", error.get_message());
            }
            Err("Parse errors occurred".to_string())
        },
    }
}