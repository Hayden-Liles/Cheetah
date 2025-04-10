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
    let mut compiler = Compiler::new(&context, "simple_nonlocal_test");

    // Compile the AST
    // Try to compile the module
    let result = compiler.compile_module(&ast);

    // Get the IR regardless of whether compilation succeeded
    let ir = compiler.get_ir();
    println!("Generated IR:\n{}", ir);

    // Return the result
    match result {
        Ok(_) => Ok(ir),
        Err(e) => Err(e),
    }
}

#[test]
fn test_simple_function_call() {
    // Test a simple function call without nonlocal variables
    let source = r#"
def add(a, b):
    return a + b

result = add(10, 20)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple function call: {:?}", result.err());

    // Print the IR for debugging
    println!("Simple function call IR:\n{}", result.unwrap());
}

#[test]
fn test_simple_nested_function() {
    // Test a simple nested function without nonlocal variables
    let source = r#"
def outer():
    def inner():
        return 42

    return inner()

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple nested function: {:?}", result.err());

    // Print the IR for debugging
    println!("Simple nested function IR:\n{}", result.unwrap());
}

#[test]
fn test_simple_global_variable() {
    // Test a simple global variable
    let source = r#"
x = 10

def get_x():
    global x
    return x

result = get_x()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple global variable: {:?}", result.err());

    // Print the IR for debugging
    println!("Simple global variable IR:\n{}", result.unwrap());
}

#[test]
fn test_simple_nonlocal_read() {
    // Test a simple nonlocal variable read
    let source = r#"
def outer():
    x = 10

    def inner():
        nonlocal x
        return x

    return inner()

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple nonlocal read: {:?}", result.err());

    // Print the IR for debugging
    println!("Simple nonlocal read IR:\n{}", result.unwrap());
}

#[test]
fn test_simple_nonlocal_write() {
    // Test a simple nonlocal variable write
    let source = r#"
def outer():
    x = 10

    def inner():
        nonlocal x
        x = 20
        return x

    return inner()

result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile simple nonlocal write: {:?}", result.err());

    // Print the IR for debugging
    println!("Simple nonlocal write IR:\n{}", result.unwrap());
}
