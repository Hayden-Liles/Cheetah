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

// Stage 6: Control Flow Statement Tests
// These tests verify the if/else, while, and for statements

#[test]
fn test_if_statement() {
    let source = r#"
def func_with_if():
    x = 10
    if x > 5:
        y = 20
    else:
        y = 30
    return y
    "#;
    
    let ir = compile_to_ir(source, "if_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for the basic blocks used in if/else structure
    assert!(ir.contains("then"), "Should contain 'then' basic block");
    assert!(ir.contains("else"), "Should contain 'else' basic block");
    assert!(ir.contains("if.end"), "Should contain 'if.end' basic block");
    assert!(ir.contains("br i1"), "Should contain conditional branch instruction");
}

#[test]
fn test_nested_if_statements() {
    let source = r#"
def func_with_nested_if():
    x = 10
    y = 20
    if x > 5:
        if y > 15:
            z = 30
        else:
            z = 40
    else:
        z = 50
    return z
    "#;
    
    let ir = compile_to_ir(source, "nested_if_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Multiple instances of conditional branches for nested ifs
    let branches = ir.matches("br i1").count();
    assert!(branches >= 2, "Should contain at least 2 conditional branches for nested ifs");
}

#[test]
fn test_while_loop() {
    let source = r#"
def func_with_while():
    x = 0
    while x < 10:
        x = x + 1
    return x
    "#;
    
    let ir = compile_to_ir(source, "while_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for the basic blocks used in while loop structure
    assert!(ir.contains("while.cond"), "Should contain 'while.cond' basic block");
    assert!(ir.contains("while.body"), "Should contain 'while.body' basic block");
    assert!(ir.contains("while.end"), "Should contain 'while.end' basic block");
    assert!(ir.contains("br i1"), "Should contain conditional branch instruction");
}

#[test]
fn test_while_loop_with_else() {
    let source = r#"
def func_with_while_else():
    x = 0
    while x < 0:  # Never enters the loop
        x = x + 1
    else:
        x = 42
    return x
    "#;
    
    let ir = compile_to_ir(source, "while_else_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for the else block in while loop
    assert!(ir.contains("while.else"), "Should contain 'while.else' basic block");
}

#[test]
fn test_for_loop() {
    let source = r#"
def func_with_for():
    sum = 0
    for i in [1, 2, 3, 4, 5]:
        sum = sum + i
    return sum
    "#;
    
    let ir = compile_to_ir(source, "for_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for the basic blocks used in for loop structure
    assert!(ir.contains("for.init"), "Should contain 'for.init' basic block");
    assert!(ir.contains("for.cond"), "Should contain 'for.cond' basic block");
    assert!(ir.contains("for.body"), "Should contain 'for.body' basic block");
    assert!(ir.contains("for.inc"), "Should contain 'for.inc' basic block");
    assert!(ir.contains("for.end"), "Should contain 'for.end' basic block");
}

#[test]
fn test_for_loop_with_else() {
    let source = r#"
def func_with_for_else():
    sum = 0
    for i in []:  # Empty list, never enters the loop
        sum = sum + i
    else:
        sum = 42
    return sum
    "#;
    
    let ir = compile_to_ir(source, "for_else_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for the else block in for loop
    assert!(ir.contains("for.else"), "Should contain 'for.else' basic block");
}

#[test]
fn test_break_statement() {
    let source = r#"
def func_with_break():
    x = 0
    while x < 10:
        x = x + 1
        if x == 5:
            break
    return x
    "#;
    
    let ir = compile_to_ir(source, "break_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // There should be an unconditional branch in the loop body for the break statement
    // that jumps to the loop end
    assert!(ir.contains("while.end"), "Should contain 'while.end' basic block");
    assert!(ir.contains("br label"), "Should contain unconditional branch instruction");
}

#[test]
fn test_continue_statement() {
    let source = r#"
def func_with_continue():
    x = 0
    sum = 0
    while x < 10:
        x = x + 1
        if x % 2 == 0:
            continue
        sum = sum + x
    return sum
    "#;
    
    let ir = compile_to_ir(source, "continue_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // There should be an unconditional branch in the loop body for the continue statement
    // that jumps to the loop condition
    assert!(ir.contains("while.cond"), "Should contain 'while.cond' basic block");
    assert!(ir.contains("br label"), "Should contain unconditional branch instruction");
}

#[test]
fn test_nested_loops_with_break() {
    let source = r#"
def func_with_nested_loops():
    x = 0
    y = 0
    while x < 5:
        x = x + 1
        while y < 5:
            y = y + 1
            if y == 3:
                break
    return x + y
    "#;
    
    let ir = compile_to_ir(source, "nested_loops_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Verify the presence of nested loop blocks and break functionality
    assert!(ir.contains("while.cond"), "Should contain 'while.cond' basic block");
}

// Stage 7: Assignment Statement Tests
// These tests verify different types of assignments

#[test]
fn test_simple_assignment() {
    let source = r#"
def func_with_assignment():
    x = 42
    y = 3.14
    z = True
    return x
    "#;
    
    let ir = compile_to_ir(source, "assignment_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for alloca and store instructions for variables
    assert!(ir.contains("alloca"), "Should contain 'alloca' instructions for variables");
    assert!(ir.contains("store"), "Should contain 'store' instructions for assignments");
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

// Stage 8: Expression Statement Tests
// These tests verify expression statements are compiled correctly

#[test]
fn test_binary_operations() {
    let source = r#"
def func_with_binary_ops():
    a = 10
    b = 20
    c = a + b
    d = a - b
    e = a * b
    f = a / b
    g = a % b
    return c + d + e + f + g
    "#;
    
    let ir = compile_to_ir(source, "binary_ops_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for arithmetic operations
    assert!(ir.contains("add") || ir.contains("fadd"), "Should contain addition operations");
    assert!(ir.contains("sub") || ir.contains("fsub"), "Should contain subtraction operations");
    assert!(ir.contains("mul") || ir.contains("fmul"), "Should contain multiplication operations");
}

#[test]
fn test_comparison_operations() {
    let source = r#"
def func_with_comparisons():
    a = 10
    b = 20
    c = a == b
    d = a != b
    e = a < b
    f = a <= b
    g = a > b
    h = a >= b
    return c or d or e or f or g or h
    "#;
    
    let ir = compile_to_ir(source, "comparison_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for comparison operations
    assert!(ir.contains("icmp") || ir.contains("fcmp"), "Should contain comparison operations");
}

#[test]
fn test_unary_operations() {
    let source = r#"
def func_with_unary_ops():
    a = 10
    b = -a
    c = +a
    d = not True
    return b + c
    "#;
    
    let ir = compile_to_ir(source, "unary_ops_module").expect("Compilation failed");
    debug_ir(&ir);
    
    // Check for negation operation
    assert!(ir.contains("sub") || ir.contains("fsub") || ir.contains("neg"), 
            "Should contain negation operation");
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