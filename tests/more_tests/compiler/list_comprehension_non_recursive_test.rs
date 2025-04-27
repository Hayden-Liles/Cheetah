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

    // Non-recursive implementations are always used

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok("Compilation successful".to_string()),
        Err(err) => Err(format!("Compilation error: {}", err)),
    }
}

#[test]
fn test_large_list_comprehension() {
    let source = r#"
# Create a large list comprehension that would cause stack overflow with recursive implementation
large_list = [x * x for x in range(100000)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile large list comprehension: {:?}", result.err());
}

#[test]
fn test_nested_list_comprehension_non_recursive() {
    let source = r#"
# Nested list comprehension that would be problematic with recursive implementation
matrix = [[i * j for j in range(100)] for i in range(100)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested list comprehension: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_complex_conditions() {
    let source = r#"
# List comprehension with multiple complex conditions
filtered = [x for x in range(10000) if x % 2 == 0 if x % 3 == 0 if x % 5 != 0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with complex conditions: {:?}", result.err());
}

#[test]
fn test_list_comprehension_with_function_calls_non_recursive() {
    let source = r#"
# Define a function
def is_special(x):
    if x % 7 == 0 and x % 11 == 0:
        return 1  # True
    else:
        return 0  # False

# List comprehension with function calls
special_numbers = [x for x in range(10000) if is_special(x)]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list comprehension with function calls: {:?}", result.err());
}
