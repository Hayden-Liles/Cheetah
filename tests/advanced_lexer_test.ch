# This is an advanced Cheetah program showing various language features

def fibonacci(n):
    """
    Calculate the nth Fibonacci number recursively
    This uses triple-quoted strings for documentation
    """
    if n <= 1:
        return n
    else:
        return fibonacci(n-1) + fibonacci(n-2)

# Class definition
class Calculator:
    def __init__(self, value=0):
        self.value = value
    
    def add(self, x):
        self.value += x
        return self.value
    
    def multiply(self, x):
        self.value *= x
        return self.value

# Testing variables and operators
x = 10
y = 3.14
name = "Cheetah"

# Testing string operations and escape sequences
message = "Hello, " + name + "!\n"
special_chars = "Unicode: \u{1F60A} and hex: \x41\x42\x43"

# Testing conditional statements
if x > 5:
    print("x is greater than 5")
    if y > 3:
        print("y is also greater than 3")
elif x == 5:
    print("x equals 5")
else:
    print("x is less than 5")

# Testing loops
sum = 0
for i in range(5):
    sum += i
    print("Current sum:", sum)

# Testing function calls
fib_10 = fibonacci(10)
print(f"The 10th Fibonacci number is: {fib_10}")

# Testing class instantiation and method calls
calc = Calculator(5)
calc.add(10)
result = calc.multiply(2)
print(f"Calculator result: {result}")

# Testing list comprehension and operations
numbers = [1, 2, 3, 4, 5]
squares = [x**2 for x in numbers if x % 2 == 0]
print("Squares of even numbers:", squares)