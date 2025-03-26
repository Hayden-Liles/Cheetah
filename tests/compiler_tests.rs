// tests/minimal_compiler_tests.rs
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
    let compiler = Compiler::new(&context, module_name);
    
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

// Stage 1: Basic Function Tests
// These tests verify the basic structure of the IR generation

#[test]
fn test_empty_function() {
    let source = "def empty_func(): pass";
    let ir = compile_to_ir(source, "empty_func_module").expect("Compilation failed");
    
    // Debug output to see what's being generated
    debug_ir(&ir);
    
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
    assert!(ir.contains("main"), "Generated IR should contain 'main' function");
    // More specific assertions can be added as compiler features develop
}

#[test]
fn test_simple_module_structure() {
    let source = "def main(): pass";
    let ir = compile_to_ir(source, "simple_module").expect("Compilation failed");
    
    debug_ir(&ir);
    
    // Very basic validation - just checking some expected LLVM structure
    assert!(ir.contains("source_filename"), "Should include module metadata");
    assert!(ir.contains("define"), "Should contain function definitions");
}

// Stage 2: Class Structure Tests
// These tests verify the class hierarchy generation

#[test]
fn test_simple_class() {
    let source = r#"
class SimpleClass:
    pass
    "#;
    
    let ir = compile_to_ir(source, "simple_class_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Simple verification - just check that it compiles for now
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
}

#[test]
fn test_class_with_method() {
    let source = r#"
class ClassWithMethod:
    def method(self):
        pass
    "#;
    
    let ir = compile_to_ir(source, "class_method_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Simple verification - just check that it compiles for now
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
}

// Stage 3: Basic Statement Tests
// These test individual statement types

#[test]
fn test_pass_statement() {
    let source = r#"
def func_with_pass():
    pass
    "#;
    
    let ir = compile_to_ir(source, "pass_module").expect("Compilation failed");
    debug_ir(&ir);
    
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
}

#[test]
fn test_empty_return() {
    let source = r#"
def func_with_return():
    return
    "#;
    
    let ir = compile_to_ir(source, "return_module").expect("Compilation failed");
    debug_ir(&ir);
    
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
    // As your compiler advances, you could check for "ret void" instruction
}

// Stage 4: Function Call Tests
// Tests basic function calls once function compilation is working

#[test]
fn test_function_call() {
    let source = r#"
def callee():
    pass

def caller():
    callee()
    "#;
    
    let ir = compile_to_ir(source, "function_call_module").expect("Compilation failed");
    debug_ir(&ir);
    
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
    // Eventually you'd check for "call" instructions
}

// Stage 5: Basic Expression Tests
// These tests verify simple expressions once expression compilation is working

#[test]
fn test_simple_literal() {
    let source = r#"
def literal_func():
    x = 42  # Simple integer assignment
    "#;
    
    let ir = compile_to_ir(source, "literal_module").expect("Compilation failed");
    debug_ir(&ir);
    
    assert!(ir.contains("define"), "Generated IR should contain function definitions");
    // Eventually check for "store i64 42" or similar
}

// Error Handling Test

#[test]
fn test_compile_error_handling() {
    let source = "def malformed(: pass";  // Syntax error
    
    match compile_to_ir(source, "error_module") {
        Ok(ir) => {
            debug_ir(&ir);
            panic!("Should have failed to compile malformed code");
        },
        Err(e) => {
            println!("Expected error: {}", e);
            // Successfully detected the error
        }
    }
}

// More advanced tests can be added as your compiler implementation progresses