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
    let mut compiler = Compiler::new(&context, "dict_manual_operations_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_dict_manual_creation_with_integers() {
    let source = r#"
# Create a dictionary manually with integer keys
data = {}
data[1] = 1
data[2] = 4
data[3] = 9
data[4] = 16
data[5] = 25

# Access some values
value_1 = data[1]
value_2 = data[2]
value_3 = data[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile manual dictionary creation with integers: {:?}", result.err());
}

#[test]
fn test_dict_manual_creation_with_for_loop() {
    let source = r#"
# Create a dictionary directly
data = {1: 1, 2: 4, 3: 9, 4: 16, 5: 25}

# Access some values
value_1 = data[1]
value_2 = data[2]
value_3 = data[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile manual dictionary creation with for loop: {:?}", result.err());
}

#[test]
fn test_dict_manual_creation_with_string_keys() {
    let source = r#"
# Create a dictionary manually with string keys
data = {}
data["one"] = 1
data["two"] = 2
data["three"] = 3

# Access some values
value_1 = data["one"]
value_2 = data["two"]
value_3 = data["three"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile manual dictionary creation with string keys: {:?}", result.err());
}

#[test]
fn test_dict_update_existing_keys() {
    let source = r#"
# Create a dictionary and update existing keys
data = {}
data[1] = 10
data[2] = 20
data[3] = 30

# Update existing keys
data[1] = 100
data[2] = 200

# Access updated values
value_1 = data[1]
value_2 = data[2]
value_3 = data[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary update with existing keys: {:?}", result.err());
}
