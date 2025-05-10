print("── Test 0: empty list ──")
empty = []
print(empty)                # → []

print("\n── Test 1: homogeneous scalar lists ──")
ints     = [1, 2, 3]
strings  = ["x", "y", "z"]
floats   = [0.1, 0.2, 0.3]
print(ints)                 # → [1, 2, 3]
print(strings)              # → ["x", "y", "z"]
print(floats)               # → [0.1, 0.2, 0.3]

print("\n── Test 2: heterogeneous list ──")
mixed = [42, "answer", 3.14]
print(mixed)                # → [42, "answer", 3.14]

print("\n── Test 3: nested & deeply‑nested lists ──")
nested      = [ints, strings]
deep_nested = [nested, [ints], [[99]]]
print(nested)               # → [[1, 2, 3], ["x", "y", "z"]]
print(deep_nested)          # → [[[1, 2, 3], ["x", "y", "z"]], [[1, 2, 3]], [[99]]]

print("\n── Test 4: list mixed with scalars and lists ──")
combo = [ints, 1000, strings]
print(combo)                # → [[1, 2, 3], 1000, ["x", "y", "z"]]

print("\n── Test 5: comprehension – simple ──")
squares = [x * x for x in [1, 2, 3, 4]]
print(squares)              # → [1, 4, 9, 16]

print("\n── Test 6: comprehension with predicate ──")
evens = [x for x in [1, 2, 3, 4, 5, 6] if x % 2 == 0]
print(evens)                # → [2, 4, 6]

print("\n── Test 7: comprehension nested in list literal ──")
comp_mix = [ "start", [y for y in ["a", "bb", "ccc"]], 7 ]
print(comp_mix)             # → ["start", ["a", "bb", "ccc"], 7]

print("\n── Test 8: trailing comma acceptance ──")
trailing = [1, 2, 3, ]
print(trailing)             # → [1, 2, 3]

print("\n── Test 9: starred‑element unpacking ──")
first, *middle, last = [10, 11, 12, 13, 14]
print(first)                # → 10
print(middle)               # → [11, 12, 13]
print(last)                 # → 14

print("\n── Test 10: empty sub‑list inside list ──")
with_empty = [ [], [1], [] ]
print(with_empty)           # → [[], [1], []]

print("\n── Test 11: big list (performance / stack overflow) ──")
big = [x for x in [i for i in range(0, 2000)]]   # 2 000 elements
length_val = len(big)
first_three = big[:3]
print("Length:", length_val)   # → Length: 2000
print("First 3:", first_three)   # slice‑print sanity check
print("\n── All list tests done ──")
