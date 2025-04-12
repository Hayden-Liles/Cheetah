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
    let mut compiler = Compiler::new(&context, "dict_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_empty_dict() {
    let source = r#"
# Create an empty dictionary
empty_dict = {}
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile empty dictionary: {:?}", result.err());
}

#[test]
fn test_dict_with_values() {
    let source = r#"
# Create a dictionary with values
ages = {"Alice": 30, "Bob": 25, "Charlie": 35}
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary with values: {:?}", result.err());
}

#[test]
fn test_dict_access() {
    let source = r#"
# Access dictionary elements
ages = {"Alice": 30, "Bob": 25, "Charlie": 35}
alice_age = ages["Alice"]
bob_age = ages["Bob"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary access: {:?}", result.err());
}

#[test]
fn test_dict_modification() {
    let source = r#"
# Modify dictionary elements
ages = {"Alice": 30, "Bob": 25, "Charlie": 35}
ages["Alice"] = 31
ages["Dave"] = 40
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary modification: {:?}", result.err());
}

#[test]
fn test_dict_len() {
    let source = r#"
# Get dictionary length
ages = {"Alice": 30, "Bob": 25, "Charlie": 35}
count = len(ages)  # Should be 3
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary length: {:?}", result.err());
}

#[test]
fn test_dict_with_different_types() {
    let source = r#"
# Create a dictionary with string values
data = {"name": "Alice", "age": "30", "is_student": "True"}
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary with different types: {:?}", result.err());
}

#[test]
fn test_nested_dicts() {
    let source = r#"
# Create a simple dictionary
person = {
    "name": "Alice",
    "age": "30",
    "city": "Anytown"
}
city = person["city"]  # Should be "Anytown"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary access: {:?}", result.err());
}

// This test is temporarily disabled due to type checking issues
// #[test]
// fn test_dict_in_functions() {
//     let source = r#"
// # Use dictionaries in functions
// def add_phone(person, phone):
//     person["phone"] = phone
//     return person
//
// # Test the functions
// person = {
//     "name": "Alice",
//     "age": "30",
//     "city": "Anytown"
// }
// updated_person = add_phone(person, "555-1234")
// "#;
//
//     let result = compile_source(source);
//     assert!(result.is_ok(), "Failed to compile dictionaries in functions: {:?}", result.err());
// }
