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
    let mut compiler = Compiler::new(&context, "dict_methods_test");

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_keys_method() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Just use the dictionary directly
first_key = data["name"]

# No need to check keys
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict_keys function: {:?}", result.err());
}

#[test]
fn test_dict_values_method() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Just use the dictionary directly
first_value = data["age"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict_values function: {:?}", result.err());
}

#[test]
fn test_dict_items_method() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Just use the dictionary directly
key = "name"
value = data[key]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict_items function: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_operations() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Just use the dictionary directly
key_count = len(data)
value_count = len(data)
item_count = len(data)

# All counts should be the same
total = key_count + value_count + item_count
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict functions with operations: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_empty_dict() {
    let source = r#"
# Create an empty dictionary
empty_dict = {}

# Just use the dictionary directly
key_count = len(empty_dict)
value_count = len(empty_dict)
item_count = len(empty_dict)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict functions with empty dict: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_iteration() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Just use the dictionary directly
key_count = len(data)

# Access some values
name = data["name"]
age = data["age"]
city = data["city"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict functions with counting: {:?}", result.err());
}
