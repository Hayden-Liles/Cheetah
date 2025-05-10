print("\n── Test 1: Simple list comprehension ──")
squares = [x * x for x in [1, 2, 3, 4]]
print(squares)  # → [1, 4, 9, 16]

print("\n── Test 2: List comprehension with predicate ──")
evens = [x for x in [1, 2, 3, 4, 5, 6] if x % 2 == 0]
print(evens)  # → [2, 4, 6]

print("\n── Test 3: List comprehension with addition ──")
plus_one = [x + 1 for x in [1, 2, 3, 4]]
print(plus_one)  # → [2, 3, 4, 5]

print("\n── Test 4: List comprehension with subtraction ──")
minus_one = [x - 1 for x in [1, 2, 3, 4]]
print(minus_one)  # → [0, 1, 2, 3]

print("\n── Test 5: List comprehension with division ──")
halves = [x / 2 for x in [2, 4, 6, 8]]
print(halves)  # → [1.0, 2.0, 3.0, 4.0]

print("\n── Test 6: List comprehension with complex expression ──")
complex_expr = [x * x + 2 * x + 1 for x in [1, 2, 3, 4]]
print(complex_expr)  # → [4, 9, 16, 25]
