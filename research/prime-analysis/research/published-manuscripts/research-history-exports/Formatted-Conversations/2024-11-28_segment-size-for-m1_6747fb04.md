# Segment Size for M1

- **Conversation ID:** `6747fb04-8060-8001-b23d-535d9b52fe37`
- **Date:** 2024-11-28

---

### User (2024-11-28 00:09:24)

what segment size should you use with python code on an m1 macbook pro with 16gb of ram?

### Assistant (2024-11-28 00:09:24)

The segment size to use in Python code on an M1 MacBook Pro with 16GB of RAM depends on the type of data and computation. Here are general guidelines for various scenarios:

### 1. **Data Processing**
   - **Pandas DataFrames**: Keep chunk sizes between **500,000 to 1,000,000 rows** for efficient processing, depending on the complexity of operations.
   - **Large Files (e.g., CSVs)**: Use chunks of **10MB to 100MB**. For larger datasets, monitor memory usage and adjust.

### 2. **Machine Learning**
   - If you're using frameworks like TensorFlow or PyTorch, the batch size should fit the VRAM of the GPU (8-core GPU in M1). Start with **batch sizes of 32 or 64**, then scale based on memory usage.
   - For CPU-only operations, batch sizes of **128 or 256** work well unless the model is very large.

### 3. **Numerical Computations (NumPy/Scipy)**
   - Divide computations into **1GB to 2GB chunks** to leave enough memory for OS and other applications.
   - Monitor memory using `psutil` or `tracemalloc` if running iterative processes.

### 4. **Parallel Processing**
   - The M1's unified memory allows multiple cores to share memory efficiently. Divide work into **4 to 8 segments**, aligning with the number of CPU performance cores (4 on the M1).

### 5. **Customizable Approach**
   - Use `multiprocessing` or `joblib` to dynamically determine chunk sizes:
     ```python
     import multiprocessing
     available_memory = 16 * (2**30)  # 16 GB in bytes
     num_cores = multiprocessing.cpu_count()
     chunk_size = available_memory // num_cores // 10  # Reserve ~10% per core
     ```

### Tips:
   - Use monitoring tools like `Activity Monitor` to observe real-time memory usage.
   - Avoid setting chunk sizes too high, as it can lead to memory swapping, significantly slowing down operations.
   - Use memory-efficient libraries such as Dask or Vaex for large-scale processing.

### User (2024-11-28 00:11:52)

what is an optimal segment size for the prime number program we wrote if I am running it on an m1 macbook pro with 16gb of ram? If I am using it to sieve prime numbers upto 10 billion?

### Assistant (2024-11-28 00:11:53)

For sieving prime numbers up to **10 billion** on an M1 MacBook Pro with 16GB of RAM, the optimal segment size should balance memory usage and processing efficiency. Here’s how to estimate and choose the segment size:

---

### Key Considerations:
1. **Memory Requirements for the Sieve**:
   - A sieve needs an array of booleans to mark prime and non-prime numbers.
   - Each boolean typically requires **1 bit**. To sieve up to \( N \):
     \[
     \text{Memory required} = \frac{N}{8} \text{ bytes}.
     \]
   - For \( N = 10^{10} \):
     \[
     \text{Memory required} = \frac{10^{10}}{8} = 1.25 \, \text{GB}.
     \]
   - This fits comfortably in your 16GB RAM, but if you use a segmented sieve, each segment will require only a fraction of this.

2. **Segmented Sieving**:
   - For a segmented sieve, you process the range in chunks (segments).
   - Smaller segments reduce memory usage but increase I/O and computation overhead.
   - Larger segments improve computational efficiency but require more memory.

3. **Performance of M1 Architecture**:
   - Unified memory makes memory operations faster.
   - Multi-core performance allows parallelism, so segment sizes can be larger.

---

### Recommended Segment Size:
For sieving up to \( 10^{10} \), aim for a segment size that uses **~1GB of memory per segment**. This balances memory usage and computational efficiency.

   - **Segment Size Calculation**:
     \[
     \text{Segment Size} = 10^{10} \times \frac{\text{1GB}}{\text{Total Memory for Full Sieve (1.25GB)}} = \frac{10^{10}}{1.25 \times 10^{10}} \times 10^9 \, \text{numbers per segment}.
     \]
     Each segment should cover approximately **800 million numbers**.

   - In bytes:
     \[
     \text{Memory per segment} = \frac{800 \times 10^6}{8} = 100 \, \text{MB per segment}.
     \]

---

### Practical Implementation:
#### 1. **Segment Size in Code**:
Set a segment size of **800 million numbers** or approximately **100MB per segment** for a boolean array.

#### 2. **Example Python Code for Segmented Sieve**:
```python
def segmented_sieve(limit, segment_size=800_000_000):
    import math

    primes = []
    sqrt_limit = int(math.sqrt(limit)) + 1
    sieve = [True] * sqrt_limit

    # Regular sieve to find small primes up to sqrt(limit)
    for i in range(2, sqrt_limit):
        if sieve[i]:
            primes.append(i)
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    # Segmented sieve
    low = sqrt_limit
    high = low + segment_size

    while low < limit:
        if high > limit:
            high = limit

        segment = [True] * (high - low)

        for prime in primes:
            # Find the starting point in the current segment
            start = max(prime * prime, (low + prime - 1) // prime * prime)

            for j in range(start, high, prime):
                segment[j - low] = False

        # Collect primes in this segment
        for i in range(high - low):
            if segment[i]:
                primes.append(low + i)

        low += segment_size
        high += segment_size

    return primes

# Example usage:
primes_up_to_10_billion = segmented_sieve(10_000_000_000)
```

---

### Additional Optimization Tips:
1. **Parallel Processing**:
   - Split segments into chunks for parallel execution using `multiprocessing.Pool`.

2. **Memory Monitoring**:
   - Use a tool like `psutil` to monitor memory usage during execution.

3. **Disk Storage**:
   - If you save primes to disk for later use, write them to a file incrementally after processing each segment.

---

This setup ensures efficient sieving while keeping memory usage well within the M1's capacity.

### User (2024-11-28 00:18:20)

using numpy and bitarray should I reconsider the segment size for my machine? or is 800 million fine?

### Assistant (2024-11-28 00:18:21)

Using `numpy` and `bitarray` can significantly improve memory efficiency and processing speed, which could allow for reconsidering the segment size on your M1 MacBook Pro. Here's a detailed breakdown:

---

### **Memory Considerations with `numpy` and `bitarray`:**
1. **Memory Usage:**
   - `bitarray` uses **1 bit per number**, compared to a boolean array in `numpy`, which typically uses 1 byte per number (8 bits). This reduces memory requirements by a factor of 8.
   - With `bitarray`, the memory required for a segment of 800 million numbers is:
     \[
     \text{Memory} = \frac{800 \times 10^6}{8} = 100 \, \text{MB}.
     \]

2. **Comparison to Full Sieve:**
   - Sieving up to \(10^{10}\) requires \( \frac{10^{10}}{8} = 1.25 \, \text{GB} \), which is feasible for your 16GB of RAM. However, for a segmented sieve, a segment size of **800 million** numbers (~100MB) is still highly efficient and leaves ample memory for computation.

3. **Larger Segment Size:**
   - With `bitarray`, increasing the segment size is more feasible because of reduced memory usage. For example:
     - Doubling to **1.6 billion numbers** requires:
       \[
       \text{Memory} = \frac{1.6 \times 10^9}{8} = 200 \, \text{MB}.
       \]
     - This is still manageable and reduces I/O overhead.

---

### **Optimal Segment Size with `numpy` + `bitarray`:**
- **800 million** numbers is a safe default.
- You could increase it up to **1.6 billion** numbers if:
  - You're not running other memory-intensive applications.
  - You want to reduce the number of segments processed.

---

### **Implementation Adjustments for `bitarray`:**
Here’s an example of a `segmented_sieve` using `bitarray`:

```python
from bitarray import bitarray
import math

def segmented_sieve(limit, segment_size=800_000_000):
    primes = []
    sqrt_limit = int(math.sqrt(limit)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    # Regular sieve to find small primes up to sqrt(limit)
    for i in range(2, sqrt_limit):
        if sieve[i]:
            primes.append(i)
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    # Segmented sieve
    low = sqrt_limit
    high = low + segment_size

    while low < limit:
        if high > limit:
            high = limit

        segment = bitarray(high - low)
        segment.setall(True)

        for prime in primes:
            # Find the starting point in the current segment
            start = max(prime * prime, (low + prime - 1) // prime * prime)

            for j in range(start, high, prime):
                segment[j - low] = False

        # Collect primes in this segment
        for i in range(high - low):
            if segment[i]:
                primes.append(low + i)

        low += segment_size
        high += segment_size

    return primes

# Example usage:
primes_up_to_10_billion = segmented_sieve(10_000_000_000, segment_size=1_600_000_000)
```

---

### **Pros and Cons of Increasing Segment Size:**
#### Pros:
- Fewer iterations and less overhead for segment transitions.
- Faster execution due to reduced I/O and memory swapping.

#### Cons:
- Larger segments could exhaust memory if combined with other applications or processes.
- Slightly higher computation time for managing larger segments.

---

### **Final Recommendation:**
- Stick with **800 million numbers** for a balanced approach.
- If you monitor memory usage and see it remains consistently low, increase the segment size incrementally to **1.6 billion** for better performance.
- Use tools like `psutil` or `memory_profiler` to test the impact of your segment size on memory usage.

### User (2024-11-28 00:21:44)

Ok I am testing 10 billion with a segment size of 800 mil

### Assistant (2024-11-28 00:21:44)

Great! Testing with a segment size of **800 million** is a solid choice, especially for sieving up to **10 billion**. With this segment size, the program should balance memory usage and efficiency quite well on your M1 MacBook Pro.

---

### **Things to Monitor During Testing:**

1. **Memory Usage**:
   - Ensure memory usage stays within safe limits (e.g., under 80% of available RAM). Use tools like `Activity Monitor` or Python’s `psutil` module.

2. **Execution Time**:
   - Note the time it takes to complete the sieve. This will help you evaluate whether adjustments to the segment size could improve performance.

3. **CPU Utilization**:
   - Ensure your CPU cores are utilized efficiently. The M1's performance cores should handle this task well, but you may consider parallelizing across segments if needed.

---

### **Troubleshooting Tips:**

1. **If Memory Usage Is Too High**:
   - Reduce the segment size to **400 million** or smaller.
   - Consider optimizing the sieve logic further to reduce overhead.

2. **If Execution Time Feels Slow**:
   - Increase the segment size incrementally, e.g., to **1.2 billion** or **1.6 billion**, if memory usage allows.
   - Add multiprocessing to process multiple segments concurrently.

3. **Debugging**:
   - Validate intermediate outputs (e.g., primes from smaller ranges) to ensure correctness before running the full sieve.

---

### **Sample Output Validation**:
After your program completes, you can validate the correctness of the sieve by checking known prime counts. For example:
- The total number of primes up to \(10^{10}\) is **455,052,511**.
- Use a smaller range (e.g., up to \(10^8\)) and verify against known data to ensure your implementation is accurate.

---

Let me know how it goes or if you encounter any challenges during the test! 🚀

### User (2024-11-28 00:28:12)

<windows>
<instructions>
You are being provided with textfield content from windows the user has asked you to focus on.
If the content of a textfield is too long to include, it will be truncated and replaced with <truncated__content /> (this tag is not part of the original content).
</instructions>
<window>
<title>Downloads — ubuntu@ip-10-40-0-39: ~ — python3 fasterprimes.py — 204×54</title>
<app_name>Terminal</app_name>
<textfields>
<textfield id="0"><truncated__content/>    segment_primes = [low + i for i in range(segment_length) if is_prime_segment[i] and (low + i) % 6 in [1, 5]]
  File "/Users/adminamn/Downloads/fasterprimes.py", line 74, in <listcomp>
    segment_primes = [low + i for i in range(segment_length) if is_prime_segment[i] and (low + i) % 6 in [1, 5]]
KeyboardInterrupt

(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 10000
^CTraceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 87, in <module>
    primes = compute_primes_segmented(M, segment_size)
  File "/Users/adminamn/Downloads/fasterprimes.py", line 66, in compute_primes_segmented
    start = max(P * P, ((low + P - 1) // P) * P)
KeyboardInterrupt

(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000
Enter the segment size (e.g., 1000000): 1000000
Number of primes up to 100000000: 5761455
Time taken: 9.03 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000
Enter the segment size (e.g., 1000000): 1000000
Number of primes up to 100000000: 5761455
Time taken: 9.01 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000
Enter the segment size (e.g., 1000000): 100000
Number of primes up to 100000000: 5761455
Time taken: 9.31 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000
Enter the segment size (e.g., 1000000): 10000000
Number of primes up to 100000000: 5761455
Time taken: 8.91 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000
Enter the segment size (e.g., 1000000): 100000000
Number of primes up to 100000000: 5761455
Time taken: 8.87 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 1000000000
Number of primes up to 1000000000: 50847534
Time taken: 107.77 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 10000 
Number of primes up to 1000000000: 50847534
Time taken: 181.35 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 10000000000
Number of primes up to 1000000000: 50847534
Time taken: 108.16 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 1000000  
^CTraceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 87, in <module>
    primes = compute_primes_segmented(M, segment_size)
  File "/Users/adminamn/Downloads/fasterprimes.py", line 71, in compute_primes_segmented
    is_prime_segment[idx] = False
KeyboardInterrupt

(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 100000000
Number of primes up to 1000000000: 50847534
Time taken: 93.51 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 10000000
Number of primes up to 1000000000: 50847534
Time taken: 92.83 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 1000000
Number of primes up to 1000000000: 50847534
Time taken: 93.77 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 134217728
Number of primes up to 1000000000: 50847534
Time taken: 94.77 seconds
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 1000000): 16777216
Number of primes up to 1000000000: 50847534
Time taken: 92.60 seconds
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py 
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
  File "/Users/adminamn/Downloads/fasterprimes.py", line 86
    if segment_size < 1000
                          ^
SyntaxError: expected ':'
(base) adminamn@MacBook-Pro Downloads % 
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
  File "/Users/adminamn/Downloads/fasterprimes.py", line 86
    if segment_size < 1000
                          ^
SyntaxError: expected ':'
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
  File "/Users/adminamn/Downloads/fasterprimes.py", line 95
    print(f"Last Prime" + str(primes.pop())
         ^
SyntaxError: '(' was never closed
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (e.g., 16777216): 1
Number of primes up to 1000000000: 50847534
Time taken: 93.49 seconds
Last Prime999999937
(base) adminamn@MacBook-Pro Downloads % 
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
  File "/Users/adminamn/Downloads/fasterprimes.py", line 86
    if str(segment_size) = "":
       ^^^^^^^^^^^^^^^^^
SyntaxError: cannot assign to function call here. Maybe you meant '==' instead of '='?
(base) adminamn@MacBook-Pro Downloads % 
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (Default -  16777216): 
Traceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 85, in <module>
    segment_size = int(input("Enter the segment size (Default -  16777216): "))
ValueError: invalid literal for int() with base 10: ''
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (Default -  16777216): 
Number of primes up to 1000000000: 50847534
Time taken: 92.56 seconds
Last Prime Found: 999999937
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (Default: 16777216): 
^CTraceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 89, in <module>
    primes = compute_primes_segmented(M, segment_size)
  File "/Users/adminamn/Downloads/fasterprimes.py", line 71, in compute_primes_segmented
    is_prime_segment[idx] = False
KeyboardInterrupt

(base) adminamn@MacBook-Pro Downloads % vi primes.py     
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000
Enter the segment size (Default: 16777216): 
Number of primes up to 1000: 168
Time taken: 0.00 seconds
Last Prime Found: 997
Traceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 96, in <module>
    print(fstr(math.ceil(math.log(M, 2))) + " bits required for Max " + str(M))
NameError: name 'fstr' is not defined. Did you mean: 'str'?
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000
Enter the segment size (Default: 16777216): 
Number of primes up to 1000: 168
Time taken: 0.00 seconds
Last Prime Found: 997
10 bits required for Max 1000
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 1000000000
Enter the segment size (Default: 16777216): 
Number of primes up to 1000000000: 50847534
Time taken: 92.49 seconds
Last Prime Found: 999999937
30 bits required for Max 1000000000
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 10000000000
Enter the segment size (Default: 16777216): 
Number of primes up to 10000000000: 455052511
Time taken: 989.81 seconds
Last Prime Found: 9999999967
34 bits required for Max 10000000000
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 100000000000
Enter the segment size (Default: 16777216): 
^CTraceback (most recent call last):
  File "/Users/adminamn/Downloads/fasterprimes.py", line 89, in <module>
    primes = compute_primes_segmented(M, segment_size)
  File "/Users/adminamn/Downloads/fasterprimes.py", line 71, in compute_primes_segmented
    is_prime_segment[idx] = False
KeyboardInterrupt
^C
(base) adminamn@MacBook-Pro Downloads % 
(base) adminamn@MacBook-Pro Downloads % vi fasterprimes.py     
(base) adminamn@MacBook-Pro Downloads % python3 fasterprimes.py
Enter the value of M: 10000000000
Enter the segment size (Default: 800000000): 

</textfield>
</textfields>
</window>
</windows>

### User (2024-11-28 00:28:12)

Ok I think I need to adjust the program... it is running fine it seems at 800 million segments, its been running for maybe 10 minutes now at about 4gb of memory used... only thing is that the cpu usage is fairly low, it is consistently around 15 percent and really not ever higher than 25% - what is the best approach now? to add multiprocessing back?

### Assistant (2024-11-28 00:28:13)

Yes, adding **multiprocessing** back is the best approach to improve CPU utilization, as your CPU is underutilized. The M1 MacBook Pro has **8 CPU cores** (4 performance cores and 4 efficiency cores), so you can divide the work across these cores for better parallelization. Here's how you can proceed:

---

### **Steps to Add Multiprocessing**

1. **Divide the Segments**:
   - Split the range \( M \) into independent segments.
   - Assign each segment to a separate process.

2. **Use Python’s `multiprocessing` Module**:
   - Create a pool of workers.
   - Assign each worker a chunk of the range to sieve.

3. **Combine Results**:
   - Merge the primes found by all workers after processing.

---

### **Updated Multiprocessing Implementation**

Here’s an example of how to update your code:

```python
import math
from multiprocessing import Pool, cpu_count
from bitarray import bitarray

def sieve_segment(low, high, small_primes):
    """Sieve a segment of the range [low, high)"""
    segment_size = high - low
    is_prime_segment = bitarray(segment_size)
    is_prime_segment.setall(True)

    for prime in small_primes:
        start = max(prime * prime, ((low + prime - 1) // prime) * prime)
        for j in range(start, high, prime):
            is_prime_segment[j - low] = False

    return [low + i for i in range(segment_size) if is_prime_segment[i]]

def compute_primes_segmented(M, segment_size):
    """Compute primes up to M using a segmented sieve with multiprocessing"""
    sqrt_limit = int(math.sqrt(M)) + 1
    sieve = bitarray(sqrt_limit)
    sieve.setall(True)

    # Regular sieve to find small primes
    for i in range(2, sqrt_limit):
        if sieve[i]:
            for j in range(i * i, sqrt_limit, i):
                sieve[j] = False

    small_primes = [i for i in range(2, sqrt_limit) if sieve[i]]

    # Divide the range into chunks
    chunks = [(low, min(low + segment_size, M), small_primes)
              for low in range(sqrt_limit, M, segment_size)]

    # Use multiprocessing to process chunks
    with Pool(processes=cpu_count()) as pool:
        results = pool.starmap(sieve_segment, chunks)

    # Combine results
    primes = small_primes + [prime for segment in results for prime in segment]

    return primes

if __name__ == "__main__":
    M = int(input("Enter the value of M: "))
    segment_size = int(input("Enter the segment size (Default: 800000000): ") or 800000000)

    import time
    start_time = time.time()
    primes = compute_primes_segmented(M, segment_size)
    end_time = time.time()

    print(f"Number of primes up to {M}: {len(primes)}")
    print(f"Time taken: {end_time - start_time:.2f} seconds")
    print(f"Last Prime Found: {primes[-1]}")
```

---

### **Why This Will Improve Performance**
1. **Parallel Processing**:
   - Each CPU core processes a separate chunk of the range, maximizing CPU usage.

2. **Scalable Segment Size**:
   - Adjust the segment size based on memory and CPU availability.

3. **Efficient Merge**:
   - The `pool.starmap` function handles parallel execution, and merging results is straightforward.

---

### **Monitoring and Fine-Tuning**
1. **CPU Usage**:
   - Use `htop` or `Activity Monitor` to confirm that all cores are utilized.
   - Ensure that CPU usage approaches 90-100% during execution.

2. **Memory Usage**:
   - Monitor memory to ensure the segment size doesn’t exceed available RAM.

3. **Segment Size Adjustment**:
   - If memory usage is too low, increase the segment size.
   - For faster execution, test smaller segments with more parallel processes.

---

### **Expected Results**
After adding multiprocessing:
- CPU utilization should increase significantly (close to full utilization of performance cores).
- Execution time should decrease, especially for larger values of \( M \).

Let me know how it works or if you encounter issues! 🚀

