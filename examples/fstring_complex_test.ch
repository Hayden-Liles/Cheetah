# Test f-strings with more complex expressions

# Define a function
def add(a, b):
    return a + b

# Create variables
x = 10
y = 20

# Test f-strings with function calls
print(f"add(x, y) = {add(x, y)}")

# Test f-strings with arithmetic expressions
print(f"x + y = {x + y}, x * y = {x * y}, x / y = {x / y}")

# Test f-strings with nested expressions
print(f"Complex expression: {add(x, y) * 2 - 5}")

# Test f-strings with multiple expressions
print(f"Multiple expressions: {x} + {y} = {x + y}")

# Test f-strings with boolean expressions
print(f"x > y: {x > y}, x < y: {x < y}, x == y: {x == y}")
