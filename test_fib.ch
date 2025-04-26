def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def main():
    print("Computing Fibonacci(20)...")
    result = fibonacci(20)
    print("Result:", result)

main()
