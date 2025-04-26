def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def main():
    print("Computing Fibonacci(30)...")
    result = fibonacci(30)
    print(f"Result: {result}")

main()
