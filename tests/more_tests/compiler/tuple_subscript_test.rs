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
    let mut compiler = Compiler::new(&context, "tuple_subscript_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_tuple_subscript_basic() {
    let source = r#"
# Create a tuple
t = (1, 2, 3)

# Access elements using subscript
first = t[0]
second = t[1]
third = t[2]

# Verify access works correctly
sum = first + second + third
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic tuple subscript: {:?}", result.err());
}

#[test]
#[ignore = "Nested tuple indexing not fully supported yet"]
fn test_tuple_subscript_nested() {
    let source = r#"
# Create a nested tuple
t = (1, (2, 3), 4)

# Access elements using subscript
first = t[0]
nested = t[1]
third = t[2]

# For now, we need to store the nested tuple in a variable
# and then unpack it
nested_tuple = nested
a, b = nested_tuple

# Verify access works correctly
sum = first + a + b + third
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple subscript: {:?}", result.err());
}

#[test]
fn test_tuple_subscript_in_expressions() {
    let source = r#"
# Create a tuple
t = (10, 20, 30)

# Use subscript in expressions
sum = t[0] + t[1] + t[2]
product = t[0] * t[1]
difference = t[2] - t[0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple subscript in expressions: {:?}", result.err());
}

#[test]
fn test_tuple_subscript_in_function() {
    let source = r#"
# Function that uses tuple subscript
def sum_tuple(t):
    # We need to handle the tuple parameter correctly
    # The function sees t as a tuple with 2 elements
    return t[0] + t[1]

# Create a tuple and call the function
t = (1, 2)
result = sum_tuple(t)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple subscript in function: {:?}", result.err());
}

#[test]
fn test_tuple_subscript_out_of_bounds() {
    let source = r#"
# Create a tuple
t = (1, 2, 3)

# Access an out-of-bounds index
value = t[3]
"#;

    let result = compile_source(source);
    assert!(result.is_err(), "Expected error for out-of-bounds tuple subscript");
    assert!(result.unwrap_err().contains("out of range"), "Error message should mention out of range");
}

#[test]
fn test_tuple_subscript_non_constant_index() {
    let source = r#"
# Create a tuple
t = (1, 2, 3)

# Use a variable as index
i = 1
value = t[i]
"#;

    let result = compile_source(source);
    assert!(result.is_err(), "Expected error for non-constant tuple index");
    assert!(result.unwrap_err().contains("Dynamic tuple indexing"), "Error message should mention dynamic indexing");
}
