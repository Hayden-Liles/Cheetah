# Debug script to help diagnose stack overflow issues
# This script gradually increases the number of iterations to find the breaking point

# Start with a small number of iterations
print("Starting stack overflow debugging")
print("=================================")

# Function to test a specific number of iterations
def test_iterations(count):
    print("Testing", count, "iterations...")

    # Run the loop
    for i in range(count):
        # Only print every 10,000 iterations to reduce output
        if i % 10000 == 0:
            print("Progress:", i)

    # If we get here, the test passed
    print("SUCCESS: Completed", count, "iterations")
    return 1  # Return an integer instead of a boolean

# Test with increasing iteration counts
iterations = [
    10000,      # 10K
    50000,      # 50K
    100000,     # 100K
    200000,     # 200K
    300000,     # 300K
    400000,     # 400K
    500000,     # 500K
    600000,     # 600K
    700000,     # 700K
    800000,     # 800K
    900000,     # 900K
    1000000     # 1M
]

# Run tests with increasing iteration counts
for count in iterations:
    success = test_iterations(count)
    if success == 0:
        print("Stack overflow detected at", count, "iterations")
        break

    # Add a separator between tests
    print("---------------------------------")

print("Stack overflow debugging complete")
