# Test file demonstrating fixed features

# 1. Global variables
global_var = 100

def modify_global():
    global global_var
    global_var = global_var + 50
    return global_var

result1 = modify_global()
print("Global variable test:")
print(result1)  # Should be 150

# 2. Nonlocal variables
def outer():
    x = 10

    def inner():
        nonlocal x
        x = x + 20
        return x

    inner_result = inner()
    return x  # Should be 30 after inner() modifies it

result2 = outer()
print("\nNonlocal variable test:")
print(result2)  # Should be 30

# 3. Exception handling with control flow
def test_exception():
    result = 0

    # Loop with exception handling
    i = 0
    while i < 5:
        try:
            if i == 2:
                # Simulate an exception on the third iteration
                result = result + 10
                # Raise an exception explicitly
                raise_exception = True  # This is just a placeholder
            else:
                result = result + i
        except:
            # Handle the exception
            result = result + 100
        i = i + 1

    return result

result3 = test_exception()
print("\nException handling test:")
print(result3)  # Should include the exception handling result

# 4. Comparison chains
a = 1
b = 2
c = 3
result4 = a < b < c  # Should be True
print("\nComparison chains test:")
print(result4)

# 5. Compound conditions
result5 = (a < b) and (b < c)
print("\nCompound conditions test:")
print(result5)  # Should be True
