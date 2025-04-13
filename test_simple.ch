# Simple test file for non-recursive expression compilation

# Basic expressions
a = 1
b = 2
c = a + b
print(c)

# If expression
d = 10 if c > 2 else 5
print(d)

# Boolean operations
e = True and False
f = True or False
print(e)
print(f)

# Comparison operations
g = a < b
h = a == b
i = a != b
print(g)
print(h)
print(i)

# Nested expressions
j = (a + b) * (c - 1)
print(j)

# List comprehension
k = [x * 2 for x in [1, 2, 3, 4, 5]]
print(k)

# Dictionary
m = {"a": 1, "b": 2, "c": 3}
print(m["a"])
print(m["b"])
print(m["c"])
