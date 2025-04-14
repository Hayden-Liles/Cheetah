# Optimization Demo
# This file demonstrates the memory optimizations in the Cheetah language

# Starting optimization demo
print("Starting optimization demo...")

# Test range iterator optimization
print("Testing range iterator optimization...")
print("Iterating through a range of 10,000,000 elements")

# Create a large range and iterate through it
sum = 0
for i in range(10000000):
    sum += i

# Output the result
print("Range sum:")
print(sum)

# Test chunking optimization
print("Testing chunking optimization...")
print("Performing chunked computation on 1,000,000 elements")

# Create a large computation that benefits from chunking
result = 0
for i in range(1000000):
    # Do some computation that benefits from chunking
    if i % 2 == 0:
        result += i
    else:
        result -= i

print("Chunked computation result:")
print(result)

print("All optimization tests completed!")
