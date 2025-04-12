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
    let mut compiler = Compiler::new(&context, "tuple_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_empty_tuple() {
    let source = r#"
# Create an empty tuple
empty_tuple = ()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile empty tuple: {:?}", result.err());
}

#[test]
fn test_single_element_tuple() {
    let source = r#"
# Create a single-element tuple
single_tuple = (42,)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile single-element tuple: {:?}", result.err());
}

#[test]
fn test_multi_element_tuple() {
    let source = r#"
# Create a multi-element tuple
multi_tuple = (1, 2, 3, 4, 5)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multi-element tuple: {:?}", result.err());
}

#[test]
fn test_tuple_with_different_types() {
    let source = r#"
# Create a tuple with different types
mixed_tuple = (1, "hello", True, 3.14)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple: {:?}", result.err());
}

#[test]
fn test_nested_tuples() {
    let source = r#"
# Create nested tuples
nested_tuple = (1, (2, 3), (4, (5, 6)))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuples: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking() {
    let source = r#"
# Create a tuple and unpack it
tuple_value = (1, 2, 3)
a, b, c = tuple_value
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple unpacking: {:?}", result.err());
}

#[test]
fn test_nested_tuple_unpacking() {
    let source = r#"
# Create a nested tuple and unpack it
tuple_value = (1, (2, 3))
a, (b, c) = tuple_value
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking_with_assignment() {
    let source = r#"
# Create variables first to avoid type checking issues
a = 0
b = 0
c = 0
# Then unpack a tuple into them
a, b, c = (1, 2, 3)
# Use the variables
sum = a + b + c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple unpacking with assignment: {:?}", result.err());
}

#[test]
fn test_direct_tuple_unpacking() {
    let source = r#"
# Directly create variables from tuple unpacking
a, b, c = (1, 2, 3)
# Use the variables
sum = a + b + c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile direct tuple unpacking: {:?}", result.err());
}

#[test]
fn test_nested_direct_tuple_unpacking() {
    let source = r#"
# Directly create variables from nested tuple unpacking
a, (b, c) = (1, (2, 3))
# Use the variables
sum = a + b + c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested direct tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking_in_function() {
    let source = r#"
def unpack_tuple(t):
    a, b, c = t
    return a + b + c

result = unpack_tuple((1, 2, 3))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple unpacking in function: {:?}", result.err());
}

#[test]
fn test_nested_tuple_unpacking_in_function() {
    let source = r#"
def process_nested_tuple(t):
    a, (b, c) = t
    return a * (b + c)

result = process_nested_tuple((5, (2, 3)))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple unpacking in function: {:?}", result.err());
}

#[test]
fn test_multiple_tuple_parameters() {
    let source = r#"
def process_tuples(t1, t2):
    a, b = t1
    c, d = t2
    return a + b + c + d

result = process_tuples((1, 2), (3, 4))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multiple tuple parameters: {:?}", result.err());
}

#[test]
fn test_tuple_as_function_argument() {
    let source = r#"
def process_tuple(t):
    return t

result = process_tuple((1, 2, 3))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple as function argument: {:?}", result.err());
}

#[test]
fn test_tuple_as_function_return_value() {
    let source = r#"
def create_tuple():
    return (1, 2, 3)

result = create_tuple()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple as function return value: {:?}", result.err());
}
