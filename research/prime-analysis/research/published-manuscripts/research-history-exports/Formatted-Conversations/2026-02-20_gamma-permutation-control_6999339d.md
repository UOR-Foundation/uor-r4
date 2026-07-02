# Gamma permutation control

- **Conversation ID:** `6999339d-6648-8326-a3fe-183f9d13c862`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:25:35)

You are a stateless compute worker.

CRITICAL: You MUST write a file named
gamma_permutation_control.json
and not print the JSON inline.

Use:
with open(“gamma_permutation_control.json”,“w”) as f:
json.dump(output,f)

No stdout JSON allowed.

⸻

Goal

Test whether predictive power depends specifically on the TRUE zeta zero frequencies, not merely any quasi-periodic phase extracted from ψ.

⸻

Step 1 — build dataset (same as previous successful experiment)
	•	uniform t grid (65536)
	•	restrict x to [1e5, 9e6]
	•	TRAIN 70%, TEST 30%
	•	forward windows Δ = 0.01
	•	residual R = (P − E)/sqrt(E)
	•	y = R > median(R_train)

⸻

Step 2 — baseline model (TRUE γ)

Use first 30 true zeta zeros (same list).

Fit amplitudes on TRAIN:
f_train ≈ Σ a_k cos(γ_k t)+b_k sin(γ_k t)

Compute φ_true and phase-bin predictor (B=12).
Compute TEST AUC.

Call this:
AUC_true

⸻

Step 3 — permutation control

Repeat 20 times:

Shuffle the γ list randomly (permute frequencies relative to their amplitudes) OR equivalently randomly reassign each γ_k to a different amplitude pair (a_k,b_k).

For each permutation:
	•	build φ_perm
	•	build same phase-bin predictor
	•	compute TEST AUC

Collect all 20 AUC values.

Compute:
	•	mean_perm_auc
	•	std_perm_auc
	•	max_perm_auc

⸻

Step 4 — output file

Write ONLY a file:

gamma_permutation_control.json

with fields:
{
“AUC_true”: value,
“perm_auc_list”: […20 values…],
“mean_perm_auc”: value,
“std_perm_auc”: value,
“max_perm_auc”: value
}

No printing. No plots. No extra files.

### Assistant (2026-02-20 23:25:36)

import numpy as np, os, json, math, statistics, random
psi_path="/mnt/data/chebyshev_psi.npz"
pr_path="/mnt/data/primes.npz"
psi=np.load(psi_path)
psi.files, {k:psi[k].shape for k in psi.files}

### Tool (2026-02-20 23:25:52)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'],
 {'N': (),
  'step': (),
  'x_grid': (100000,),
  'psi_grid': (100000,),
  'psi_minus_x': (100000,)})

### Assistant (2026-02-20 23:25:54)

x=psi["x_grid"]; step=int(psi["step"]); N=int(psi["N"])
x[:5], x[-5:], step, N

### Tool (2026-02-20 23:25:56)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 100,
 10000000)

### Assistant (2026-02-20 23:25:57)

pr=np.load(pr_path)
pr.files, {k:pr[k].shape for k in pr.files}

### Tool (2026-02-20 23:26:00)

(['N', 'primes', 'is_prime'],
 {'N': (), 'primes': (664579,), 'is_prime': (10000001,)})

### Assistant (2026-02-20 23:26:01)

import numpy as np, math, json, random
from pathlib import Path

# Load datasets
psi = np.load("/mnt/data/chebyshev_psi.npz")
x_grid = psi["x_grid"].astype(np.float64)
psi_minus_x_grid = psi["psi_minus_x"].astype(np.float64)

pr = np.load("/mnt/data/primes.npz")
primes = pr["primes"].astype(np.int64)

# Parameters
tN = 65536
x_min = 1e5
x_max = 9e6
Delta = 0.01
B = 12
train_frac = 0.70
n_perm = 20
rng = np.random.default_rng(12345)

# Build uniform t-grid
t_min = math.log(x_min)
t_max = math.log(x_max)
t = np.linspace(t_min, t_max, tN, dtype=np.float64)
x = np.exp(t)

# Interpolate psi_minus_x at x
# x_grid is step 100; use np.interp
psi_minus_x = np.interp(x, x_grid, psi_minus_x_grid)  # linear
f = psi_minus_x / np.sqrt(x)  # scaled fluctuation signal for fitting

# Prime counts in forward multiplicative window [x, x*exp(Delta)]
x2 = x * math.exp(Delta)

# Count primes in (x, x2]
# Use searchsorted on primes array for vectorized counts
left_idx = np.searchsorted(primes, x, side="right")  # > x
right_idx = np.searchsorted(primes, x2, side="right")  # <= x2
P_counts = (right_idx - left_idx).astype(np.float64)

# Expected counts E = integral_{x}^{x2} du/log u (numerical Gauss-Legendre 16)
# We'll do fixed 16-point Gauss-Legendre on [0,1], map to [x,x2]
nodes, weights = np.polynomial.legendre.leggauss(16)
# map nodes from [-1,1] to [0,1]
u01 = 0.5*(nodes+1.0)
w01 = 0.5*weights

# Vectorized integration
# For each interval: u = x + (x2-x)*u01
dx = (x2 - x)
# broadcast
u = x[:,None] + dx[:,None]*u01[None,:]
integrand = 1.0/np.log(u)
E = (dx[:,None] * integrand * w01[None,:]).sum(axis=1)
# Safety: avoid zeros
E = np.maximum(E, 1e-12)

R = (P_counts - E)/np.sqrt(E)

# Train/test split by index
n_train = int(train_frac * tN)
train_idx = np.arange(n_train)
test_idx = np.arange(n_train, tN)

R_train = R[train_idx]
median_R_train = float(np.median(R_train))
y = (R > median_R_train).astype(np.int8)
y_train = y[train_idx]
y_test = y[test_idx]

# First 30 nontrivial zeros imaginary parts (gamma_k)
gamma_true = np.array([
    14.134725141734693, 21.022039638771554, 25.010857580145688, 30.424876125859513,
    32.935061587739190, 37.586178158825671, 40.918719012147495, 43.327073280914999,
    48.005150881167159, 49.773832477672302, 52.970321477714460, 56.446247697063394,
    59.347044002602353, 60.831778524609809, 65.112544048081606, 67.079810529494173,
    69.546401711173979, 72.067157674481907, 75.704690699083933, 77.144840068874805,
    79.337375020249367, 82.910380854086030, 84.735492980517050, 87.425274613125229,
    88.809111207634465, 92.491899270558484, 94.651344040519886, 95.870634228245309,
    98.831194218193692, 101.31785100573139
], dtype=np.float64)

K = len(gamma_true)

def fit_amplitudes(gamma, t_train, f_train):
    # Design matrix: [cos(gamma_k t), sin(gamma_k t)] for k=1..K
    cosM = np.cos(np.outer(t_train, gamma))
    sinM = np.sin(np.outer(t_train, gamma))
    X = np.concatenate([cosM, sinM], axis=1)
    # Solve least squares
    beta, *_ = np.linalg.lstsq(X, f_train, rcond=None)
    a = beta[:K]
    b = beta[K:]
    return a, b

def phase_from_params(gamma, a, b, t_all):
    # complex coefficient c_k = a_k - i b_k so that Re(c_k e^{i gamma t}) = a cos + b sin
    c = a - 1j*b
    Z = np.exp(1j*np.outer(t_all, gamma)) @ c
    phi = np.angle(Z)
    # map to [0, 2pi)
    phi = np.mod(phi, 2*np.pi)
    return phi

def phase_bin_predictor_auc(phi, y, train_idx, test_idx, B):
    # Bin edges
    edges = np.linspace(0.0, 2*np.pi, B+1)
    train_bins = np.digitize(phi[train_idx], edges) - 1
    test_bins = np.digitize(phi[test_idx], edges) - 1
    # clamp
    train_bins = np.clip(train_bins, 0, B-1)
    test_bins = np.clip(test_bins, 0, B-1)
    # estimate P(y=1 | bin) on train with Laplace smoothing
    p = np.zeros(B, dtype=np.float64)
    for b in range(B):
        idx = train_idx[train_bins == b]
        if idx.size == 0:
            p[b] = 0.5
        else:
            # Laplace smoothing
            yb = y[idx]
            p[b] = (yb.sum() + 1.0) / (yb.size + 2.0)
    scores = p[test_bins]
    labels = y[test_idx].astype(np.int8)
    # AUC via rank method, handle ties
    order = np.argsort(scores)
    sorted_scores = scores[order]
    sorted_labels = labels[order]
    n_pos = sorted_labels.sum()
    n_neg = sorted_labels.size - n_pos
    if n_pos == 0 or n_neg == 0:
        return float("nan")
    # Compute ranks with tie handling: average ranks for ties
    ranks = np.empty_like(sorted_scores, dtype=np.float64)
    i=0
    rank=1
    while i < sorted_scores.size:
        j=i
        while j < sorted_scores.size and sorted_scores[j]==sorted_scores[i]:
            j+=1
        avg_rank = 0.5*(rank + (rank + (j-i) -1))
        ranks[i:j]=avg_rank
        rank += (j-i)
        i=j
    # Sum ranks of positives
    sum_ranks_pos = ranks[sorted_labels==1].sum()
    auc = (sum_ranks_pos - n_pos*(n_pos+1)/2) / (n_pos*n_neg)
    return float(auc)

# Fit on train with true gammas
a_true, b_true = fit_amplitudes(gamma_true, t[train_idx], f[train_idx])
phi_true = phase_from_params(gamma_true, a_true, b_true, t)
AUC_true = phase_bin_predictor_auc(phi_true, y, train_idx, test_idx, B)

# Permutation control
perm_auc_list = []
for _ in range(n_perm):
    perm = rng.permutation(K)
    gamma_perm = gamma_true[perm]
    # keep (a,b) paired with original index but applied to permuted gammas
    phi_perm = phase_from_params(gamma_perm, a_true, b_true, t)
    aucp = phase_bin_predictor_auc(phi_perm, y, train_idx, test_idx, B)
    perm_auc_list.append(aucp)

perm_auc_arr = np.array(perm_auc_list, dtype=np.float64)
mean_perm_auc = float(np.nanmean(perm_auc_arr))
std_perm_auc = float(np.nanstd(perm_auc_arr, ddof=1))
max_perm_auc = float(np.nanmax(perm_auc_arr))

output = {
    "AUC_true": float(AUC_true),
    "perm_auc_list": [float(v) for v in perm_auc_list],
    "mean_perm_auc": mean_perm_auc,
    "std_perm_auc": std_perm_auc,
    "max_perm_auc": max_perm_auc
}

out_path = Path("/mnt/data/gamma_permutation_control.json")
with open(out_path, "w") as f:
    json.dump(output, f)

### Assistant (2026-02-20 23:27:01)

Done. Your precious control experiment output has been written to:

[gamma_permutation_control.json](sandbox:/mnt/data/gamma_permutation_control.json)

