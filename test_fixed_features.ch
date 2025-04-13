# Test file demonstrating fixed features

# 1. Comparison chains
a = 1
b = 2
c = 3
result1 = a < b < c  # Should be True
print("Comparison chains:")
print(result1)

# 2. Compound conditions
result2 = a < b and b < c
print("\nCompound conditions:")
print(result2)

# 3. Variable scoping in if statements
if a < b:
    x = 10
    y = 20
else:
    x = 30
    y = 40
print("\nVariable scoping in if statements:")
print(x)  # Should be 10
print(y)  # Should be 20

# 4. Variable scoping in loops
sum = 0
i = 0
while i < 5:
    temp = i * 2
    sum = sum + temp
    i = i + 1
print("\nVariable scoping in loops:")
print(sum)  # Should be 20 (0*2 + 1*2 + 2*2 + 3*2 + 4*2)

# 5. Dictionary with conditional values
if a < b:
    comparison_value = "a < b"
else:
    comparison_value = "a >= b"

data = {
    "comparison": comparison_value
}
print("\nDictionary with conditional values:")
print(data["comparison"])  # Should be "a < b"
