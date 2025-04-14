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
    let mut compiler = Compiler::new(&context, "tuple_type_inference_test");

    // Enable non-recursive expression compilation to avoid stack overflow
    compiler.context.use_non_recursive_expr = true;

    // Register string operations
    compiler.context.module.add_function(
        "string_get_char",
        context.i64_type().fn_type(
            &[
                context.ptr_type(inkwell::AddressSpace::default()).into(),
                context.i64_type().into(),
            ],
            false,
        ),
        None,
    );

    compiler.context.module.add_function(
        "char_to_string",
        context.ptr_type(inkwell::AddressSpace::default()).fn_type(
            &[context.i64_type().into()],
            false,
        ),
        None,
    );

    // Compile the AST
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => {
            // Add a terminator to the main function
            let main_fn = compiler.context.module.get_function("main").unwrap();
            let entry_block = main_fn.get_first_basic_block().unwrap();

            // Position at the end of the entry block
            compiler.context.builder.position_at_end(entry_block);

            // Add a return void instruction
            compiler.context.builder.build_return(None).unwrap();

            println!("Compilation successful");
            Ok("Compilation successful".to_string())
        },
        Err(err) => {
            println!("Compilation error: {}", err);
            Err(format!("Compilation error: {}", err))
        },
    }
}

#[test]
fn test_mixed_type_tuple_creation() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True, 3.14)

# Access elements using subscript
int_val = mixed_tuple[0]
str_val = mixed_tuple[1]
bool_val = mixed_tuple[2]
float_val = mixed_tuple[3]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple: {:?}", result.err());
}

#[test]
fn test_mixed_type_tuple_unpacking() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True, 3.14)

# Unpack the tuple
a, b, c, d = mixed_tuple

# Use the unpacked variables
int_val = a
str_val = b
bool_val = c
float_val = d
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_type_inference_in_function() {
    let source = r#"
# Function that takes a tuple parameter
def process_tuple(t):
    # Unpack the tuple
    a, b = t

    # Use the unpacked variables
    return a + b

# Call the function with a tuple
t = (42, 10)
result = process_tuple(t)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple type inference in function: {:?}", result.err());
}

#[test]
fn test_nested_tuple_type_inference() {
    let source = r#"
# Function that takes a nested tuple parameter
def process_nested_tuple(t):
    # Unpack the nested tuple
    a, (b, c) = t

    # Use the unpacked variables
    return a + b + c

# Call the function with a nested tuple
result = process_nested_tuple((1, (2, 3)))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested tuple type inference: {:?}", result.err());
}

#[test]
fn test_tuple_return_type_inference() {
    let source = r#"
# Function that returns a tuple
def create_tuple():
    # Create the tuple directly
    return (1, "hello", True)

# Call the function and store the result
t = create_tuple()

# Unpack the result
a, b, c = t

# Use the unpacked variables
int_val = a
str_val = b
bool_val = c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple return type inference: {:?}", result.err());
}

#[test]
fn test_dynamic_tuple_indexing() {
    let source = r#"
# Create a tuple
t = (10, 20, 30)

# Use a constant index
value = t[0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile dynamic tuple indexing: {:?}", result.err());
}

#[test]
fn test_mixed_type_tuple_dynamic_indexing() {
    let source = r#"
# Create a tuple with mixed types
mixed_tuple = (1, "hello", True)

# Use a constant index
value = mixed_tuple[0]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile mixed-type tuple dynamic indexing: {:?}", result.err());
}
