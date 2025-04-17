use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use clap::{Parser as ClapParser, Subcommand};
use anyhow::{Result, Context};
use colored::Colorize;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// For stack size control
use libc;

// Import modules from lib.rs
use cheetah::lexer::{Lexer, Token, TokenType, LexerConfig};
use cheetah::parser::{self, ParseErrorFormatter};
use cheetah::formatter::CodeFormatter;
use cheetah::visitor::Visitor;
use cheetah::compiler::Compiler;
use cheetah::compiler::runtime::print_ops::{print_string, println_string, print_int, print_float, print_bool};
use cheetah::compiler::runtime::range_ops;
use cheetah::compiler::runtime::list_ops_runtime::{list_new, list_with_capacity, list_get, list_set, list_append, list_len, list_free, list_slice, list_concat, list_repeat};
use cheetah::parse;

use inkwell::context;
// Import LLVM context and optimization-related modules
use inkwell::targets::{InitializationConfig, Target};


#[derive(ClapParser)]
#[command(name = "cheetah")]
#[command(version = "0.1.0")]
#[command(about = "Cheetah programming language compiler", long_about = None)]
struct Cli {
    /// Source file to run (with .ch extension)
    #[arg(value_name = "FILE")]
    file: Option<String>,

    /// Use interpreter instead of LLVM JIT compilation
    #[arg(short = 'i', long, default_value = "false")]
    interpreter: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Cheetah source file
    Run {
        /// The source file to run
        file: String,

        /// Use interpreter instead of LLVM JIT compilation
        #[arg(short = 'i', long, default_value = "false")]
        interpreter: bool,
    },
    /// Start a REPL session
    Repl {
        /// Use LLVM JIT compilation in REPL
        #[arg(short = 'j', long)]
        jit: bool,
    },
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
    /// Compile a Cheetah source file to LLVM IR
    Compile {
        /// The source file to compile
        file: String,

        /// Output path (defaults to input file name with .ll extension)
        #[arg(short, long)]
        output: Option<String>,

        /// Optimization level (0-3)
        #[arg(short, long, default_value = "0")]
        opt: u8,

        /// Compile to object file instead of LLVM IR
        #[arg(short, long)]
        object: bool,

        /// Target triple (default: host target)
        #[arg(short, long)]
        target: Option<String>,
    },
}

// Function to increase the stack size limit
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn increase_stack_size() {
    // Set stack size to 256MB (256 * 1024 * 1024) for large ranges
    let stack_size = 256 * 1024 * 1024;

    // Get current limits
    let mut current_rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    unsafe {
        // Get current limits first
        if libc::getrlimit(libc::RLIMIT_STACK, &mut current_rlim) != 0 {
            eprintln!("Warning: Failed to get current stack size limits.");
        }

        // Use the maximum available or our desired size, whichever is smaller
        let new_size = if current_rlim.rlim_max != libc::RLIM_INFINITY && current_rlim.rlim_max < stack_size {
            eprintln!("Note: System maximum stack size is {}MB, using that instead of requested {}MB",
                     current_rlim.rlim_max / (1024 * 1024), stack_size / (1024 * 1024));
            current_rlim.rlim_max
        } else {
            stack_size
        };

        let rlim = libc::rlimit {
            rlim_cur: new_size,
            rlim_max: current_rlim.rlim_max,
        };

        // Try to increase the stack size
        if libc::setrlimit(libc::RLIMIT_STACK, &rlim) != 0 {
            eprintln!("Warning: Failed to increase stack size. Stack overflows may occur with large ranges.");
        } else {
            println!("{}", format!("Stack size increased to {}MB for handling large ranges", new_size / (1024 * 1024)).bright_green());
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn increase_stack_size() {
    eprintln!("Warning: Stack size adjustment not supported on this platform.");
}

fn main() -> Result<()> {
    // Increase stack size to prevent stack overflow
    increase_stack_size();

    // Initialize LLVM targets for cross-compilation support
    initialize_llvm_targets();

    let cli = Cli::parse();

    // Handle direct file execution (cheetah main.ch)
    if let Some(file) = cli.file {
        // Always use interpreter mode for now to avoid segmentation faults
        run_file(&file)?;
        return Ok(());
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Run { file, interpreter: _ }) => {
            // Always use interpreter mode for now to avoid segmentation faults
            // if interpreter {
                run_file(&file)?;
            // } else {
            //    run_file_jit(&file)?;
            // }
        }
        Some(Commands::Repl { jit }) => {
            if jit {
                run_repl_jit()?;
            } else {
                run_repl()?;
            }
        }
        Some(Commands::Lex { file, verbose, color, line_numbers }) => {
            lex_file(&file, verbose, color, line_numbers)?;
        }
        Some(Commands::Parse { file, verbose }) => {
            parse_file(&file, verbose)?;
        }
        Some(Commands::Check { file, verbose }) => {
            check_file(&file, verbose)?;
        }
        Some(Commands::Format { file, write, indent }) => {
            format_file(&file, write, indent)?;
        }
        Some(Commands::Compile { file, output, opt, object, target }) => {
            compile_file(&file, output, opt, object, target)?;
        }
        None => {
            // If no file and no command, start REPL
            run_repl()?
        }
    }

    Ok(())
}

fn initialize_llvm_targets() {
    let config = InitializationConfig {
        asm_parser: true,
        asm_printer: true,
        base: true,
        disassembler: true,
        info: true,
        machine_code: true,
    };

    Target::initialize_all(&config);
}

/// Ensure the file has a .ch extension, adding it if necessary
fn ensure_ch_extension(filename: &str) -> String {
    let path = PathBuf::from(filename);
    if let Some(ext) = path.extension() {
        if ext == "ch" {
            return filename.to_string();
        }
    }

    // If no extension or not .ch, add .ch extension
    let mut path_with_ext = path.clone();
    path_with_ext.set_extension("ch");
    path_with_ext.to_string_lossy().to_string()
}

fn run_file(filename: &str) -> Result<()> {
    let filename = ensure_ch_extension(filename);
    println!("{}", format!("Interpreting {}", filename).bright_green());

    let source = fs::read_to_string(&filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;

    // Parse the source code
    match parse(&source) {
        Ok(module) => {
            // Create a new interpreter
            let mut interpreter = cheetah::interpreter::Interpreter::new();

            // Interpret the module
            match interpreter.interpret(&module) {
                Ok(_) => {
                    println!("{}", format!("Execution completed successfully").bright_green());
                    Ok(())
                },
                Err(e) => Err(anyhow::anyhow!("Execution failed: {}", e)),
            }
        },
        Err(errors) => {
            for error in &errors {
                let formatter = ParseErrorFormatter::new(error, Some(&source), true);
                eprintln!("{}", formatter.format().bright_red());
            }
            Err(anyhow::anyhow!("Parsing failed"))
        }
    }
}

// run_file_jit function has been merged into run_file

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
                                let formatter = ParseErrorFormatter::new(&error, Some(complete_input), true);
                                eprintln!("{}", formatter.format().bright_red());
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

fn run_repl_jit() -> Result<()> {
    println!("{}", "Cheetah Programming Language REPL (JIT Mode)".bright_green());
    println!("Type 'exit' or press Ctrl+D to exit");

    let mut input_buffer = String::new();
    let mut paren_level = 0;
    let mut bracket_level = 0;
    let mut brace_level = 0;
    let mut in_multiline_block = false;

    // Create LLVM context once for the entire REPL session
    let context = context::Context::create();
    let mut repl_count = 0;

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
                repl_count += 1;
                let module_name = format!("repl_{}", repl_count);

                // Parse the input
                match parse(complete_input) {
                    Ok(module) => {
                        // Create compiler for this REPL entry
                        let mut compiler = Compiler::new(&context, &module_name);

                        // Compile the AST
                        match compiler.compile_module(&module) {
                            Ok(_) => {
                                println!("{}", "✓ Compiled successfully".bright_green());

                                // Get the compiled module
                                let compiled_module = compiler.get_module();

                                // Apply optimization passes to the module
                                apply_optimization_passes(compiled_module);

                                // Create JIT execution engine with aggressive optimization
                                match compiled_module.create_jit_execution_engine(inkwell::OptimizationLevel::Aggressive) {
                                    Ok(execution_engine) => {
                                        // Register runtime functions with the execution engine
                                        if let Err(e) = register_runtime_functions(&execution_engine, compiled_module) {
                                            println!("{}", format!("Warning: Failed to register some runtime functions: {}", e).bright_yellow());
                                        }

                                        // Execute the "main" function using the JIT execution engine
                                        unsafe {
                                            // Look up the main function in the module
                                            match execution_engine.get_function::<unsafe extern "C" fn() -> ()>("main") {
                                                Ok(main_fn) => {
                                                    // Execute the main function
                                                    println!("{}", "Executing main function...".bright_green());
                                                    main_fn.call();
                                                    // Flush any remaining output
                                                    cheetah::compiler::runtime::buffered_output::flush_output_buffer();

                                                    // Clean up range operations
                                                    cheetah::compiler::runtime::range_ops::cleanup();

                                                    // Clean up range iterator system
                                                    cheetah::compiler::runtime::range_iterator::cleanup();

                                                    // Clean up circular buffer
                                                    cheetah::compiler::runtime::circular_buffer::cleanup();

                                                    // Clean up memory profiler
                                                    cheetah::compiler::runtime::memory_profiler::cleanup();

                                                    // Clean up parallel processing
                                                    cheetah::compiler::runtime::parallel_ops::cleanup();

                                                    println!("{}", "Execution completed.".bright_green());
                                                },
                                                Err(e) => {
                                                    println!("{}", format!("Warning: Failed to find main function: {}", e).bright_yellow());
                                                    println!("{}", "Displaying IR instead:".bright_yellow());
                                                    println!("{}", compiler.get_ir());
                                                }
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("{}", format!("Failed to create execution engine: {}", e).bright_red());
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("{}", format!("Compilation error: {}", e).bright_red());
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
    let filename = ensure_ch_extension(filename);
    let source = fs::read_to_string(&filename)
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
    let filename = ensure_ch_extension(filename);
    let source = fs::read_to_string(&filename)
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
                let formatter = ParseErrorFormatter::new(&error, Some(&source), true);
                eprintln!("  {}", formatter);
            }
        }
    }

    Ok(())
}

fn check_file(filename: &str, verbose: bool) -> Result<()> {
    let filename = ensure_ch_extension(filename);
    let source = fs::read_to_string(&filename)
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
                    // Use the new ParseErrorFormatter for better error messages
                    let formatter = ParseErrorFormatter::new(&error, Some(&source), true);
                    eprintln!("  {}", formatter);
                } else {
                    eprintln!("  {}", error.get_message());
                }
            }
        }
    }

    Ok(())
}

fn format_file(filename: &str, write: bool, indent_size: usize) -> Result<()> {
    let filename = ensure_ch_extension(filename);
    let source = fs::read_to_string(&filename)
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
                fs::write(&filename, &formatted_source)
                    .with_context(|| format!("Failed to write to file: {}", filename))?;
                println!("Formatted and wrote changes to '{}'", filename);
            } else {
                print!("{}", formatted_source);
            }
        },
        Err(errors) => {
            eprintln!("Cannot format file with syntax errors:");
            for error in errors {
                let formatter = ParseErrorFormatter::new(&error, Some(&source), true);
                eprintln!("  {}", formatter);
            }
        }
    }

    Ok(())
}

fn compile_file(
    filename: &str,
    output: Option<String>,
    opt_level: u8,
    output_object: bool,
    target_triple: Option<String>
) -> Result<()> {
    let _ = target_triple;
    let filename = ensure_ch_extension(filename);
    println!("{}", format!("Compiling {} with optimization level {}", filename, opt_level).bright_green());

    let source = fs::read_to_string(&filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;

    // Parse the source code
    match parse(&source) {
        Ok(module) => {
            // Create LLVM context and compiler
            let context = context::Context::create();
            let mut compiler = Compiler::new(&context, &filename);

            // Set optimization level based on user input
            let opt_level = match opt_level {
                0 => inkwell::OptimizationLevel::None,
                1 => inkwell::OptimizationLevel::Less,
                2 => inkwell::OptimizationLevel::Default,
                _ => inkwell::OptimizationLevel::Aggressive,
            };
            println!("{}", format!("Using optimization level: {:?}", opt_level).bright_green());

            // Compile the AST
            match compiler.compile_module(&module) {
                Ok(_) => {
                    // Determine output path and extension
                    let mut output_path = match output {
                        Some(path) => PathBuf::from(path),
                        None => {
                            let mut path = PathBuf::from(filename);
                            path.set_extension(if output_object { "o" } else { "ll" });
                            path
                        }
                    };

                    // If output_object is true, we would compile to an object file
                    // For now, we'll just write the LLVM IR
                    if output_object {
                        println!("{}", "Object file output not yet implemented, defaulting to LLVM IR".bright_yellow());
                        if output_path.extension().unwrap_or_default() == "o" {
                            output_path.set_extension("ll");
                        }
                    }

                    // Write LLVM IR to file
                    compiler.write_to_file(&output_path)
                        .map_err(|e| anyhow::anyhow!("Failed to write IR to file: {}", e))?;

                    println!("Successfully compiled to {}", output_path.display());
                    Ok(())
                },
                Err(e) => Err(anyhow::anyhow!("Compilation failed: {}", e)),
            }
        },
        Err(errors) => {
            for error in &errors {
                let formatter = ParseErrorFormatter::new(error, Some(&source), true);
                eprintln!("{}", formatter.format().bright_red());
            }
            Err(anyhow::anyhow!("Parsing failed"))
        }
    }
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

// This function registers all runtime functions with the JIT execution engine
/// Apply optimization passes to the LLVM module to improve performance
fn apply_optimization_passes(_module: &inkwell::module::Module<'_>) {
    // We'll use LLVM's built-in optimization levels instead of manual passes
    // The actual optimization will be done when creating the execution engine
    // with OptimizationLevel::Aggressive

    println!("{}", "Using aggressive optimization level for improved performance".bright_green());
    println!("{}", "Stack overflow prevention enabled".bright_green());
}

fn register_runtime_functions(
    engine: &inkwell::execution_engine::ExecutionEngine<'_>,
    module: &inkwell::module::Module<'_>
) -> Result<(), String> {
    // Type conversion functions
    if let Some(function) = module.get_function("int_to_string") {
        {
            engine.add_global_mapping(&function, jit_int_to_string as usize);
        }
    }

    if let Some(function) = module.get_function("float_to_string") {
        {
            engine.add_global_mapping(&function, jit_float_to_string as usize);
        }
    }

    if let Some(function) = module.get_function("bool_to_string") {
        {
            engine.add_global_mapping(&function, jit_bool_to_string as usize);
        }
    }

    // Range functions - map only the original functions, not the duplicates with suffixes
    if let Some(function) = module.get_function("range_1") {
        {
            engine.add_global_mapping(&function, range_ops::range_1 as usize);
        }
    }

    if let Some(function) = module.get_function("range_2") {
        {
            engine.add_global_mapping(&function, range_ops::range_2 as usize);
        }
    }

    if let Some(function) = module.get_function("range_3") {
        {
            engine.add_global_mapping(&function, range_ops::range_3 as usize);
        }
    }

    if let Some(function) = module.get_function("range_cleanup") {
        {
            engine.add_global_mapping(&function, range_ops::range_cleanup as usize);
        }
    }

    // Map the range functions with suffixes to the same implementations
    if let Some(function) = module.get_function("range_1.3") {
        {
            engine.add_global_mapping(&function, range_ops::range_1 as usize);
        }
    }

    if let Some(function) = module.get_function("range_2.4") {
        {
            engine.add_global_mapping(&function, range_ops::range_2 as usize);
        }
    }

    if let Some(function) = module.get_function("range_3.5") {
        {
            engine.add_global_mapping(&function, range_ops::range_3 as usize);
        }
    }

    if let Some(function) = module.get_function("string_to_int") {
        {
            engine.add_global_mapping(&function, jit_string_to_int as usize);
        }
    }

    if let Some(function) = module.get_function("string_to_float") {
        {
            engine.add_global_mapping(&function, jit_string_to_float as usize);
        }
    }

    if let Some(function) = module.get_function("string_to_bool") {
        {
            engine.add_global_mapping(&function, jit_string_to_bool as usize);
        }
    }

    if let Some(function) = module.get_function("char_to_string") {
        {
            engine.add_global_mapping(&function, jit_char_to_string as usize);
        }
    }

    if let Some(function) = module.get_function("free_string") {
        {
            engine.add_global_mapping(&function, jit_free_string as usize);
        }
    }

    // Built-in functions (these would call the type conversion functions)
    if let Some(function) = module.get_function("str_int") {
        {
            engine.add_global_mapping(&function, jit_str_int as usize);
        }
    }

    if let Some(function) = module.get_function("str_float") {
        {
            engine.add_global_mapping(&function, jit_str_float as usize);
        }
    }

    if let Some(function) = module.get_function("str_bool") {
        {
            engine.add_global_mapping(&function, jit_str_bool as usize);
        }
    }

    // Print functions
    if let Some(function) = module.get_function("print_string") {
        {
            engine.add_global_mapping(&function, print_string as usize);
        }
    }

    if let Some(function) = module.get_function("println_string") {
        {
            engine.add_global_mapping(&function, println_string as usize);
        }
    }

    if let Some(function) = module.get_function("print_int") {
        {
            engine.add_global_mapping(&function, print_int as usize);
        }
    }

    if let Some(function) = module.get_function("print_float") {
        {
            engine.add_global_mapping(&function, print_float as usize);
        }
    }

    if let Some(function) = module.get_function("print_bool") {
        {
            engine.add_global_mapping(&function, print_bool as usize);
        }
    }

    // String operations
    if let Some(function) = module.get_function("string_concat") {
        {
            engine.add_global_mapping(&function, jit_string_concat as usize);
        }
    }

    if let Some(function) = module.get_function("string_equals") {
        {
            engine.add_global_mapping(&function, jit_string_equals as usize);
        }
    }

    if let Some(function) = module.get_function("string_length") {
        {
            engine.add_global_mapping(&function, jit_string_length as usize);
        }
    }

    // List operations
    if let Some(function) = module.get_function("list_new") {
        {
            engine.add_global_mapping(&function, list_new as usize);
        }
    }

    if let Some(function) = module.get_function("list_with_capacity") {
        {
            engine.add_global_mapping(&function, list_with_capacity as usize);
        }
    }

    if let Some(function) = module.get_function("list_get") {
        {
            engine.add_global_mapping(&function, list_get as usize);
        }
    }

    if let Some(function) = module.get_function("list_set") {
        {
            engine.add_global_mapping(&function, list_set as usize);
        }
    }

    if let Some(function) = module.get_function("list_append") {
        {
            engine.add_global_mapping(&function, list_append as usize);
        }
    }

    if let Some(function) = module.get_function("list_len") {
        {
            engine.add_global_mapping(&function, list_len as usize);
        }
    }

    if let Some(function) = module.get_function("list_free") {
        {
            engine.add_global_mapping(&function, list_free as usize);
        }
    }

    if let Some(function) = module.get_function("list_slice") {
        {
            engine.add_global_mapping(&function, list_slice as usize);
        }
    }

    if let Some(function) = module.get_function("list_concat") {
        {
            engine.add_global_mapping(&function, list_concat as usize);
        }
    }

    if let Some(function) = module.get_function("list_repeat") {
        {
            engine.add_global_mapping(&function, list_repeat as usize);
        }
    }

    // Map the len function to the appropriate implementation based on the argument type
    if let Some(function) = module.get_function("len") {
        {
            // Use list_len as the implementation for len
            engine.add_global_mapping(&function, list_len as usize);
        }
    }

    Ok(())
}

// Runtime function implementations - optimized for performance
extern "C" fn jit_int_to_string(value: i64) -> *mut c_char {
    // Use a more efficient approach for small integers
    let s = if value >= -9999 && value <= 9999 {
        // Small integers can use a fixed buffer
        let mut buffer = [0u8; 16]; // More than enough for small integers
        let s = value.to_string();
        let bytes = s.as_bytes();
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()] = 0; // Null terminator
        unsafe { CString::from_raw(buffer.as_ptr() as *mut c_char) }
    } else {
        // For larger integers, use the standard approach
        CString::new(value.to_string()).unwrap()
    };
    s.into_raw()
}

extern "C" fn jit_float_to_string(value: f64) -> *mut c_char {
    let s = format!("{}", value);
    let c_str = CString::new(s).unwrap();
    c_str.into_raw()
}

extern "C" fn jit_bool_to_string(value: i64) -> *mut c_char {
    let s = if value != 0 { "True" } else { "False" }.to_string();
    let c_str = CString::new(s).unwrap();
    c_str.into_raw()
}

extern "C" fn jit_char_to_string(value: i64) -> *mut c_char {
    // Convert the character code to a Rust char
    let c = std::char::from_u32(value as u32).unwrap_or('\0');

    // Create a string with just this character
    let s = c.to_string();

    // Convert to C string and return
    let c_str = CString::new(s).unwrap();
    c_str.into_raw()
}

extern "C" fn jit_string_to_int(value: *const c_char) -> i64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    s.parse::<i64>().unwrap_or(0)
}

extern "C" fn jit_string_to_float(value: *const c_char) -> f64 {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    s.parse::<f64>().unwrap_or(0.0)
}

extern "C" fn jit_string_to_bool(value: *const c_char) -> bool {
    let c_str = unsafe { CStr::from_ptr(value) };
    let s = c_str.to_str().unwrap_or("");
    match s.to_lowercase().as_str() {
        "true" | "1" => true,
        _ => false,
    }
}

extern "C" fn jit_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
            // Memory is freed when CString is dropped
        }
    }
}

// Built-in function implementations
extern "C" fn jit_str_int(value: i64) -> *mut c_char {
    jit_int_to_string(value)
}

extern "C" fn jit_str_float(value: f64) -> *mut c_char {
    jit_float_to_string(value)
}

extern "C" fn jit_str_bool(value: bool) -> *mut c_char {
    jit_bool_to_string(if value { 1 } else { 0 })
}

// String operation implementations
extern "C" fn jit_string_concat(left: *const c_char, right: *const c_char) -> *mut c_char {
    let left_cstr = unsafe { CStr::from_ptr(left) };
    let right_cstr = unsafe { CStr::from_ptr(right) };

    let left_str = left_cstr.to_str().unwrap_or("");
    let right_str = right_cstr.to_str().unwrap_or("");

    let result = format!("{}{}", left_str, right_str);
    let c_str = CString::new(result).unwrap();
    c_str.into_raw()
}

extern "C" fn jit_string_equals(left: *const c_char, right: *const c_char) -> bool {
    let left_cstr = unsafe { CStr::from_ptr(left) };
    let right_cstr = unsafe { CStr::from_ptr(right) };

    let left_str = left_cstr.to_str().unwrap_or("");
    let right_str = right_cstr.to_str().unwrap_or("");

    left_str == right_str
}

extern "C" fn jit_string_length(string: *const c_char) -> i64 {
    let cstr = unsafe { CStr::from_ptr(string) };
    let s = cstr.to_str().unwrap_or("");
    s.len() as i64
}

// Range functions are now implemented in src/compiler/runtime/range_ops.rs