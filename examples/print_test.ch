# Test the print function in Cheetah

# Print a string
print("Hello, Cheetah!")

# Print multiple values
print("The answer is", 42)

# Print a calculation
print("2 + 2 =", 2 + 2)

# Print a boolean
print("Is Cheetah awesome?", True)

# Print without arguments (just a newline)
print()

# Print multiple types
print("String:", "text", "Integer:", 123, "Float:", 3.14, "Boolean:", False)

# Define and call a function that uses print
def greet(name):
    print("Hello,", name, "!")
    return 0

# Call the function
greet("World")

# Print in a loop
for i in range(3):
    print("Loop iteration", i)
