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
    let mut compiler = Compiler::new(&context, "dict_manual_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => Err(format!("Compilation error: {}", e)),
    }
}

#[test]
fn test_dict_with_list_iteration() {
    let source = r#"
# Create a list of numbers
numbers = [1, 2, 3, 4, 5]

# Create a dictionary manually
data = {}
for x in numbers:
    data[x] = x*x

# Access some values
value_1 = data[1]
value_2 = data[2]
value_3 = data[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary with list iteration: {:?}", result.err());
}
