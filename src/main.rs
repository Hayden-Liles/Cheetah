mod lexer;

use std::fs;
use std::io::{self, Write};
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use colored::Colorize;

use crate::lexer::{Lexer, Token, TokenType, LexerConfig};

#[derive(Parser)]
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
        Commands::Check { file, verbose } => {
            check_file(&file, verbose)?;
        }
        Commands::Format { file, write } => {
            format_file(&file, write)?;
        }
    }

    Ok(())
}

fn run_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Create a lexer with default configuration
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Check for lexical errors
    let errors = lexer.get_errors();
    if !errors.is_empty() {
        eprintln!("Lexical errors found in '{}':", filename);
        for error in errors {
            eprintln!("{}", error);
        }
        return Ok(());
    }
    
    // TODO: Add parsing and interpretation steps
    println!("Successfully lexed file: {}", filename);
    println!("Found {} tokens", tokens.len());
    
    Ok(())
}

fn run_repl() -> Result<()> {
    println!("{}", "Cheetah Programming Language REPL".bright_green());
    println!("Type 'exit' or press Ctrl+D to exit");
    
    // Keep track of input state for multi-line input support
    let mut input_buffer = String::new();
    let mut paren_level = 0;
    let mut bracket_level = 0;
    let mut brace_level = 0;
    let mut in_multiline_block = false;
    let mut prompt = ">>> ".bright_green().to_string();
    
    loop {
        // Show appropriate prompt
        if !input_buffer.is_empty() {
            prompt = "... ".bright_yellow().to_string();
        } else {
            prompt = ">>> ".bright_green().to_string();
        }
        
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            // EOF (Ctrl+D)
            break;
        }
        
        let input = input.trim_end(); // Preserve leading whitespace but remove trailing
        
        if input_buffer.is_empty() && input == "exit" {
            break;
        }
        
        // Add the input to our buffer
        input_buffer.push_str(input);
        input_buffer.push('\n');
        
        // Check if we should continue collecting input (unclosed parentheses, indentation, etc.)
        update_repl_state(&input, &mut paren_level, &mut bracket_level, &mut brace_level, &mut in_multiline_block);
        
        let should_execute = !in_multiline_block && paren_level == 0 && bracket_level == 0 && brace_level == 0 && 
                              (input.trim().is_empty() || !input.trim().ends_with(':'));
        
        if should_execute {
            // Process the complete input
            let complete_input = input_buffer.trim();
            
            if !complete_input.is_empty() {
                // Tokenize the input
                let mut lexer = Lexer::new(complete_input);
                let tokens = lexer.tokenize();
                
                // Check for errors
                let errors = lexer.get_errors();
                if !errors.is_empty() {
                    for error in errors {
                        eprintln!("{}", error.to_string().bright_red());
                    }
                } else {
                    // Display tokens if no errors
                    for token in &tokens {
                        match &token.token_type {
                            TokenType::Invalid(_) => println!("{}", format!("{}", token).bright_red()),
                            _ => println!("{}", format_token_for_repl(token, true)),
                        }
                    }
                    
                    // TODO: Add parsing and interpretation
                }
            }
            
            // Reset the buffer and state for the next input
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
    // Count parentheses, brackets, and braces
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
    
    // Check for multiline block
    if input.trim().ends_with(':') {
        *in_multiline_block = true;
    } else if input.trim().is_empty() && *in_multiline_block {
        // Empty line ends a multiline block
        *in_multiline_block = false;
    }
}

fn lex_file(filename: &str, verbose: bool, use_color: bool, line_numbers: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Create lexer with default config
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Check for lexical errors
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
        // Display detailed token information including position
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
        // Display simplified token information
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

fn check_file(filename: &str, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Create a lexer with more strict configuration for checking
    let config = LexerConfig {
        enforce_indent_consistency: true,
        standard_indent_size: 4,
        tab_width: 4,
        allow_tabs_in_indentation: false,
        strict_line_joining: true,
        allow_trailing_semicolon: false,
    };
    
    let mut lexer = Lexer::with_config(&source, config);
    let _tokens = lexer.tokenize();
    
    // Check for lexical errors
    let errors = lexer.get_errors();
    if errors.is_empty() {
        println!("✓ No lexical errors found in '{}'", filename);
    } else {
        eprintln!("✗ Lexical errors found in '{}':", filename);
        for error in errors {
            if verbose {
                // Detailed error reporting with line context and suggestion
                eprintln!("  Line {}, Col {}: {}", error.line, error.column, error.message);
                eprintln!("  {}", error.snippet);
                eprintln!("  {}^", " ".repeat(error.column + 1));
                if let Some(suggestion) = &error.suggestion {
                    eprintln!("  Suggestion: {}", suggestion);
                }
                eprintln!();
            } else {
                // Simplified error reporting
                eprintln!("  {}", error);
            }
        }
    }
    
    // TODO: Add parsing and semantic analysis steps
    
    Ok(())
}

fn format_file(filename: &str, write: bool) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Create a lexer with standard configuration
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    // Check for lexical errors
    let errors = lexer.get_errors();
    if !errors.is_empty() {
        eprintln!("Cannot format file with lexical errors:");
        for error in errors {
            eprintln!("  {}", error);
        }
        return Ok(());
    }
    
    // TODO: Implement code formatting logic here
    
    // For now, just echo the original source
    let formatted_source = source.clone();
    
    if write {
        fs::write(filename, formatted_source)
            .with_context(|| format!("Failed to write to file: {}", filename))?;
        println!("Formatted and wrote changes to '{}'", filename);
    } else {
        // Print to stdout
        print!("{}", formatted_source);
    }
    
    Ok(())
}

/// Format the token output based on token type for better readability
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
    
    // For REPL, we use a more compact format than the general token format
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