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

    // Compile the AST
    match compiler.compile_module(&ast) {
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

# Get the keys as a list
keys = data.keys()

# Check the keys
first_key = keys[0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict.keys() method: {:?}", result.err());
}

#[test]
fn test_dict_values_method() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Get the values as a list
values = data.values()

# Check the values
first_value = values[0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict.values() method: {:?}", result.err());
}

#[test]
fn test_dict_items_method() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Get the items as a list of tuples
items = data.items()

# Check the items
first_item = items[0]
key = first_item[0]
value = first_item[1]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict.items() method: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_operations() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Get the keys and check the length
keys = data.keys()
key_count = len(keys)

# Get the values and check the length
values = data.values()
value_count = len(values)

# Get the items and check the length
items = data.items()
item_count = len(items)

# All counts should be the same
total = key_count + value_count + item_count
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict methods with operations: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_empty_dict() {
    let source = r#"
# Create an empty dictionary
empty_dict = {}

# Get the keys, values, and items
keys = empty_dict.keys()
values = empty_dict.values()
items = empty_dict.items()

# Check the lengths
key_count = len(keys)
value_count = len(values)
item_count = len(items)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict methods with empty dict: {:?}", result.err());
}

#[test]
fn test_dict_methods_with_iteration() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Iterate over keys
keys = data.keys()
key_count = 0
for key in keys:
    key_count = key_count + 1

# Iterate over values
values = data.values()
value_count = 0
for value in values:
    value_count = value_count + 1

# Iterate over items
items = data.items()
item_count = 0
for item in items:
    item_count = item_count + 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dict methods with iteration: {:?}", result.err());
}
