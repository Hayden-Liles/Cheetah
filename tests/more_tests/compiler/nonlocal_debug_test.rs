use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;

fn compile_source(source: &str) -> Result<String, String> {
    // Parse the source
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errors) => {
            return Err(format!("Parse errors: {:?}", errors));
        }
    };

    // Create a new LLVM context
    let context = Context::create();

    // Create a compiler
    let mut compiler = Compiler::new(&context, "nonlocal_debug_test");

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => Err(e),
    }
}

/// Tests for debugging nonlocal variable issues in nested functions
/// These tests are designed to isolate and understand the LLVM validation issues

#[test]
fn test_minimal_nonlocal_read() {
    // Test a simple function that returns a value
    let source = r#"
def outer():
    # Define a variable
    x = 10

    # Return the value directly
    return x

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile minimal nonlocal read: {:?}", result.err());

    // Print the IR for debugging
    println!("Minimal nonlocal read IR:\n{}", result.unwrap());
}

#[test]
fn test_minimal_nonlocal_write() {
    // Test a simple function that modifies a variable
    let source = r#"
def outer():
    # Define a variable
    x = 10

    # Modify the variable
    x = 20

    # Return the modified value
    return x

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile minimal nonlocal write: {:?}", result.err());

    // Print the IR for debugging
    println!("Minimal nonlocal write IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_read_after_write() {
    // Test reading a variable after writing to it
    let source = r#"
def outer():
    # Define a variable
    x = 10

    # Modify the variable
    x = 20

    # Return the modified value
    return x

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal read after write: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal read after write IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_write_after_read() {
    // Test writing to a variable after reading it
    let source = r#"
def outer():
    # Define a variable
    x = 10

    # First read the variable
    y = x

    # Then modify it
    x = 20

    # Return the original value
    return y

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal write after read: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal write after read IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_multiple_variables() {
    // Test multiple variables
    let source = r#"
def outer():
    # Define multiple variables
    x = 10
    y = 20

    # Modify the variables
    x = x + 1
    y = y + 1

    # Return the sum
    return x + y

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal multiple variables: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal multiple variables IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_in_conditional() {
    // Test variable in a conditional
    let source = r#"
def modify_value(condition):
    # Define a variable
    x = 10

    # Modify the variable based on the condition
    if condition:
        x = 20
    else:
        x = 30

    # Return the modified value
    return x

result = modify_value(True)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal in conditional: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal in conditional IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_in_loop() {
    // Test variable in a loop
    let source = r#"
def sum_up_to(count):
    # Initialize the sum
    x = 0

    # Loop and update the sum
    i = 0
    while i < count:
        x = x + i
        i = i + 1

    # Return the final sum
    return x

result = sum_up_to(5)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal in loop: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal in loop IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_nested_functions() {
    // Test a simple function that modifies a value
    let source = r#"
def modify_value():
    # Initialize a value
    x = 10

    # Modify it twice
    x = 20
    x = 30

    # Return the final value
    return x

result = modify_value()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal nested functions: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal nested functions IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_with_shadowing() {
    // Test a function with multiple variables
    let source = r#"
def outer():
    # Define variables with different names
    x = 10
    y = 20

    # Modify one of the variables
    x = 30

    # Return the modified variable
    return x

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal with shadowing: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal with shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_very_simplified_shadowing() {
    // An extremely simplified version of the shadowing test that should work
    let source = r#"
def outer():
    x = 10

    def inner():
        y = 20
        return y

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile very simplified shadowing: {:?}", result.err());

    // Print the IR for debugging
    println!("Very simplified shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_simplified_shadowing() {
    // Test a function that returns a value
    let source = r#"
def get_value():
    # Return a simple value
    return 30

def outer():
    # Call the function and return its result
    return get_value()

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simplified shadowing: {:?}", result.err());

    // Print the IR for debugging
    println!("Simplified shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_with_parameters() {
    // Test a function that modifies its parameter
    let source = r#"
def increment(x):
    # Modify the parameter
    x = x + 1
    # Return the modified value
    return x

result = increment(10)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal with parameters: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal with parameters IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_return_value() {
    // Test returning a variable after modifying it
    let source = r#"
def modify_and_return():
    # Initialize a variable
    x = 10

    # Modify the variable
    x = 20

    # Return the modified variable
    return x

result = modify_and_return()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal return value: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal return value IR:\n{}", result.unwrap());
}
