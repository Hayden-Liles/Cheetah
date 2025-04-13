# Dictionary conditional values test

a = 1
b = 2

# Dictionary with conditional values
if a < b:
    comparison_value = "a < b"
else:
    comparison_value = "a >= b"

data = {
    "comparison": comparison_value
}
print("Dictionary with conditional values:")
print(data["comparison"])  # Should be "a < b"
