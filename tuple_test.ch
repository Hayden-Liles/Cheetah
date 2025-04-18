# Test tuple creation and access
print("Starting tuple test")
t = (1, 2, 3)
print("Tuple:", t)

# Test tuple unpacking
a, b, c = t
print("Unpacked:", a, b, c)

# Test nested tuples
nested = ((1, 2), (3, 4))
print("Nested tuple:", nested)

# Test tuple as function argument
def process_tuple(t):
    print("Processing tuple:", t)
    return t

result = process_tuple((5, 6))
print("Result:", result)
