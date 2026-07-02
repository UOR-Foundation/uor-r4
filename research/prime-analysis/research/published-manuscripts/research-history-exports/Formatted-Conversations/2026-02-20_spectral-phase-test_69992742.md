# Spectral Phase Test

- **Conversation ID:** `69992742-889c-832e-ab3c-67c944d5a7cd`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:32:37)

You are a stateless compute worker.

Upload:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)

Goal: Using TRUE zeta-zero frequencies (first 30), test whether the spectral phase φ(t) predicts local density deviations of primes (proxied by increments of ψ).

⸻

Step 1 — uniform log grid and signal

Load x_grid, psi_minus_x, and also reconstruct psi_grid = psi_minus_x + x_grid.

t = log(x_grid)

Interpolate psi_grid onto uniform t_uniform of 65536 points by interpolating psi as a function of t (not ψ−x).

Let psi_u(t) be the interpolated ψ on uniform grid.
Let x_u = exp(t_uniform).

⸻

Step 2 — define a local “prime activity” proxy

On the uniform grid, define local activity as a finite difference:
A(t_i) = psi_u(t_i+Δ) - psi_u(t_i)
Use Δ = 50 samples (fixed lag in t grid).

This measures accumulated log p mass in a local multiplicative interval.

Normalize A by its mean across the grid to get A_norm.

⸻

Step 3 — spectral phase from TRUE zeros

Use the first 30 TRUE γ values (hardcode list same as before).

Fit amplitudes (a_k,b_k) on TRAIN only (first 70%) to approximate f=psi_minus_x on uniform grid.

Build Z(t)=Σ (a_k - i b_k) exp(i γ_k t)
φ(t)=unwrap(angle(Z(t)))

⸻

Step 4 — phase binning test

Bin φ modulo 2π into 12 equal bins.

For each bin, compute mean(A_norm) on TEST only (last 30%).

Report:
	•	per-bin mean(A_norm)
	•	max-min spread across bins
	•	one-way ANOVA p-value (or permutation test) for “activity depends on phase bin”

Also report correlation between cos(φ) and A_norm on TEST.

⸻

Output

Return exactly one JSON file: phase_vs_activity.json
Include:
	•	test correlation corr(cosφ, A_norm)
	•	per-bin means (12 numbers)
	•	spread (max-min)
	•	significance p-value
	•	all constants used (Δ lag, bin count)

No plots. No extra files.

### Assistant (2026-02-20 22:32:38)

import numpy as np, json, math, os, statistics
path="/mnt/data/chebyshev_psi.npz"
data=np.load(path)
data.files

### Tool (2026-02-20 22:32:57)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-20 22:32:58)

import numpy as np, math, json, os
from numpy.linalg import lstsq

npz=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=npz["x_grid"].astype(float)
psi_minus_x=npz["psi_minus_x"].astype(float)
# reconstruct psi_grid
psi_grid=psi_minus_x + x_grid
# verify if psi_grid in file matches closely
if "psi_grid" in npz.files:
    psi_grid_file=npz["psi_grid"].astype(float)
    maxdiff=float(np.max(np.abs(psi_grid_file-psi_grid)))
else:
    maxdiff=None
maxdiff

### Tool (2026-02-20 22:33:02)

0.0

### Assistant (2026-02-20 22:33:03)

t=np.log(x_grid)
# ensure sorted
idx=np.argsort(t)
t=t[idx]; psi_grid=psi_grid[idx]
N=65536
t_uniform=np.linspace(float(t[0]), float(t[-1]), N)
psi_u=np.interp(t_uniform, t, psi_grid)
x_u=np.exp(t_uniform)
f_u=psi_u - x_u  # psi_minus_x on uniform grid

# Step2 activity with lag
lag=50
A=psi_u[lag:] - psi_u[:-lag]
A_mean=float(A.mean())
A_norm=A / A_mean
# align phi later to length N-lag
len(A_norm), N

### Tool (2026-02-20 22:33:07)

(65486, 65536)

### Assistant (2026-02-20 22:33:08)

gammas = np.array([
14.134725141734693,
21.022039638771554,
25.010857580145688,
30.424876125859513,
32.935061587739189,
37.586178158825671,
40.918719012147495,
43.327073280914999,
48.005150881167159,
49.773832477672302,
52.970321477714460,
56.446247697063394,
59.347044002602353,
60.831778524609809,
65.112544048081607,
67.079810529494173,
69.546401711173979,
72.067157674481907,
75.704690699083933,
77.144840068874805,
79.337375020249367,
82.910380854086030,
84.735492980517050,
87.425274613125229,
88.809111207634465,
92.491899270558484,
94.651344040519886,
95.870634228245309,
98.831194218193692,
101.317851005731390
], dtype=float)
len(gammas)

### Tool (2026-02-20 22:33:16)

30

### Assistant (2026-02-20 22:33:17)

N=len(t_uniform)
train_end=int(0.7*N)
t_train=t_uniform[:train_end]
y_train=f_u[:train_end]

# Design matrix: [cos(g1 t), sin(g1 t), ..., cos(gm t), sin(gm t)]
m=len(gammas)
X=np.empty((train_end, 2*m), dtype=np.float64)
for k,g in enumerate(gammas):
    X[:,2*k]=np.cos(g*t_train)
    X[:,2*k+1]=np.sin(g*t_train)
coef, residuals, rank, s = lstsq(X, y_train, rcond=None)
a = coef[0::2]
b = coef[1::2]
rank, residuals[:5] if len(residuals)>0 else residuals

### Tool (2026-02-20 22:33:21)

(60, array([29152137.85682261]))

### Assistant (2026-02-20 22:33:24)

t_all=t_uniform
Z=np.zeros(N, dtype=np.complex128)
for k,g in enumerate(gammas):
    Z += (a[k] - 1j*b[k]) * np.exp(1j*g*t_all)
phi=np.unwrap(np.angle(Z))
phi_aligned=phi[:-lag]  # align with A_norm length
# test indices for aligned arrays
N_aligned=len(A_norm)
train_end_aligned=int(0.7*N_aligned)  # consistent split on aligned length
test_slice=slice(train_end_aligned, N_aligned)
A_test=A_norm[test_slice]
phi_test=phi_aligned[test_slice]
# Phase binning
bin_count=12
phase_mod=np.mod(phi_test, 2*np.pi)
bins = np.floor(phase_mod/(2*np.pi)*bin_count).astype(int)
bins = np.clip(bins, 0, bin_count-1)
per_bin_means=[]
counts=[]
for j in range(bin_count):
    mask=(bins==j)
    counts.append(int(mask.sum()))
    if mask.any():
        per_bin_means.append(float(A_test[mask].mean()))
    else:
        per_bin_means.append(float("nan"))
spread=float(np.nanmax(per_bin_means)-np.nanmin(per_bin_means))
counts[:3], spread

### Tool (2026-02-20 22:33:31)

([1822, 1732, 1823], 2.163725234205087)

### Assistant (2026-02-20 22:33:33)

p_value=None
try:
    import scipy.stats as st
    groups=[A_test[bins==j] for j in range(bin_count) if np.any(bins==j)]
    fstat, p_value = st.f_oneway(*groups)
    p_value=float(p_value)
    fstat=float(fstat)
except Exception as e:
    e, type(e)

### Assistant (2026-02-20 22:33:37)

fstat, p_value

### Tool (2026-02-20 22:33:40)

(57.14803404809271, 1.2568157077622946e-125)

### Assistant (2026-02-20 22:33:42)

cosphi=np.cos(phi_test)
corr=float(np.corrcoef(cosphi, A_test)[0,1])
corr

### Tool (2026-02-20 22:33:44)

0.02059551314800184

### Assistant (2026-02-20 22:33:47)

out = {
    "test_correlation_corr_cosphi_A_norm": corr,
    "per_bin_mean_A_norm_test": per_bin_means,
    "spread_max_minus_min": spread,
    "significance_p_value_one_way_anova": p_value,
    "constants": {
        "lag_samples_Delta": lag,
        "bin_count": bin_count,
        "uniform_t_points": int(N),
        "zero_count": int(len(gammas)),
        "train_fraction": 0.7,
        "t_uniform_min": float(t_uniform[0]),
        "t_uniform_max": float(t_uniform[-1]),
        "gamma_values": [float(x) for x in gammas],
    }
}
out_path="/mnt/data/phase_vs_activity.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, os.path.getsize(out_path)

### Tool (2026-02-20 22:33:52)

('/mnt/data/phase_vs_activity.json', 1484)

### Assistant (2026-02-20 22:33:55)

Done. Your precious single JSON artifact is ready:

[Download phase_vs_activity.json](sandbox:/mnt/data/phase_vs_activity.json)

