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
fn test_range_basic() {
    let source = r#"
# Basic range with one argument
for i in range(5):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic range: {:?}", result.err());
}

#[test]
fn test_range_start_stop() {
    let source = r#"
# Range with start and stop
for i in range(2, 7):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with start and stop: {:?}", result.err());
}

#[test]
fn test_range_start_stop_step() {
    let source = r#"
# Range with start, stop, and step
for i in range(1, 10, 2):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with start, stop, and step: {:?}", result.err());
}

#[test]
fn test_range_negative_step() {
    let source = r#"
# Range with negative step
for i in range(10, 0, -1):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with negative step: {:?}", result.err());
}

#[test]
fn test_range_variable_arguments() {
    let source = r#"
# Range with variable arguments
start = 1
stop = 10
step = 2
for i in range(start, stop, step):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with variable arguments: {:?}", result.err());
}

#[test]
fn test_range_expression_arguments() {
    let source = r#"
# Range with expression arguments
a = 1
b = 5
c = a + 1
d = b * 2
for i in range(c, d):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with expression arguments: {:?}", result.err());
}

#[test]
fn test_range_empty() {
    let source = r#"
# Empty range (start >= stop with positive step)
for i in range(5, 5):
    x = i  # This should not execute
else:
    y = 10  # This should execute
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile empty range: {:?}", result.err());
}

#[test]
fn test_range_nested() {
    let source = r#"
# Nested ranges
for i in range(3):
    for j in range(3):
        x = i * j
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested ranges: {:?}", result.err());
}

#[test]
fn test_range_sum() {
    let source = r#"
# Calculate sum using range
sum = 0
for i in range(1, 6):  # 1 + 2 + 3 + 4 + 5 = 15
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range sum: {:?}", result.err());
}

#[test]
fn test_range_factorial() {
    let source = r#"
# Calculate factorial using range
n = 5
factorial = 1
for i in range(1, 6):  # 5! = 1 * 2 * 3 * 4 * 5 = 120
    factorial = factorial * i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range factorial: {:?}", result.err());
}
