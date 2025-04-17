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
    let mut compiler = Compiler::new(&context, "tuple_operations_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_tuple_creation() {
    let source = r#"
# Create a simple tuple
t = (1, 2, 3)

# Create a nested tuple
nested = ((1, 2), (3, 4), (5, 6))

# Create a tuple with mixed types
mixed = (1, "hello", True)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple creation: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking() {
    let source = r#"
# Create a tuple
t = (10, 20, 30)

# Unpack the tuple
a, b, c = t

# Verify the values
result = a == 10 and b == 20 and c == 30
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_constant_indexing() {
    let source = r#"
# Create a tuple
t = (5, 10, 15, 20)

# Access elements with constant indices
first = t[0]
second = t[1]
third = t[2]
fourth = t[3]

# Verify the values
result = first == 5 and second == 10 and third == 15 and fourth == 20
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple constant indexing: {:?}", result.err());
}

#[test]
fn test_nested_tuple_access() {
    let source = r#"
# Create a nested tuple
t = ((1, 2), (3, 4), (5, 6))

# Access elements
first_tuple = t[0]
second_tuple = t[1]
third_tuple = t[2]

# Access nested elements
first_first = first_tuple[0]
first_second = first_tuple[1]
second_first = second_tuple[0]
second_second = second_tuple[1]

# Verify the values
result = first_first == 1 and first_second == 2 and second_first == 3 and second_second == 4
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple access: {:?}", result.err());
}

#[test]
fn test_tuple_function_return() {
    let source = r#"
# Function that returns a tuple
def get_tuple():
    return (10, 20, 30)

# Call the function and use the result
t = get_tuple()
first = t[0]
second = t[1]
third = t[2]

# Verify the values
result = first == 10 and second == 20 and third == 30
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple function return: {:?}", result.err());
}

#[test]
fn test_tuple_function_argument() {
    let source = r#"
# Function that takes a tuple and returns an element
def get_second_element(t):
    return t[1]

# Create a tuple and call the function
t = (10, 20, 30)
second = get_second_element(t)

# Verify the value
result = second == 20
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple function argument: {:?}", result.err());
}

#[test]
fn test_tuple_nested_function() {
    let source = r#"
# Function that returns a nested tuple
def get_nested_tuple():
    return ((1, 2), (3, 4))

# Function that takes a tuple and returns a nested element
def get_nested_element(t):
    inner = t[1]
    return inner[0]

# Call the functions
t = get_nested_tuple()
value = get_nested_element(t)

# Verify the value
result = value == 3
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple nested function: {:?}", result.err());
}
