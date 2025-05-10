print("\n── List Comprehensions ──")
# Simple list comprehension
squares = [x * x for x in [1, 2, 3, 4]]
print(squares)  # → [1, 4, 9, 16]

# List comprehension with predicate
evens = [x for x in [1, 2, 3, 4, 5, 6] if x % 2 == 0]
print(evens)  # → [2, 4, 6]
