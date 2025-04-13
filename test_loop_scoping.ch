# Loop scoping test

# Variable scoping in loops
sum = 0
i = 0
while i < 5:
    temp = i * 2
    sum = sum + temp
    i = i + 1
print("Variable scoping in loops:")
print(sum)  # Should be 20 (0*2 + 1*2 + 2*2 + 3*2 + 4*2)
