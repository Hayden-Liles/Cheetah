print("\n── Test: Simple list comprehension ──")
# Simple list comprehension
simple_list = [i for i in range(0, 10)]
print("Simple list:", simple_list)

print("\n── Test: List comprehension with inner list ──")
# List comprehension with inner list
inner_list = [i for i in range(0, 10)]
outer_list = [x for x in inner_list]
print("Outer list:", outer_list)

print("\n── All list tests done ──")
