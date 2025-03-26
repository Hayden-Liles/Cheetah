use cheetah::compiler::Compiler;
use cheetah::parse;
use inkwell::context::Context;

#[test]
fn test_basic_ir_generation() {
    let source = "def main(): pass";
    
    // Parse the source code
    let ast = match parse(source) {
        Ok(module) => module,
        Err(errors) => {
            for error in &errors {
                println!("Error: {}", error.get_message());
            }
            panic!("Failed to parse source code");
        }
    };
    
    // Create LLVM context and compiler
    let context = Context::create();
    let compiler = Compiler::new(&context, "test_module");
    
    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => {
            // Print the generated IR for inspection
            let ir = compiler.get_ir();
            println!("Generated LLVM IR:\n{}", ir);
            
            // Simple verification that IR was generated
            assert!(ir.contains("define"), "Generated IR should contain function definitions");
        },
        Err(e) => panic!("Compilation failed: {}", e),
    }
}