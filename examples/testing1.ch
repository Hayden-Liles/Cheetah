def fibonacci(n):
    seq = [0] * n  # Pre-allocate the list
    a, b = 0, 1
    for i in range(n):
        seq[i] = a  # Use indexing instead of append
        a, b = b, a + b
    return seq

fib_sequence = fibonacci(10)
print("Fibonacci sequence:")
for i in range(1_000_000):
    print(fib_sequence[i])