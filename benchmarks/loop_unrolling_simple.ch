# Simple loop unrolling benchmark

# Small loop that should be fully unrolled
print("Small loop (should be fully unrolled):")
result = 0
for i in range(8):  # Small loop with 8 iterations
    result = result + i
print("Result:")
print(result)

# Medium loop that should be partially unrolled
print("Medium loop (should be partially unrolled):")
result = 0
for i in range(100):  # Medium loop
    result = result + i
print("Result:")
print(result)

# Large loop that should be chunked
print("Large loop (should be chunked):")
result = 0
for i in range(10000):  # Large loop
    result = result + i
print("Result:")
print(result)

# Print a separator to make the output more visible
print("=======================")
print("BENCHMARK COMPLETED")
print("=======================")
