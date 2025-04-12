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

    // Create a new LLVM context and compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok("Compilation successful".to_string()),
        Err(err) => Err(format!("Compilation error: {}", err)),
    }
}

#[test]
fn test_basic_list_comprehension() {
    let source = r#"
# Basic list comprehension
squares = [x * x for x in range(10)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic list comprehension: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_condition() {
    let source = r#"
# List comprehension with condition
even_squares = [x * x for x in range(10) if x % 2 == 0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with condition: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_multiple_conditions() {
    let source = r#"
# List comprehension with multiple conditions
special_numbers = [x for x in range(20) if x % 2 == 0 if x % 3 == 0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with multiple conditions: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_complex_expression() {
    let source = r#"
# List comprehension with complex expression
expressions = [x * x + 2 * x - 1 for x in range(5)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with complex expression: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_function_call() {
    let source = r#"
# List comprehension with function call
def square(x):
    return x * x

squared = [square(x) for x in range(5)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with function call: {:?}", result.err());
}

#[test]
#[ignore = "Nested list comprehension not fully implemented yet"]
fn test_nested_list_comprehension() {
    let source = r#"
# Nested list comprehension
matrix = [[i * j for j in range(3)] for i in range(3)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested list comprehension: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_string() {
    let source = r#"
# List comprehension with string
text = "Hello"
chars = [c for c in text]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with string: {:?}", result.err());
}

#[test]
#[ignore = "List comprehension with list not fully implemented yet"]
fn test_list_comprehension_with_list() {
    let source = r#"
# List comprehension with list
numbers = [1, 2, 3, 4, 5]
doubled = [x * 2 for x in numbers]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with list: {:?}", result.err());
}
