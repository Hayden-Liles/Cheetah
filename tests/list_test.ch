def main():
    # Create a list
    my_list = [1, 2, 3, 4, 5]
    
    # Print the list
    print("List:", my_list)
    
    # Get the length of the list
    print("Length:", len(my_list))
    
    # Access elements
    print("First element:", my_list[0])
    print("Last element:", my_list[4])
    
    # Append to the list
    my_list.append(6)
    print("After append:", my_list)
    
    # Concatenate lists
    other_list = [7, 8, 9]
    combined = my_list + other_list
    print("Combined list:", combined)
    
    # Slice the list
    slice = my_list[1:4]
    print("Slice [1:4]:", slice)
    
    # Repeat the list
    repeated = my_list * 2
    print("Repeated list:", repeated)
