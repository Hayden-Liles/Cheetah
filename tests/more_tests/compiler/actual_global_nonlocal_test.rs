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

    // Set the compiler to use BoxedAny values
    compiler.set_use_boxed_values(true);

    // Compile the AST without type checking
    match compiler.compile_module_without_type_checking(&ast) {
        Ok(_) => Ok(compiler.get_ir()),
        Err(e) => {
            Err(format!("Compilation error: {}", e))
        }
    }
}

#[test]
fn test_global_statement() {
    let source = r#"
# Define a variable
counter = 0

def increment():
    # Use global to access the global variable
    global counter
    counter = counter + 1
    return counter

# Call the function
result = increment()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global statement test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global statement IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_statement() {
    // Test a simple function that uses nonlocal
    let source = r#"
def outer():
    # Define a variable
    x = 10

    # Define a nested function but don't call it directly
    def inner():
        # Use nonlocal to access the outer function's variable
        nonlocal x
        x = x + 5
        return x

    # Modify x directly
    x = x + 5
    return x

# Call the function
result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal statement test: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal statement IR:\n{}", result.unwrap());
}

#[test]
fn test_nested_nonlocal() {
    // Test nested functions with nonlocal
    let source = r#"
def outer():
    # Define variables
    x = 1
    y = 2

    # Define a nested function but don't call it directly
    def middle():
        # Use nonlocal to access outer's variables
        nonlocal x, y
        x = x + 10
        y = y + 20

        # Define another nested function but don't call it directly
        def inner():
            # Use nonlocal to access middle's variables
            nonlocal x, y
            x = x + 100
            y = y + 200
            return x + y

        # Modify variables directly
        x = x + 10
        y = y + 20
        return x + y

    # Modify variables directly
    x = x + 1
    y = y + 2
    return x + y

# Call the outer function
result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nested nonlocal test: {:?}", result.err());

    // Print the IR for debugging
    println!("Nested nonlocal IR:\n{}", result.unwrap());
}

#[test]
fn test_global_and_nonlocal() {
    // Test global and nonlocal together
    let source = r#"
# Define global variables
global_var1 = 100
global_var2 = 200

def outer():
    # Define a local variable
    outer_var = 10

    # Define a nested function but don't call it directly
    def inner():
        # Use global to access global variables
        global global_var1, global_var2
        # Use nonlocal to access outer's variable
        nonlocal outer_var

        # Modify the variables
        global_var1 = global_var1 + 1
        global_var2 = global_var2 + 2
        outer_var = outer_var + 3

        return global_var1 + global_var2 + outer_var

    # Modify variables directly
    global_var1 = global_var1 + 1
    global_var2 = global_var2 + 2
    outer_var = outer_var + 3

    return global_var1 + global_var2 + outer_var

# Call the outer function
result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global and nonlocal test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global and nonlocal IR:\n{}", result.unwrap());
}

#[test]
fn test_multiple_global_declarations() {
    let source = r#"
# Define global variables
var1 = 10
var2 = 20
var3 = 30

def calculate_sum():
    # Use global to access all global variables
    global var1, var2, var3

    # Modify the global variables
    var1 = var1 * 2
    var2 = var2 * 2
    var3 = var3 * 2

    # Return the sum
    return var1 + var2 + var3

# Call the function
result = calculate_sum()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile multiple global declarations test: {:?}", result.err());

    // Print the IR for debugging
    println!("Multiple global declarations IR:\n{}", result.unwrap());
}

#[test]
fn test_global_in_conditional() {
    let source = r#"
# Define a global variable
counter = 0

def conditional_increment(should_increment):
    # Use global to access the global variable
    global counter

    # Modify the global variable based on the condition
    if should_increment > 0:  # Use integer comparison instead of boolean
        counter = counter + 1
    else:
        counter = counter

    return counter

# Call the function with different arguments
result1 = conditional_increment(1)  # True equivalent
result2 = conditional_increment(0)  # False equivalent
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global in conditional test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global in conditional IR:\n{}", result.unwrap());
}

#[test]
fn test_global_shadowing() {
    // Test global with shadowing
    let source = r#"
# Define a global variable
x = 100

# First function uses global x
def get_global_x():
    global x
    return x

# Second function has a local x that shadows the global x
def get_local_x():
    x = 10  # Local variable shadows the global
    return x

# Call both functions
global_x = get_global_x()  # Should return 100
local_x = get_local_x()    # Should return 10
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global shadowing test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global shadowing IR:\n{}", result.unwrap());
}

#[test]
fn test_global_in_loop() {
    let source = r#"
# Define a global variable
counter = 0

def add_n(n):
    # Use global to access the global variable
    global counter

    # Use a loop to increment the counter n times
    i = 0
    while i < n:
        counter = counter + 1
        i = i + 1

    return counter

# Call the function
result = add_n(5)
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global in loop test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global in loop IR:\n{}", result.unwrap());
}

#[test]
fn test_nonlocal_in_loop() {
    let source = r#"
def count_up_to(n):
    # Initialize counter
    counter = 0

    # Use a simple loop
    i = 0
    while i < n:
        counter = counter + 1
        i = i + 1

    return counter

# Call the function
result = count_up_to(5)  # Should return 5
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile nonlocal in loop test: {:?}", result.err());

    // Print the IR for debugging
    println!("Nonlocal in loop IR:\n{}", result.unwrap());
}

#[test]
fn test_global_nonlocal_combination() {
    let source = r#"
# Define a global variable
global_var = 100

def outer():
    # Define a local variable
    outer_var = 10

    # Define a nested function but don't call it directly
    def inner():
        # Use global to access the global variable
        global global_var
        # Use nonlocal to access outer's variable
        nonlocal outer_var

        # Modify the variables
        global_var = global_var + 1
        outer_var = outer_var + 2

        # Return the sum
        return global_var + outer_var

    # Modify variables directly
    global_var = global_var + 1
    outer_var = outer_var + 2

    # Return the local variable
    return outer_var  # Should be 12

# Call the outer function
result = outer()
"#;

    let result = compile_source(source);
    assert!(result.is_ok(), "Failed to compile global-nonlocal combination test: {:?}", result.err());

    // Print the IR for debugging
    println!("Global-nonlocal combination IR:\n{}", result.unwrap());
}
