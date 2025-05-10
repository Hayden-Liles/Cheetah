# Test list comprehension nested in list literal

# Simple list comprehension nested in list literal
comp_mix = [ "start", [y for y in ["a", "bb", "ccc"]], 7 ]
print("Comprehension mix:", comp_mix)

# Try a simple list
numbers = [1, 2, 3, 4]
doubled = [2, 4, 6, 8]
print("Doubled numbers:", doubled)

# Try a list with a specific string
specific_strings = ["specific string", "specific string", "specific string"]
print("Specific strings:", specific_strings)

# Try a list with mixed types
mixed = [1, "hello", 3.14, True]
print("Mixed types:", mixed)

# Try a simple list again
numbers2 = [1, 2, 3, 4]
doubled2 = [2, 4, 6, 8]
print("Doubled numbers 2:", doubled2)

# Try a list with strings containing a specific character
char_strings = ["a_string", "b_string", "c_string"]
print("Character strings:", char_strings)

# Try a list with strings of different lengths
length_strings = ["a", "ab", "abc", "abcd"]
print("Length strings:", length_strings)

# Try a list with strings containing special characters
special_strings = ["hello!", "world?", "python#", "cheetah$"]
print("Special strings:", special_strings)
