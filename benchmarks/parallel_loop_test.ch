# Parallel loop test
# This script tests the parallel loop processing capabilities

print("Testing parallel loop processing")
print("===============================")

# Function to perform a CPU-intensive operation
def expensive_operation(n):
    result = 0
    for i in range(1000):  # Simulate work
        result = result + (n * i) % 1000
    return result

# Sequential processing of a small range (should not be parallelized)
print("\nSequential processing (small range):")
result = 0
for i in range(100):
    result = result + expensive_operation(i)
print("Result:", result)

# Parallel processing of a medium range (should be parallelized)
print("\nParallel processing (medium range):")
result = 0
for i in range(2000):
    result = result + expensive_operation(i)
print("Result:", result)

# Parallel processing of a large range (should be parallelized)
print("\nParallel processing (large range):")
result = 0
for i in range(10000):
    result = result + expensive_operation(i)
print("Result:", result)

# Parallel processing with a list
print("\nParallel processing with a list:")
numbers = []
for i in range(5000):
    numbers.append(i)

result = 0
for num in numbers:
    result = result + expensive_operation(num)
print("Result:", result)

print("\nAll tests completed successfully!")
print("\nNote: To see parallel processing statistics, check the output after program completion.")
