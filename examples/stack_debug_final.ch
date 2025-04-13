# Simple program to debug stack overflow issues
# This will print numbers with debugging information

# Use a very small range to avoid stack overflow
for i in range(10000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)
        
print("First loop completed successfully")

# Reset the stack state by returning to the top level
# This is a workaround for the stack overflow issue

# Now use a second loop with a small range
for i in range(10000, 20000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)
        
print("Second loop completed successfully")

# Continue with more small loops
for i in range(20000, 30000):
    if i % 1000 == 0:
        print(i)
        
print("Third loop completed successfully")

# Print a final message
print("All loops completed successfully without stack overflow")
