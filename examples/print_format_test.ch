print("=== Test 1: Basic Types ===")
print("String: Hello, World!")
print("Integer:", 42)
print("Float:", 3.14159)
print("Boolean:", True)
print("Boolean:", False)

print("=== Test 2: Multiple Values ===")
print("Multiple values:", 1, 2, 3, 4, 5)
print("Mixed types:", "string", 42, 3.14, True)

print("=== Test 3: Empty Prints and Whitespace ===")
print()  # Empty print should produce a newline
print("")  # Empty string should produce a newline
print(" ")  # Space should be printed
print("\t")  # Tab should be printed

print("=== Test 4: Special Characters ===")
print("Newline in string:\nSecond line")
print("Tab in string:\tTabbed text")
print("Quotes: \"Double\" and \'Single\'")
print("Backslash: \\")

print("=== Test 5: Number Formats ===")
print("Integer:", 0)
print("Integer:", -42)
print("Large integer:", 1000000)
print("Float:", 0.0)
print("Float:", -3.14)
print("Scientific notation:", 1.23e-4)
print("Float precision:", 1.2345678901234567)

print("=== Test 6: Min and Max Functions ===")
print("min(5, 10) =")
print(min(5, 10))
print("max(5, 10) =")
print(max(5, 10))
print("min(5.5, 10.5) =")
print(min(5.5, 10.5))
print("max(5.5, 10.5) =")
print(max(5.5, 10.5))
print("min(5, 10.5) =")
print(min(5, 10.5))
print("max(5, 10.5) =")
print(max(5, 10.5))

print("=== Test 7: Consecutive Prints ===")
print("Line 1")
print("Line 2")
print("Line 3")

print("=== Test 8: Print in a Loop ===")
for i in range(1, 6):
    print("Loop iteration:", i)

print("=== Test 9: Print Expressions ===")
print("Addition:", 5 + 10)
print("Subtraction:", 10 - 5)
print("Multiplication:", 5 * 10)
print("Division:", 10 / 5)
print("Complex expression:", (5 + 10) * (10 - 5) / 5)

print("=== Test 10: String Concatenation ===")
print("Concatenated: " + "Hello" + " " + "World!")
