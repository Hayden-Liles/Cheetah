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
fn test_optimized_range_loop_basic() {
    let source = r#"
# Basic range with one argument
for i in range(5):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic range: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_start_stop() {
    let source = r#"
# Range with start and stop
for i in range(2, 7):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with start and stop: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_start_stop_step() {
    let source = r#"
# Range with start, stop, and step
for i in range(1, 10, 2):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with start, stop, and step: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_negative_step() {
    let source = r#"
# Range with negative step
for i in range(10, 0, -1):
    x = i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range with negative step: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_variable_arguments() {
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
fn test_optimized_range_loop_nested() {
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
fn test_optimized_range_loop_with_break() {
    let source = r#"
# Range loop with break
sum = 0
for i in range(10):
    if i > 5:
        break
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range loop with break: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_with_continue() {
    let source = r#"
# Range loop with continue
sum = 0
for i in range(10):
    if i % 2 == 0:
        continue
    sum = sum + i  # Only add odd numbers
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range loop with continue: {:?}", result.err());
}

#[test]
fn test_optimized_range_loop_with_else() {
    let source = r#"
# Range loop with else clause
found = False
for i in range(10):
    if i == 20:  # This will never be true
        found = True
        break
else:
    found = False  # This should execute
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile range loop with else: {:?}", result.err());
}
