# Example of dynamic tuple indexing

# Create a tuple
t = (10, 20, 30, 40, 50)

# Use a variable as index
i = 2
print("Value at index", i, "is", t[i])  # Should print 30

# Use a calculated index
base = 1
offset = 2
index = base + offset
print("Value at calculated index", index, "is", t[index])  # Should print 40

# Create a nested tuple
nested = ((1, 2), (3, 4), (5, 6))
j = 1
inner = nested[j]  # Should get (3, 4)
print("Inner tuple at index", j, "is", inner)
print("First element of inner tuple is", inner[0])  # Should print 3
