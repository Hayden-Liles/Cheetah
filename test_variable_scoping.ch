# Variable scoping test

# Variable scoping in if statements
a = 1
b = 2
if a < b:
    x = 10
    y = 20
else:
    x = 30
    y = 40
print("Variable scoping in if statements:")
print(x)  # Should be 10
print(y)  # Should be 20
