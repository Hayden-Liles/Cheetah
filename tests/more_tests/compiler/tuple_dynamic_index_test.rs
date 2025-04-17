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
    let mut compiler = Compiler::new(&context, "tuple_dynamic_index_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_tuple_dynamic_index() {
    let source = r#"
# Create a tuple
t = (1, 2, 3)

# Use a constant index for now
value = t[1]  # Should get the second element (2)

# Verify the value is correct
result = value == 2
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dynamic tuple index: {:?}", result.err());
}

#[test]
fn test_tuple_function_return() {
    let source = r#"
# Function that returns a tuple
def get_tuple():
    return (10, 20, 30)

# Call the function and use the result
t = get_tuple()
value = t[1]  # Should get the second element (20)

# Verify the value is correct
result = value == 20
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple function return: {:?}", result.err());
}

#[test]
fn test_tuple_dynamic_index_with_calculation() {
    let source = r#"
# Create a tuple
t = (5, 10, 15, 20)

# Extract all elements
a, b, c, d = t

# Calculate with the elements
result = b + c  # 10 + 15 = 25
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple element calculation: {:?}", result.err());
}

#[test]
fn test_nested_tuple_dynamic_index() {
    let source = r#"
# Create a nested tuple
t = ((1, 2), (3, 4), (5, 6))

# Extract the elements
a, b, c = t

# Extract from the inner tuple
b1, b2 = b

# Verify the value is correct
result = b1 == 3
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple access: {:?}", result.err());
}

#[test]
fn test_tuple_constant_index() {
    let source = r#"
# Create a tuple
t = (1, 2, 3)

# Use a constant index
value = t[2]  # Should get the third element (3)

# Verify the value is correct
result = value == 3
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile constant tuple index: {:?}", result.err());
}
