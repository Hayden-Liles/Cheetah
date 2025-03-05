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
    },
    /// Check a file for syntax errors
    Check {
        /// The source file to check
        file: String,
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
        Commands::Lex { file, verbose, color } => {
            lex_file(&file, verbose, color)?;
        }
        Commands::Check { file } => {
            check_file(&file)?;
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
    
    loop {
        print!("{} ", ">>>".bright_green());
        io::stdout().flush()?;
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            // EOF (Ctrl+D)
            break;
        }
        
        let input = input.trim();
        if input == "exit" {
            break;
        }
        
        // Tokenize the input
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        
        // Check for errors
        let errors = lexer.get_errors();
        if !errors.is_empty() {
            for error in errors {
                eprintln!("{}", error.bright_red());
            }
        }
        
        // Display tokens
        for token in &tokens {
            match &token.token_type {
                TokenType::Invalid(_) => println!("{}", format!("{}", token).bright_red()),
                _ => println!("{}", token),
            }
        }
        
        // TODO: Add parsing and interpretation
    }
    
    println!("Goodbye!");
    Ok(())
}

fn lex_file(filename: &str, verbose: bool, use_color: bool) -> Result<()> {
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
                eprintln!("{}", error.bright_red());
            } else {
                eprintln!("{}", error);
            }
        }
    }
    
    println!("Tokens from file '{}':", filename);
    
    if verbose {
        // Display detailed token information including position
        for (i, token) in tokens.iter().enumerate() {
            let token_str = format!("{:4}: {}", i, token);
            
            if use_color {
                match &token.token_type {
                    TokenType::Def | TokenType::If | TokenType::Else | TokenType::For | 
                    TokenType::While | TokenType::Return => println!("{}", token_str.bright_blue()),
                    TokenType::Identifier(_) => println!("{}", token_str.bright_yellow()),
                    TokenType::StringLiteral(_) => println!("{}", token_str.bright_green()),
                    TokenType::IntLiteral(_) | TokenType::FloatLiteral(_) => println!("{}", token_str.bright_cyan()),
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
                    _ => println!("{}", token),
                }
            } else {
                println!("{}", token);
            }
        }
    }
    
    Ok(())
}

fn check_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;
    
    // Create a lexer with more strict configuration for checking
    let config = LexerConfig {
        enforce_indent_consistency: true,
        standard_indent_size: 4,
        tab_width: 4,
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
            eprintln!("  {}", error);
        }
    }
    
    // TODO: Add parsing and semantic analysis steps
    
    Ok(())
}

/// Format the token output based on token type for better readability
fn format_token_output(token: &Token, use_color: bool) -> String {
    if !use_color {
        return format!("{}", token);
    }
    
    match &token.token_type {
        TokenType::Invalid(_) => format!("{}", token).bright_red().to_string(),
        TokenType::Indent | TokenType::Dedent | TokenType::Newline => {
            format!("{}", token).bright_magenta().to_string()
        },
        _ => format!("{}", token).to_string(),
    }
}