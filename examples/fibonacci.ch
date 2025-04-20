def fibonacci(n):
    """Generate Fibonacci sequence up to n terms."""
    fib_sequence = []
    a, b = 0, 1
    for _ in range(n):
        fib_sequence.append(a)
        a, b = b, a + b
    return fib_sequence

# Generate and print the first 10 Fibonacci numbers
print(fibonacci(10))