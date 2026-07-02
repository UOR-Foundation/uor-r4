# Sacks Spiral Prime Visualization

- **Conversation ID:** `66ed1c59-7fd0-8001-9c25-57ef0f5bee4f`
- **Date:** 2024-09-20

---

### User (2024-09-20 02:55:21)

Please plot the Sacks spiral highlighting prime numbers.

### Assistant (2024-09-20 02:56:06)

import numpy as np
import matplotlib.pyplot as plt

# Function to check if a number is prime
def is_prime(n):
    if n <= 1:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    for i in range(3, int(np.sqrt(n)) + 1, 2):
        if n % i == 0:
            return False
    return True

# Generate Sacks spiral
def sacks_spiral(n_points):
    theta = []
    r = []
    primes = []
    
    for i in range(1, n_points + 1):
        angle = i * np.pi / 2
        radius = np.sqrt(i)
        theta.append(angle)
        r.append(radius)
        if is_prime(i):
            primes.append(i)
    
    return np.array(theta), np.array(r), primes

# Define the number of points in the spiral
n_points = 1000

# Generate the spiral and highlight primes
theta, r, primes = sacks_spiral(n_points)

# Plot the spiral
fig, ax = plt.subplots(figsize=(8, 8), subplot_kw={'projection': 'polar'})
ax.plot(theta, r, lw=0.5, color='lightgray')

# Highlight primes
prime_theta = [theta[i-1] for i in primes]
prime_r = [r[i-1] for i in primes]
ax.scatter(prime_theta, prime_r, color='red', s=10, label="Primes")

# Add title and legend
ax.set_title("Sacks Spiral Highlighting Prime Numbers", va='bottom')
ax.legend(loc="upper right")

plt.show()

### Tool (2024-09-20 02:56:06)

<<ImageDisplayed>>

![Attachment](../file-Lx7oh32EfKGqIR0VV8UfSnk3-6e5253a0-e16e-4465-a87c-404f5c0d527e.png)

### Assistant (2024-09-20 02:56:06)

Here is the Sacks spiral highlighting the prime numbers in red. The spiral grows outward with each number, and the primes form distinct points along the spiral, emphasizing their distribution in this geometric visualization. Let me know if you'd like to adjust or explore the visualization further!

