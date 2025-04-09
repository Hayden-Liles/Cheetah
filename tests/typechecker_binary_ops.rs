use cheetah::compiler::types::TypeError;
use cheetah::typechecker;

// Helper function to check if a type error contains a specific message
fn error_contains(error: &TypeError, message: &str) -> bool {
    format!("{:?}", error).contains(message)
}

#[test]
fn test_arithmetic_binary_ops() {
    // Test all arithmetic binary operations with integers
    let source = r#"
# Addition
a = 10 + 20

# Subtraction
b = 30 - 15

# Multiplication
c = 5 * 4

# Division
d = 20 / 4

# Floor division
e = 20 // 4

# Modulo
f = 10 % 3

# Power
g = 2 ** 3
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid integer arithmetic operations");
}

#[test]
fn test_float_binary_ops() {
    // Test all arithmetic binary operations with floats
    let source = r#"
# Addition
a = 10.5 + 20.5

# Subtraction
b = 30.5 - 15.2

# Multiplication
c = 5.5 * 4.0

# Division
d = 20.0 / 4.0

# Floor division
e = 20.0 // 4.0

# Modulo
f = 10.5 % 3.2

# Power
g = 2.5 ** 3.0
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid float arithmetic operations");
}

#[test]
fn test_mixed_numeric_binary_ops() {
    // Test all arithmetic binary operations with mixed numeric types
    let source = r#"
# Addition
a = 10 + 20.5

# Subtraction
b = 30.5 - 15

# Multiplication
c = 5 * 4.0

# Division
d = 20 / 4.0

# Floor division
e = 20.0 // 4

# Modulo
f = 10 % 3.2

# Power
g = 2 ** 3.5
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid mixed numeric arithmetic operations");
}

#[test]
fn test_string_binary_ops() {
    // Test binary operations with strings
    let source = r#"
# String concatenation
a = "hello" + " world"

# String repetition
b = "hello" * 3
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid string operations");
}

#[test]
fn test_list_binary_ops() {
    // Test binary operations with lists
    let source = r#"
# List concatenation
a = [1, 2, 3] + [4, 5, 6]

# List repetition
b = [1, 2, 3] * 3
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid list operations");
}

#[test]
fn test_bitwise_binary_ops() {
    // Test all bitwise binary operations
    let source = r#"
# Bitwise AND
a = 10 & 7

# Bitwise OR
b = 10 | 7

# Bitwise XOR
c = 10 ^ 7

# Left shift
d = 10 << 2

# Right shift
e = 10 >> 2
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid bitwise operations");
}

#[test]
fn test_comparison_binary_ops() {
    // Test all comparison operations
    let source = r#"
# Equal
a = 10 == 10

# Not equal
b = 10 != 20

# Less than
c = 10 < 20

# Less than or equal
d = 10 <= 10

# Greater than
e = 20 > 10

# Greater than or equal
f = 10 >= 10

# String comparisons
g = "apple" < "banana"
h = "apple" == "apple"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid comparison operations");
}

#[test]
fn test_boolean_binary_ops() {
    // Test boolean operations
    let source = r#"
# AND
a = True and False

# OR
b = True or False

# Complex boolean expressions
c = (10 > 5) and (20 < 30)
d = (10 == 10) or (5 != 5)
e = not (10 > 20)
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid boolean operations");
}

#[test]
fn test_invalid_arithmetic_ops() {
    // Test invalid arithmetic operations
    let source = r#"
# Cannot add string and integer
a = "hello" + 10
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_err(), "Type checking should fail for invalid arithmetic operations");
    
    if let Err(error) = result {
        assert!(error_contains(&error, "InvalidOperator"), 
                "Expected InvalidOperator error, got {:?}", error);
        assert!(error_contains(&error, "+"), 
                "Error should mention the '+' operator");
    }
}

#[test]
fn test_invalid_bitwise_ops() {
    // Test invalid bitwise operations
    let source = r#"
# Cannot perform bitwise AND on float
a = 10.5 & 7
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_err(), "Type checking should fail for invalid bitwise operations");
    
    if let Err(error) = result {
        assert!(error_contains(&error, "InvalidOperator"), 
                "Expected InvalidOperator error, got {:?}", error);
        assert!(error_contains(&error, "&"), 
                "Error should mention the '&' operator");
    }
}

#[test]
fn test_invalid_string_ops() {
    // Test invalid string operations
    let source = r#"
# Cannot subtract from string
a = "hello" - "h"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_err(), "Type checking should fail for invalid string operations");
    
    if let Err(error) = result {
        assert!(error_contains(&error, "InvalidOperator"), 
                "Expected InvalidOperator error, got {:?}", error);
        assert!(error_contains(&error, "-"), 
                "Error should mention the '-' operator");
    }
}

#[test]
fn test_complex_binary_expressions() {
    // Test complex expressions with multiple binary operations
    let source = r#"
# Complex arithmetic expression
a = 10 + 20 * 30 / 2 - 5

# Complex boolean expression
b = (10 > 5 and 20 < 30) or (40 == 40 and not (50 < 40))

# Mixed expression
c = (10 + 5) * 2 > 15 * 2 - 10
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid complex expressions");
}

#[test]
fn test_binary_ops_with_variables() {
    // Test binary operations with variables
    let source = r#"
# Initialize variables
x = 10
y = 20.5
z = "hello"

# Operations with variables
a = x + y
b = x * 2
c = y / x
d = z + " world"
e = x > y
f = (x < 20) and (y > 10)
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid operations with variables");
}

#[test]
fn test_binary_ops_in_functions() {
    // Test binary operations in function definitions
    let source = r#"
def add(x, y):
    return x + y

def multiply(x, y):
    return x * y

def compare(x, y):
    return x > y and x != y

# Call functions
a = add(10, 20)
b = multiply(5, 4)
c = compare(20, 10)
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid operations in functions");
}

#[test]
fn test_binary_ops_in_conditionals() {
    // Test binary operations in conditional statements
    let source = r#"
x = 10
y = 20

if x + y > 25:
    z = "greater"
else:
    z = "less"

if x * 2 == y:
    w = "equal"
elif x - 5 < 0:
    w = "negative"
else:
    w = "other"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid operations in conditionals");
}

#[test]
fn test_binary_ops_with_function_calls() {
    // Test binary operations with function calls
    let source = r#"
def get_value():
    return 10

def get_string():
    return "hello"

# Operations with function calls
a = get_value() + 20
b = get_value() * 3
c = get_string() + " world"
d = get_value() > 5
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid operations with function calls");
}

#[test]
fn test_binary_ops_type_inference() {
    // Test type inference with binary operations
    let source = r#"
# Integer operations should result in integers
a = 10 + 20
b = 30 - 15
c = 5 * 4

# Float operations should result in floats
d = 10.5 + 20.5
e = 30.0 - 15.0

# Mixed operations should result in floats
f = 10 + 20.5
g = 5 * 4.0

# String operations
h = "hello" + " world"

# Boolean operations
i = 10 > 5
j = True and False
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for type inference with binary operations");
}
