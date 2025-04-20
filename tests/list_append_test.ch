# Test list.append method

# Create a list
numbers = [1, 2, 3]

# Append a value
numbers.append(4)

# Print the list
print("List after append:", numbers)

# Append another value
numbers.append(5)

# Print the list again
print("List after second append:", numbers)

# Create an empty list and append to it
empty = []
empty.append(10)
empty.append(20)
print("Empty list after appends:", empty)

# Test with different types
mixed = ["hello"]
mixed.append(42)
mixed.append(True)
mixed.append(3.14)
print("Mixed type list:", mixed)
