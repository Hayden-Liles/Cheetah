# Test subscript operations

# Test list subscript
my_list = [10, 20, 30, 40, 50]
print("List:")
print(my_list)

list_0 = my_list[0]
print("List[0]:")
print(list_0)

list_2 = my_list[2]
print("List[2]:")
print(list_2)

# Test tuple subscript
my_tuple = (100, 200, 300, 400, 500)
print("Tuple:")
print(my_tuple)

tuple_0 = my_tuple[0]
print("Tuple[0]:")
print(tuple_0)

tuple_2 = my_tuple[2]
print("Tuple[2]:")
print(tuple_2)

# Test nested tuple
nested_tuple = ((1, 2), (3, 4), (5, 6))
print("Nested tuple:")
print(nested_tuple)

nested_1 = nested_tuple[1]
print("Nested tuple[1]:")
print(nested_1)

nested_1_0 = nested_tuple[1][0]
print("Nested tuple[1][0]:")
print(nested_1_0)
