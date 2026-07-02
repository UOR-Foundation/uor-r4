# Hilbert robustness sweep

- **Conversation ID:** `699937f8-4ca0-8329-8776-ec8c9a187348`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:43:54)

You are a stateless compute worker.

You MUST write a file named:
hilbert_robustness_sweep.json
Do NOT print JSON to stdout.

Goal: Robustness sweep for Hilbert-phase prime residual prediction across detrend window W and forward window Δ.

⸻

Common pipeline
	•	Load x_grid, psi_minus_x, primes.
	•	Build uniform t_uniform (65536) and interpolate psi_minus_x to f_raw.
	•	Restrict x to [1e5, 9e6].
	•	Time-ordered split: 70% train, 30% test.

For each (W, Δ):
	1.	Detrend: f = f_raw - moving_average(f_raw, window=W)
	2.	Hilbert analytic signal → φ_mod = angle(A) mod 2π
	3.	Forward window: count primes P in [x, x*exp(Δ))
	4.	Expected E = x*(exp(Δ)-1)/log(x)
	5.	Residual R=(P-E)/sqrt(E)
	6.	Label y = 1 if R > median(R_train) else 0
	7.	Fit Fourier logistic on TRAIN using features up to 3rd harmonic:
[1, cosφ, sinφ, cos2φ, sin2φ, cos3φ, sin3φ]
	8.	Evaluate on TEST: AUC, accuracy@0.5, Brier

⸻

Sweep grid

W in {500, 1000, 2000, 4000}
Δ in {0.005, 0.01, 0.02}

(12 runs total)

⸻

Output

Write ONLY hilbert_robustness_sweep.json containing:
	•	x_range used
	•	for each (W,Δ): {AUC, accuracy, Brier, train_pos_rate, test_pos_rate}
	•	identify best AUC run and best Brier run

No plots. No other files.

### Assistant (2026-02-20 23:43:56)

import numpy as np, os, json, math
psi_path="/mnt/data/chebyshev_psi.npz"
pr_path="/mnt/data/primes.npz"
psi=np.load(psi_path)
psi.files, pr_path

### Tool (2026-02-20 23:44:03)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'], '/mnt/data/primes.npz')

### Assistant (2026-02-20 23:44:06)

pr=np.load(pr_path)
pr.files[:20], {k: pr[k].shape for k in pr.files}

### Tool (2026-02-20 23:44:09)

(['N', 'primes', 'is_prime'],
 {'N': (), 'primes': (664579,), 'is_prime': (10000001,)})

### Assistant (2026-02-20 23:44:11)

x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
x_grid.shape, psi_minus_x.shape, (x_grid[0], x_grid[-1]), (psi_minus_x[0], psi_minus_x[-1])

### Tool (2026-02-20 23:44:13)

((100000,), (100000,), (1, 9999901), (-1.0, -1490.5413775146008))

### Assistant (2026-02-20 23:44:16)

step=psi["step"].item()
step, x_grid[1]-x_grid[0], x_grid[10]-x_grid[0]

### Tool (2026-02-20 23:44:17)

(100, 100, 1000)

### Assistant (2026-02-20 23:44:20)

import numpy as np, math, json, os
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score

# Load data
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"].astype(float)
psi_minus_x=psi["psi_minus_x"].astype(float)

pr=np.load("/mnt/data/primes.npz")
primes=pr["primes"].astype(np.int64)

# Restrict x range
x_min, x_max = 1e5, 9e6
mask = (x_grid>=x_min) & (x_grid<=x_max)
x_r = x_grid[mask]
y_r = psi_minus_x[mask]

t_grid = np.log(x_r)
t_uniform = np.linspace(t_grid[0], t_grid[-1], 65536)
x_uniform = np.exp(t_uniform)

# interpolate psi_minus_x to uniform t
f_raw = np.interp(t_uniform, t_grid, y_r)

def moving_average_reflect(x, window:int):
    if window<=1:
        return x.copy()
    # ensure odd window for symmetric smoothing
    w = int(window)
    if w%2==0:
        w += 1
    pad = w//2
    xpad = np.pad(x, pad_width=pad, mode='reflect')
    kernel = np.ones(w, dtype=float)/w
    return np.convolve(xpad, kernel, mode='valid')

def hilbert_analytic(x):
    # FFT-based Hilbert transform, returns analytic signal
    N = x.shape[0]
    Xf = np.fft.fft(x)
    h = np.zeros(N)
    if N%2==0:
        h[0]=1
        h[N//2]=1
        h[1:N//2]=2
    else:
        h[0]=1
        h[1:(N+1)//2]=2
    return np.fft.ifft(Xf*h)

def prime_counts_in_windows(x_vals, delta):
    # primes p with x<=p< x*exp(delta)
    upper = x_vals*np.exp(delta)
    lo = np.searchsorted(primes, x_vals, side='left')
    hi = np.searchsorted(primes, upper, side='left')
    return (hi-lo).astype(np.int32)

def expected_counts(x_vals, delta):
    return x_vals*(np.exp(delta)-1.0)/np.log(x_vals)

Ws=[500,1000,2000,4000]
Ds=[0.005,0.01,0.02]

n = len(x_uniform)
split = int(0.7*n)
results={}
for W in Ws:
    # detrend
    ma = moving_average_reflect(f_raw, W)
    f = f_raw - ma
    A = hilbert_analytic(f)
    phi = np.angle(A)
    phi_mod = np.mod(phi, 2*np.pi)
    # features
    c1=np.cos(phi_mod); s1=np.sin(phi_mod)
    c2=np.cos(2*phi_mod); s2=np.sin(2*phi_mod)
    c3=np.cos(3*phi_mod); s3=np.sin(3*phi_mod)
    X = np.column_stack([np.ones(n), c1,s1,c2,s2,c3,s3])
    X_train, X_test = X[:split], X[split:]
    x_train, x_test = x_uniform[:split], x_uniform[split:]
    for D in Ds:
        P = prime_counts_in_windows(x_uniform, D).astype(float)
        E = expected_counts(x_uniform, D)
        # guard tiny E (shouldn't happen)
        E = np.clip(E, 1e-9, None)
        R = (P - E)/np.sqrt(E)
        R_train = R[:split]
        med = np.median(R_train)
        y = (R > med).astype(int)
        y_train, y_test = y[:split], y[split:]
        # fit logistic
        clf = LogisticRegression(C=1e6, solver='lbfgs', max_iter=2000)
        clf.fit(X_train, y_train)
        p_test = clf.predict_proba(X_test)[:,1]
        auc = roc_auc_score(y_test, p_test)
        acc = float(np.mean((p_test>=0.5)==y_test))
        brier = float(np.mean((p_test - y_test)**2))
        key=f"W{W}_D{D}"
        results[key]={
            "W": int(W),
            "Delta": float(D),
            "AUC": float(auc),
            "accuracy": acc,
            "Brier": brier,
            "train_pos_rate": float(y_train.mean()),
            "test_pos_rate": float(y_test.mean()),
        }

# identify best runs
best_auc_key=max(results, key=lambda k: results[k]["AUC"])
best_brier_key=min(results, key=lambda k: results[k]["Brier"])
out={
    "x_range_used":[float(x_r[0]), float(x_r[-1])],
    "t_uniform_N": int(n),
    "split_train_frac": 0.7,
    "runs": results,
    "best_AUC_run": {"key": best_auc_key, **{k: results[best_auc_key][k] for k in ["W","Delta","AUC","accuracy","Brier"]}},
    "best_Brier_run": {"key": best_brier_key, **{k: results[best_brier_key][k] for k in ["W","Delta","AUC","accuracy","Brier"]}},
}

out_path="/mnt/data/hilbert_robustness_sweep.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, best_auc_key, best_brier_key, results[best_auc_key], results[best_brier_key]

### Tool (2026-02-20 23:44:53)

('/mnt/data/hilbert_robustness_sweep.json',
 'W1000_D0.01',
 'W500_D0.01',
 {'W': 1000,
  'Delta': 0.01,
  'AUC': 0.8500142458776063,
  'accuracy': 0.7829205025176745,
  'Brier': 0.15674233189255302,
  'train_pos_rate': 0.4999891008174387,
  'test_pos_rate': 0.47825644677279894},
 {'W': 500,
  'Delta': 0.01,
  'AUC': 0.8469358880767717,
  'accuracy': 0.7875998168963939,
  'Brier': 0.1560112655751158,
  'train_pos_rate': 0.4999891008174387,
  'test_pos_rate': 0.47825644677279894})

### Assistant (2026-02-20 23:45:17)

Wrote the file you demanded (because humans love demanding things):

[hilbert_robustness_sweep.json](sandbox:/mnt/data/hilbert_robustness_sweep.json)

