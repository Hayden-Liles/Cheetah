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

    // Create a compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_block_scoping() {
    let source = r#"
# Variables in different blocks
x = 10  # Global scope

if True:
    y = 20  # Block scope
    z = x + y  # Can access global scope
else:
    y = 30  # Different block scope
    w = x + y  # Can access global scope

# y should not be accessible here
x = x + 5  # But x is still accessible
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile block scoping test: {:?}", result.err());

    // Print the IR for debugging
    println!("Block scoping IR:\n{}", result.unwrap());
}

#[test]
fn test_function_scoping() {
    let source = r#"
# Global variable
global_var = 100

def test_function(param1, param2):
    # Function scope variable
    local_var = param1 + param2

    # For now, we can't access global variables from functions
    # So we'll just return the local variable
    return local_var

# Call the function
result = test_function(10, 20)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile function scoping test: {:?}", result.err());

    // Print the IR for debugging
    println!("Function scoping IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_scopes() {
    let source = r#"
# Global scope
x = 1

def outer_function(a):
    # Outer function scope
    y = a

    if y > 10:
        # If block scope
        z = y * 2

        while z > 0:
            # While loop scope
            w = z - 1
            z = w

    return y

# Call the function
result = outer_function(10)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested scopes test: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested scopes IR:\n{}", result.unwrap());
}

#[test]
fn test_loop_scoping() {
    let source = r#"
# Global scope
sum = 0

# Simple loop with a fixed number of iterations
i = 0
temp = 0
while i < 5:
    temp = i * 2
    sum = sum + temp
    i = i + 1

# Another simple loop
counter = 5
temp_counter = 0
while counter > 0:
    temp_counter = counter
    sum = sum + temp_counter
    counter = counter - 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile loop scoping test: {:?}", result.err());

    // Print the IR for debugging
    println!("Loop scoping IR:\n{}", result.unwrap());
}

#[test]
fn test_shadowing() {
    let source = r#"
# Global scope
x = 10

def test_function(x):  # Parameter shadows global x
    # Function scope
    y = x + 5  # Uses parameter x, not global x

    if True:
        # Block scope
        x = 20  # Shadows parameter x
        z = x + y  # Uses block x, not parameter x

    return x  # Returns parameter x, not block x

# Call the function
result = test_function(15)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile shadowing test: {:?}", result.err());

    // Print the IR for debugging
    println!("Shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_block_scoping() {
    let source = r#"
# Test nested block scoping with multiple levels
x = 1

if x > 0:
    y = 2

    if y > 1:
        z = 3

        if z > 2:
            w = 4
            result = x + y + z + w  # Should be 10

        # w is not accessible here
        result2 = x + y + z  # Should be 6

    # z is not accessible here
    result3 = x + y  # Should be 3

# y is not accessible here
result4 = x  # Should be 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested block scoping test: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested block scoping IR:\n{}", result.unwrap());
}

#[test]
fn test_variable_reuse_in_different_scopes() {
    let source = r#"
# Test using different variable names in different scopes

if True:
    a = 10
    result1 = a

# Use a different variable name in a different scope
if True:
    b = 20
    result2 = b

# Create a function with its own scope
def test_func():
    # Use yet another variable name in the function scope
    c = 30
    return c

result3 = test_func()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile variable reuse test: {:?}", result.err());

    // Print the IR for debugging
    println!("Variable reuse IR:\n{}", result.unwrap());
}

#[test]
fn test_loop_variable_scoping() {
    let source = r#"
# Test loop variable scoping

# Initialize counter
i = 0
sum_outer = 0

# Outer loop
while i < 3:
    sum_outer = sum_outer + i

    # Initialize inner counter with same name in a nested scope
    j = 0
    sum_inner = 0

    # Inner loop with its own scope
    while j < 2:
        sum_inner = sum_inner + j
        j = j + 1

    # j from inner loop is still accessible here
    # but it should have its own value separate from outer i
    result_inner = sum_inner  # Should be 1 (0+1)

    # Increment outer counter
    i = i + 1

# Final result should be sum of 0, 1, 2 = 3
result_outer = sum_outer
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile loop variable scoping test: {:?}", result.err());

    // Print the IR for debugging
    println!("Loop variable scoping IR:\n{}", result.unwrap());
}
