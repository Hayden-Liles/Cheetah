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
fn test_basic_for_loop() {
    let source = r#"
# Basic for loop with range
sum = 0
for i in range(5):
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic for loop: {:?}", result.err());
}

#[test]
fn test_for_loop_with_break() {
    let source = r#"
# For loop with break
sum = 0
for i in range(10):
    # Simple break without condition
    break
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with break: {:?}", result.err());
}

#[test]
fn test_for_loop_with_continue() {
    let source = r#"
# For loop with continue
sum = 0
for i in range(10):
    # Simple continue without condition
    continue
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with continue: {:?}", result.err());
}

#[test]
fn test_for_loop_with_else() {
    let source = r#"
# For loop with else
found = False
for i in range(10):
    found = True
else:
    found = False
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile for loop with else: {:?}", result.err());
}

#[test]
fn test_nested_for_loops() {
    let source = r#"
# Nested for loops
sum = 0
for i in range(5):
    for j in range(5):
        sum = sum + i * j
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested for loops: {:?}", result.err());
}
