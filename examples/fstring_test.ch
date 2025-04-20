name = "Alice"
age = 30
is_active = True

# Print the variables to verify their values
print("name = " + name)
print("age = " + str(age))
print("is_active = " + str(is_active))

# Basic string interpolation
print(f"User {name} is {age} years old.")

# Multiple values
print(f"Name: {name}, Age: {age}")

# Expressions in f-strings
print(f"Next year you will be {age + 1} years old")

# Boolean values
print(f"Active status: {is_active}")

# Nested expressions
print(f"Complex calculation: {2 * (3 + 4)}")

# Multiple interpolations
print(f"First: {1}, Second: {2}, Third: {3}")
