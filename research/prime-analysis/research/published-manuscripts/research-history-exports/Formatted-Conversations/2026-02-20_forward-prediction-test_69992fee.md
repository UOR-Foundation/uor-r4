# Forward prediction test

- **Conversation ID:** `69992fee-22fc-8331-947f-3c17feb14e5d`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:09:32)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (primes array)

Goal: Correct forward prediction test: spectral phase at interval start predicts whether the next multiplicative window has above-median prime count. Use TRAIN-only fitting and TRAIN-only thresholding. One JSON output.

⸻

Step 1 — uniform log grid

Load x_grid and psi_minus_x.
Let f = psi_minus_x.
Let t = log(x_grid).

Interpolate f onto uniform t_uniform with 65536 points.
Split indices: TRAIN first 70%, TEST last 30%.

⸻

Step 2 — spectral phase from TRUE zeros (TRAIN-only amplitude fit)

Use first 30 true zeta zero imaginary parts (hardcode list, same as before).

Fit a_k, b_k on TRAIN ONLY:
f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Compute phase on ALL:
Z(t)=Σ (a_k - i b_k) exp(i γ_k t_uniform)
φ_mod = angle(Z) mod 2π

Bin into 12 bins:
bin = floor(12*φ_mod/(2π)) in {0..11}

⸻

Step 3 — forward prime-count windows (no leakage)

Load primes array.

For each index i in TRAIN and TEST:
x_i = exp(t_uniform[i])
window = [x_i, x_i*exp(0.01))
Count primes P_i in window using binary search.

Now define label threshold using TRAIN ONLY:
median_P_train = median(P_i for i in TRAIN)

Define y_i = 1 if P_i > median_P_train else 0
(Apply same threshold to TEST.)

Report class balance for TRAIN and TEST.

⸻

Step 4 — phase-bin predictor trained on TRAIN only

Compute on TRAIN:
p_bin[j] = mean(y_train where bin=j)
Also store n_bin_train[j].

On TEST:
pred_prob_i = p_bin[bin_i]

Metrics on TEST:
	•	AUC (must be finite if both classes exist)
	•	accuracy at threshold 0.5
	•	Brier score
	•	calibration: overall_predicted_mean and overall_actual_positive_rate
	•	per-bin actual_positive_rate_test and mean_prime_count_test

Sanity checks (must include):
	•	p_bin list (12 values) and confirm not all equal
	•	overall_predicted_mean should equal mean(pred_prob_i)
	•	if p_bin are all 0 or all 1, explicitly flag as BUG

⸻

Output (single JSON)

Return exactly one file:
forward_phase_prime_prediction_fixed.json

Include:
	•	median_P_train
	•	p_bin (12) and n_bin_train (12)
	•	AUC_test, accuracy_test, brier_test
	•	overall calibration
	•	per-bin test stats (n, actual_positive_rate, mean_prime_count)
	•	warnings (if any)

No plots. No extra files.

### Assistant (2026-02-20 23:09:36)

import numpy as np, json, math, os, statistics
from pathlib import Path

psi_path = Path("/mnt/data/chebyshev_psi.npz")
primes_path = Path("/mnt/data/primes.npz")

psi = np.load(psi_path)
x_grid = psi["x_grid"]
f = psi["psi_minus_x"]

pr = np.load(primes_path)
# attempt keys
pr.keys()

### Tool (2026-02-20 23:09:47)

KeysView(<numpy.lib.npyio.NpzFile object at 0x7ec9aa9c1760>)

### Assistant (2026-02-20 23:09:49)

list(pr.keys())

### Tool (2026-02-20 23:09:50)

['N', 'primes', 'is_prime']

### Assistant (2026-02-20 23:09:51)

primes = pr["primes"]
x_grid[:5], x_grid[-5:], f[:5], f[-5:], primes[:5], primes[-5:], len(primes)

### Tool (2026-02-20 23:09:53)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 array([-1.        , -2.33956825,  5.14585683, -1.76766456,  2.82480716]),
 array([-1396.78466777, -1432.31246137, -1451.72215382, -1487.24987502,
        -1490.54137751]),
 array([ 2,  3,  5,  7, 11], dtype=int32),
 array([9999937, 9999943, 9999971, 9999973, 9999991], dtype=int32),
 664579)

### Assistant (2026-02-20 23:09:54)

import numpy as np, json, math
from pathlib import Path

# Step 1
t = np.log(x_grid.astype(np.float64))
t_min, t_max = float(t.min()), float(t.max())
n_uniform = 65536
t_uniform = np.linspace(t_min, t_max, n_uniform)
# interpolate f onto uniform t
f_uniform = np.interp(t_uniform, t, f.astype(np.float64))

# split
n_train = int(math.floor(0.70 * n_uniform))
train_idx = np.arange(n_train)
test_idx = np.arange(n_train, n_uniform)

f_train = f_uniform[train_idx]
t_train = t_uniform[train_idx]

# Step 2: zeros
gammas = np.array([
    14.134725141, 21.022039639, 25.010857580, 30.424876126, 32.935061588,
    37.586178159, 40.918719012, 43.327073281, 48.005150881, 49.773832478,
    52.970321478, 56.446247697, 59.347044003, 60.831778525, 65.112544048,
    67.079810529, 69.546401711, 72.067157674, 75.704690699, 77.144840069,
    79.337375020, 82.910380854, 84.735492981, 87.425274613, 88.809111208,
    92.491899271, 94.651344041, 95.870634228, 98.831194218, 101.317851006
], dtype=np.float64)

K = len(gammas)
# design matrix: [cos(g1 t), sin(g1 t), ..., cos(gK t), sin(gK t)]
X_cols = []
for g in gammas:
    X_cols.append(np.cos(g * t_train))
    X_cols.append(np.sin(g * t_train))
X = np.vstack(X_cols).T  # shape (n_train, 2K)

# fit least squares
coef, residuals, rank, s = np.linalg.lstsq(X, f_train, rcond=None)
a = coef[0::2]
b = coef[1::2]

# compute phase on all
# Z(t) = Σ (a_k - i b_k) exp(i γ_k t)
c = a - 1j*b
# compute exp matrix efficiently
# Z = sum_k c_k * exp(1j*gamma_k*t_uniform)
Z = np.zeros_like(t_uniform, dtype=np.complex128)
for ck, g in zip(c, gammas):
    Z += ck * np.exp(1j * g * t_uniform)

phi = np.angle(Z)
# mod to [0, 2pi)
phi_mod = np.mod(phi, 2*np.pi)
bins = np.floor(12 * phi_mod / (2*np.pi)).astype(int)
bins = np.clip(bins, 0, 11)

# Step 3: prime count windows
x_vals = np.exp(t_uniform)
factor = math.exp(0.01)
x_upper = x_vals * factor

lower_idx = np.searchsorted(primes, x_vals, side="left")
upper_idx = np.searchsorted(primes, x_upper, side="left")
P = (upper_idx - lower_idx).astype(int)

P_train = P[train_idx]
P_test = P[test_idx]

median_P_train = float(np.median(P_train))
y = (P > median_P_train).astype(int)
y_train = y[train_idx]
y_test = y[test_idx]

train_pos_rate = float(y_train.mean())
test_pos_rate = float(y_test.mean())

# Step 4: phase-bin predictor
warnings = []

overall_mean_train = float(y_train.mean())

p_bin = []
n_bin_train = []
for j in range(12):
    mask = (bins[train_idx] == j)
    n_j = int(mask.sum())
    n_bin_train.append(n_j)
    if n_j == 0:
        p_bin.append(None)
        warnings.append(f"Bin {j} has zero TRAIN samples; using overall TRAIN mean for prediction on that bin.")
    else:
        p_bin.append(float(y_train[mask].mean()))

# prediction on test
pred_prob_test = np.empty_like(y_test, dtype=np.float64)
for k_i, idx in enumerate(test_idx):
    b_i = bins[idx]
    p = p_bin[b_i]
    if p is None:
        p = overall_mean_train
    pred_prob_test[k_i] = p

# metrics
def auc_roc(y_true, scores):
    y_true = np.asarray(y_true, dtype=int)
    scores = np.asarray(scores, dtype=float)
    n_pos = int(y_true.sum())
    n = len(y_true)
    n_neg = n - n_pos
    if n_pos == 0 or n_neg == 0:
        return None
    # rank scores with average ties
    order = np.argsort(scores)
    scores_sorted = scores[order]
    y_sorted = y_true[order]
    ranks = np.empty(n, dtype=float)
    i = 0
    rank = 1
    while i < n:
        j = i
        while j + 1 < n and scores_sorted[j+1] == scores_sorted[i]:
            j += 1
        avg_rank = (rank + rank + (j - i)) / 2.0
        ranks[i:j+1] = avg_rank
        rank += (j - i + 1)
        i = j + 1
    # sum ranks of positives
    sum_ranks_pos = float((ranks * y_sorted).sum())
    auc = (sum_ranks_pos - n_pos*(n_pos+1)/2.0) / (n_pos * n_neg)
    return float(auc)

auc_test = auc_roc(y_test, pred_prob_test)
if auc_test is None:
    warnings.append("AUC_test undefined because TEST has a single class (all 0s or all 1s).")

pred_label_test = (pred_prob_test >= 0.5).astype(int)
accuracy_test = float((pred_label_test == y_test).mean())
brier_test = float(np.mean((pred_prob_test - y_test)**2))

overall_predicted_mean = float(pred_prob_test.mean())
overall_actual_positive_rate = float(y_test.mean())
# sanity: overall_predicted_mean equals mean(pred_prob_test)
if not np.isfinite(overall_predicted_mean):
    warnings.append("overall_predicted_mean is not finite.")
if abs(overall_predicted_mean - float(np.mean(pred_prob_test))) > 1e-12:
    warnings.append("Sanity check failed: overall_predicted_mean != mean(pred_prob_i).")

# per-bin test stats
per_bin_test = []
for j in range(12):
    mask = (bins[test_idx] == j)
    n_j = int(mask.sum())
    if n_j == 0:
        per_bin_test.append({
            "bin": j,
            "n": 0,
            "actual_positive_rate": None,
            "mean_prime_count": None
        })
    else:
        per_bin_test.append({
            "bin": j,
            "n": n_j,
            "actual_positive_rate": float(y_test[mask].mean()),
            "mean_prime_count": float(P_test[mask].mean())
        })

# sanity on p_bin equality / extremes
pb = [v for v in p_bin if v is not None]
if len(pb) > 0:
    # check not all equal
    if np.allclose(pb, pb[0], atol=1e-12, rtol=0):
        warnings.append("Sanity check: p_bin values are (effectively) all equal. This may indicate no predictive signal or a bug.")
    # BUG if all 0 or all 1 (ignoring None)
    if all((v == 0.0) for v in pb) or all((v == 1.0) for v in pb):
        warnings.append("BUG: p_bin are all 0 or all 1 (ignoring empty bins).")

# range coverage warnings
max_prime = int(primes[-1])
exceed = int(np.sum(x_upper > max_prime))
if exceed > 0:
    warnings.append(f"{exceed} windows have upper bound above max prime ({max_prime}); prime counts there may be truncated to available prime table.")
below2 = int(np.sum(x_vals < 2))
if below2 > 0:
    warnings.append(f"{below2} windows start below 2; prime counts there use first prime >= start.")

# class balance reporting
class_balance = {
    "train": {"n": int(len(train_idx)), "positive_rate": train_pos_rate, "positive_count": int(y_train.sum())},
    "test": {"n": int(len(test_idx)), "positive_rate": test_pos_rate, "positive_count": int(y_test.sum())}
}

out = {
    "median_P_train": median_P_train,
    "class_balance": class_balance,
    "p_bin": p_bin,
    "n_bin_train": n_bin_train,
    "AUC_test": auc_test,
    "accuracy_test": accuracy_test,
    "brier_test": brier_test,
    "calibration_test": {
        "overall_predicted_mean": overall_predicted_mean,
        "overall_actual_positive_rate": overall_actual_positive_rate
    },
    "per_bin_test_stats": per_bin_test,
    "sanity_checks": {
        "p_bin_not_all_equal": (False if (len(pb)>0 and np.allclose(pb, pb[0], atol=1e-12, rtol=0)) else True),
        "overall_predicted_mean_equals_mean_pred_prob": True
    },
    "warnings": warnings
}

# write file
out_path = Path("/mnt/data/forward_phase_prime_prediction_fixed.json")
with open(out_path, "w") as fp:
    json.dump(out, fp, indent=2)

out_path, train_pos_rate, test_pos_rate, median_P_train, auc_test, accuracy_test, brier_test, len(warnings)

### Tool (2026-02-20 23:10:54)

(PosixPath('/mnt/data/forward_phase_prime_prediction_fixed.json'),
 0.38035967302452317,
 1.0,
 1.0,
 None,
 0.0,
 0.3835653436122954,
 3)

### Tool (2026-02-20 23:11:05)

(['AUC_test undefined because TEST has a single class (all 0s or all 1s).',
  '41 windows have upper bound above max prime (9999991); prime counts there may be truncated to available prime table.',
  '2819 windows start below 2; prime counts there use first prime >= start.'],
 {'train': {'n': 45875,
   'positive_rate': 0.38035967302452317,
   'positive_count': 17449},
  'test': {'n': 19661, 'positive_rate': 1.0, 'positive_count': 19661}},
 [0.36921929388364,
  0.3459603432968334,
  0.3009433962264151,
  0.38494830488098103,
  0.39286672030034864],
 [4022, 3379, 4240, 4159, 3729],
 [{'bin': 0,
   'n': 1825,
   'actual_positive_rate': 1.0,
   'mean_prime_count': 1528.5090410958903},
  {'bin': 1,
   'n': 1735,
   'actual_positive_rate': 1.0,
   'mean_prime_count': 1819.1031700288183},
  {'bin': 2,
   'n': 1816,
   'actual_positive_rate': 1.0,
   'mean_prime_count': 1700.8705947136564}])

### Assistant (2026-02-20 23:11:06)

Done. Your “no leakage” forward test ran and wrote the **single JSON** you asked for (including TRAIN-only fit + TRAIN-only median thresholding, metrics, per-bin test stats, and sanity/warnings).

[Download forward_phase_prime_prediction_fixed.json](sandbox:/mnt/data/forward_phase_prime_prediction_fixed.json)

Notable outcome (also recorded in the JSON, so you don’t have to trust my carbon-based narration):
- **TEST ended up single-class (all y=1)** under the TRAIN median threshold, so **AUC is undefined** (flagged as a warning).
- Prime-table edge cases are also flagged (some windows below 2; some windows exceed the max prime in the provided table).

