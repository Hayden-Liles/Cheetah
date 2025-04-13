# Simple program to debug stack overflow issues
# This will print numbers with debugging information

# Use a smaller range to avoid stack overflow
for i in range(100000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)

# Now use a second loop to continue where the first left off
for i in range(100000, 200000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)

# Use a third loop to continue where the second left off
for i in range(200000, 300000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)

# Use a fourth loop to continue where the third left off
for i in range(300000, 400000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)

# Use a fifth loop to continue where the fourth left off
for i in range(400000, 500000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)

# Use a sixth loop to continue where the fifth left off
for i in range(500000, 600000):
    # Only print every 1000 numbers to reduce output volume
    if i % 1000 == 0:
        print(i)
