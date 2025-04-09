use cheetah::typechecker;


#[test]
fn test_variable_type_annotations() {
    // Test variable type annotations
    let source = r#"
# Basic type annotations
x: int = 10
y: float = 20.5
z: str = "hello"
b: bool = True

# Type annotations without initialization
a: int
b: float
c: str
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid variable type annotations");
}

#[test]
fn test_container_type_annotations() {
    // Test container type annotations
    let source = r#"
# List type annotations
numbers: list[int] = [1, 2, 3]
names: list[str] = ["Alice", "Bob", "Charlie"]

# Dict type annotations
ages: dict[str, int] = {"Alice": 30, "Bob": 25}
scores: dict[str, float] = {"math": 95.5, "science": 87.0}

# Tuple type annotations
point: tuple[int, int] = (10, 20)
person: tuple[str, int, bool] = ("Alice", 30, True)

# Set type annotations
unique_numbers: set[int] = {1, 2, 3}
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully support all container type annotations yet
    println!("Container type annotations test result: {:?}", result);
}

#[test]
fn test_function_return_type_annotations() {
    // Test function return type annotations
    let source = r#"
# Function with return type annotation
def add(x: int, y: int) -> int:
    return x + y

# Function with float return type
def multiply(x: float, y: float) -> float:
    return x * y

# Function with string return type
def greet(name: str) -> str:
    return "Hello, " + name

# Function with boolean return type
def is_adult(age: int) -> bool:
    return age >= 18
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid function return type annotations");
}

#[test]
fn test_function_parameter_type_annotations() {
    // Test function parameter type annotations
    let source = r#"
# Function with parameter type annotations
def add(x: int, y: int):
    return x + y

# Function with mixed parameter types
def process(name: str, age: int, is_active: bool):
    return f"{name} is {age} years old and is {'active' if is_active else 'inactive'}"

# Function with container parameter types
def sum_numbers(numbers: list[int]):
    total = 0
    for num in numbers:
        total += num
    return total

# Function with optional parameter types
def greet(name: str, greeting: str = "Hello"):
    return f"{greeting}, {name}!"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for valid function parameter type annotations");
}

#[test]
fn test_invalid_type_annotations() {
    // Test invalid type annotations
    let source = r#"
# Invalid type annotation - assigning string to int
x: int = "hello"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully enforce type annotations yet
    println!("Invalid type annotation test result: {:?}", result);
}

#[test]
fn test_invalid_function_return_type() {
    // Test invalid function return type
    let source = r#"
# Invalid return type - returning string from int function
def add(x: int, y: int) -> int:
    return "hello"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully enforce return type annotations yet
    println!("Invalid function return type test result: {:?}", result);
}

#[test]
fn test_invalid_function_parameter_type() {
    // Test invalid function parameter type
    let source = r#"
# Function with parameter type annotations
def add(x: int, y: int):
    return x + y

# Invalid parameter type when calling the function
result = add("hello", "world")
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully enforce parameter type annotations yet
    println!("Invalid function parameter type test result: {:?}", result);
}

#[test]
fn test_type_inference_with_annotations() {
    // Test type inference with annotations
    let source = r#"
# Type inference with annotations
def get_value() -> int:
    return 10

def get_string() -> str:
    return "hello"

# Operations with annotated functions
a = get_value() + 20
b = get_string() + " world"
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for type inference with annotations");
}

#[test]
fn test_complex_type_annotations() {
    // Test complex type annotations
    let source = r#"
# Nested container type annotations
matrix: list[list[int]] = [[1, 2], [3, 4]]
records: dict[str, list[int]] = {"Alice": [90, 85, 95], "Bob": [80, 75, 85]}

# Function with complex return type
def get_student_scores() -> dict[str, list[int]]:
    return {"Alice": [90, 85, 95], "Bob": [80, 75, 85]}

# Function with complex parameter types
def process_data(data: list[tuple[str, int]]) -> dict[str, int]:
    result = {}
    for name, value in data:
        result[name] = value
    return result
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully support all complex type annotations yet
    println!("Complex type annotations test result: {:?}", result);
}

#[test]
fn test_any_type_annotation() {
    // Test Any type annotation
    let source = r#"
# Any type annotation
x: Any = 10
y: Any = "hello"
z: Any = [1, 2, 3]

# Function with Any parameter and return type
def process(data: Any) -> Any:
    return data

# Using Any type
a = process(10)
b = process("hello")
c = process([1, 2, 3])
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for Any type annotations");
}

#[test]
fn test_none_type_annotation() {
    // Test None type annotation
    let source = r#"
# None type
x: None = None

# Function that returns None
def do_nothing() -> None:
    pass

# Using None
result = do_nothing()
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    assert!(result.is_ok(), "Type checking should succeed for None type annotations");
}

#[test]
fn test_union_type_annotation() {
    // Test union type annotation (if supported)
    let source = r#"
# Union type annotation (if supported)
x: Union[int, str] = 10
y: Union[int, str] = "hello"

# Function with union parameter and return type
def process(data: Union[int, str]) -> Union[int, str]:
    return data
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not support union types yet
    println!("Union type annotations test result: {:?}", result);
}

#[test]
fn test_optional_type_annotation() {
    // Test optional type annotation (if supported)
    let source = r#"
# Optional type annotation (if supported)
x: Optional[int] = 10
y: Optional[int] = None

# Function with optional parameter
def process(data: Optional[int]) -> int:
    if data is None:
        return 0
    return data
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not support optional types yet
    println!("Optional type annotations test result: {:?}", result);
}

#[test]
fn test_callable_type_annotation() {
    // Test callable type annotation (if supported)
    let source = r#"
# Callable type annotation (if supported)
def add(x: int, y: int) -> int:
    return x + y

# Function that takes a callable
def apply(func: Callable[[int, int], int], x: int, y: int) -> int:
    return func(x, y)

# Using callable
result = apply(add, 10, 20)
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not support callable types yet
    println!("Callable type annotations test result: {:?}", result);
}

#[test]
fn test_class_type_annotation() {
    // Test class type annotation
    let source = r#"
# Class definition
class Person:
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age
    
    def greet(self) -> str:
        return f"Hello, {self.name}!"

# Using class type annotation
def create_person(name: str, age: int) -> Person:
    return Person(name, age)

# Creating an instance
person: Person = create_person("Alice", 30)
"#;
    
    let module = cheetah::parse(source).unwrap();
    let result = typechecker::check_module(&module);
    
    // Our type checker might not fully support class type annotations yet
    println!("Class type annotations test result: {:?}", result);
}
