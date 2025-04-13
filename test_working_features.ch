# Test file demonstrating working features of the non-recursive expression compiler

# Basic expressions
a = 1
b = 2
c = a + b
print("Basic arithmetic:")
print(c)  # Should print 3

# If expressions
d = 10 if c > 2 else 5
print("\nIf expressions:")
print(d)  # Should print 10

# Boolean operations
e = True and False
f = True or False
print("\nBoolean operations:")
print(e)  # Should print False
print(f)  # Should print True

# Comparison operations
g = a < b
h = a == b
i = a != b
print("\nComparison operations:")
print(g)  # Should print True
print(h)  # Should print False
print(i)  # Should print True

# Nested expressions
j = (a + b) * (c - 1)
print("\nNested expressions:")
print(j)  # Should print 6
