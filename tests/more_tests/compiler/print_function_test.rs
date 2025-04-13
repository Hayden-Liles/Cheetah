use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;

/// Helper function to compile a source string and return the result
pub fn compile_source(source: &str) -> Result<String, String> {
    // Parse the source
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errors) => {
            return Err(format!("Parse errors: {:?}", errors));
        }
    };

    // Create a new LLVM context and compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "print_test_module");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => {
            // Get the generated IR for inspection
            let ir = compiler.get_ir();
            Ok(ir)
        },
        Err(err) => Err(format!("Compilation error: {}", err)),
    }
}

#[test]
fn test_basic_print_string() {
    // Test basic string printing
    let source = r#"
# Basic string printing
print("Hello, Cheetah!")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic string print: {:?}", result.err());

    let ir = result.unwrap();
    // Verify the IR contains a call to print_string function
    assert!(ir.contains("print_string") || ir.contains("println_string"),
            "IR doesn't contain print_string function call");
}

#[test]
fn test_print_multiple_arguments() {
    // Test printing multiple arguments
    let source = r#"
# Print multiple values
print("The answer is", 42)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with multiple arguments: {:?}", result.err());

    let ir = result.unwrap();
    // Verify the IR contains calls to both print_string and print_int
    assert!(ir.contains("print_string"), "IR doesn't contain print_string function call");
    assert!(ir.contains("print_int"), "IR doesn't contain print_int function call");
}

#[test]
fn test_print_empty() {
    // Test printing with no arguments (just a newline)
    let source = r#"
# Print without arguments
print()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile empty print: {:?}", result.err());

    let ir = result.unwrap();
    // Verify the IR contains a call to println_string with an empty string
    assert!(ir.contains("println_string"), "IR doesn't contain println_string function call");
}

#[test]
fn test_print_different_types() {
    // Test printing different data types
    let source = r#"
# Print different data types
print("String:", "text")
print("Integer:", 123)
print("Float:", 3.14)
print("Boolean:", True)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with different types: {:?}", result.err());

    let ir = result.unwrap();
    // Verify the IR contains calls to different print functions
    assert!(ir.contains("print_string"), "IR doesn't contain print_string function call");
    assert!(ir.contains("print_int"), "IR doesn't contain print_int function call");
    assert!(ir.contains("print_float"), "IR doesn't contain print_float function call");
    assert!(ir.contains("print_bool"), "IR doesn't contain print_bool function call");
}

#[test]
fn test_print_in_function() {
    // Test printing inside a function
    let source = r#"
# Print inside a function
def greet():
    print("Hello, World!")
    return 0

# Call the function
result = greet()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print in function: {:?}", result.err());
}

#[test]
fn test_print_in_loop() {
    // Test printing in a loop
    let source = r#"
# Print in a loop
for i in range(3):
    print("Loop iteration", i)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print in loop: {:?}", result.err());
}

#[test]
fn test_print_in_conditional() {
    // Test printing in conditional statements
    let source = r#"
# Print in conditional statements
x = 10
if x > 5:
    print("x is greater than 5")
else:
    print("x is not greater than 5")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print in conditional: {:?}", result.err());
}

#[test]
fn test_print_with_expressions() {
    // Test printing with expressions
    let source = r#"
# Print with expressions
a = 10
b = 20
c = a + b
print("Sum:", c)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with expressions: {:?}", result.err());
}

#[test]
fn test_print_special_characters() {
    // Test printing special characters
    let source = r#"
# Print special characters
print("Special chars: \n \t \\ \" '")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with special characters: {:?}", result.err());
}

#[test]
fn test_print_long_string() {
    // Test printing a very long string
    let source = r#"
# Print a very long string
print("This is a very long string that should test the compiler's ability to handle large string literals. " * 10)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with long string: {:?}", result.err());
}

#[test]
fn test_print_nested_expressions() {
    // Test printing with nested expressions
    let source = r#"
# Print with nested expressions
a = 5
b = 10
c = 15
d = a + b
print("Complex expression:", d)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with nested expressions: {:?}", result.err());
}

#[test]
fn test_print_function_calls() {
    // Test printing results of function calls
    let source = r#"
# Define a function
def calculate(x, y):
    return x * y + x

result = calculate(5, 3)
# Print result of function call
print("Result:", result)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile print with function calls: {:?}", result.err());
}

#[test]
fn test_print_multiple_lines() {
    // Test multiple print statements in sequence
    let source = r#"
# Multiple print statements
print("Line 1")
print("Line 2")
print("Line 3")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multiple print statements: {:?}", result.err());

    let ir = result.unwrap();
    // Count the number of print_string or println_string calls
    let print_calls = ir.matches("print_string").count() + ir.matches("println_string").count();
    assert!(print_calls >= 3, "IR doesn't contain at least 3 print function calls");
}
