def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def main():
    print("Computing Fibonacci(30)...")
    start_time = __time__()
    result = fibonacci(30)
    end_time = __time__()
    print(f"Result: {result}")
    print(f"Time: {end_time - start_time:.6f} seconds")
