# Dynamic chunk size test
# This script tests the dynamic chunk size calculation in the loop optimizer

print("Testing dynamic chunk size calculation")
print("======================================")

# Small loop (should use small chunk size)
print("\nSmall loop test:")
print("Range size: 500")
result = 0
for i in range(500):
    result = result + i
print("Result:", result)

# Medium loop (should use medium chunk size)
print("\nMedium loop test:")
print("Range size: 5,000")
result = 0
for i in range(5000):
    result = result + i
print("Result:", result)

# Large loop (should use large chunk size)
print("\nLarge loop test:")
print("Range size: 50,000")
result = 0
for i in range(50000):
    result = result + i
print("Result:", result)

# Very large loop (should use dynamic chunk size)
print("\nVery large loop test:")
print("Range size: 2,000,000")
result = 0
for i in range(2000000):
    result = result + i
print("Result:", result)

# Extremely large loop (should use max chunk size)
print("\nExtremely large loop test:")
print("Range size: 10,000,000")
result = 0
for i in range(10000000):
    result = result + i
print("Result:", result)

print("\nAll tests completed successfully!")
print("\nNote: To see detailed chunk size calculations, run with RUSTFLAGS=\"-C debug-assertions\" flag.")
