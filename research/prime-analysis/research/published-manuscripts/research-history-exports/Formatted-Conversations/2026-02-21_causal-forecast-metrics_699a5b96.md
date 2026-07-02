# Causal forecast metrics

- **Conversation ID:** `699a5b96-4b08-8333-b81d-4ad8fd9d8ae1`
- **Date:** 2026-02-21

---

### User (2026-02-21 20:27:52)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Causal forecast of prime residual activity using ONLY past ψ-signal features (no future leakage).

Recompute everything:
1) primes up to N=10_000_000
2) ψ(x) on step-100 grid incl prime powers
3) f(x)=ψ(x)−x
4) uniform log grid size 65536 (t=log x)
Let u(t) be interpolated f on this grid.

Define target at each time index i:
- Δ=0.01
- x=exp(t[i])
- P = #primes in [x, x*exp(Δ))
- E = x*(exp(Δ)-1)/log(x)
- R=(P−E)/sqrt(E)
Label y=1 if R > median(R_train) else 0

Split time-ordered: 70% TRAIN, 30% TEST.

Features at time i must use ONLY u up to i (past-causal).
Compute features from a trailing window of length W=512 samples:
- mean(u), std(u)
- mean(abs(diff(u)))
- energy in 3 FFT bands of u-window: [low, mid, high] within [0,0.02] cycles/sample
- last value u[i]
- slope of u over window (linear fit)
- optionally: sign of u[i] and sign of slope

Fit a logistic regression on TRAIN to predict y at the SAME i (or y at i+1; pick i+1 for true forecast):
Use y_{i+1} with features from window ending at i (one-step-ahead).

Evaluate on TEST:
- AUC
- Brier
- accuracy at 0.5

Also report baseline:
- predict y_{i+1} using y_i (persistence baseline)

Output JSON:
{
"setup":{"W":512,"Delta":0.01,"forecast_horizon":1},
"metrics":{"AUC":...,"Brier":...,"accuracy":...},
"baseline_persistence":{"AUC":...,"Brier":...,"accuracy":...}
}

No other text.

### Assistant (2026-02-21 20:27:53)

import numpy as np, math, time, json, sys, statistics, os, textwrap

### Assistant (2026-02-21 20:28:33)

import numpy as np, math, time
N = 10_000_000
start_time=time.time()
# Sieve using numpy for speed/memory
is_prime = np.ones(N+1, dtype=bool)
is_prime[:2]=False
is_prime[4::2]=False
is_prime[2]=True
limit=int(math.isqrt(N))
for p in range(3, limit+1, 2):
    if is_prime[p]:
        step = p*2
        start = p*p
        is_prime[start:N+1:step]=False
primes = np.nonzero(is_prime)[0]
len(primes), time.time()-start_time

### Tool (2026-02-21 20:28:37)

(664579, 0.05929851531982422)

### Assistant (2026-02-21 20:28:42)

start_time=time.time()
psi_delta = np.zeros(N+1, dtype=np.float64)
logs = np.log(primes.astype(np.float64))
for p, lp in zip(primes, logs):
    pk = p
    # add log p at prime powers
    while pk <= N:
        psi_delta[pk] += lp
        pk *= p
psi = np.cumsum(psi_delta)
time.time()-start_time, psi[-1]

### Tool (2026-02-21 20:28:45)

(0.9080533981323242, 9998539.403345892)

### Assistant (2026-02-21 20:28:49)

x_step = np.arange(100, N+1, 100, dtype=np.int64)
psi_grid = psi[x_step]
f_grid = psi_grid - x_step.astype(np.float64)
t_step = np.log(x_step.astype(np.float64))
len(x_step), f_grid[:3], f_grid[-1]

### Tool (2026-02-21 20:28:51)

(100000, array([-5.95468877,  6.14585683, -0.76766456]), -1460.5966541077942)

### Assistant (2026-02-21 20:28:54)

M = 65536
t_min = math.log(float(x_step[0]))
t_max = math.log(float(x_step[-1]))
t = np.linspace(t_min, t_max, M, dtype=np.float64)
u = np.interp(t, t_step, f_grid)
# determine valid indices where x*exp(Delta) <= N
Delta=0.01
expD = math.exp(Delta)
x = np.exp(t)
valid = x*expD <= N
m = int(np.argmax(~valid)) if (~valid).any() else M
m, M

### Tool (2026-02-21 20:28:57)

(65479, 65536)

### Assistant (2026-02-21 20:28:59)

pi = np.cumsum(is_prime.astype(np.int32))
pi[-1], pi[10]

### Tool (2026-02-21 20:29:00)

(664579, 4)

### Assistant (2026-02-21 20:29:03)

start_time=time.time()
x_m = x[:m]
a = x_m
b = x_m * expD
start_int = np.ceil(a).astype(np.int64)
end_int = (np.ceil(b).astype(np.int64) - 1)
start_int[start_int<2]=2
end_int[end_int>N]=N
# counts
P = pi[end_int] - pi[start_int-1]
# expectation
E = x_m*(expD-1.0)/np.log(x_m)
R = (P - E)/np.sqrt(E)
time.time()-start_time, R[:5], np.isfinite(R).all()

### Tool (2026-02-21 20:29:07)

(0.013872385025024414,
 array([1.67344534, 1.67326603, 1.67308671, 1.67290741, 1.67272811]),
 True)

### Assistant (2026-02-21 20:29:10)

train_end = int(0.7*m)
threshold = np.median(R[:train_end])
y = (R > threshold).astype(np.int8)
threshold, y.mean(), y[:10]

### Tool (2026-02-21 20:29:12)

(-0.06422663783045693,
 0.5081629224636907,
 array([1, 1, 1, 1, 1, 1, 1, 1, 1, 1], dtype=int8))

### Assistant (2026-02-21 20:29:33)

from numpy.lib.stride_tricks import sliding_window_view
W=512
# limit u to m indices for causality
u_m = u[:m]
windows = sliding_window_view(u_m, window_shape=W)  # shape (m-W+1, W)
windows.shape

### Tool (2026-02-21 20:29:35)

(64968, 512)

### Assistant (2026-02-21 20:29:44)

start_time=time.time()
win_use = windows[:-1]  # (n_samples, W)
n_samples = win_use.shape[0]
mean_u = win_use.mean(axis=1)
std_u = win_use.std(axis=1, ddof=0)
mad_diff = np.abs(np.diff(win_use, axis=1)).mean(axis=1)
last_u = win_use[:, -1]
# slope via linear regression on indices 0..W-1
idx = np.arange(W, dtype=np.float64)
idx_mean = idx.mean()
idx_centered = idx - idx_mean
den = np.sum(idx_centered**2)
# slope = sum((x-xmean)*(y-ymean))/den
# compute numerator: dot(idx_centered, y) - ymean*sum(idx_centered)=dot since sum idx_centered=0
num = win_use @ idx_centered
slope = num / den
# FFT band energies
U = np.fft.rfft(win_use, axis=1)
power = (U.real**2 + U.imag**2)
freqs = np.fft.rfftfreq(W, d=1.0)
# band masks within [0,0.02]
mask_total = (freqs >= 0) & (freqs <= 0.02)
# define bands
edges = [0.0, 0.02/3, 0.04/3, 0.02]
mask_low = (freqs >= edges[0]) & (freqs < edges[1])
mask_mid = (freqs >= edges[1]) & (freqs < edges[2])
mask_high = (freqs >= edges[2]) & (freqs <= edges[3])
energy_low = power[:, mask_low].sum(axis=1)
energy_mid = power[:, mask_mid].sum(axis=1)
energy_high = power[:, mask_high].sum(axis=1)
sign_u = np.sign(last_u)
sign_slope = np.sign(slope)
# assemble X
X = np.column_stack([mean_u, std_u, mad_diff, energy_low, energy_mid, energy_high, last_u, slope, sign_u, sign_slope]).astype(np.float64)
y_target = y[W:W+n_samples]  # y_{i+1}
time.time()-start_time, X.shape, y_target.shape, y_target.mean()

### Tool (2026-02-21 20:29:53)

(2.703249454498291, (64967, 10), (64967,), 0.5086582418766451)

### Assistant (2026-02-21 20:29:58)

from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
split = int(0.7*n_samples)
X_train, X_test = X[:split], X[split:]
y_train, y_test = y_target[:split], y_target[split:]
model = Pipeline([
    ("scaler", StandardScaler()),
    ("lr", LogisticRegression(max_iter=2000, n_jobs=None, solver="lbfgs"))
])
start_time=time.time()
model.fit(X_train, y_train)
fit_time=time.time()-start_time
proba = model.predict_proba(X_test)[:,1]
pred = (proba>=0.5).astype(np.int8)
metrics = {
    "AUC": float(roc_auc_score(y_test, proba)),
    "Brier": float(brier_score_loss(y_test, proba)),
    "accuracy": float(accuracy_score(y_test, pred))
}
fit_time, metrics

### Tool (2026-02-21 20:30:01)

(3.9241576194763184,
 {'AUC': 0.6088451312502223,
  'Brier': 0.3985937626961094,
  'accuracy': 0.5349648555743677})

### Assistant (2026-02-21 20:30:10)

y_persist = y[W-1:W-1+n_samples].astype(np.float64)
# test slice
y_persist_test = y_persist[split:]
proba_base = y_persist_test
pred_base = (proba_base>=0.5).astype(np.int8)
baseline = {
    "AUC": float(roc_auc_score(y_test, proba_base)),
    "Brier": float(brier_score_loss(y_test, proba_base)),
    "accuracy": float(accuracy_score(y_test, pred_base))
}
baseline

### Tool (2026-02-21 20:30:13)

{'AUC': 0.907675472553682,
 'Brier': 0.09209378687599405,
 'accuracy': 0.907906213124006}

### Assistant (2026-02-21 20:30:15)

{
  "setup": {
    "W": 512,
    "Delta": 0.01,
    "forecast_horizon": 1
  },
  "metrics": {
    "AUC": 0.6088451312502223,
    "Brier": 0.3985937626961094,
    "accuracy": 0.5349648555743677
  },
  "baseline_persistence": {
    "AUC": 0.907675472553682,
    "Brier": 0.09209378687599405,
    "accuracy": 0.907906213124006
  }
}

