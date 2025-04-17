# Debug test file to diagnose segmentation fault
print("Starting debug test")

# Test 1: Simple print
print("Test 1: Simple print - OK")

# Test 2: Variable assignment
x = 5
print("Test 2: Variable assignment - OK, x =", x)

# Test 3: Simple math
y = x + 10
print("Test 3: Simple math - OK, y =", y)

# Test 4: Create a list
my_list = [1, 2, 3]
print("Test 4: Create a list - OK, list =", my_list)

# Test 5: Access list element
first = my_list[0]
print("Test 5: Access list element - OK, first =", first)

# Test 6: Create a range object without using it
r = range(5)
print("Test 6: Create a range object - OK")

# Test 7: Simple for loop with small range
print("Test 7: Simple for loop with range(3)")
for i in range(3):
    print("  i =", i)

print("All tests completed successfully")
