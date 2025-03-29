// tests/compiler_tests.rs
use cheetah::compiler::Compiler;
use cheetah::parse;
use inkwell::context::Context;

// Helper function to compile source code to IR and return the IR string
fn compile_to_ir(source: &str, module_name: &str) -> Result<String, String> {
    // Parse the source code
    let ast = match parse(source) {
        Ok(module) => module,
        Err(errors) => {
            let error_messages = errors.iter()
                .map(|e| e.get_message())
                .collect::<Vec<String>>()
                .join("\n");
            return Err(format!("Parsing failed: {}", error_messages));
        }
    };
    
    // Create LLVM context and compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, module_name);
    
    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => Err(format!("Compilation failed: {}", e)),
    }
}

// Print the IR for debugging
fn debug_ir(ir: &str) {
    println!("Generated IR:\n{}", ir);
}

#[test]
fn test_annotated_assignment() {
    let source = r#"
def func_with_ann_assignment():
    x: int = 42
    return x
    "#;
    
    let ir = compile_to_ir(source, "ann_assignment_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for alloca and store instructions for the variable
    assert!(ir.contains("alloca"), "Should contain 'alloca' instruction");
    assert!(ir.contains("store"), "Should contain 'store' instruction for assignment");
}

#[test]
fn test_augmented_assignment() {
    let source = r#"
def func_with_aug_assignment():
    x = 10
    x += 5
    y = 20
    y *= 2
    return x + y
    "#;
    
    let ir = compile_to_ir(source, "aug_assignment_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for load, add/mul, and store operations
    assert!(ir.contains("load"), "Should contain 'load' instructions");
    assert!(ir.contains("add") || ir.contains("fadd"), "Should contain addition operations");
    assert!(ir.contains("mul") || ir.contains("fmul"), "Should contain multiplication operations");
}