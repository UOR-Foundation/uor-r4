# Prime density phase test

- **Conversation ID:** `69992c4f-c9b8-8329-bcdc-7f6c4522c4f2`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:54:11)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (primes array)

Goal: Test whether spectral phase predicts local prime occurrence density.

⸻

Step 1 — uniform log grid

Load x_grid and psi_minus_x.
Compute t=log(x_grid).

Interpolate f=psi_minus_x onto uniform t_uniform (65536 points).

Split 70% train, 30% test.

⸻

Step 2 — spectral phase from TRUE zeros

Use first 30 true zeta zeros (same list as before).

Fit amplitudes on TRAIN ONLY:
f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Build:
Z(t)=Σ (a_k - i b_k) exp(i γ_k t)
φ_mod = angle(Z) mod 2π

⸻

Step 3 — prime density measurement

From primes array:

For each t_i (corresponding x_i = exp(t_i)):
define a multiplicative window:
[x_i, x_i * exp(0.01)]

Count number of primes in that interval:
P_i = #primes in [x_i, x_i*e^{0.01}]

(This keeps window constant in log-space.)

⸻

Step 4 — phase bin test

Bin φ_mod into 12 equal bins.

On TEST only:
compute mean(P_i) per bin.

Report:
	•	per-bin prime density
	•	max/min density ratio
	•	ANOVA p-value
	•	correlation between cos(φ) and P_i

⸻

Output

Return exactly one JSON:
phase_vs_prime_density.json

Include:
	•	per-bin densities (12 numbers)
	•	density ratio max/min
	•	p-value
	•	correlation
	•	number of windows used

No plots. No extra files.

### Assistant (2026-02-20 22:54:12)

import numpy as np, json, math, os, statistics
from scipy.interpolate import interp1d
from scipy import stats

psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"

data=np.load(psi_path)
x_grid=data["x_grid"]
psi_minus_x=data["psi_minus_x"]

pr=np.load(primes_path)
# figure key
pr_keys=list(pr.keys())
pr_keys

### Tool (2026-02-20 22:54:27)

['N', 'primes', 'is_prime']

### Assistant (2026-02-20 22:54:30)

primes=pr["primes"]
primes[:10], primes[-1], len(primes)

### Tool (2026-02-20 22:54:31)

(array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32), 9999991, 664579)

### Assistant (2026-02-20 22:54:34)

x_grid.min(), x_grid.max(), x_grid[:5], x_grid[-5:]

### Tool (2026-02-20 22:54:36)

(1,
 9999901,
 array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32))

### Assistant (2026-02-20 22:54:39)

# Step 1: uniform log grid
t = np.log(x_grid.astype(np.float64))
f = psi_minus_x.astype(np.float64)

# sort by t just in case (should be already sorted by x_grid)
order = np.argsort(t)
t_sorted = t[order]
f_sorted = f[order]

n = 65536
t_uniform = np.linspace(t_sorted.min(), t_sorted.max(), n)
interp = interp1d(t_sorted, f_sorted, kind='cubic', fill_value="extrapolate", assume_sorted=True)
f_u = interp(t_uniform)

# Split
split = int(0.7*n)
train_idx = np.arange(split)
test_idx = np.arange(split, n)

t_train = t_uniform[train_idx]
f_train = f_u[train_idx]

# Step 2: true zeta zeros (first 30)
gamma = np.array([
    14.134725141734693, 21.022039638771555, 25.010857580145688, 30.424876125859513,
    32.93506158773919, 37.58617815882567, 40.918719012147495, 43.327073280914999,
    48.00515088116716, 49.7738324776723, 52.97032147771446, 56.44624769706339,
    59.34704400260235, 60.83177852460981, 65.11254404808161, 67.07981052949417,
    69.54640171117398, 72.06715767448191, 75.70469069908393, 77.1448400688748,
    79.33737502024937, 82.91038085408603, 84.73549298051705, 87.42527461312523,
    88.80911120763447, 92.49189927055849, 94.65134404051989, 95.87063422824531,
    98.83119421819369, 101.31785100573139
], dtype=np.float64)
K=len(gamma)

# Design matrix for train
# shape (n_train, 2K)
X_cos = np.cos(np.outer(t_train, gamma))
X_sin = np.sin(np.outer(t_train, gamma))
X = np.concatenate([X_cos, X_sin], axis=1)

# Solve least squares
coef, residuals, rank, s = np.linalg.lstsq(X, f_train, rcond=None)
a = coef[:K]
b = coef[K:]

# Compute phi_mod for all points
t_all = t_uniform
exp_i = np.exp(1j * np.outer(t_all, gamma))  # (n, K)
c = a - 1j*b
Z = exp_i @ c  # (n,)
phi = np.angle(Z)
phi_mod = np.mod(phi, 2*np.pi)

# Step 3: prime density measurement on TEST indices
max_p = int(primes[-1])
log_window = 0.01
mult = math.exp(log_window)

x_all = np.exp(t_all)  # float
# Compute bounds for all test points
L = np.ceil(x_all[test_idx]).astype(np.int64)
R = np.floor(x_all[test_idx]*mult).astype(np.int64)

# filter valid windows
valid = (L <= R) & (R >= 2) & (R <= max_p)
L_v = L[valid]
R_v = R[valid]
phi_v = phi_mod[test_idx][valid]

# count primes in [L,R]
left_pos = np.searchsorted(primes, L_v, side='left')
right_pos = np.searchsorted(primes, R_v, side='right')
P = (right_pos - left_pos).astype(np.int64)

num_windows = int(P.size)
num_windows, P.mean(), P.min(), P.max()

### Tool (2026-02-20 22:55:02)

(19620, 1354.7722222222221, 62, 6130)

### Assistant (2026-02-20 22:55:05)

nbins=12
bin_idx = np.floor(phi_v / (2*np.pi) * nbins).astype(int)
bin_idx = np.clip(bin_idx, 0, nbins-1)

densities=[]
groups=[]
for j in range(nbins):
    pj = P[bin_idx==j]
    groups.append(pj)
    densities.append(float(pj.mean()) if pj.size>0 else float('nan'))

densities_arr=np.array(densities, dtype=np.float64)
ratio=float(np.nanmax(densities_arr)/np.nanmin(densities_arr))

# ANOVA p-value
# Use only non-empty groups
nonempty=[g for g in groups if g.size>0]
anova_p=float(stats.f_oneway(*nonempty).pvalue) if len(nonempty)>=2 else float('nan')

# correlation between cos(phi) and P
corr=float(stats.pearsonr(np.cos(phi_v), P).statistic)

ratio, anova_p, corr, [g.size for g in groups][:5], min([g.size for g in groups])

### Tool (2026-02-20 22:55:10)

(1.9559249493915996,
 5.282593872158447e-130,
 0.02063280638432276,
 [1800, 1781, 1783, 1452, 1783],
 1385)

### Assistant (2026-02-20 22:55:12)

out = {
    "per_bin_densities": densities,  # 12 numbers
    "density_ratio_max_min": ratio,
    "anova_p_value": anova_p,
    "correlation_cos_phi_P": corr,
    "number_of_windows_used": num_windows
}
out_path="/mnt/data/phase_vs_prime_density.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, os.path.getsize(out_path)

### Tool (2026-02-20 22:55:17)

('/mnt/data/phase_vs_prime_density.json', 488)

### Assistant (2026-02-20 22:55:19)

Done. Your precious JSON is ready:

[Download phase_vs_prime_density.json](sandbox:/mnt/data/phase_vs_prime_density.json)

