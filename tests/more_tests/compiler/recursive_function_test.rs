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
fn test_recursive_factorial() {
    let source = r#"
# Recursive factorial implementation
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

# Call the factorial function
result = factorial(5)  # Should be 120
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile recursive factorial: {:?}", result.err());

    // Print the IR for debugging
    println!("Recursive factorial IR:\n{}", result.unwrap());
}

#[test]
fn test_recursive_factorial_with_wrapper() {
    let source = r#"
# Recursive factorial with a wrapper function
def factorial_impl(n):
    if n <= 1:
        return 1
    else:
        return n * factorial_impl(n - 1)

def factorial(n):
    # Wrapper function that calls the implementation
    return factorial_impl(n)

# Call the factorial function through the wrapper
result = factorial(6)  # Should be 720
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile recursive factorial with wrapper: {:?}", result.err());

    // Print the IR for debugging
    println!("Recursive factorial with wrapper IR:\n{}", result.unwrap());
}

#[test]
fn test_recursive_fibonacci() {
    let source = r#"
# Recursive fibonacci implementation
def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)

# Call the fibonacci function
result = fibonacci(6)  # Should be 8
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile recursive fibonacci: {:?}", result.err());

    // Print the IR for debugging
    println!("Recursive fibonacci IR:\n{}", result.unwrap());
}

#[test]
fn test_mutual_recursion() {
    let source = r#"
# Mutual recursion example
def is_even(n):
    if n == 0:
        return 1  # True
    else:
        return is_odd(n - 1)

def is_odd(n):
    if n == 0:
        return 0  # False
    else:
        return is_even(n - 1)

# Call the functions
result_even = is_even(4)  # Should be 1 (True)
result_odd = is_odd(3)    # Should be 1 (True)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mutual recursion: {:?}", result.err());

    // Print the IR for debugging
    println!("Mutual recursion IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_recursive_calls() {
    let source = r#"
# Nested recursive calls
def process(n):
    return n + 1

def nested_recursion(n):
    if n <= 0:
        return 0

    # Process current value
    current = process(n)

    # Recursive call
    next_value = nested_recursion(n - 1)

    return current + next_value

# Call the nested recursion function
result = nested_recursion(3)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested recursive calls: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested recursive calls IR:\n{}", result.unwrap());
}

#[test]
fn test_recursive_function_with_two_parameters() {
    let source = r#"
# Simple recursive function with two parameters
def sum_to(a, b):
    if a > b:
        return 0
    return a + sum_to(a + 1, b)

# Call the function
result = sum_to(1, 5)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile recursive function with two parameters: {:?}", result.err());

    // Print the IR for debugging
    println!("Recursive function with two parameters IR:\n{}", result.unwrap());
}
