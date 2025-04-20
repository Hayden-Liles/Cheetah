# Simple print test
print("=== Test 1: Basic Types ===")
print("String: Hello, World!")
print("Integer: 42")
print("Float: 3.14159")
print("Boolean: True")
print("Boolean: False")

print("=== Test 2: Empty Prints and Whitespace ===")
print()  # Empty print should produce a newline
print("")  # Empty string should produce a newline
print(" ")  # Space should be printed

print("=== Test 3: Min and Max Functions ===")
print("min(5, 10) =")
print(min(5, 10))
print("max(5, 10) =")
print(max(5, 10))

print("=== Test 4: Consecutive Prints ===")
print("Line 1")
print("Line 2")
print("Line 3")

print("=== Test 5: Print in a Loop ===")
for i in range(1, 6):
    print("Loop iteration " + str(i))

print("=== Test 6: Print Expressions ===")
print("Addition: " + str(5 + 10))
print("Subtraction: " + str(10 - 5))
print("Multiplication: " + str(5 * 10))
print("Division: " + str(10 / 5))
