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
    
    // Compile the AST
    compiler.compile_module(&ast)?;
    
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
    
    // Check for various ways the add instruction might appear
    let contains_add = ir.contains("add") || ir.contains("add ") || 
                       ir.contains("fadd") || ir.contains("add i64");
    
    assert!(contains_add, "IR doesn't contain addition instruction");
    assert!(ir.contains("mul") || ir.contains("mul ") || 
            ir.contains("fmul") || ir.contains("mul i64"), 
            "IR doesn't contain multiplication instruction");
}

#[test]
fn test_if_condition() {
    let source = r#"
x = 42
if x > 40:
    y = 1
else:
    y = 0
"#;
    
    let result = compile_source(source);
    assert!(result.is_ok());
    
    let ir = result.unwrap();
    
    // Verify the IR contains expected elements
    assert!(ir.contains("icmp"));
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
    assert!(ir.contains("i64"));  // Integer type
    assert!(ir.contains("double"));  // Float type
    assert!(ir.contains("i1"));   // Boolean type
}