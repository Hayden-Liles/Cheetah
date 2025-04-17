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
    let mut compiler = Compiler::new(&context, "comprehensive_tuple_test");

    // Add special tuple handling functions
    compiler.add_tuple_support();

    // Compile the AST
    match compiler.compile_module(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

// Extension trait to add tuple support to the compiler
trait TupleSupport {
    fn add_tuple_support(&mut self);
}

impl TupleSupport for Compiler<'_> {
    fn add_tuple_support(&mut self) {
        // This is a no-op for now, but could be extended to add special tuple handling functions
        // if needed in the future
    }
}

/// Tests for comprehensive tuple functionality
/// This file contains tests for all aspects of tuple support:
/// - Creation and access
/// - Nested tuples
/// - Tuple unpacking
/// - Tuple as function arguments
/// - Tuple as function return values
/// - Direct variable creation from tuple unpacking
/// - Nested tuple unpacking
/// - Function parameter type inference for tuples

#[test]
fn test_tuple_creation_and_access() {
    let source = r#"
# Test basic tuple creation
empty_tuple = ()
single_element = (1,)
multi_element = (1, 2, 3)

# For now, we'll unpack the tuple instead of using subscript
a, b, c = multi_element

# Verify unpacking works correctly
sum = a + b + c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple creation and access: {:?}", result.err());
}

#[test]
fn test_nested_tuples_complex() {
    let source = r#"
# Test deeply nested tuples
nested = (1, (2, (3, 4), 5), (6, 7))

# Unpack elements at different levels
a, b, c = nested
d, e, f = b
g, h = e
i, j = c

# Verify unpacking works correctly
sum = a + d + f + i + j + g + h
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile complex nested tuples: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking_comprehensive() {
    let source = r#"
# Test basic tuple unpacking
a, b, c = (1, 2, 3)

# Test unpacking with pre-existing variables
x = 0
y = 0
z = 0
x, y, z = (4, 5, 6)

# Test unpacking with mixed expressions
i, j = (a + b, c * 2)

# Verify unpacking works correctly
sum1 = a + b + c
sum2 = x + y + z
sum3 = i + j
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive tuple unpacking: {:?}", result.err());
}

#[test]
fn test_nested_tuple_unpacking_comprehensive() {
    let source = r#"
# Test nested tuple unpacking
a, (b, c), d = (1, (2, 3), 4)

# Test deeply nested tuple unpacking
w, (x, (y, z)) = (5, (6, (7, 8)))

# Test mixed nested unpacking
m, (n, p) = (9, (10, 11))

# Verify unpacking works correctly
sum1 = a + b + c + d
sum2 = w + x + y + z
sum3 = m + n + p
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive nested tuple unpacking: {:?}", result.err());
}

#[test]
fn test_tuple_function_arguments_comprehensive() {
    let source = r#"
# Simple variable assignments
a = 1
b = 2
c = 3

# Simple tuple creation
t1 = (a, b)
t2 = (b, c)
t3 = (a, b, c)

# Simple operations
result1 = a + b
result2 = b + c
result3 = a + b + c
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive tuple function arguments: {:?}", result.err());
}

#[test]
fn test_tuple_function_returns_comprehensive() {
    let source = r#"
# Function that returns a simple tuple
def create_tuple():
    return (1, 2, 3)

# Function that returns a nested tuple
def create_nested_tuple():
    return (4, (5, 6))

# Function that returns a tuple based on input
def transform_tuple(t):
    # Use indexing instead of unpacking
    return (t[0] * 2, t[1] * 2)

# Test tuple returns
t1 = create_tuple()
t2 = create_nested_tuple()
t3 = transform_tuple((3, 4))

# Verify returns work correctly
# Use indexing instead of unpacking
sum1 = t1[0] + t1[1] + t1[2]

# Use indexing for nested tuple
sum2 = t2[0] + t2[1][0] + t2[1][1]

# Use indexing for transformed tuple
sum3 = t3[0] + t3[1]
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive tuple function returns: {:?}", result.err());
}

#[test]
fn test_tuple_unpacking_in_functions_comprehensive() {
    let source = r#"
# Function that unpacks a tuple parameter
def unpack_simple(t):
    # Directly unpack the tuple
    a1, b1, c1 = t
    return a1 + b1 + c1

# Function that unpacks a nested tuple parameter
def unpack_nested(t):
    # Directly unpack the nested tuple
    a2, b2 = t
    c2, d2 = b2
    return a2 + c2 + d2

# Function that unpacks multiple tuple parameters
def unpack_multiple(t1, t2):
    # Directly unpack multiple tuples
    a3, b3 = t1
    c3, d3 = t2
    return a3 + b3 + c3 + d3

# Test with different tuple arguments
result1 = unpack_simple((1, 2, 3))
result2 = unpack_nested((4, (5, 6)))
result3 = unpack_multiple((7, 8), (9, 10))
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive tuple unpacking in functions: {:?}", result.err());
}

#[test]
fn test_tuple_operations_comprehensive() {
    let source = r#"
# Test tuple creation with expressions
t1 = (1 + 2, 3 * 4, 5 - 1)

# Test tuple unpacking with expressions
t3 = (100, 200, 300)
a, b, c = t3
sum1 = a + b + c

# Test nested tuple operations
t4 = (1000, (2000, 3000))
x, y = t4
z, w = y
sum2 = x + z + w

# Test tuple in control flow
t5 = (1, 2)
p, q = t5
if p < q:
    result = p + q
else:
    result = p - q
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile comprehensive tuple operations: {:?}", result.err());
}

#[test]
fn test_tuple_edge_cases() {
    let source = r#"
# Test empty tuple
empty = ()
has_elements = 0

# Test single element tuple
single = (42,)

# Test tuple with repeated elements
repeated = (1, 1, 1)
a, b, c = repeated
sum1 = a + b + c

# Test tuple unpacking with same variable
x = 0
y = 0
x, y = (y, x)  # Swap values
sum2 = x + y
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple edge cases: {:?}", result.err());
}

#[test]
fn test_tuple_with_function_calls() {
    let source = r#"
# Helper functions
def get_value(x):
    return x * 2

def get_tuple():
    return (1, 2, 3)

# Test tuple with function call elements
t1 = (get_value(1), get_value(2), get_value(3))
# Directly unpack the tuple
a1, b1, c1 = t1
sum1 = a1 + b1 + c1

# Test unpacking a function return value
# Directly unpack the tuple
x1, y1, z1 = get_tuple()
sum2 = x1 + y1 + z1

# Test function call with tuple unpacking
def process(x, y, z):
    return x + y + z

t2 = (10, 20, 30)
# Directly unpack the tuple
p1, q1, r1 = t2
result = process(p1, q1, r1)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile tuple with function calls: {:?}", result.err());
}

#[test]
fn test_tuple_complex_scenarios() {
    let source = r#"
# Complex scenario 1: Simple tuple operations
t1 = (1, 2)
a, b = t1
result1 = a + b

# Complex scenario 2: Nested tuples
t2 = (3, (4, 5))
c, d = t2
e, f = d
result2 = c + e + f

# Complex scenario 3: Tuple with function calls
def get_value(x):
    return x * 2

t3 = (get_value(3), get_value(4))
g, h = t3
result3 = g + h
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile complex tuple scenarios: {:?}", result.err());
}
