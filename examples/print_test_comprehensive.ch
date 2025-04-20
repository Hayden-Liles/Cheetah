# Comprehensive test for print function with various data structures

# Basic types
print("\n=== Basic Types ===")
print("Integer:", 42)
print("Float:", 3.14159)
print("Boolean:", True, False)
print("None:", None)
print("String:", "Hello, World!")

# Lists
print("\n=== Lists ===")
empty_list = []
print("Empty list:", empty_list)

numbers = [1, 2, 3, 4, 5]
print("Number list:", numbers)

mixed_list = [1, "two", 3.0, True, None]
print("Mixed list:", mixed_list)

# Nested lists
print("\n=== Nested Lists ===")
nested_list = [1, [2, 3], [4, [5, 6]]]
print("Nested list:", nested_list)

matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
print("Matrix:", matrix)

# Dictionaries
print("\n=== Dictionaries ===")
empty_dict = {}
print("Empty dictionary:", empty_dict)

person = {"name": "John", "age": 30, "is_student": False}
print("Person dictionary:", person)

# Nested dictionaries
print("\n=== Nested Dictionaries ===")
nested_dict = {
    "person": {"name": "Alice", "age": 25},
    "scores": {"math": 95, "science": 98}
}
print("Nested dictionary:", nested_dict)

# Mixed nesting
print("\n=== Mixed Nesting ===")
complex_structure = {
    "name": "Complex Example",
    "data": [1, 2, {"nested": [3, 4, {"deep": 5}]}],
    "metadata": {"created": "today", "tags": ["test", "example"]}
}
print("Complex structure:", complex_structure)

# List of dictionaries
print("\n=== List of Dictionaries ===")
users = [
    {"id": 1, "name": "Alice", "roles": ["admin", "user"]},
    {"id": 2, "name": "Bob", "roles": ["user"]},
    {"id": 3, "name": "Charlie", "roles": ["moderator", "user"]}
]
print("Users:", users)

# Dictionary of lists
print("\n=== Dictionary of Lists ===")
categories = {
    "fruits": ["apple", "banana", "cherry"],
    "vegetables": ["carrot", "broccoli", "spinach"],
    "numbers": [1, 2, 3, 4, 5]
}
print("Categories:", categories)

# Multiple arguments
print("\n=== Multiple Arguments ===")
print("Multiple values:", 1, "two", 3.0, [4, 5], {"six": 6})

# Modifying and printing
print("\n=== Modifying and Printing ===")
test_list = [1, 2, 3]
print("Before append:", test_list)
test_list.append(4)
print("After append:", test_list)

# Print formatting with multiple types
print("\n=== Print Formatting ===")
name = "Alice"
age = 30
scores = [95, 87, 92]
print("Name:", name, "Age:", age, "Scores:", scores)

# Print with expressions
print("\n=== Print with Expressions ===")
a = 5
b = 10
print("Sum:", a + b, "Product:", a * b, "List:", [a, b, a + b])

# Print with string concatenation
print("\n=== String Concatenation ===")
first = "Hello"
last = "World"
print("Concatenated:", first + " " + last)

# Print with boolean expressions
print("\n=== Boolean Expressions ===")
x = 5
y = 10
print("Comparisons:", x < y, x > y, x == y, x != y)

print("\n=== End of Test ===")
