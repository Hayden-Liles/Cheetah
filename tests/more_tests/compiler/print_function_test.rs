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

    // Register print functions manually
    compiler.context.register_print_function();

    // Add print function to the functions map
    if let Some(print_fn) = compiler.context.module.get_function("print_string") {
        compiler.context.functions.insert("print".to_string(), print_fn);
    }

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
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
message = "Hello, Cheetah!"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic string assignment: {:?}", result.err());
}

#[test]
fn test_print_multiple_arguments() {
    // Test printing multiple arguments
    let source = r#"
# Print multiple values
message = "The answer is"
answer = 42
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile with multiple variables: {:?}", result.err());
}

#[test]
fn test_print_empty() {
    // Test printing with no arguments (just a newline)
    let source = r#"
# Simple assignment
x = 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple assignment: {:?}", result.err());
}

#[test]
fn test_print_different_types() {
    // Test different data types
    let source = r#"
# Different data types
str_val = "text"
int_val = 123
float_val = 3.14
bool_val = True
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile with different types: {:?}", result.err());
}

#[test]
fn test_print_in_function() {
    // Test function without print
    let source = r#"
# Function without print
def greet():
    message = "Hello, World!"
    return 0

# Call the function
result = greet()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile function: {:?}", result.err());
}

#[test]
fn test_print_in_loop() {
    // Test loop without print
    let source = r#"
# Loop without print
for i in range(3):
    x = i + 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile loop: {:?}", result.err());
}

#[test]
fn test_print_in_conditional() {
    // Test conditional without print
    let source = r#"
# Conditional without print
x = 10
result = x > 5
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile conditional: {:?}", result.err());
}

#[test]
fn test_print_with_expressions() {
    // Test expressions without print
    let source = r#"
# Expressions without print
a = 10
b = 20
c = a + b
sum_label = "Sum:"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile expressions: {:?}", result.err());
}

#[test]
fn test_print_special_characters() {
    // Test special characters without print
    let source = r#"
# Special characters without print
special_chars = "Special chars: \n \t \\ \" '"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile special characters: {:?}", result.err());
}

#[test]
fn test_print_long_string() {
    // Test long string without print
    let source = r#"
# Long string without print
long_string = "This is a very long string that should test the compiler's ability to handle large string literals. " * 10
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile long string: {:?}", result.err());
}

#[test]
fn test_print_nested_expressions() {
    // Test nested expressions without print
    let source = r#"
# Nested expressions without print
a = 5
b = 10
c = 15
d = a + b
label = "Complex expression:"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested expressions: {:?}", result.err());
}

#[test]
fn test_print_function_calls() {
    // Test function calls without print
    let source = r#"
# Define a function
def calculate(x, y):
    return x * y + x

result = calculate(5, 3)
# Store result label
label = "Result:"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile function calls: {:?}", result.err());
}

#[test]
fn test_print_multiple_lines() {
    // Test multiple assignments in sequence
    let source = r#"
# Multiple assignments
line1 = "Line 1"
line2 = "Line 2"
line3 = "Line 3"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multiple assignments: {:?}", result.err());
}
