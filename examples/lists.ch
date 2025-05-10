print("\n── Test 7: comprehension nested in list literal ──")
comp_mix = [ "start", [y for y in ["a", "bb", "ccc"]], 7 ]
print(comp_mix)             # → ["start", ["a", "bb", "ccc"], 7]