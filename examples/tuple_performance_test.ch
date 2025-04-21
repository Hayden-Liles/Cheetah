# Create deeply nested tuples to test type inference performance
def create_nested_tuple(depth, width):
    if depth <= 0:
        return 1
    
    result = []
    for i in range(width):
        result.append(create_nested_tuple(depth - 1, width))
    
    return tuple(result)

# Create a 4-level 3-element tuple
nested_tuple = create_nested_tuple(4, 3)
print("Created nested tuple")

# Create tuples with mixed types
mixed_tuple1 = (1, "hello", 3.14)
mixed_tuple2 = (True, 42, "world")
mixed_tuple3 = ((1, 2), (3.0, 4.0), ("a", "b"))

# Combine tuples
combined = (mixed_tuple1, mixed_tuple2, mixed_tuple3)
print("Created combined tuple")

# Print results
print("Nested tuple type:", type(nested_tuple))
print("Mixed tuple type:", type(mixed_tuple1))
print("Combined tuple type:", type(combined))
