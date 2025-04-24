use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;

fn compile_source(source: &str) -> Result<String, String> {
    // Parse the source
    let ast = parse(source).map_err(|errors| {
        format!("Parse errors: {:?}", errors)
    })?;

    // Create a compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");

    // Compile the AST without type checking
    compiler.compile_module_without_type_checking(&ast)?;

    // Return the LLVM IR
    Ok(compiler.get_ir())
}

#[test]
fn test_simple_arithmetic() {
    let source = r#"
a = 40  # Use variables instead of direct constants
b = 2
x = a + b  # This should generate an add instruction
y = x * b
z = y // b
"#;

    let result = compile_source(source);
    assert!(result.is_ok());

    let ir = result.unwrap();
    println!("Generated IR:\n{}", ir);

    // Check for BoxedAny operations
    let contains_add = ir.contains("boxed_any_add");

    assert!(contains_add, "IR doesn't contain boxed_any_add instruction");
    assert!(ir.contains("boxed_any_multiply"),
            "IR doesn't contain boxed_any_multiply instruction");
    assert!(ir.contains("boxed_any_floor_div"),
            "IR doesn't contain boxed_any_floor_div instruction");
}

#[test]
fn test_if_condition() {
    let source = r#"
x = 42
y = 0
if x > 40:
    y = 1
"#;

    let result = compile_source(source);
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok());

    let ir = result.unwrap();

    // Verify the IR contains expected elements
    assert!(ir.contains("boxed_any_greater_than"));
    assert!(ir.contains("br"));
}

#[test]
fn test_variable_assignment() {
    let source = r#"
x = 42  # Integer
y = 3.14  # Float
z = True  # Boolean
w = None  # None value
"#;

    let result = compile_source(source);
    assert!(result.is_ok());

    let ir = result.unwrap();

    // Verify the IR contains expected elements
    assert!(ir.contains("store"));
    assert!(ir.contains("boxed_any_from_int"));  // Integer type
    assert!(ir.contains("boxed_any_from_float"));  // Float type
    assert!(ir.contains("boxed_any_from_bool"));   // Boolean type
    assert!(ir.contains("boxed_any_none"));   // None type
}