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
    let mut compiler = Compiler::new(&context, "dict_function_return_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_return_direct() {
    let source = r#"
def create_dict():
    return {"key": "value"}

d = create_dict()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile direct dictionary return: {:?}", result.err());
}

#[test]
fn test_dict_return_variable() {
    let source = r#"
def create_dict():
    d = {"key": "value"}
    return d

result = create_dict()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary variable return: {:?}", result.err());
}

#[test]
fn test_dict_return_and_access() {
    let source = r#"
def create_dict():
    return {"name": "Alice", "age": "30"}

person = create_dict()
name = person["name"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary return and access: {:?}", result.err());
}
