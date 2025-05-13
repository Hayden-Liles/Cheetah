print("\n── Test 11: big list (performance / stack overflow) ──")
big = [x for x in [i for i in range(0, 2000)]]   # 2 000 elements
length_val = len(big)
first_three = big[:3]
print("Length:", length_val)   # → Length: 2000
print(big)
print("First 3:", first_three)   # slice‑print sanity check
print("\n── All list tests done ──")
