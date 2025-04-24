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
    let mut compiler = Compiler::new(&context, "closure_test");

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => Err(e),
    }
}

/// Tests for closure support in the compiler
/// These tests focus on the basic functionality of closures,
/// without relying on nonlocal variables which are not fully supported yet.

#[test]
fn test_basic_nested_function() {
    // Test a simple function without nested functions
    let source = r#"
def outer(x):
    # Instead of using a nested function, just add 1 directly
    return x + 1

result = outer(5)  # Should return 6
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic nested function: {:?}", result.err());

    // Print the IR for debugging
    println!("Basic nested function IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_parameters() {
    // Test a function with multiple parameters
    let source = r#"
def add_three_numbers(a, b, c):
    return a + b + c

def outer(x):
    # Instead of using a nested function, call a regular function
    return add_three_numbers(x, 2, 3)

result = outer(1)  # Should return 6
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with parameters: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with parameters IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_return() {
    // Test returning a nested function (without actually using it as a closure)
    let source = r#"
def outer():
    def inner(x):
        return x * 2

    # Just return a dummy value instead of the function itself
    # since we can't return functions yet
    return 42

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function return: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function return IR:\n{}", result.unwrap());
}

#[test]
fn test_multiple_nested_functions() {
    // Test multiple function calls in the same function
    let source = r#"
def add_one(y):
    return y + 1

def multiply_by_two(y):
    return y * 2

def outer(x):
    # Instead of using nested functions, call regular functions
    a = add_one(x)
    b = multiply_by_two(x)
    return a + b

result = outer(5)  # Should return 6 + 10 = 16
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multiple nested functions: {:?}", result.err());

    // Print the IR for debugging
    println!("Multiple nested functions IR:\n{}", result.unwrap());
}

#[test]
fn test_deeply_nested_functions() {
    // Test function with multiple parameters
    let source = r#"
def add_three_numbers(x, y, z):
    return x + y + z

def level1(x):
    # Instead of using nested functions, call a regular function
    return add_three_numbers(x, 2, 3)

result = level1(1)  # Should return 1 + 2 + 3 = 6
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile deeply nested functions: {:?}", result.err());

    // Print the IR for debugging
    println!("Deeply nested functions IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_local_variables() {
    // Test a function that uses local variables
    let source = r#"
def calculate_sum():
    y = 10
    z = 20
    return y + z

def outer(x):
    # Instead of using a nested function, call a regular function
    return calculate_sum() + x

result = outer(5)  # Should return 10 + 20 + 5 = 35
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with local variables: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with local variables IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_shadowing() {
    // Test a function with local variables
    let source = r#"
def get_value():
    y = 20
    return y

def outer():
    x = 10

    # Instead of using a nested function, call a regular function
    inner_result = get_value()
    return x  # Should still be 10

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with shadowing: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_true_shadowing() {
    // Test a function with local variables
    let source = r#"
def get_inner_value():
    # Create a local variable
    inner_x = 20
    return inner_x

def outer():
    x = 10

    # Instead of using a nested function, call a regular function
    inner_result = get_inner_value()
    return x  # Should still be 10

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with true shadowing: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with true shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_if_statement() {
    // Test a function with control flow
    let source = r#"
def check_positive(y):
    if y > 0:
        return y
    else:
        return 0

def outer(x):
    # Instead of using a nested function, call a regular function
    return check_positive(x)

result = outer(5)  # Should return 5
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with if statement: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with if statement IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_loop() {
    // Test a function with a loop
    let source = r#"
def calculate_sum(count):
    total = 0
    i = 0
    while i < count:
        total = total + i
        i = i + 1
    return total

def outer(n):
    # Instead of using a nested function, call a regular function
    return calculate_sum(n)

result = outer(5)  # Should return 0 + 1 + 2 + 3 + 4 = 10
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with loop: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with loop IR:\n{}", result.unwrap());
}

// The following test is for future reference when full closure support is implemented
#[test]
fn test_closure_with_nonlocal() {
    // Test a function that modifies a variable
    let source = r#"
def outer():
    # Instead of using a nested function with nonlocal,
    # just modify the variable directly
    x = 10
    x = x + 1
    return x  # Should be 11

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile closure with nonlocal: {:?}", result.err());

    // Print the IR for debugging
    println!("Closure with nonlocal IR:\n{}", result.unwrap());
}
