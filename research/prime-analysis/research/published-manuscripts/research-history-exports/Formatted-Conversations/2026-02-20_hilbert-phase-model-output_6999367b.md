# Hilbert Phase Model Output

- **Conversation ID:** `6999367b-7408-8327-9de9-b6ca5a436661`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:38:15)

You are a stateless compute worker.

You MUST write a file named:
hilbert_phase_gating_model.json
Do NOT print JSON to stdout.

Goal: Use Hilbert phase φ(t) of the detrended fluctuation field to predict forward prime-density residual with a smooth gating model, evaluated out-of-sample.

⸻

Step 1 — replicate the successful Hilbert phase pipeline
	1.	Load x_grid, psi_minus_x.
	2.	t=log(x_grid). Interpolate psi_minus_x onto uniform t_uniform (65536).
	3.	Detrend by subtracting moving average window W=2000 samples.
	4.	Compute analytic signal via Hilbert transform and φ_mod = angle(A) mod 2π.

Restrict x to [1e5, 9e6].
Split in time order: 70% train, 30% test.

⸻

Step 2 — forward prime-density residual target

Using primes array, for each kept index:
	•	x=exp(t_uniform[i])
	•	Δ=0.01
	•	P = #primes in [x, x*exp(Δ))
	•	E = x*(exp(Δ)-1)/log(x)
	•	R=(P−E)/sqrt(E)

Label:
y=1 if R > median(R_train) else 0

⸻

Step 3 — smooth phase→probability model

Fit on TRAIN ONLY a circular logistic model using Fourier features:
Features: [1, cos(φ), sin(φ), cos(2φ), sin(2φ), cos(3φ), sin(3φ)]
Train logistic regression (or equivalent) to predict y.

Evaluate on TEST:
	•	AUC
	•	accuracy@0.5
	•	Brier

Also compute the old baseline (12-bin empirical p_bin) and report its TEST AUC/Brier for comparison.

⸻

Output

Write ONLY hilbert_phase_gating_model.json with:
{
“W”:2000,
“Delta”:0.01,
“train_pos_rate”:…,
“test_pos_rate”:…,
“fourier_model”: {“AUC”:…, “accuracy”:…, “Brier”:…, “coeffs”:[…]},
“bin12_model”: {“AUC”:…, “Brier”:…}
}

No plots. No other files. No stdout JSON.

### Assistant (2026-02-20 23:38:16)

import numpy as np, json, math, os
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score

psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"

psi=np.load(psi_path)
psi.files

### Tool (2026-02-20 23:38:34)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-20 23:38:37)

x_grid=psi["x_grid"].astype(float)
psi_minus_x=psi["psi_minus_x"].astype(float)
x_grid.min(), x_grid.max(), len(x_grid)

### Tool (2026-02-20 23:38:38)

(1.0, 9999901.0, 100000)

### Assistant (2026-02-20 23:38:39)

W=2000
xmin, xmax = 1e5, 9e6
mask=(x_grid>=xmin)&(x_grid<=xmax)
x_sel=x_grid[mask]
pmx_sel=psi_minus_x[mask]
t_sel=np.log(x_sel)

# uniform t grid
n_uniform=65536
t_uniform=np.linspace(t_sel.min(), t_sel.max(), n_uniform)
pmx_uniform=np.interp(t_uniform, t_sel, pmx_sel)

# moving average detrend
kernel=np.ones(W)/W
trend=np.convolve(pmx_uniform, kernel, mode="same")
detrended=pmx_uniform - trend

# Hilbert transform
analytic=hilbert(detrended)
phi=np.mod(np.angle(analytic), 2*np.pi)

phi[:5], detrended[:5], t_uniform[0], t_uniform[-1]

### Tool (2026-02-20 23:38:45)

(array([1.54203697, 1.52981021, 1.53353331, 1.52942473, 1.53327037]),
 array([35.6846576 , 33.52439064, 31.36481987, 29.20594529, 27.0477669 ]),
 11.51293546492023,
 16.012724135239992)

### Assistant (2026-02-20 23:38:47)

pr=np.load(primes_path)
pr.files, pr[list(pr.files)[0]].dtype, pr[list(pr.files)[0]].shape[:10]

### Tool (2026-02-20 23:38:49)

(['N', 'primes', 'is_prime'], dtype('int32'), ())

### Assistant (2026-02-20 23:38:53)

primes=pr["primes"]
is_prime=pr["is_prime"]
primes[:10], primes[-1], len(primes)

### Tool (2026-02-20 23:38:54)

(array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32), 9999991, 664579)

### Assistant (2026-02-20 23:38:56)

Delta=0.01
x_u=np.exp(t_uniform)
x2_u=x_u*np.exp(Delta)

# searchsorted expects same dtype; primes int32. bounds floats ok.
idx1=np.searchsorted(primes, x_u, side="left")
idx2=np.searchsorted(primes, x2_u, side="left")
P=(idx2-idx1).astype(float)

E = x_u*(np.exp(Delta)-1.0)/np.log(x_u)
R = (P - E)/np.sqrt(E)

n=len(R)
n_train=int(0.7*n)
train_slice=slice(0,n_train)
test_slice=slice(n_train,n)

R_train=R[train_slice]
med=np.median(R_train)

y=(R>med).astype(int)
y_train=y[train_slice]; y_test=y[test_slice]
train_pos=y_train.mean(); test_pos=y_test.mean()
train_pos, test_pos, med

### Tool (2026-02-20 23:39:04)

(0.4999891008174387, 0.47825644677279894, -0.022816595752842755)

### Assistant (2026-02-20 23:39:06)

phi_train=phi[train_slice]; phi_test=phi[test_slice]
def fourier_features(ph):
    return np.column_stack([
        np.ones_like(ph),
        np.cos(ph), np.sin(ph),
        np.cos(2*ph), np.sin(2*ph),
        np.cos(3*ph), np.sin(3*ph),
    ])

X_train=fourier_features(phi_train)
X_test=fourier_features(phi_test)

clf=LogisticRegression(fit_intercept=False, solver="lbfgs", max_iter=2000)
clf.fit(X_train, y_train)
proba_test=clf.predict_proba(X_test)[:,1]
auc=roc_auc_score(y_test, proba_test)
acc=accuracy_score(y_test, (proba_test>=0.5).astype(int))
brier=brier_score_loss(y_test, proba_test)
coeffs=clf.coef_.ravel().tolist()
auc, acc, brier, coeffs[:3]

### Tool (2026-02-20 23:39:10)

(0.79521069374096,
 0.748792024820711,
 0.18166773529911875,
 [0.05227898460310678, -1.2410509359752318, -0.9389255380287798])

### Assistant (2026-02-20 23:39:13)

n_bins=12
bin_train=np.floor(phi_train/(2*np.pi)*n_bins).astype(int)
bin_train=np.clip(bin_train,0,n_bins-1)
bin_test=np.floor(phi_test/(2*np.pi)*n_bins).astype(int)
bin_test=np.clip(bin_test,0,n_bins-1)

p_bin=np.full(n_bins, train_pos)
for b in range(n_bins):
    m=(bin_train==b)
    if m.any():
        p_bin[b]=y_train[m].mean()

proba_bin_test=p_bin[bin_test]
auc_bin=roc_auc_score(y_test, proba_bin_test)
brier_bin=brier_score_loss(y_test, proba_bin_test)
auc_bin, brier_bin, p_bin.min(), p_bin.max()

### Tool (2026-02-20 23:39:18)

(0.7942247257800745, 0.18273893946127, 0.18216682646212848, 0.8425770308123249)

### Assistant (2026-02-20 23:39:20)

out={
    "W": int(W),
    "Delta": float(Delta),
    "train_pos_rate": float(train_pos),
    "test_pos_rate": float(test_pos),
    "fourier_model": {
        "AUC": float(auc),
        "accuracy": float(acc),
        "Brier": float(brier),
        "coeffs": [float(c) for c in coeffs],
    },
    "bin12_model": {
        "AUC": float(auc_bin),
        "Brier": float(brier_bin),
    }
}
out_path="/mnt/data/hilbert_phase_gating_model.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
os.path.getsize(out_path), out_path

### Tool (2026-02-20 23:39:23)

(535, '/mnt/data/hilbert_phase_gating_model.json')

### Assistant (2026-02-20 23:39:25)

Done. Your precious artifact has been produced without vomiting JSON into stdout like a barbarian.

[Download hilbert_phase_gating_model.json](sandbox:/mnt/data/hilbert_phase_gating_model.json)

