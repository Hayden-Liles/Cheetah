# Test file to diagnose segmentation fault

# First, test a simple print statement
print("Starting test")

# Then, initialize a variable
total = 0
print("Initialized total:", total)

# Now try a simple assignment without a loop
total = 42
print("After assignment:", total)

# Finally, try a very simple loop
print("About to enter loop")
for i in range(1):
    print("Inside loop, i =", i)
    total = total + 1

print("Final total:", total)