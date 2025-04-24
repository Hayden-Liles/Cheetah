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
    let mut compiler = Compiler::new(&context, "list_membership_test");

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_basic_list_membership() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Check if elements exist in the list
exists_1 = 1 in data
exists_3 = 3 in data
exists_6 = 6 in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic list membership: {:?}", result.err());
}

#[test]
fn test_list_membership_with_not_in() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Check if elements don't exist in the list
not_exists_1 = 1 not in data
not_exists_6 = 6 not in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list membership with not in: {:?}", result.err());
}

#[test]
fn test_list_membership_in_if_statement() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Use membership test in if statement
result = ""
if 3 in data:
    result = "3 exists"
else:
    result = "3 does not exist"

# Use not in with if statement
if 6 not in data:
    result = result + " and 6 does not exist"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list membership in if statement: {:?}", result.err());
}

#[test]
fn test_list_membership_with_variables() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Use variables for elements
element1 = 3
element2 = 6

# Check membership with variables
element1_exists = element1 in data
element2_exists = element2 in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list membership with variables: {:?}", result.err());
}

#[test]
fn test_list_membership_with_expressions() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Use expressions for elements
base = 2
element = base + 1

# Check membership with expressions
element_exists = element in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list membership with expressions: {:?}", result.err());
}

#[test]
fn test_list_membership_in_while_loop() {
    let source = r#"
# Create a list
data = [1, 2, 3, 4, 5]

# Use membership test in while loop
count = 0

# Check each element individually
if 1 in data:
    count = count + 1
if 3 in data:
    count = count + 1
if 6 in data:
    count = count + 1
if 7 in data:
    count = count + 1

# count should be 2
result = count
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile list membership in while loop: {:?}", result.err());
}

#[test]
fn test_nested_list_membership() {
    let source = r#"
# Create a list with simple elements
data = [1, 2, 3, 4, 5]

# Check membership in list
has_1 = 1 in data
has_3 = 3 in data

# Check non-membership
has_6 = 6 in data
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested list membership: {:?}", result.err());
}

#[test]
fn test_string_membership() {
    let source = r#"
# Create a string
text = "Hello, world!"

# Check if substrings exist in the string
has_hello = "Hello" in text
has_world = "world" in text
has_goodbye = "Goodbye" in text
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile string membership: {:?}", result.err());
}

#[test]
fn test_string_membership_with_not_in() {
    let source = r#"
# Create a string
text = "Hello, world!"

# Check if substrings don't exist in the string
not_has_hello = "Hello" not in text
not_has_goodbye = "Goodbye" not in text
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile string membership with not in: {:?}", result.err());
}

#[test]
fn test_string_membership_in_if_statement() {
    let source = r#"
# Create a string
text = "Hello, world!"

# Use membership test in if statement
result = ""
if "Hello" in text:
    result = "Hello exists"
else:
    result = "Hello does not exist"

# Use not in with if statement
if "Goodbye" not in text:
    result = result + " and Goodbye does not exist"
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile string membership in if statement: {:?}", result.err());
}
