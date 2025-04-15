# 16500000 + is where seg fault happens

total = 0
for i in range(17000000):
    total += i
    
# Only print the final result
print(f"Final sum: {total}")