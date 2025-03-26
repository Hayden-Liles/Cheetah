use std::fs;
use std::io::{self, Write};
use clap::{Parser as ClapParser, Subcommand};
use anyhow::{Result, Context};
use colored::Colorize;

// Import modules from lib.rs
use cheetah::lexer::{Lexer, Token, TokenType, LexerConfig};
use cheetah::parser::{self, ParseError};
use cheetah::formatter::CodeFormatter;
use cheetah::visitor::Visitor;


#[derive(ClapParser)]
#[command(name = "cheetah")]
#[command(version = "0.1.0")]
#[command(about = "Cheetah programming language interpreter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Cheetah source file
    Run {
        /// The source file to run
        file: String,
    },
    /// Start a REPL session
    Repl,
    /// Lex a file and print the tokens (for debugging)
    Lex {
        /// The source file to lex
        file: String,
        
        /// Show detailed token information
        #[arg(short, long)]
        verbose: bool,
        
        /// Highlight token types with colors
        #[arg(short, long)]
        color: bool,
        
        /// Show line numbers in output
        #[arg(short = 'n', long)]
        line_numbers: bool,
    },
    /// Parse a file and print the AST (for debugging)
    Parse {
        /// The source file to parse
        file: String,
        
        /// Show detailed AST information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Check a file for syntax errors
    Check {
        /// The source file to check
        file: String,
        
        /// Show detailed information about errors
        #[arg(short, long)]
        verbose: bool,
    },
    /// Format a Cheetah source file
    Format {
        /// The source file to format
        file: String,
        
        /// Write changes to file instead of stdout
        #[arg(short, long)]
        write: bool,
        
        /// Indentation size (number of spaces)
        #[arg(short, long, default_value = "4")]
        indent: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            run_file(&file)?;
        }
        Commands::Repl => {
            run_repl()?;
        }
        Commands::Lex { file, verbose, color, line_numbers } => {
            lex_file(&file, verbose, color, line_numbers)?;
        }
        Commands::Parse { file, verbose } => {
            parse_file(&file, verbose)?;
        }
        Commands::Check { file, verbose } => {
            check_file(&file, verbose)?;
        }
        Commands::Format { file, write, indent } => {
            format_file(&file, write, indent)?;
        }
    }

    Ok(())
}

fn run_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // First, lex the file
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let lexer_errors = lexer.get_errors();
    if !lexer_errors.is_empty() {
        eprintln!("Lexical errors found in '{}':", filename);
        for error in lexer_errors {
            eprintln!("{}", error);
        }
        return Ok(());
    }
    
    // Then, parse the tokens with the new parser interface
    match parser::parse(tokens) {
        Ok(module) => {
            println!("Successfully parsed file: {}", filename);
            println!("AST contains {} top-level statements", module.body.len());
            // Here you would execute the parsed code in a future interpreter
        },
        Err(errors) => {
            eprintln!("Syntax errors found in '{}':", filename);
            for error in errors {
                eprintln!("  {}", error.get_message());
            }
        }
    }
    
    Ok(())
}

fn run_repl() -> Result<()> {
    println!("{}", "Cheetah Programming Language REPL".bright_green());
    println!("Type 'exit' or press Ctrl+D to exit");
    
    let mut input_buffer = String::new();
    let mut paren_level = 0;
    let mut bracket_level = 0;
    let mut brace_level = 0;
    let mut in_multiline_block = false;
    
    loop {
        let prompt = if !input_buffer.is_empty() {
            "... ".bright_yellow().to_string()
        } else {
            ">>> ".bright_green().to_string()
        };
        
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }
        
        let input = input.trim_end();
        
        if input_buffer.is_empty() && input == "exit" {
            break;
        }
        
        input_buffer.push_str(input);
        input_buffer.push('\n');
        
        update_repl_state(&input, &mut paren_level, &mut bracket_level, &mut brace_level, &mut in_multiline_block);
        
        let should_execute = !in_multiline_block && paren_level == 0 && bracket_level == 0 && brace_level == 0 && 
                                    (input.trim().is_empty() || !input.trim().ends_with(':'));
        
        if should_execute {
            let complete_input = input_buffer.trim();
            
            if !complete_input.is_empty() {
                // First lexical analysis
                let mut lexer = Lexer::new(complete_input);
                let tokens = lexer.tokenize();
                
                let lexer_errors = lexer.get_errors();
                if !lexer_errors.is_empty() {
                    for error in lexer_errors {
                        eprintln!("{}", error.to_string().bright_red());
                    }
                } else {
                    // Then try parsing with the new parser interface
                    match parser::parse(tokens.clone()) {
                        Ok(_module) => {
                            println!("{}", "✓ Parsed successfully".bright_green());
                            // Here you would execute the parsed code in a future interpreter
                            
                            // For now, just print the tokens
                            if input.starts_with("tokens") || input.starts_with("lexer") {
                                for token in &tokens {
                                    match &token.token_type {
                                        TokenType::Invalid(_) => println!("{}", format!("{}", token).bright_red()),
                                        _ => println!("{}", format_token_for_repl(token, true)),
                                    }
                                }
                            }
                        },
                        Err(errors) => {
                            for error in errors {
                                eprintln!("{}", error.get_message().bright_red());
                            }
                        }
                    }
                }
            }
            
            input_buffer.clear();
            paren_level = 0;
            bracket_level = 0;
            brace_level = 0;
            in_multiline_block = false;
        }
    }
    
    println!("Goodbye!");
    Ok(())
}

/// Updates the REPL state based on the current line of input
fn update_repl_state(input: &str, paren_level: &mut usize, bracket_level: &mut usize, brace_level: &mut usize, in_multiline_block: &mut bool) {
    for c in input.chars() {
        match c {
            '(' => *paren_level += 1,
            ')' => if *paren_level > 0 { *paren_level -= 1 },
            '[' => *bracket_level += 1,
            ']' => if *bracket_level > 0 { *bracket_level -= 1 },
            '{' => *brace_level += 1,
            '}' => if *brace_level > 0 { *brace_level -= 1 },
            _ => {}
        }
    }
    
    if input.trim().ends_with(':') {
        *in_multiline_block = true;
    } else if input.trim().is_empty() && *in_multiline_block {
        *in_multiline_block = false;
    }
}

fn lex_file(filename: &str, verbose: bool, use_color: bool, line_numbers: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let errors = lexer.get_errors();
    if !errors.is_empty() {
        eprintln!("Lexical errors found in '{}':", filename);
        for error in errors {
            if use_color {
                eprintln!("{}", error.to_string().bright_red());
            } else {
                eprintln!("{}", error);
            }
        }
    }
    
    println!("Tokens from file '{}':", filename);
    
    if verbose {
        for (i, token) in tokens.iter().enumerate() {
            let mut token_str = String::new();
            
            if line_numbers {
                token_str = format!("{:4}: ", i);
            }
            
            token_str.push_str(&format!("{}", token));
            
            if use_color {
                match &token.token_type {
                    TokenType::Def | TokenType::If | TokenType::Else | TokenType::For | 
                    TokenType::While | TokenType::Return => println!("{}", token_str.bright_blue()),
                    TokenType::Identifier(_) => println!("{}", token_str.bright_yellow()),
                    TokenType::StringLiteral(_) | TokenType::RawString(_) | 
                    TokenType::FString(_) | TokenType::BytesLiteral(_) => {
                        println!("{}", token_str.bright_green())
                    },
                    TokenType::IntLiteral(_) | TokenType::FloatLiteral(_) |
                    TokenType::BinaryLiteral(_) | TokenType::OctalLiteral(_) |
                    TokenType::HexLiteral(_) => println!("{}", token_str.bright_cyan()),
                    TokenType::Invalid(_) => println!("{}", token_str.bright_red()),
                    TokenType::Indent | TokenType::Dedent => println!("{}", token_str.bright_magenta()),
                    _ => println!("{}", token_str),
                }
            } else {
                println!("{}", token_str);
            }
        }
    } else {
        for token in &tokens {
            if use_color {
                match &token.token_type {
                    TokenType::Invalid(_) => println!("{}", format!("{}", token).bright_red()),
                    _ => println!("{}", format_token(token, use_color)),
                }
            } else {
                println!("{}", token);
            }
        }
    }
    
    Ok(())
}

/// New function to parse a file and print the AST
fn parse_file(filename: &str, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // First, lex the file
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let lexer_errors = lexer.get_errors();
    if !lexer_errors.is_empty() {
        eprintln!("Lexical errors found in '{}':", filename);
        for error in lexer_errors {
            eprintln!("{}", error);
        }
        return Ok(());
    }
    
    // Then, parse the tokens with the new parser interface
    match parser::parse(tokens) {
        Ok(module) => {
            println!("Successfully parsed file: {}", filename);
            
            if verbose {
                // Use the AstPrinter to display the AST structure
                use cheetah::visitor::AstPrinter;
                let mut printer = AstPrinter::new();
                let output = printer.visit_module(&module);
                println!("AST Structure:");
                println!("{}", output);
            } else {
                // Just print summary info
                println!("AST contains {} top-level statements", module.body.len());
                
                // Print the first few statements as a preview
                let max_preview = 5;
                let preview_count = std::cmp::min(max_preview, module.body.len());
                
                if preview_count > 0 {
                    println!("Top-level statements:");
                    for (i, stmt) in module.body.iter().take(preview_count).enumerate() {
                        println!("  {}: {}", i + 1, stmt);
                    }
                    
                    if module.body.len() > max_preview {
                        println!("  ... and {} more", module.body.len() - max_preview);
                    }
                }
            }
        },
        Err(errors) => {
            eprintln!("Syntax errors found in '{}':", filename);
            for error in errors {
                eprintln!("  {}", error.get_message());
            }
        }
    }
    
    Ok(())
}

fn check_file(filename: &str, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Check for lexical errors first
    let config = LexerConfig {
        enforce_indent_consistency: true,
        standard_indent_size: 4,
        tab_width: 4,
        allow_tabs_in_indentation: false,
        allow_trailing_semicolon: false,
    };
    
    let mut lexer = Lexer::with_config(&source, config);
    let tokens = lexer.tokenize();
    
    let lexer_errors = lexer.get_errors();
    if !lexer_errors.is_empty() {
        eprintln!("✗ Lexical errors found in '{}':", filename);
        for error in lexer_errors {
            if verbose {
                eprintln!("  Line {}, Col {}: {}", error.line, error.column, error.message);
                eprintln!("  {}", error.snippet);
                eprintln!("  {}^", " ".repeat(error.column + 1));
                if let Some(suggestion) = &error.suggestion {
                    eprintln!("  Suggestion: {}", suggestion);
                }
                eprintln!();
            } else {
                eprintln!("  {}", error);
            }
        }
        return Ok(());
    }
    
    // Then check for syntax errors using the new parser interface
    match parser::parse(tokens) {
        Ok(_) => {
            println!("✓ No syntax errors found in '{}'", filename);
        },
        Err(errors) => {
            eprintln!("✗ Syntax errors found in '{}':", filename);
            for error in errors {
                if verbose {
                    // Get error details and display with context
                    let (line, column, message) = match &error {
                        ParseError::UnexpectedToken { expected, found, line, column } => 
                            (*line, *column, format!("Expected {}, found {:?}", expected, found)),
                        ParseError::InvalidSyntax { message, line, column } => 
                            (*line, *column, message.clone()),
                        ParseError::EOF { expected, line, column } => 
                            (*line, *column, format!("Unexpected end of file, expected {}", expected)),
                    };
                    
                    eprintln!("  Line {}, Col {}: {}", line, column, message);
                    
                    // Trying to extract line from source for context
                    if let Some(context) = get_line_context(&source, line) {
                        eprintln!("  {}", context);
                        eprintln!("  {}^", " ".repeat(column + 1));
                    }
                    eprintln!();
                } else {
                    eprintln!("  {}", error.get_message());
                }
            }
        }
    }
    
    Ok(())
}

// Helper function to get a line of context from source code
fn get_line_context(source: &str, line_num: usize) -> Option<String> {
    if line_num == 0 {
        return None;
    }
    
    let lines: Vec<&str> = source.lines().collect();
    if line_num <= lines.len() {
        Some(lines[line_num - 1].to_string())
    } else {
        None
    }
}

fn format_file(filename: &str, write: bool, indent_size: usize) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // First, check for lexical errors
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    let lexer_errors = lexer.get_errors();
    if !lexer_errors.is_empty() {
        eprintln!("Cannot format file with lexical errors:");
        for error in lexer_errors {
            eprintln!("  {}", error);
        }
        return Ok(());
    }
    
    // Then parse into AST using the new parser interface
    match parser::parse(tokens) {
        Ok(module) => {
            // Format the AST
            let mut formatter = CodeFormatter::new(indent_size);
            formatter.visit_module(&module);
            let formatted_source = formatter.get_output().to_string();
            
            if write {
                fs::write(filename, &formatted_source)
                    .with_context(|| format!("Failed to write to file: {}", filename))?;
                println!("Formatted and wrote changes to '{}'", filename);
            } else {
                print!("{}", formatted_source);
            }
        },
        Err(errors) => {
            eprintln!("Cannot format file with syntax errors:");
            for error in errors {
                eprintln!("  {}", error.get_message());
            }
        }
    }
    
    Ok(())
}

/// Format the token output based on token type
fn format_token(token: &Token, use_color: bool) -> String {
    if !use_color {
        return format!("{}", token);
    }
    
    match &token.token_type {
        TokenType::Invalid(_) => format!("{}", token).bright_red().to_string(),
        TokenType::Indent | TokenType::Dedent | TokenType::Newline => {
            format!("{}", token).bright_magenta().to_string()
        },
        TokenType::Identifier(_) => format!("{}", token).bright_yellow().to_string(),
        TokenType::Def | TokenType::If | TokenType::Else | TokenType::For | 
        TokenType::While | TokenType::Return => format!("{}", token).bright_blue().to_string(),
        TokenType::StringLiteral(_) | TokenType::RawString(_) | 
        TokenType::FString(_) | TokenType::BytesLiteral(_) => format!("{}", token).bright_green().to_string(),
        TokenType::IntLiteral(_) | TokenType::FloatLiteral(_) |
        TokenType::BinaryLiteral(_) | TokenType::OctalLiteral(_) |
        TokenType::HexLiteral(_) => format!("{}", token).bright_cyan().to_string(),
        _ => format!("{}", token).to_string(),
    }
}

/// Format tokens for REPL output with syntax highlighting
fn format_token_for_repl(token: &Token, use_color: bool) -> String {
    if !use_color {
        return format!("{}", token);
    }
    
    let token_desc = match &token.token_type {
        TokenType::Invalid(msg) => format!("Invalid: {}", msg).bright_red().to_string(),
        TokenType::Identifier(name) => format!("Identifier: {}", name).bright_yellow().to_string(),
        TokenType::IntLiteral(val) => format!("Int: {}", val).bright_cyan().to_string(),
        TokenType::FloatLiteral(val) => format!("Float: {}", val).bright_cyan().to_string(),
        TokenType::StringLiteral(val) => format!("String: \"{}\"", val).bright_green().to_string(),
        TokenType::RawString(val) => format!("RawString: r\"{}\"", val).bright_green().to_string(),
        TokenType::FString(val) => format!("FString: f\"{}\"", val).bright_green().to_string(),
        TokenType::BytesLiteral(bytes) => {
            let bytes_str = bytes.iter()
                .map(|b| format!("\\x{:02x}", b))
                .collect::<Vec<_>>()
                .join("");
            format!("Bytes: b\"{}\"", bytes_str).bright_green().to_string()
        },
        TokenType::BinaryLiteral(val) => format!("Binary: 0b{:b}", val).bright_cyan().to_string(),
        TokenType::OctalLiteral(val) => format!("Octal: 0o{:o}", val).bright_cyan().to_string(),
        TokenType::HexLiteral(val) => format!("Hex: 0x{:x}", val).bright_cyan().to_string(),
        TokenType::Indent => "Indent".bright_magenta().to_string(),
        TokenType::Dedent => "Dedent".bright_magenta().to_string(),
        TokenType::Newline => "Newline".bright_magenta().to_string(),
        _ => format!("{:?}", token.token_type),
    };
    
    format!("{} at {}:{}", token_desc, token.line, token.column)
}