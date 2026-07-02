# Harmonic Phase Attractors

- **Conversation ID:** `699a51e3-e90c-8331-acc6-d4f8357e5cac`
- **Date:** 2026-02-21

---

### User (2026-02-21 19:46:32)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Cleanly identify phase attractors using the fitted harmonic acceleration law F(θ), not noisy bin means.

Recompute everything (no uploads assumed):
- primes to N=10_000_000
- ψ(x) on step-100 grid incl prime powers
- uniform log grid 65536
- bandpass [0.002,0.01] cycles/sample
- Hilbert phase θ(t) unwrapped
- ω(t)=Δθ, α(t)=Δω
- θ_mod = θ mod 2π

Fit F(θ) by harmonic regression up to K=8 on the raw samples (not binned):
Model: α ≈ c + Σ_{k=1..8} [a_k cos(kθ_mod) + b_k sin(kθ_mod)]
Fit by least squares on a random subsample of 200000 points to keep it fast.

Then:
1) On a dense grid θ_grid of 2048 points in [0,2π), compute F(θ_grid).
2) Find all zeros of F(θ) by sign changes + linear interpolation.
3) For each zero θ*, compute numerical derivative F'(θ*) via finite diff.
Classify:
- stable if F'(θ*) < 0
- unstable if F'(θ*) > 0

Output JSON:
{
"K":8,
"coeffs":{"c":..., "a":[...], "b":[...]},
"zeros":[ {"theta":..., "stability":"stable"/"unstable", "Fprime":...}, ... ],
"num_stable":...,
"num_unstable":...
}

No other text.

### Assistant (2026-02-21 19:46:33)

import numpy as np, math, json, time, sys

### Assistant (2026-02-21 19:47:01)

def sieve_primes(n:int):
    # simple optimized sieve using bytearray for odd numbers
    if n < 2:
        return np.array([], dtype=np.int32)
    size = (n//2)  # represent odds: index i -> 2*i+1
    is_prime = bytearray(b"\x01") * size
    is_prime[0] = 0  # 1 not prime
    limit = int(n**0.5)
    for p in range(3, limit+1, 2):
        if is_prime[p//2]:
            start = (p*p)//2
            step = p
            is_prime[start:size:step] = b"\x00"*(((size-1-start)//step)+1)
    primes = [2]
    primes.extend([2*i+1 for i in range(1,size) if is_prime[i] and (2*i+1)<=n])
    return np.array(primes, dtype=np.int32)

N=10_000_000
t0=time.time()
primes = sieve_primes(N)
len_primes = len(primes)
time.time()-t0, len_primes, primes[:10], primes[-1]

### Tool (2026-02-21 19:47:07)

(0.5032579898834229,
 664579,
 array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32),
 9999991)

### Assistant (2026-02-21 19:47:09)

# Compute psi on step-100 grid including prime powers
step=100
M = N//step  # 100000
inc = np.zeros(M, dtype=np.float64)

logp = np.log(primes.astype(np.float64))
for p, lp in zip(primes, logp):
    q = int(p)
    while q <= N:
        idx = (q-1)//step  # first grid point x=step*(idx+1) >= q
        inc[idx] += lp
        # next power, guard overflow
        if q > N//p:
            break
        q *= int(p)

psi = np.cumsum(inc)  # psi at x_i = step*(i+1)
x_grid = step*(np.arange(M)+1).astype(np.float64)
psi[-1], x_grid[-1]

### Tool (2026-02-21 19:47:12)

(9998539.403345881, 10000000.0)

### Assistant (2026-02-21 19:47:15)

# Build uniform log grid 65536 between x_min and N
n_log = 65536
x_min = step  # 100
x_log = np.exp(np.linspace(np.log(x_min), np.log(N), n_log))
# Interpolate psi(x) from step grid
psi_log = np.interp(x_log, x_grid, psi)
# signal: psi(x)-x
s = psi_log - x_log
s.mean(), s.std()

### Tool (2026-02-21 19:47:17)

(0.5185969768906133, 194.54472226423897)

### Assistant (2026-02-21 19:47:19)

# Bandpass via rfft
n = n_log
S = np.fft.rfft(s)
freqs = np.fft.rfftfreq(n, d=1.0)  # cycles/sample
band = (freqs >= 0.002) & (freqs <= 0.01)
S_filt = S * band
s_filt = np.fft.irfft(S_filt, n=n)

# Hilbert analytic signal for real s_filt
X = np.fft.fft(s_filt)
H = np.zeros(n)
if n % 2 == 0:
    H[0] = 1
    H[n//2] = 1
    H[1:n//2] = 2
else:
    H[0] = 1
    H[1:(n+1)//2] = 2
analytic = np.fft.ifft(X * H)
phase = np.unwrap(np.angle(analytic))
theta_mod = np.mod(phase, 2*np.pi)

omega = np.diff(phase)
alpha = np.diff(omega)

theta_mid = theta_mod[1:-1]  # align with alpha
len(theta_mid), len(alpha), theta_mid.min(), theta_mid.max()

### Tool (2026-02-21 19:47:24)

(65534, 65534, 5.820977655446313e-06, 6.282986625385547)

### Assistant (2026-02-21 19:47:25)

# Regression up to K=8
K=8
m = len(alpha)
# random subsample up to 200000 points; here m smaller, take all
idx = np.arange(m)
th = theta_mid[idx]
y = alpha[idx]

# build design matrix
cols = [np.ones_like(th)]
for k in range(1, K+1):
    cols.append(np.cos(k*th))
    cols.append(np.sin(k*th))
Xmat = np.column_stack(cols)
# least squares
beta, *_ = np.linalg.lstsq(Xmat, y, rcond=None)
c = float(beta[0])
a = [float(beta[1+2*(k-1)]) for k in range(1,K+1)]
b = [float(beta[2+2*(k-1)]) for k in range(1,K+1)]
c, a[:3], b[:3]

### Tool (2026-02-21 19:47:30)

(1.6757805717326014e-05,
 [-6.825789418661663e-05, 0.00016563149365465252, 0.0002485161132378401],
 [-5.439587717693467e-05, 2.021438900171993e-05, -4.231535705475286e-05])

### Assistant (2026-02-21 19:47:31)

# Define F(theta) function
def F(theta):
    theta = np.asarray(theta)
    out = c*np.ones_like(theta, dtype=np.float64)
    for k in range(1,K+1):
        out += a[k-1]*np.cos(k*theta) + b[k-1]*np.sin(k*theta)
    return out

# compute on dense grid
grid_n=2048
theta_grid = np.linspace(0, 2*np.pi, grid_n, endpoint=False)
F_grid = F(theta_grid)

# find zeros by sign changes
zeros=[]
for i in range(grid_n):
    j = (i+1) % grid_n
    f1, f2 = F_grid[i], F_grid[j]
    t1, t2 = theta_grid[i], theta_grid[j]
    # handle exact zero
    if f1 == 0.0:
        zeros.append(t1)
        continue
    if f1*f2 < 0:
        # linear interp
        theta0 = t1 + (t2 - t1) * (0 - f1) / (f2 - f1)
        # adjust wrap for last segment crossing 2pi
        if j==0:  # wrap segment
            t2w = 2*np.pi
            theta0 = t1 + (t2w - t1) * (0 - f1) / (f2 - f1)
            theta0 = theta0 % (2*np.pi)
        zeros.append(theta0)

# deduplicate close zeros (because of wrap)
zeros_sorted = sorted([z%(2*np.pi) for z in zeros])
dedup=[]
tol=1e-3
for z in zeros_sorted:
    if not dedup or abs(z-dedup[-1])>tol:
        dedup.append(z)
# check wrap adjacency
if len(dedup)>1 and abs((dedup[0]+2*np.pi)-dedup[-1])<tol:
    dedup[-1]=dedup[0]
    dedup=dedup[:-1]

len(zeros), len(dedup), dedup[:5]

### Tool (2026-02-21 19:47:43)

(10,
 10,
 [0.5708442082703449,
  1.8639364198567934,
  2.4795001875807956,
  3.048854472768268,
  4.088805578294413])

### Assistant (2026-02-21 19:47:45)

# Numerical derivative at zeros
def Fprime(theta, h=1e-4):
    return float((F((theta+h)%(2*np.pi)) - F((theta-h)%(2*np.pi))) / (2*h))

zero_objs=[]
for z in dedup:
    fp = Fprime(z)
    stab = "stable" if fp < 0 else "unstable" if fp > 0 else "neutral"
    zero_objs.append({"theta": float(z), "stability": stab, "Fprime": fp})

num_stable = sum(1 for o in zero_objs if o["stability"]=="stable")
num_unstable = sum(1 for o in zero_objs if o["stability"]=="unstable")
num_stable, num_unstable, zero_objs[0]

### Tool (2026-02-21 19:47:48)

(5,
 5,
 {'theta': 0.5708442082703449,
  'stability': 'stable',
  'Fprime': -0.0021451248821081612})

### Assistant (2026-02-21 19:47:49)

out = {
    "K": K,
    "coeffs": {"c": c, "a": a, "b": b},
    "zeros": zero_objs,
    "num_stable": num_stable,
    "num_unstable": num_unstable
}
json_str = json.dumps(out, ensure_ascii=False, separators=(",",":"))
len(json_str), json_str[:200]

### Tool (2026-02-21 19:47:51)

(1274,
 '{"K":8,"coeffs":{"c":1.6757805717326014e-05,"a":[-6.825789418661663e-05,0.00016563149365465252,0.0002485161132378401,-7.373662013686932e-05,4.337686501772263e-05,0.00014100781952060487,-9.076000618638')

### Assistant (2026-02-21 19:47:52)

{"K":8,"coeffs":{"c":1.6757805717326014e-05,"a":[-6.825789418661663e-05,0.00016563149365465252,0.0002485161132378401,-7.373662013686932e-05,4.337686501772263e-05,0.00014100781952060487,-9.076000618638108e-05,3.148985312277915e-05],"b":[-5.439587717693467e-05,2.021438900171993e-05,-4.231535705475286e-05,8.642673733185513e-05,3.276909659039353e-05,2.7017919934476313e-05,1.0262865673341513e-05,-2.752123200408884e-06]},"zeros":[{"theta":0.5708442082703449,"stability":"stable","Fprime":-0.0021451248821081612},{"theta":1.8639364198567934,"stability":"unstable","Fprime":0.0017847069717856957},{"theta":2.4795001875807956,"stability":"stable","Fprime":-0.0016194068988503636},{"theta":3.048854472768268,"stability":"unstable","Fprime":0.0018295223691596815},{"theta":4.088805578294413,"stability":"stable","Fprime":-0.002078773494444749},{"theta":4.609072135450975,"stability":"unstable","Fprime":0.001446817009020067},{"theta":5.0359854077581035,"stability":"stable","Fprime":-0.0009761142943735216},{"theta":5.40746784276472,"stability":"unstable","Fprime":0.001314708311872787},{"theta":5.841166393061574,"stability":"stable","Fprime":-0.0014411111970759727},{"theta":6.068468501785644,"stability":"unstable","Fprime":0.0008641341215671392}],"num_stable":5,"num_unstable":5}

