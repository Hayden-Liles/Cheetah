# Test big integer operations

# Test with small numbers (fits in i64)
a = 1234567890
b = 9876543210
print("a =")
print(a)
print("b =")
print(b)
print("a + b =")
print(a + b)
print("a - b =")
print(a - b)
print("a * b =")
print(a * b)
print("a / b =")
print(a / b)

# Test with larger numbers (requires BigInt)
c = 9223372036854775807  # i64::MAX
print("\nc =")
print(c)
print("c + 1 =")
print(c + 1)
print("c + c =")
print(c + c)
print("c * 2 =")
print(c * 2)

# Test with a simple Fibonacci calculation
print("\nFibonacci numbers:")
fib0 = 0
fib1 = 1
print("F(0) =")
print(fib0)
print("F(1) =")
print(fib1)

# Calculate F(2)
fib2 = fib0 + fib1
print("F(2) =")
print(fib2)

# Calculate F(3)
fib3 = fib1 + fib2
print("F(3) =")
print(fib3)

# Calculate F(4)
fib4 = fib2 + fib3
print("F(4) =")
print(fib4)

# Calculate F(5)
fib5 = fib3 + fib4
print("F(5) =")
print(fib5)

# Calculate F(50)
print("\nCalculating F(50)...")
fib_a = 0
fib_b = 1
fib_i = 2
while fib_i <= 50:
    fib_temp = fib_a
    fib_a = fib_b
    fib_b = fib_temp + fib_b
    fib_i = fib_i + 1

print("F(50) =")
print(fib_b)

# Calculate F(100)
print("\nCalculating F(100)...")
fib_a = 0
fib_b = 1
fib_i = 2
while fib_i <= 100:
    fib_temp = fib_a
    fib_a = fib_b
    fib_b = fib_temp + fib_b
    fib_i = fib_i + 1

print("F(100) =")
print(fib_b)

# Calculate F(200)
print("\nCalculating F(200)...")
fib_a = 0
fib_b = 1
fib_i = 2
while fib_i <= 200:
    fib_temp = fib_a
    fib_a = fib_b
    fib_b = fib_temp + fib_b
    fib_i = fib_i + 1

print("F(200) =")
print(fib_b)
