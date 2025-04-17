# Example of tuple functionality in Cheetah

# Create a simple tuple
simple_tuple = (1, 2, 3)
print("Simple tuple:", simple_tuple)

# Tuple with different types
mixed_tuple = (1, "hello", True)
print("Mixed tuple:", mixed_tuple)

# Nested tuples
nested_tuple = ((1, 2), (3, 4))
print("Nested tuple:", nested_tuple)

# Tuple unpacking
a, b, c = simple_tuple
print("Unpacked values:", a, b, c)

# Tuple as function argument
def process_tuple(t):
    print("Processing tuple:", t)
    return t

result = process_tuple((5, 6))
print("Result:", result)

# Tuple as function return value
def create_tuple():
    return (7, 8, 9)

returned_tuple = create_tuple()
print("Returned tuple:", returned_tuple)

# Tuple unpacking with nested tuples
(x, y), (z, w) = nested_tuple
print("Unpacked nested tuple:", x, y, z, w)

# Empty tuple
empty_tuple = ()
print("Empty tuple:", empty_tuple)

print("All tuple tests completed successfully!")
