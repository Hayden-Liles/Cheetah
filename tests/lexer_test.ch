# This is a sample Cheetah program
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

# Calculate factorial of 5
result = factorial(5)
print("Factorial of 5 is", result)

# Simple if-else example
x = 10
if x > 5:
    print("x is greater than 5")
    if x > 8:
        print("x is also greater than 8")
else:
    print("x is less than or equal to 5")

# Loop example
for i in range(5):
    print("Iteration", i)