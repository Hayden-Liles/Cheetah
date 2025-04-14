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
    let mut compiler = Compiler::new(&context, "dict_comprehension_test");

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_basic_dict_comprehension() {
    let source = r#"
# Basic dictionary comprehension
squares = {x: x*x for x in range(5)}

# Access some values
value_0 = squares[0]
value_1 = squares[1]
value_2 = squares[2]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile basic dictionary comprehension: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_condition() {
    let source = r#"
# Dictionary comprehension with condition
even_squares = {x: x*x for x in range(10) if x % 2 == 0}

# Access some values
value_0 = even_squares[0]
value_2 = even_squares[2]
value_4 = even_squares[4]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with condition: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_complex_expressions() {
    let source = r#"
# Dictionary comprehension with complex expressions
data = {str(x): x*x + 1 for x in range(5)}

# Access some values using string keys
value_0 = data["0"]
value_1 = data["1"]
value_2 = data["2"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with complex expressions: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_list_iteration() {
    let source = r#"
# Create a list of numbers
numbers = [1, 2, 3, 4, 5]

# Create a dictionary manually
data = {}
for x in numbers:
    data[x] = x*x

# Access some values
value_1 = data[1]
value_2 = data[2]
value_3 = data[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with list iteration: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_range_iteration() {
    let source = r#"
# Dictionary comprehension with range iteration
squares = {i: i*i for i in range(5)}

# Access some values
value_0 = squares[0]
value_1 = squares[1]
value_2 = squares[2]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with range iteration: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_string_keys() {
    let source = r#"
# Dictionary comprehension with string keys
data = {"key_" + str(x): x for x in range(5)}

# Access some values
value_0 = data["key_0"]
value_1 = data["key_1"]
value_2 = data["key_2"]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with string keys: {:?}", result.err());
}

#[test]
fn test_dict_comprehension_with_tuple_values() {
    let source = r#"
# Dictionary comprehension with tuple values
data = {x: (x, x*x) for x in range(5)}

# Access some values and their components
pair_0 = data[0]
first_0 = pair_0[0]
second_0 = pair_0[1]

pair_1 = data[1]
first_1 = pair_1[0]
second_1 = pair_1[1]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dictionary comprehension with tuple values: {:?}", result.err());
}
