# Phase Bin Activity Classifier

- **Conversation ID:** `69992839-0920-832e-81bf-769865ce4fe1`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:36:42)

You are a stateless compute worker.

Upload:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)

Goal: Build a simple out-of-sample predictor of “high prime activity” using ONLY spectral phase bins from TRUE zeta zeros.

⸻

Step 1 — uniform log grid

Load x_grid and psi_minus_x. Reconstruct psi_grid = psi_minus_x + x_grid.
t=log(x_grid)

Interpolate psi_grid onto uniform t_uniform with 65536 points.
Let psi_u(t) be interpolated ψ.
Let f_u(t) be interpolated (ψ−x).

Split:
train = first 70%
test = last 30%

⸻

Step 2 — define activity label

Define local activity:
A(t_i)=psi_u(t_{i+Δ}) - psi_u(t_i), with Δ=50 samples.

Normalize A by mean(A) over TRAIN only:
A_norm = A / mean_train(A)

Define binary label on TRAIN/TEST:
y=1 if A_norm > median_train(A_norm) else 0
(so “high activity” is above median)

⸻

Step 3 — spectral phase

Use first 30 TRUE zeta zeros (hardcode list from previous run).

Fit amplitudes a_k,b_k to f_u on TRAIN only:
f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Compute Z(t)=Σ (a_k - i b_k) exp(i γ_k t)
φ(t)=angle(Z) (unwrap optional), then take φ_mod = φ mod 2π.

Bin φ_mod into 12 equal bins.
Feature is just the bin index (0..11), one-hot encoded.

⸻

Step 4 — predictive model

Fit a logistic regression on TRAIN:
y ~ one_hot(phase_bin)

Evaluate on TEST:
	•	accuracy
	•	AUC
	•	confusion matrix (threshold 0.5)

Also report per-bin learned log-odds weights (or estimated probabilities per bin).

⸻

Output

Return exactly one JSON: phase_bin_activity_classifier.json
Include:
	•	accuracy_test, auc_test
	•	confusion matrix counts
	•	predicted probability per bin (12 numbers)
	•	constants used (Δ, bins)

No plots. No extra files.

### Assistant (2026-02-20 22:36:43)

import numpy as np, json, math, os, pathlib

path = "/mnt/data/chebyshev_psi.npz"
data = np.load(path)
x_grid = data["x_grid"].astype(float)
psi_minus_x = data["psi_minus_x"].astype(float)

# Step 1: reconstruct psi and interpolate onto uniform log-grid
psi_grid = psi_minus_x + x_grid
t = np.log(x_grid)

# ensure sorted by t
order = np.argsort(t)
t = t[order]
psi_grid = psi_grid[order]
f_grid = psi_minus_x[order]

N = 65536
t_uniform = np.linspace(t[0], t[-1], N)
psi_u = np.interp(t_uniform, t, psi_grid)
f_u = np.interp(t_uniform, t, f_grid)

# Step 2: activity label
Delta = 50
M = N - Delta
A = psi_u[Delta:] - psi_u[:-Delta]

train_end = int(0.7 * M)
A_train = A[:train_end]
mean_train_A = A_train.mean()
A_norm = A / mean_train_A
median_train = np.median(A_norm[:train_end])
y = (A_norm > median_train).astype(int)

# Step 3: spectral phase using first 30 TRUE zeta zeros (imag parts)
gamma = np.array([
    14.134725141734693, 21.022039638771554, 25.010857580145688, 30.424876125859513,
    32.935061587739189, 37.586178158825671, 40.918719012147495, 43.327073280914999,
    48.005150881167159, 49.773832477672302, 52.970321477714460, 56.446247697063394,
    59.347044002602353, 60.831778524609809, 65.112544048081607, 67.079810529494173,
    69.546401711173979, 72.067157674481907, 75.704690699083934, 77.144840068874805,
    79.337375020249367, 82.910380854086030, 84.735492980517050, 87.425274613125229,
    88.809111207634465, 92.491899270558484, 94.651344040519886, 95.870634228245309,
    98.831194218193692, 101.31785100573139
], dtype=float)

# Build design matrix on TRAIN: [cos(g_k t), sin(g_k t)]
t_train = t_uniform[:train_end]
Xc = np.cos(np.outer(t_train, gamma))
Xs = np.sin(np.outer(t_train, gamma))
X = np.hstack([Xc, Xs])  # shape (train_end, 60)

# Fit least squares to f_u on TRAIN
f_train = f_u[:train_end]
coef, *_ = np.linalg.lstsq(X, f_train, rcond=None)
a = coef[:len(gamma)]
b = coef[len(gamma):]

# Compute phase for all samples (aligned with A/y indices, so use t_uniform[:M])
t_feat = t_uniform[:M]
Z_re = np.cos(np.outer(t_feat, gamma)) @ a + np.sin(np.outer(t_feat, gamma)) @ b  # real part of sum
# Imag part of Z(t) = sum (a_k sin(g t) - b_k cos(g t)) because (a - i b) e^{i g t}
Z_im = (np.sin(np.outer(t_feat, gamma)) @ a) - (np.cos(np.outer(t_feat, gamma)) @ b)

phi = np.arctan2(Z_im, Z_re)  # angle in (-pi, pi]
phi_mod = np.mod(phi, 2*np.pi)

bins = 12
bin_width = 2*np.pi / bins
phase_bin = np.floor(phi_mod / bin_width).astype(int)
phase_bin = np.clip(phase_bin, 0, bins-1)

# Step 4: "logistic regression" with one-hot is equivalent to per-bin MLE.
# We'll compute smoothed per-bin probabilities on TRAIN.
eps = 1e-12
prob_per_bin = np.zeros(bins, dtype=float)
counts = np.zeros(bins, dtype=int)

for bidx in range(bins):
    mask = (phase_bin[:train_end] == bidx)
    n = int(mask.sum())
    counts[bidx] = n
    if n == 0:
        # if unseen, default to overall train prevalence (smoothed)
        p = (y[:train_end].sum() + 0.5) / (train_end + 1.0)
    else:
        # Laplace smoothing (Jeffreys-ish): add 0.5 pseudo-counts
        p = (y[:train_end][mask].sum() + 0.5) / (n + 1.0)
    prob_per_bin[bidx] = float(np.clip(p, eps, 1-eps))

# Predict on TEST
test_mask = np.arange(M) >= train_end
y_test = y[test_mask]
bins_test = phase_bin[test_mask]
p_test = prob_per_bin[bins_test]

yhat = (p_test >= 0.5).astype(int)

# Confusion matrix: TN, FP, FN, TP
tn = int(((y_test == 0) & (yhat == 0)).sum())
fp = int(((y_test == 0) & (yhat == 1)).sum())
fn = int(((y_test == 1) & (yhat == 0)).sum())
tp = int(((y_test == 1) & (yhat == 1)).sum())

accuracy = float((yhat == y_test).mean())

# AUC (rank-based, handles ties)
def auc_roc(y_true, scores):
    y_true = np.asarray(y_true, dtype=int)
    scores = np.asarray(scores, dtype=float)
    pos = (y_true == 1)
    neg = (y_true == 0)
    n_pos = pos.sum()
    n_neg = neg.sum()
    if n_pos == 0 or n_neg == 0:
        return float("nan")
    # Rank scores with average ranks for ties
    order = np.argsort(scores)
    ranks = np.empty_like(order, dtype=float)
    ranks[order] = np.arange(1, len(scores) + 1, dtype=float)
    # tie handling
    sorted_scores = scores[order]
    i = 0
    while i < len(scores):
        j = i
        while j + 1 < len(scores) and sorted_scores[j+1] == sorted_scores[i]:
            j += 1
        if j > i:
            avg_rank = (i + 1 + j + 1) / 2.0
            ranks[order[i:j+1]] = avg_rank
        i = j + 1
    sum_ranks_pos = ranks[pos].sum()
    auc = (sum_ranks_pos - n_pos*(n_pos+1)/2.0) / (n_pos*n_neg)
    return float(auc)

auc = auc_roc(y_test, p_test)

out = {
    "accuracy_test": accuracy,
    "auc_test": auc,
    "confusion_matrix": {"tn": tn, "fp": fp, "fn": fn, "tp": tp},
    "predicted_probability_per_bin": [float(x) for x in prob_per_bin.tolist()],
    "constants": {"Delta": Delta, "bins": bins}
}

out_path = "/mnt/data/phase_bin_activity_classifier.json"
with open(out_path, "w") as f:
    json.dump(out, f, indent=2)

out_path

### Tool (2026-02-20 22:37:28)

'/mnt/data/phase_bin_activity_classifier.json'

### Assistant (2026-02-20 22:37:32)

Here. Your shiny JSON file, produced exactly as specified, using **only** phase bins from the **first 30 true zeta zeros** and labels derived from the **true** Chebyshev ψ data.

[Download phase_bin_activity_classifier.json](sandbox:/mnt/data/phase_bin_activity_classifier.json)

