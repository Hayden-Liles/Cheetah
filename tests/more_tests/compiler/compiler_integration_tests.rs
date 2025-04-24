use cheetah::parse;
use cheetah::compiler::Compiler;
use inkwell::context::Context;

fn compile_source(source: &str) -> Result<String, String> {
    // Parse the source
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errors) => {
            return Err(format!("Parse errors: {:?}", errors));
        }
    };

    // Create a compiler
    let context = Context::create();
    let mut compiler = Compiler::new(&context, "test_module");

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_fizzbuzz() {
    // Test a simplified FizzBuzz implementation
    let source = r#"
i = 1
result = ""
if i % 15 == 0:
    result = "FizzBuzz"
elif i % 3 == 0:
    result = "Fizz"
elif i % 5 == 0:
    result = "Buzz"
else:
    # Use string concatenation instead of str() function
    result = "" + i
i = i + 1
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "FizzBuzz compilation failed: {:?}", result.err());

    let ir = result.unwrap();
    // Check for conditional branches (if/else)
    assert!(ir.contains("br"));
    // Check for BoxedAny operations
    assert!(ir.contains("boxed_any_modulo"));
    assert!(ir.contains("boxed_any_equals"));
    assert!(ir.contains("boxed_any_add"));
}

#[test]
fn test_nested_arithmetic() {
    // Test complex arithmetic expression compilation
    let source = r#"
a = 10
b = 20
c = 30
result = a + b * c - (a * b) / c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Nested arithmetic compilation failed: {:?}", result.err());

    let ir = result.unwrap();
    // Check for BoxedAny arithmetic operations
    assert!(ir.contains("boxed_any_add"));
    assert!(ir.contains("boxed_any_multiply"));
    assert!(ir.contains("boxed_any_subtract"));
    assert!(ir.contains("boxed_any_divide"));
}

#[test]
fn test_variable_scopes() {
    // Test variable scoping with a simplified approach
    let source = r#"
x = 10
y = 0
z = 0
if x > 5:
    y = 20
    z = x + y
result = z  # should be accessible outside the if block
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Variable scoping test failed: {:?}", result.err());
}

#[test]
fn test_compound_conditions() {
    // Test complex boolean expressions
    let source = r#"
a = 10
b = 20
c = 30
result = False

if a < b and b < c:
    if not (a == 0 or c == 0):
        result = True

if a < b < c and not (a == 0 or c == 0):
    # This is the same condition as above but expressed differently
    alternative = True
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Compound conditions test failed: {:?}", result.err());
}

#[test]
fn test_loop_control() {
    // Test loop control flow (break, continue)
    let source = r#"
sum = 0
i = 0
while i < 10:
    i = i + 1
    if i % 2 == 0:
        continue  # Skip even numbers
    if i > 8:
        break  # Stop when i > 8
    sum = sum + i
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Loop control test failed: {:?}", result.err());

    let ir = result.unwrap();
    // Check for branching that would indicate continue/break
    assert!(ir.contains("br label"));
}

#[test]
fn test_type_conversions() {
    // Test implicit type conversions
    let source = r#"
i = 10       # int
f = 3.14     # float
result = i + f  # Should convert i to float
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Type conversion test failed: {:?}", result.err());

    let ir = result.unwrap();
    // With BoxedAny, we use boxed_any_add instead of sitofp
    assert!(ir.contains("boxed_any_add"));
}