print("Creating nested list comprehension...")
big = [x for x in [i for i in range(0, 2_000_000)]]   # 2_000_000 elements
length_val = len(big)
first_three = big[:3]
print("Length:", length_val)
print("First 3:", first_three)

# Test direct range list creation for comparison
print("\nCreating direct range list...")
direct = [i for i in range(0, 2_000_000)]
print("Direct Length:", len(direct))
print("Direct First 3:", direct[:3])

print("\n── All list tests done ──")
