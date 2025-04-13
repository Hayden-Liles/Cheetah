# Test file for non-recursive expression compilation

# Simple expressions
a = 1
b = 2
c = a + b

# Print the result
print(c)

# Create a loop with many iterations to test stack overflow
for i in range(10000000):
    print(i)
