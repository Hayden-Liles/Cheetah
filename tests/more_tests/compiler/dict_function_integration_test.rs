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
    let mut compiler = Compiler::new(&context, "dict_function_integration_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_as_function_parameter() {
    let source = r#"
# Define a function that takes a dictionary as a parameter
def get_dict_value(data, key):
    return data[key]

# Test the function
data = {"name": "Alice", "age": "30", "city": "New York"}
name = get_dict_value(data, "name")
age = get_dict_value(data, "age")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function parameter: {:?}", result.err());
}

#[test]
fn test_dict_as_function_return_value() {
    let source = r#"
# Define a function that returns a dictionary
def create_dict():
    data = {"key": "value"}
    return data

# Test the function
result = create_dict()
value = result["key"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary as function return value: {:?}", result.err());
}

#[test]
fn test_dict_modification_in_function() {
    let source = r#"
# Define a function that modifies a dictionary
def add_phone(person, phone):
    person["phone"] = phone
    return person

# Test the function
person = {
    "name": "Alice",
    "age": "30",
    "city": "New York"
}
updated_person = add_phone(person, "555-1234")
phone = updated_person["phone"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary modification in function: {:?}", result.err());
}

#[test]
fn test_dict_creation_in_function() {
    let source = r#"
# Define a function that creates a dictionary
def create_simple_dict():
    result = {"key": "value"}
    return result

# Test the function
person = create_simple_dict()
value = person["key"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary creation in function: {:?}", result.err());
}

#[test]
fn test_nested_dict_functions() {
    let source = r#"
# Simple dictionary access
data = {"user": "Alice"}
name = data["user"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary functions: {:?}", result.err());
}

#[test]
fn test_dict_with_function_calls_as_values() {
    let source = r#"
# Define functions to use as values
def square(x):
    return x * x

def cube(x):
    return x * x * x

# Create a dictionary with function results as values
def create_math_dict(x):
    return {
        "square": square(x),
        "cube": cube(x),
        "double": x * 2
    }

# Test the function
math_dict = create_math_dict(5)
square_value = math_dict["square"]
cube_value = math_dict["cube"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary with function calls as values: {:?}", result.err());
}

#[test]
fn test_dict_methods_in_functions() {
    let source = r#"
# Define a function that uses dictionary directly
def get_key(data, key):
    return data[key]

# Test the function
data = {"name": "Alice", "age": "30"}
name = get_key(data, "name")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary methods in functions: {:?}", result.err());
}

#[test]
fn test_dict_in_function_with_if_statement() {
    let source = r#"
# Define a function that uses a dictionary with if statements
def has_key(data, key):
    if key in data:
        return 1
    else:
        return 0

# Test the function
data = {"name": "Alice", "age": "30"}
result = has_key(data, "name")
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary in function with if statement: {:?}", result.err());
}
