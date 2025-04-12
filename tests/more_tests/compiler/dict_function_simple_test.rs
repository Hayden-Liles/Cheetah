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
    let mut compiler = Compiler::new(&context, "dict_function_simple_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_as_function_parameter_simple() {
    let source = r#"
# Define a function that takes a dictionary as a parameter
def get_name(person):
    return person["name"]

# Test the function
person = {"name": "Alice"}
name = get_name(person)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function parameter: {:?}", result.err());
}

#[test]
fn test_dict_as_function_return_value_simple() {
    let source = r#"
# Define a function that returns a dictionary
def create_dict():
    person = {"name": "Alice"}
    return person

# Test the function
person = create_dict()
name = person["name"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function return value: {:?}", result.err());
}
