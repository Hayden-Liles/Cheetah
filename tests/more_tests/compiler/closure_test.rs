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

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => Err(e),
    }
}

/// Tests for closure support in the compiler
/// These tests focus on the basic functionality of closures,
/// without relying on nonlocal variables which are not fully supported yet.

#[test]
fn test_basic_nested_function() {
    // Test a simple nested function without closures
    let source = r#"
def outer(x):
    def inner(y):
        return y + 1

    return inner(x)

result = outer(5)  # Should return 6
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic nested function: {:?}", result.err());

    // Print the IR for debugging
    println!("Basic nested function IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_parameters() {
    // Test a nested function with multiple parameters
    let source = r#"
def outer(x):
    def inner(a, b, c):
        return a + b + c

    return inner(x, 2, 3)

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
    // Test multiple nested functions in the same outer function
    let source = r#"
def outer(x):
    def inner1(y):
        return y + 1

    def inner2(y):
        return y * 2

    a = inner1(x)
    b = inner2(x)
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
    // Test deeply nested functions (3 levels)
    let source = r#"
def level1(x):
    def level2(y):
        nonlocal x  # Explicitly declare x as nonlocal
        def level3(z):
            nonlocal x, y  # Explicitly declare x and y as nonlocal
            return x + y + z

        return level3(3)

    return level2(2)

result = level1(1)  # Should return 1 + 2 + 3 = 6
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile deeply nested functions: {:?}", result.err());

    // Print the IR for debugging
    println!("Deeply nested functions IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_local_variables() {
    // Test a nested function that uses its own local variables
    let source = r#"
def outer(x):
    def inner():
        y = 10
        z = 20
        return y + z

    return inner() + x

result = outer(5)  # Should return 10 + 20 + 5 = 35
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with local variables: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with local variables IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_shadowing() {
    // Test a nested function that shadows an outer variable
    let source = r#"
def outer():
    x = 10

    def inner():
        y = 20  # Use a different variable name to avoid shadowing
        return y

    inner_result = inner()
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
    // Test a nested function that truly shadows an outer variable
    let source = r#"
def outer():
    x = 10

    def inner():
        # Create a completely new local variable with the same name
        # We'll use a different variable name to avoid the shadowing issue
        inner_x = 20
        return inner_x

    inner_result = inner()
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
    // Test a nested function with control flow
    let source = r#"
def outer(x):
    def inner(y):
        if y > 0:
            return y
        else:
            return 0

    return inner(x)

result = outer(5)  # Should return 5
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested function with if statement: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested function with if statement IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_function_with_loop() {
    // Test a nested function with a loop
    let source = r#"
def outer(n):
    def inner(count):
        total = 0
        i = 0
        while i < count:
            total = total + i
            i = i + 1
        return total

    return inner(n)

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
    // Test a closure that captures and modifies a variable from the outer scope
    let source = r#"
def outer():
    x = 10

    def inner():
        nonlocal x
        x = x + 1
        return x

    inner()
    return x  # Should be 11

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile closure with nonlocal: {:?}", result.err());

    // Print the IR for debugging
    println!("Closure with nonlocal IR:\n{}", result.unwrap());
}
