# Hilbert phase prediction

- **Conversation ID:** `699934e5-5414-832f-b630-6597f1531f80`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:30:49)

You are a stateless compute worker.

You MUST create a file named:

hilbert_phase_prediction.json

Do NOT print JSON to stdout.

⸻

Goal

Extract a phase directly from ψ(x)−x using a Hilbert transform in log-space and test whether it predicts forward prime-density residual.

⸻

Step 1: prepare signal

From chebyshev_psi.npz:
	•	x_grid
	•	psi_minus_x

Compute:
t = log(x_grid)

Interpolate psi_minus_x onto a uniform grid in t (size 65536).

Remove global trend:
subtract a moving average with window = 2000 samples.

Call result:
f(t)

⸻

Step 2: analytic signal

Compute analytic signal via Hilbert transform:

A(t) = f(t) + i * Hilbert(f(t))

Define phase:
φ(t) = angle(A(t)) mod 2π

⸻

Step 3: prediction dataset

Keep x in [1e5, 9e6]

Split by time order:
70% train
30% test

For each t:

Count primes in forward window:
[x, x*exp(Δ)), Δ=0.01

Expected:
E = x*(exp(Δ)-1)/log(x)

Residual:
R=(P−E)/sqrt(E)

Define label:
y = 1 if R > median(R_train)

⸻

Step 4: phase-bin predictor

Use B=12 bins on φ(t).

Compute p_bin on TRAIN.

Predict probabilities on TEST.

Compute:
	•	AUC
	•	accuracy@0.5
	•	Brier score

⸻

Step 5: output

Write ONLY:

hilbert_phase_prediction.json

containing:
{
“AUC_test”: value,
“accuracy_test”: value,
“Brier_test”: value,
“train_pos_rate”: value,
“test_pos_rate”: value
}

No plots. No other files.

### Assistant (2026-02-20 23:30:50)

import numpy as np, json, os, math, warnings
from numpy.lib.stride_tricks import sliding_window_view

psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"

psi=np.load(psi_path)
psi.files[:10], psi['x_grid'].shape, psi['psi_minus_x'].shape

### Tool (2026-02-20 23:31:03)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'], (100000,), (100000,))

### Assistant (2026-02-20 23:31:05)

x_grid=psi['x_grid']
x_grid.min(), x_grid.max(), x_grid[:5], x_grid[-5:]

### Tool (2026-02-20 23:31:07)

(1,
 9999901,
 array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32))

### Assistant (2026-02-20 23:31:09)

import scipy, scipy.signal, scipy.interpolate, scipy.stats
scipy.__version__

### Tool (2026-02-20 23:31:14)

'1.14.1'

### Assistant (2026-02-20 23:31:16)

pr=np.load(primes_path)
pr.files, {k: pr[k].shape for k in pr.files}

### Tool (2026-02-20 23:31:22)

(['N', 'primes', 'is_prime'],
 {'N': (), 'primes': (664579,), 'is_prime': (10000001,)})

### Assistant (2026-02-20 23:31:24)

import sklearn
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
sklearn.__version__

### Tool (2026-02-20 23:31:31)

'1.4.2'

### Assistant (2026-02-20 23:31:32)

from scipy.signal import hilbert

psi_minus_x=psi['psi_minus_x'].astype(np.float64)
t=np.log(x_grid.astype(np.float64))

# uniform grid
N_uniform=65536
t_uniform=np.linspace(t.min(), t.max(), N_uniform)
f_interp=np.interp(t_uniform, t, psi_minus_x)

# moving average detrend
window=2000
kernel=np.ones(window)/window
pad=window//2
# reflect padding to reduce edge artifacts
f_pad=np.pad(f_interp, (pad, pad-1 if window%2==0 else pad), mode='reflect')
ma=np.convolve(f_pad, kernel, mode='valid')
f=f_interp - ma

# analytic signal
A = hilbert(f)
phi = np.angle(A)
phi = np.mod(phi, 2*np.pi)

x = np.exp(t_uniform)

# keep range [1e5,9e6]
mask = (x>=1e5) & (x<=9e6)
x_m = x[mask]
phi_m = phi[mask]

# prime counts in forward window
primes = pr['primes'].astype(np.int64)
Delta=0.01
x2 = x_m * np.exp(Delta)
idx1 = np.searchsorted(primes, x_m, side='left')
idx2 = np.searchsorted(primes, x2, side='left')
P = (idx2 - idx1).astype(np.float64)

E = x_m*(np.exp(Delta)-1)/np.log(x_m)
R = (P - E)/np.sqrt(E)

n=len(R)
train_n=int(0.7*n)
R_train=R[:train_n]
R_test=R[train_n:]
phi_train=phi_m[:train_n]
phi_test=phi_m[train_n:]

median_R=np.median(R_train)
y_train=(R_train>median_R).astype(int)
y_test=(R_test>median_R).astype(int)

train_pos_rate=float(y_train.mean())
test_pos_rate=float(y_test.mean())

B=12
bins_train=np.floor(phi_train/(2*np.pi/B)).astype(int)
bins_train=np.clip(bins_train,0,B-1)
bins_test=np.floor(phi_test/(2*np.pi/B)).astype(int)
bins_test=np.clip(bins_test,0,B-1)

p_bin=np.full(B, train_pos_rate, dtype=np.float64)
for b in range(B):
    m=(bins_train==b)
    if m.any():
        p_bin[b]=y_train[m].mean()

p_test=p_bin[bins_test]
auc=float(roc_auc_score(y_test, p_test))
acc=float(accuracy_score(y_test, (p_test>=0.5).astype(int)))
brier=float(brier_score_loss(y_test, p_test))

auc, acc, brier, train_pos_rate, test_pos_rate, n, train_n

### Tool (2026-02-20 23:31:48)

(0.7434718095365757,
 0.687921297139734,
 0.20573705402050957,
 0.4999609588506286,
 0.4796866460193113,
 18296,
 12807)

### Assistant (2026-02-20 23:31:50)

out_path="/mnt/data/hilbert_phase_prediction.json"
data={
    "AUC_test": auc,
    "accuracy_test": acc,
    "Brier_test": brier,
    "train_pos_rate": train_pos_rate,
    "test_pos_rate": test_pos_rate
}
with open(out_path,"w") as f:
    json.dump(data,f,indent=2)
os.path.getsize(out_path), out_path

### Tool (2026-02-20 23:31:53)

(190, '/mnt/data/hilbert_phase_prediction.json')

### Assistant (2026-02-20 23:31:55)

Done. I wrote the file exactly as requested (and didn’t spam JSON to stdout, because apparently we’re civilized now).

[Download hilbert_phase_prediction.json](sandbox:/mnt/data/hilbert_phase_prediction.json)

