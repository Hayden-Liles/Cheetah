use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;

pub fn compile_source(source: &str) -> Result<String, String> {
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
        Ok(_) => Ok("Compilation successful".to_string()),
        Err(err) => Err(format!("Compilation error: {}", err)),
    }
}

#[test]
fn test_for_loop_empty_body() {
    let source = r#"
# For loop with empty body
for i in range(5):
    pass
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with empty body: {:?}", result.err());
}

#[test]
fn test_for_loop_multiple_statements() {
    let source = r#"
# For loop with multiple statements in body
sum = 0
count = 0
for i in range(10):
    sum = sum + i
    count = count + 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with multiple statements: {:?}", result.err());
}

#[test]
fn test_for_loop_conditional() {
    let source = r#"
# For loop with conditional in body
sum_even = 0
sum_odd = 0
for i in range(10):
    if i % 2 == 0:
        sum_even = sum_even + i
    else:
        sum_odd = sum_odd + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with conditional: {:?}", result.err());
}

#[test]
fn test_for_loop_early_break() {
    let source = r#"
# For loop with early break
found = False
for i in range(100):
    if i == 5:
        found = True
        break
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with early break: {:?}", result.err());
}

#[test]
fn test_for_loop_conditional_break() {
    let source = r#"
# For loop with conditional break
sum = 0
for i in range(100):
    sum = sum + i
    if sum > 50:
        break
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with conditional break: {:?}", result.err());
}

#[test]
fn test_for_loop_conditional_continue() {
    let source = r#"
# For loop with conditional continue
sum = 0
for i in range(10):
    if i % 2 == 0:
        continue
    sum = sum + i  # Only add odd numbers
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with conditional continue: {:?}", result.err());
}

#[test]
fn test_for_loop_else_no_break() {
    let source = r#"
# For loop with else clause (no break)
found = False
for i in range(5):
    if i == 10:  # This will never be true
        found = True
        break
else:
    found = False  # This will execute
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with else (no break): {:?}", result.err());
}

#[test]
fn test_for_loop_else_with_break() {
    let source = r#"
# For loop with else clause (with break)
found = False
for i in range(5):
    if i == 3:  # This will be true
        found = True
        break
else:
    found = False  # This will not execute
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with else (with break): {:?}", result.err());
}

#[test]
fn test_nested_for_loops_simple() {
    let source = r#"
# Simple nested for loops
sum = 0
for i in range(3):
    for j in range(3):
        sum = sum + (i * j)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple nested for loops: {:?}", result.err());
}

#[test]
fn test_nested_for_loops_with_break() {
    let source = r#"
# Nested for loops with break in inner loop
found = False
for i in range(5):
    for j in range(5):
        if i * j > 10:
            found = True
            break
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested for loops with break: {:?}", result.err());
}

#[test]
fn test_nested_for_loops_with_continue() {
    let source = r#"
# Nested for loops with continue in inner loop
sum = 0
for i in range(5):
    for j in range(5):
        if j % 2 == 0:
            continue
        sum = sum + (i * j)  # Only add when j is odd
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested for loops with continue: {:?}", result.err());
}

#[test]
fn test_for_loop_with_function_call() {
    let source = r#"
# Function that returns a value
def get_value():
    return 5

# For loop using function call
sum = 0
for i in range(get_value()):
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with function call: {:?}", result.err());
}

#[test]
fn test_for_loop_with_variable_update() {
    let source = r#"
# For loop that updates a variable used in the range
limit = 5
sum = 0
for i in range(limit):
    sum = sum + i
    limit = 10  # This shouldn't affect the loop range
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with variable update: {:?}", result.err());
}

#[test]
fn test_for_loop_complex_body() {
    let source = r#"
# For loop with complex body
sum = 0
for i in range(10):
    if i < 5:
        if i % 2 == 0:
            sum = sum + i
        else:
            sum = sum + (i * 2)
    else:
        if i % 2 == 0:
            sum = sum + (i * 3)
        else:
            sum = sum + (i * 4)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with complex body: {:?}", result.err());
}

#[test]
fn test_for_loop_triple_nested() {
    let source = r#"
# Triple nested for loops
sum = 0
for i in range(3):
    for j in range(3):
        for k in range(3):
            sum = sum + (i * j * k)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile triple nested for loops: {:?}", result.err());
}

#[test]
fn test_for_loop_break_outer() {
    let source = r#"
# Breaking out of outer loop from inner loop
found = False
for i in range(5):
    for j in range(5):
        if i * j > 10:
            found = True
            break
    if found:
        break
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop breaking outer loop: {:?}", result.err());
}

#[test]
fn test_for_loop_continue_outer() {
    let source = r#"
# Continuing outer loop from inner loop
sum = 0
for i in range(5):
    should_continue = False
    for j in range(5):
        if j > 2:
            should_continue = True
            break
    if should_continue:
        continue
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop continuing outer loop: {:?}", result.err());
}

#[test]
fn test_for_loop_fibonacci() {
    let source = r#"
# Calculate Fibonacci sequence using a for loop
a = 0
b = 1
for i in range(10):
    temp = a
    a = b
    b = temp + b
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop Fibonacci: {:?}", result.err());
}

#[test]
fn test_for_loop_prime_numbers() {
    let source = r#"
# Find prime numbers using nested for loops
count = 0
for n in range(2, 20):
    is_prime = True
    for i in range(2, n):
        if n % i == 0:
            is_prime = False
            break
    if is_prime:
        count = count + 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop prime numbers: {:?}", result.err());
}
