# Memory optimization test
# This test focuses on range iterator optimization

# Main function
print("Starting memory optimization test...")

# Test with small range
print("Testing small range (10)")
small_sum = 0
for i in range(10):
    small_sum += i
print("Small range sum: " + str(small_sum))

# Test with medium range
print("Testing medium range (10000)")
medium_sum = 0
for i in range(10000):
    medium_sum += i
print("Medium range sum: " + str(medium_sum))

# Test with large range
print("Testing large range (1000000)")
large_sum = 0
for i in range(1000000):
    large_sum += i
print("Large range sum: " + str(large_sum))

# Test with very large range
print("Testing very large range (10000000)")
very_large_sum = 0
for i in range(10000000):
    very_large_sum += i
print("Very large range sum: " + str(very_large_sum))

print("Memory test completed!")
