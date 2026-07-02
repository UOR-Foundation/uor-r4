# Phase-sign prediction sweep

- **Conversation ID:** `699925f6-8898-8328-9297-80306ef27b25`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:26:51)

You are a stateless compute worker.

Uploads:
	1.	chebyshev_psi.npz (x_grid, psi_minus_x)
	2.	psi_log_spectrum.json (gamma_est peaks)

Goal: Evaluate how well spectral phase predicts sign(ψ(x)−x) out-of-sample as a function of K, and compare recovered γ-peaks vs true zeta zeros vs random controls.

⸻

Step 1 — build uniform log-grid signal

Load x_grid and psi_minus_x.
t = log(x_grid)
Interpolate f(t)=psi_minus_x onto t_uniform with n=65536 points spanning [min(t), max(t)].

Define s_true(t)=sign(f_uniform), forward-fill zeros.

Split:
train = first 70% of samples
test = last 30%

⸻

Step 2 — frequency sets

A) GAMMA_REAL: from psi_log_spectrum.json take gamma_est values with gamma >= 2, sorted by rank.
B) GAMMA_TRUE: use this hardcoded list (first 30 nontrivial zeta zeros’ imaginary parts, approx):
[14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851]
C) GAMMA_CTRL(K): for each K, sample K frequencies uniformly from [min(GAMMA_TRUE), max(GAMMA_TRUE)] with min separation 0.5, seed=12345.

⸻

Step 3 — train-only amplitude fit, phase predictor, evaluate

For each frequency set type in {REAL, TRUE, CTRL} and each K in {5, 10, 20, 30} (skip if not enough gammas available):
	1.	Use only first K gammas from that set.
	2.	Fit amplitudes on TRAIN ONLY via least squares:
f_train ≈ Σ a_k cos(gamma_k t) + b_k sin(gamma_k t)
	3.	Build complex field on ALL samples:
Z(t)=Σ (a_k - i b_k) * exp(i gamma_k t)
φ(t)=unwrap(angle(Z(t)))
	4.	Predict sign using phase only:
pred = +1 if cos(φ)>0 else -1
Compute accuracy on TRAIN and TEST.

Also compute reconstruction R² on TRAIN and TEST using the real part model.

⸻

Step 4 — output (single JSON only)

Return exactly one file: gamma_phase_k_sweep.json

Include:
	•	accuracies and R² for each set type and K on train/test
	•	for CTRL, include mean over 5 different seeds (12345..12349) to reduce luck (still no extra files)
	•	a short summary: best TEST accuracy achieved and by which set/K

No plots. No extra files.

### Tool (2026-02-20 22:26:53)

Make sure to include fileciteturn0file0 in your response to cite this file, or to surface it as a link.
{
  "n_grid_points": 65536,
  "t_range": {
    "min": 0.0,
    "max": 16.118085750909312
  },
  "top_25_peaks": [
    {
      "rank": 1,
      "gamma_est": 14.03337983785895,
      "relative_power": 1.0
    },
    {
      "rank": 2,
      "gamma_est": 0.38981610660719307,
      "relative_power": 0.9419401097763271
    },
    {
      "rank": 3,
      "gamma_est": 21.050069756788425,
      "relative_power": 0.41895728747639754
    },
    {
      "rank": 4,
      "gamma_est": 24.948230822860356,
      "relative_power": 0.3021532205835205
    },
    {
      "rank": 5,
      "gamma_est": 30.405656315361057,
      "relative_power": 0.2358422235553156
    },
    {
      "rank": 6,
      "gamma_est": 32.74455295500422,
      "relative_power": 0.15452431469743916
    },
    {
      "rank": 7,
      "gamma_est": 37.42234623429053,
      "relative_power": 0.1411210153121425
    },
    {
      "rank": 8,
      "gamma_est": 40.930691193755266,
      "relative_power": 0.12140196856305495
    },
    {
      "rank": 9,
      "gamma_est": 43.269587833398425,
      "relative_power": 0.11076893095248755
    },
    {
      "rank": 10,
      "gamma_est": 47.947381112684745,
      "relative_power": 0.07796839577418356
    },
    {
      "rank": 11,
      "gamma_est": 49.89646164572071,
      "relative_power": 0.07182697556101642
    },
    {
      "rank": 12,
      "gamma_est": 56.523335458042986,
      "relative_power": 0.06521326847461034
    },
    {
      "rank": 13,
      "gamma_est": 53.01499049857825,
      "relative_power": 0.058525555336305476
    },
    {
      "rank": 14,
      "gamma_est": 60.811312630722114,
      "relative_power": 0.056821468418261405
    },
    {
      "rank": 15,
      "gamma_est": 59.25204820429334,
      "relative_power": 0.05183161301746724
    },
    {
      "rank": 16,
      "gamma_est": 65.09928980340123,
      "relative_power": 0.04468088100666298
    },
    {
      "rank": 17,
      "gamma_est": 77.18358910822423,
      "relative_power": 0.04211897364225764
    },
    {
      "rank": 18,
      "gamma_est": 67.0483703364372,
      "relative_power": 0.038787614582797995
    },
    {
      "rank": 19,
      "gamma_est": 75.62432468179544,
      "relative_power": 0.03496542331177256
    },
    {
      "rank": 20,
      "gamma_est": 69.38726697608035,
      "relative_power": 0.0346659498247792
    },
    {
      "rank": 21,
      "gamma_est": 83.03083070733213,
      "relative_power": 0.034259049199107376
    },
    {
      "rank": 22,
      "gamma_est": 79.52248574786738,
      "relative_power": 0.032594659979818946
    },
    {
      "rank": 23,
      "gamma_est": 72.1159797223307,
      "relative_power": 0.029427805988419927
    },
    {
      "rank": 24,
      "gamma_est": 87.31880788001124,
      "relative_power": 0.027819441293874102
    },
    {
      "rank": 25,
      "gamma_est": 88.87807230644002,
      "relative_power": 0.025103011009955954
    }
  ]
}

### Tool (2026-02-20 22:26:53)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-20 22:26:52)

import json, math, numpy as np
from pathlib import Path

# Paths
npz_path = Path("/mnt/data/chebyshev_psi.npz")
spectrum_path = Path("/mnt/data/psi_log_spectrum.json")

# Load data
npz = np.load(npz_path)
x_grid = npz["x_grid"].astype(float)
psi_minus_x = npz["psi_minus_x"].astype(float)

with open(spectrum_path, "r") as f:
    spectrum = json.load(f)

# Step 1: build uniform log-grid signal
t = np.log(x_grid)
t_min, t_max = float(np.min(t)), float(np.max(t))
n = 65536
t_uniform = np.linspace(t_min, t_max, n)

# Interpolate psi_minus_x onto t_uniform
# Ensure t is increasing for interpolation
order = np.argsort(t)
t_sorted = t[order]
y_sorted = psi_minus_x[order]

f_uniform = np.interp(t_uniform, t_sorted, y_sorted)

# sign with forward-fill zeros
s_true = np.sign(f_uniform).astype(int)
# forward-fill zeros using previous non-zero; for leading zeros, use first subsequent non-zero else 1
if np.any(s_true == 0):
    # find first non-zero
    nz_idx = np.flatnonzero(s_true != 0)
    if len(nz_idx) == 0:
        s_true[:] = 1
    else:
        first_nz = nz_idx[0]
        # fill leading zeros
        if first_nz > 0:
            s_true[:first_nz] = s_true[first_nz]
        # forward fill
        for i in range(first_nz + 1, len(s_true)):
            if s_true[i] == 0:
                s_true[i] = s_true[i-1]

# Split train/test
n_train = int(round(0.70 * n))
train_slice = slice(0, n_train)
test_slice = slice(n_train, n)

t_train = t_uniform[train_slice]
t_test = t_uniform[test_slice]
f_train = f_uniform[train_slice]
f_test = f_uniform[test_slice]
s_train = s_true[train_slice]
s_test = s_true[test_slice]

# Step 2: frequency sets
# A) GAMMA_REAL from spectrum json: take gamma_est values with gamma >= 2, sorted by rank
gamma_real = []
for p in spectrum.get("top_25_peaks", []):
    g = float(p.get("gamma_est"))
    if g >= 2:
        gamma_real.append(g)
# Also consider if file has other peaks list
if "peaks" in spectrum and isinstance(spectrum["peaks"], list):
    # if provided, take rank order as given
    for p in spectrum["peaks"]:
        try:
            g = float(p.get("gamma_est", p.get("gamma", p)))
        except Exception:
            continue
        if g >= 2:
            gamma_real.append(g)

# de-duplicate while preserving order
seen = set()
gamma_real_unique = []
for g in gamma_real:
    # round for dedup
    key = round(g, 12)
    if key not in seen:
        seen.add(key)
        gamma_real_unique.append(g)
gamma_real = gamma_real_unique

# B) hardcoded true list
gamma_true = [
    14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
    52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
    79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851
]

gmin, gmax = min(gamma_true), max(gamma_true)

def sample_ctrl(K, seed):
    rng = np.random.default_rng(seed)
    # rejection sampling with min separation 0.5
    selected = []
    attempts = 0
    while len(selected) < K and attempts < 200000:
        attempts += 1
        cand = float(rng.uniform(gmin, gmax))
        if all(abs(cand - s) >= 0.5 for s in selected):
            selected.append(cand)
    if len(selected) < K:
        return None
    selected.sort()
    return selected

# Step 3: fit and evaluate
def fit_and_eval(gammas):
    K = len(gammas)
    # design matrices
    # For numerical stability, create cos/sin columns
    cos_train = np.cos(np.outer(t_train, gammas))
    sin_train = np.sin(np.outer(t_train, gammas))
    X_train = np.hstack([cos_train, sin_train])

    # least squares on train
    coeffs, residuals, rank, svals = np.linalg.lstsq(X_train, f_train, rcond=None)
    a = coeffs[:K]
    b = coeffs[K:]

    # Predict f on train/test via real model
    def f_hat(tarr):
        return (np.cos(np.outer(tarr, gammas)) @ a) + (np.sin(np.outer(tarr, gammas)) @ b)

    fhat_train = f_hat(t_train)
    fhat_test = f_hat(t_test)

    # R^2
    def r2(y, yhat):
        ss_res = float(np.sum((y - yhat)**2))
        ss_tot = float(np.sum((y - float(np.mean(y)))**2))
        if ss_tot == 0:
            return 0.0
        return 1.0 - ss_res/ss_tot

    r2_train = r2(f_train, fhat_train)
    r2_test = r2(f_test, fhat_test)

    # Build complex field Z(t) on all samples: Z = Σ (a_k - i b_k) exp(i gamma_k t)
    # efficient: compute exp(i t * gammas) matrix
    exp_all = np.exp(1j * np.outer(t_uniform, gammas))
    c = a - 1j*b
    Z = exp_all @ c

    phi = np.unwrap(np.angle(Z))
    pred = np.where(np.cos(phi) > 0, 1, -1).astype(int)

    acc_train = float(np.mean(pred[train_slice] == s_train))
    acc_test = float(np.mean(pred[test_slice] == s_test))

    return {
        "K": K,
        "gammas": [float(g) for g in gammas],
        "train": {"accuracy": acc_train, "r2": float(r2_train)},
        "test": {"accuracy": acc_test, "r2": float(r2_test)},
    }

Ks = [5, 10, 20, 30]
results = {"REAL": {}, "TRUE": {}, "CTRL": {}}

# REAL
for K in Ks:
    if len(gamma_real) >= K:
        results["REAL"][str(K)] = fit_and_eval(gamma_real[:K])

# TRUE
for K in Ks:
    if len(gamma_true) >= K:
        results["TRUE"][str(K)] = fit_and_eval(gamma_true[:K])

# CTRL mean over 5 seeds
ctrl_seeds = list(range(12345, 12350))
for K in Ks:
    per_seed = []
    for seed in ctrl_seeds:
        gam = sample_ctrl(K, seed)
        if gam is None:
            continue
        per_seed.append(fit_and_eval(gam))
    if len(per_seed) == 0:
        continue
    # mean metrics
    mean_train_acc = float(np.mean([d["train"]["accuracy"] for d in per_seed]))
    mean_test_acc = float(np.mean([d["test"]["accuracy"] for d in per_seed]))
    mean_train_r2 = float(np.mean([d["train"]["r2"] for d in per_seed]))
    mean_test_r2 = float(np.mean([d["test"]["r2"] for d in per_seed]))

    results["CTRL"][str(K)] = {
        "K": K,
        "seeds": ctrl_seeds,
        "train": {"accuracy_mean": mean_train_acc, "r2_mean": mean_train_r2},
        "test": {"accuracy_mean": mean_test_acc, "r2_mean": mean_test_r2},
        "per_seed": [
            {
                "seed": seed,
                "gammas": d["gammas"],
                "train": d["train"],
                "test": d["test"],
            }
            for seed, d in zip(ctrl_seeds, per_seed)
        ],
    }

# Step 4: summary
best = {"test_accuracy": -1.0, "set": None, "K": None, "detail": None}
for set_type in ["REAL", "TRUE"]:
    for K, d in results[set_type].items():
        acc = d["test"]["accuracy"]
        if acc > best["test_accuracy"]:
            best = {"test_accuracy": acc, "set": set_type, "K": int(K), "detail": d}
for K, d in results["CTRL"].items():
    acc = d["test"]["accuracy_mean"]
    if acc > best["test_accuracy"]:
        best = {"test_accuracy": acc, "set": "CTRL", "K": int(K), "detail": d}

output = {
    "meta": {
        "n_uniform": n,
        "t_range": {"min": t_min, "max": t_max},
        "train_fraction": 0.70,
        "ctrl_seeds": ctrl_seeds,
        "min_ctrl_separation": 0.5,
        "notes": "Phase predictor: pred=+1 if cos(phi)>0 else -1; phi=unwrap(angle(Z)); Z=Σ(a_k - i b_k)exp(i gamma_k t).",
    },
    "results": results,
    "summary": {
        "best_test_accuracy": float(best["test_accuracy"]),
        "best_by": {"set_type": best["set"], "K": best["K"]},
    },
}

out_path = Path("/mnt/data/gamma_phase_k_sweep.json")
with open(out_path, "w") as f:
    json.dump(output, f, indent=2)

str(out_path), output["summary"]

### Tool (2026-02-20 22:27:28)

('/mnt/data/gamma_phase_k_sweep.json',
 {'best_test_accuracy': 0.820405879660241,
  'best_by': {'set_type': 'TRUE', 'K': 30}})

### Assistant (2026-02-20 22:28:01)

Here. Your precious out-of-sample phase-sign sweep is done, and it only took bending math into a cos(unwrap(angle(Σ…))) pretzel.

- Used `psi_log_spectrum.json` for GAMMA_REAL peaks fileciteturn0file0  
- Wrote exactly one output file: **`gamma_phase_k_sweep.json`**

**Download:** [gamma_phase_k_sweep.json](sandbox:/mnt/data/gamma_phase_k_sweep.json)

