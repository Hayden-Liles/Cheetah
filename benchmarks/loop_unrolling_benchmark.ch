# Loop unrolling benchmark

# Function with a small loop that should be fully unrolled
def small_loop_test():
    result = 0
    for i in range(8):  # Small loop with 8 iterations - should be fully unrolled
        result = result + i
    return result

# Function with a medium loop that should be partially unrolled
def medium_loop_test():
    result = 0
    for i in range(100):  # Medium loop - should be partially unrolled
        result = result + i
    return result

# Function with a large loop that should be chunked
def large_loop_test():
    result = 0
    for i in range(10000):  # Large loop - should be chunked
        result = result + i
    return result

# Run the tests and print the results
print("Small loop test result:")
print(small_loop_test())

print("Medium loop test result:")
print(medium_loop_test())

print("Large loop test result:")
print(large_loop_test())

# Benchmark by running multiple iterations
print("Running benchmarks...")

# Small loop benchmark
print("Small loop benchmark (should be fully unrolled):")
iterations = 1000000
result = 0
for _ in range(iterations):
    result = result + small_loop_test()
print("Completed iterations:")
print(iterations)
print("Result:")
print(result)

# Medium loop benchmark
print("Medium loop benchmark (should be partially unrolled):")
iterations = 10000
result = 0
for _ in range(iterations):
    result = result + medium_loop_test()
print("Completed iterations:")
print(iterations)
print("Result:")
print(result)

# Large loop benchmark
print("Large loop benchmark (should be chunked):")
iterations = 100
result = 0
for _ in range(iterations):
    result = result + large_loop_test()
print("Completed iterations:")
print(iterations)
print("Result:")
print(result)
