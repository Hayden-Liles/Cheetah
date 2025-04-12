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
    let mut compiler = Compiler::new(&context, "dict_function_minimal_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_as_function_parameter_minimal() {
    let source = r#"
# Define a function that takes a dictionary as a parameter
def identity(d):
    return d

# Test the function
d = {"key": "value"}
result = identity(d)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function parameter: {:?}", result.err());
}

#[test]
fn test_dict_as_function_return_value_minimal() {
    let source = r#"
# Define a function that returns a dictionary
def create_dict():
    return {"key": "value"}

# Test the function
d = create_dict()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function return value: {:?}", result.err());
}
