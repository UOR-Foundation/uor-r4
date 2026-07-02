# Prime Density Prediction

- **Conversation ID:** `69992ec7-5ed8-8328-a139-39d1c2df35ac`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:04:39)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (array of primes)

Goal: Test whether spectral phase at the start of a log-interval predicts the future prime density inside that interval.

No plots. One JSON output only.

⸻

Step 1 — uniform log grid and split

Load x_grid and psi_minus_x.
Define f = psi_minus_x.
Let t = log(x_grid).

Interpolate f onto a uniform grid t_uniform with 65536 points.

Define TRAIN = first 70% of t_uniform
Define TEST = last 30% of t_uniform

⸻

Step 2 — build spectral phase (TRAIN ONLY)

Use the first 30 nontrivial zeta zero imaginary parts:

[14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851]

Fit amplitudes ONLY on TRAIN:

f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Construct phase on ALL t:

Z(t) = Σ (a_k - i b_k) * exp(i γ_k t_uniform)

φ_mod(t) = angle(Z) mod 2π

⸻

Step 3 — forward prime density measurement

For each t_i in TEST:

x_i = exp(t_uniform[i])

Define a forward multiplicative window
[x_i, x_i * exp(0.01)]

Count primes in this interval:

P_i = number of primes p with x_i ≤ p < x_i * exp(0.01)

(Use binary search in primes array for efficiency.)

Record (φ_mod[i], P_i)

⸻

Step 4 — predictive test (no binning trickery)

We now test prediction ability.

4a — threshold definition

Compute the median prime count across TEST:
median_P

Define label:
y_i = 1 if P_i > median_P else 0

⸻

4b — phase-only predictor

Fit a predictor using TRAIN ONLY:

On TRAIN points:
compute mean prime count per 12 phase bins.

Let p_bin[j] = probability(y=1 | phase bin j)

Then on TEST:
predict probability for each interval using only its phase bin.

⸻

4c — metrics

Compute on TEST:
	•	AUC (ROC AUC)
	•	accuracy (threshold 0.5)
	•	Brier score (mean squared error of predicted probabilities)
	•	calibration: average predicted probability vs actual rate

Also report:
	•	number of windows used
	•	class balance

⸻

Output

Return exactly one JSON file:

forward_phase_prime_prediction.json

Include:
	•	AUC
	•	accuracy
	•	Brier score
	•	calibration info
	•	class balance
	•	windows_used

No plots. No extra files.

### Assistant (2026-02-20 23:04:40)

import numpy as np, json, math, os, statistics

# Paths
psi_path = "/mnt/data/chebyshev_psi.npz"
primes_path = "/mnt/data/primes.npz"

# Load data
psi_npz = np.load(psi_path)
x_grid = psi_npz["x_grid"]
psi_minus_x = psi_npz["psi_minus_x"]

pr_npz = np.load(primes_path)
# Try common keys
prime_key = None
for k in pr_npz.files:
    if "prime" in k.lower():
        prime_key = k
        break
if prime_key is None:
    # fallback: first array
    prime_key = pr_npz.files[0]
primes = pr_npz[prime_key]

# Ensure sorted and integer type
primes = np.asarray(primes)
if primes.ndim != 1:
    primes = primes.ravel()
primes.sort()

# Step 1: log grid and uniform interpolation
x_grid = np.asarray(x_grid, dtype=float)
f = np.asarray(psi_minus_x, dtype=float)

# Guard: x_grid must be positive for log
mask = x_grid > 0
x_grid = x_grid[mask]
f = f[mask]

t = np.log(x_grid)
# Ensure strictly increasing for interpolation
order = np.argsort(t)
t = t[order]
f = f[order]

# If there are duplicates in t, average f for duplicates to make interp well-defined
# We'll compress duplicates
dt = np.diff(t)
if np.any(dt == 0):
    # group by unique t
    uniq_t, inv = np.unique(t, return_inverse=True)
    f_acc = np.zeros_like(uniq_t)
    counts = np.zeros_like(uniq_t)
    np.add.at(f_acc, inv, f)
    np.add.at(counts, inv, 1.0)
    f = f_acc / counts
    t = uniq_t

N = 65536
t_uniform = np.linspace(t[0], t[-1], N)
f_uniform = np.interp(t_uniform, t, f)

# Split
n_train = int(np.floor(0.70 * N))
train_idx = np.arange(n_train)
test_idx = np.arange(n_train, N)

# Step 2: fit spectral model on TRAIN
gammas = np.array([
    14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
    52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
    79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851
], dtype=float)

t_tr = t_uniform[train_idx]
f_tr = f_uniform[train_idx]

# Build design matrix: [cos(g1 t) ... cos(gK t) sin(g1 t) ... sin(gK t)]
K = gammas.size
cos_part = np.cos(np.outer(t_tr, gammas))  # (n_train, K)
sin_part = np.sin(np.outer(t_tr, gammas))  # (n_train, K)
X = np.concatenate([cos_part, sin_part], axis=1)  # (n_train, 2K)

# Least squares fit
coef, residuals, rank, svals = np.linalg.lstsq(X, f_tr, rcond=None)
a = coef[:K]
b = coef[K:]

# Construct Z(t) on ALL t_uniform
t_all = t_uniform
E = np.exp(1j * np.outer(t_all, gammas))  # (N, K)
c = a - 1j * b  # (K,)
Z = E @ c  # (N,)
phi = np.angle(Z)
phi_mod = np.mod(phi, 2*np.pi)

# Step 3: forward prime density counts for ALL points (we need TRAIN too for p_bin)
window_factor = math.exp(0.01)
x_all = np.exp(t_all)

# Efficient searchsorted counts
# right endpoint is exclusive
left_idx = np.searchsorted(primes, x_all, side="left")
right_idx = np.searchsorted(primes, x_all * window_factor, side="left")
P_all = (right_idx - left_idx).astype(int)

P_test = P_all[test_idx]
phi_test = phi_mod[test_idx]
P_train = P_all[train_idx]
phi_train = phi_mod[train_idx]

# Step 4a: median across TEST and labels
median_P = float(np.median(P_test))
y_test = (P_test > median_P).astype(int)
y_train = (P_train > median_P).astype(int)  # as specified, using TEST median threshold

# Step 4b: 12 phase bins trained on TRAIN only
n_bins = 12
bin_edges = np.linspace(0.0, 2*np.pi, n_bins+1)
# Assign bin indices 0..n_bins-1
bin_train = np.clip(np.digitize(phi_train, bin_edges, right=False) - 1, 0, n_bins-1)
bin_test = np.clip(np.digitize(phi_test, bin_edges, right=False) - 1, 0, n_bins-1)

p_bin = np.zeros(n_bins, dtype=float)
bin_counts = np.zeros(n_bins, dtype=int)
for j in range(n_bins):
    m = (bin_train == j)
    bin_counts[j] = int(m.sum())
    if bin_counts[j] > 0:
        p_bin[j] = float(y_train[m].mean())
    else:
        p_bin[j] = float(y_train.mean())  # fallback to overall TRAIN rate

p_pred_test = p_bin[bin_test]

# Step 4c: metrics
def auc_roc(y_true: np.ndarray, y_score: np.ndarray) -> float:
    y_true = np.asarray(y_true, dtype=int)
    y_score = np.asarray(y_score, dtype=float)
    n_pos = int(y_true.sum())
    n = y_true.size
    n_neg = n - n_pos
    if n_pos == 0 or n_neg == 0:
        return float("nan")

    # Compute average ranks for ties
    order = np.argsort(y_score, kind="mergesort")
    scores_sorted = y_score[order]
    y_sorted = y_true[order]

    ranks = np.empty(n, dtype=float)
    i = 0
    rank = 1
    while i < n:
        j = i
        while j + 1 < n and scores_sorted[j + 1] == scores_sorted[i]:
            j += 1
        # average rank for i..j inclusive
        avg_rank = (rank + (rank + (j - i))) / 2.0
        ranks[i:j+1] = avg_rank
        rank += (j - i + 1)
        i = j + 1

    # Sum ranks of positives in sorted order
    sum_ranks_pos = float((ranks * y_sorted).sum())
    auc = (sum_ranks_pos - n_pos * (n_pos + 1) / 2.0) / (n_pos * n_neg)
    return float(auc)

auc = auc_roc(y_test, p_pred_test)

y_hat = (p_pred_test >= 0.5).astype(int)
accuracy = float((y_hat == y_test).mean())
brier = float(np.mean((p_pred_test - y_test)**2))

# Calibration info on TEST by predicted probability value (12 discrete bins)
cal_bins = []
for j in range(n_bins):
    m = (bin_test == j)
    n_j = int(m.sum())
    if n_j == 0:
        continue
    cal_bins.append({
        "phase_bin": int(j),
        "n": n_j,
        "predicted_probability": float(p_bin[j]),
        "actual_positive_rate": float(y_test[m].mean()),
        "mean_prime_count": float(P_test[m].mean()),
    })

calibration_summary = {
    "overall_predicted_mean": float(p_pred_test.mean()),
    "overall_actual_positive_rate": float(y_test.mean()),
    "per_phase_bin": cal_bins
}

class_balance = {
    "positive_rate_test": float(y_test.mean()),
    "n_positive_test": int(y_test.sum()),
    "n_negative_test": int((1 - y_test).sum()),
    "median_P_test": median_P
}

result = {
    "AUC": auc,
    "accuracy": accuracy,
    "brier_score": brier,
    "calibration": calibration_summary,
    "class_balance": class_balance,
    "windows_used": int(P_test.size),
    "details": {
        "t_uniform_points": int(N),
        "train_points": int(n_train),
        "test_points": int(N - n_train),
        "phase_bins": int(n_bins),
        "window_log_width": 0.01,
        "zeta_zeros_used": int(K),
        "prime_array_key": prime_key
    }
}

out_path = "/mnt/data/forward_phase_prime_prediction.json"
with open(out_path, "w") as f_out:
    json.dump(result, f_out, indent=2)

out_path

### Tool (2026-02-20 23:05:28)

'/mnt/data/forward_phase_prime_prediction.json'

### Assistant (2026-02-20 23:05:31)

Done. Your precious single artifact is here:

[Download forward_phase_prime_prediction.json](sandbox:/mnt/data/forward_phase_prime_prediction.json)

