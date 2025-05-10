print("\n── Test: List element access ──")
my_list = [10, 11, 12, 13, 14]
print("First element:", my_list[0])  # Should print 10
print("Last element:", my_list[4])   # Should print 14

print("\n── Test: Starred-element unpacking ──")
first, *middle, last = [10, 11, 12, 13, 14]
print("first:", first)      # Should print 10
print("middle:", middle)    # Should print [11, 12, 13]
print("last:", last)        # Should print 14
