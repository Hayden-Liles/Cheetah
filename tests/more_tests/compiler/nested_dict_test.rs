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
    let mut compiler = Compiler::new(&context, "nested_dict_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_nested_dict_creation() {
    let source = r#"
# Create a nested dictionary
data = {
    "user": {
        "name": "Alice",
        "age": "30"
    }
}
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary creation: {:?}", result.err());
}

#[test]
fn test_nested_dict_access() {
    let source = r#"
# Create a nested dictionary
data = {
    "user": {
        "name": "Alice",
        "age": "30"
    }
}

# Access nested dictionary values
user_dict = data["user"]
name = user_dict["name"]
age = user_dict["age"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary access: {:?}", result.err());
}

#[test]
fn test_nested_dict_modification() {
    let source = r#"
# Create a nested dictionary
data = {
    "user": {
        "name": "Alice",
        "age": "30"
    }
}

# Modify nested dictionary values
user_dict = data["user"]
user_dict["name"] = "Bob"
user_dict["age"] = "25"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary modification: {:?}", result.err());
}

#[test]
fn test_nested_dict_with_variables() {
    let source = r#"
# Create a nested dictionary
data = {
    "user": {
        "name": "Alice",
        "age": "30"
    }
}

# Access nested dictionary with variables
user_key = "user"
name_key = "name"

user_dict = data[user_key]
name = user_dict[name_key]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested dictionary with variables: {:?}", result.err());
}

#[test]
fn test_deeply_nested_dict() {
    let source = r#"
# Create a deeply nested dictionary
data = {
    "level1": {
        "level2": "value"
    }
}

# Access deeply nested value
level1 = data["level1"]
value = level1["level2"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile deeply nested dictionary: {:?}", result.err());
}
