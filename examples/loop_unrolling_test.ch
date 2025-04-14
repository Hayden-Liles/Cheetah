# Loop unrolling test for Cheetah

# This program tests the performance of loops with different iteration counts
# to demonstrate the benefits of loop unrolling

# Function to perform a computation-heavy task in a small loop
def small_loop_test():
    result = 0
    # This loop should be fully unrolled (8 iterations)
    for i in range(8):
        # Do some computation to make each iteration more significant
        result = result + i * i * i
    return result

# Function to perform a computation-heavy task in a medium loop
def medium_loop_test():
    result = 0
    # This loop should be partially unrolled (100 iterations)
    for i in range(100):
        # Do some computation to make each iteration more significant
        result = result + i * i
    return result

# Function to perform a computation-heavy task in a large loop
def large_loop_test():
    result = 0
    # This loop should be chunked (10000 iterations)
    for i in range(10000):
        # Do some computation to make each iteration more significant
        result = result + i
    return result

# Run each test multiple times to measure performance
print("Running loop unrolling tests...")

# Small loop test
print("\nSmall loop test (should be fully unrolled):")
small_result = small_loop_test()
print("Result:", small_result)

# Medium loop test
print("\nMedium loop test (should be partially unrolled):")
medium_result = medium_loop_test()
print("Result:", medium_result)

# Large loop test
print("\nLarge loop test (should be chunked):")
large_result = large_loop_test()
print("Result:", large_result)

# Benchmark by running multiple iterations
print("\nRunning benchmarks...")

# Small loop benchmark
print("\nSmall loop benchmark:")
iterations = 1000000
result = 0
for _ in range(iterations):
    result = result + small_loop_test()
print("Completed iterations:", iterations)
print("Result:", result)

# Medium loop benchmark
print("\nMedium loop benchmark:")
iterations = 10000
result = 0
for _ in range(iterations):
    result = result + medium_loop_test()
print("Completed iterations:", iterations)
print("Result:", result)

# Large loop benchmark
print("\nLarge loop benchmark:")
iterations = 100
result = 0
for _ in range(iterations):
    result = result + large_loop_test()
print("Completed iterations:", iterations)
print("Result:", result)

print("\nBenchmark complete!")
