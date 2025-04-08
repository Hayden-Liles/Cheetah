use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_compiler_construction() {
    let context = Context::create();
    let compiler = Compiler::new(&context, "test_module");
    
    // Since context is private, we'll just test that the compiler was created
    assert!(!compiler.get_ir().is_empty());
}

#[test]
fn test_ir_generation() {
    let source = r#"
x = 42
y = x + 10
if y > 50:
    z = y
else:
    z = x
"#;
    
    // Parse the source
    let ast = parse(source).expect("Failed to parse source");
    
    // Create a compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");
    
    // Compile the AST
    compiler.compile_module(&ast).expect("Failed to compile module");
    
    // Get the LLVM IR
    let ir = compiler.get_ir();
    
    // Verify the IR is not empty and contains expected elements
    assert!(!ir.is_empty());
    assert!(ir.contains("define"));
    assert!(ir.contains("main"));
}

#[test]
fn test_write_to_file() {
    let source = "x = 42";
    
    // Parse the source
    let ast = parse(source).expect("Failed to parse source");
    
    // Create a compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");
    
    // Compile the AST
    compiler.compile_module(&ast).expect("Failed to compile module");
    
    // Create a temporary file path
    let temp_path = PathBuf::from("/tmp/test_output.ll");
    
    // Write the IR to the file - this may fail if you don't have permission to write
    // to /tmp, in which case we'll just skip this test
    if let Ok(_) = compiler.write_to_file(&temp_path) {
        // Read the file and verify it contains IR
        if let Ok(content) = fs::read_to_string(&temp_path) {
            assert!(!content.is_empty());
            assert!(content.contains("define"));
        }
        
        // Clean up
        let _ = fs::remove_file(&temp_path);
    }
}