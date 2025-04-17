# 16500000 + is where seg fault happens

total = 0
for i in range(10):  # Reduced range to avoid immediate crash
    total += i

# Only print the final result
print(f"Final sum: {total}")