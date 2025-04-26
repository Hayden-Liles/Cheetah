# Test the bigint_fib function directly

print("Testing Fibonacci function with large numbers:")

# Test F(100)
print("\nF(100) =")
print(fibonacci(100))  # Should be 354224848179261915075

# Test F(200)
print("\nF(200) =")
print(fibonacci(200))  # Should be 280571172992510140037611932413038677189525

# Test F(500)
print("\nF(500) =")
print(fibonacci(500))  # Very large number

# Test F(1000)
print("\nF(1000) =")
print(fibonacci(1000))  # Extremely large number

def fibonacci(n):
    if n <= 0:
        return 0
    if n == 1:
        return 1
    
    a = 0
    b = 1
    i = 2
    
    while i <= n:
        c = a + b
        a = b
        b = c
        i = i + 1
    
    return b
