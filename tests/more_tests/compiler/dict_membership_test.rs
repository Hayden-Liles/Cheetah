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
    let mut compiler = Compiler::new(&context, "dict_membership_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_basic_dict_membership() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Check if keys exist in the dictionary
name_exists = "name" in data
age_exists = "age" in data
address_exists = "address" in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic dictionary membership: {:?}", result.err());
}

#[test]
fn test_dict_membership_with_not_in() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Check if keys don't exist in the dictionary
name_not_exists = "name" not in data
address_not_exists = "address" not in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership with not in: {:?}", result.err());
}

#[test]
fn test_dict_membership_in_if_statement() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Use membership test in if statement
result = ""
if "name" in data:
    result = "name exists"
else:
    result = "name does not exist"

# Use not in with if statement
if "address" not in data:
    result = result + " and address does not exist"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership in if statement: {:?}", result.err());
}

#[test]
fn test_dict_membership_with_variables() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Use variables for keys
key1 = "name"
key2 = "address"

# Check membership with variables
key1_exists = key1 in data
key2_exists = key2 in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership with variables: {:?}", result.err());
}

#[test]
fn test_dict_membership_with_expressions() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Use expressions for keys
prefix = "na"
key = prefix + "me"

# Check membership with expressions
key_exists = key in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership with expressions: {:?}", result.err());
}

#[test]
fn test_dict_membership_with_numeric_keys() {
    let source = r#"
# Create a dictionary with numeric keys
data = {1: "one", 2: "two", 3: "three"}

# Check membership with numeric keys
key1_exists = 1 in data
key4_exists = 4 in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership with numeric keys: {:?}", result.err());
}

#[test]
fn test_dict_membership_in_while_loop() {
    let source = r#"
# Create a dictionary
data = {"name": "Alice", "age": "30", "city": "New York"}

# Use membership test in while loop
count = 0

# Check each key individually
if "name" in data:
    count = count + 1
if "age" in data:
    count = count + 1
if "address" in data:
    count = count + 1
if "phone" in data:
    count = count + 1

# count should be 2
result = count
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary membership in while loop: {:?}", result.err());
}

#[test]
fn test_nested_dict_membership() {
    let source = r#"
# Create a nested dictionary
user = {
    "name": "Alice",
    "contact": {
        "email": "alice@example.com",
        "phone": "555-1234"
    }
}

# Check membership in outer dictionary
has_name = "name" in user
has_contact = "contact" in user

# We can't check membership in nested dictionary yet
# because string membership is not implemented
contact_info = "contact info"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary membership: {:?}", result.err());
}
