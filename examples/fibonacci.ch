def fibonacci(n):
    fib_sequence = [0, 1]
    print(fib_sequence[-1])
    print(fib_sequence[-2])
    while len(fib_sequence) < n:
        fib_sequence.append(fib_sequence[-1] + fib_sequence[-2])
    return fib_sequence

print(fibonacci(10000))