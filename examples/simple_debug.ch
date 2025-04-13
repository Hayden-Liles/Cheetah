# Simple debug file to test stack overflow issues

# Print a message to show we're starting
print("Starting simple debug test")

# Test with a small number of iterations first
print("Testing 10,000 iterations")
for i in range(10000):
    if i % 1000 == 0:
        print("Progress:", i)

print("Completed 10,000 iterations")

# Test with a larger number of iterations
print("Testing 100,000 iterations")
for i in range(100000):
    if i % 10000 == 0:
        print("Progress:", i)

print("Completed 100,000 iterations")

# Test with an even larger number of iterations
print("Testing 500,000 iterations")
for i in range(500000):
    if i % 50000 == 0:
        print("Progress:", i)

print("Completed 500,000 iterations")

# Final test with a very large number of iterations
print("Testing 1,000,000 iterations")
for i in range(1000000):
    if i % 100000 == 0:
        print("Progress:", i)

print("Completed 1,000,000 iterations")

print("All tests completed successfully")
