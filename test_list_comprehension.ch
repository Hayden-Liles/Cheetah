# Test list comprehension
numbers = [x for x in range(10)]
print(numbers)

# Test list comprehension with condition
even_numbers = [x for x in range(10) if x % 2 == 0]
print(even_numbers)

# Test list comprehension with function call
def square(x):
    return x * x

squares = [square(x) for x in range(10)]
print(squares)

# Test list comprehension with string formatting
formatted = ["Number: " + str(x) for x in range(5)]
print(formatted)
