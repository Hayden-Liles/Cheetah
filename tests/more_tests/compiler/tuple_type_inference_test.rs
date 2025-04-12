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
    let mut compiler = Compiler::new(&context, "tuple_type_inference_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_mixed_type_tuple_creation() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True, 3.14)

# Access elements using subscript
int_val = mixed_tuple[0]
str_val = mixed_tuple[1]
bool_val = mixed_tuple[2]
float_val = mixed_tuple[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple: {:?}", result.err());
}

#[test]
fn test_mixed_type_tuple_unpacking() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True, 3.14)

# Unpack the tuple
a, b, c, d = mixed_tuple

# Use the unpacked variables
int_val = a
str_val = b
bool_val = c
float_val = d
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_type_inference_in_function() {
    let source = r#"
# Function that takes a tuple parameter
def process_tuple(t):
    # Unpack the tuple
    a, b = t

    # Use the unpacked variables
    return a + b

# Call the function with a tuple
t = (42, 10)
result = process_tuple(t)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple type inference in function: {:?}", result.err());
}

#[test]
fn test_nested_tuple_type_inference() {
    let source = r#"
# Function that takes a nested tuple parameter
def process_nested_tuple(t):
    # Unpack the nested tuple
    a, (b, c) = t

    # Use the unpacked variables
    return a + b + c

# Call the function with a nested tuple
result = process_nested_tuple((1, (2, 3)))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple type inference: {:?}", result.err());
}

#[test]
#[ignore = "Function return type inference for tuples not fully implemented yet"]
fn test_tuple_return_type_inference() {
    let source = r#"
# Function that returns a tuple
def create_tuple():
    # Create the tuple directly
    t = (1, "hello", True)
    return t

# Call the function and store the result
t = create_tuple()

# Unpack the result
a, b, c = t

# Use the unpacked variables
int_val = a
str_val = b
bool_val = c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple return type inference: {:?}", result.err());
}

#[test]
fn test_dynamic_tuple_indexing() {
    let source = r#"
# Create a tuple
t = (10, 20, 30, 40, 50)

# Use a variable as index
i = 0
value = t[i]
i = 1
value = t[i]
i = 2
value = t[i]
i = 3
value = t[i]
i = 4
value = t[i]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dynamic tuple indexing: {:?}", result.err());
}

#[test]
fn test_mixed_type_tuple_dynamic_indexing() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True, 3.14)

# Use a variable as index
i = 0
value = mixed_tuple[i]
i = 1
value = mixed_tuple[i]
i = 2
value = mixed_tuple[i]
i = 3
value = mixed_tuple[i]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple dynamic indexing: {:?}", result.err());
}
