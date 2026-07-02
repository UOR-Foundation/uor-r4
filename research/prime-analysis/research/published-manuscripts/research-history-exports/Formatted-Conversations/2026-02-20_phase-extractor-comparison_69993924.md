# Phase Extractor Comparison

- **Conversation ID:** `69993924-1ed0-832c-8092-2c6562b3335b`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:48:52)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (primes array)

You MUST write a file named:
phase_extractor_comparison.json
Do NOT print JSON to stdout.

Goal: Compare baseline Hilbert-phase predictor vs bandpass-Hilbert-phase predictor for forward prime-density residual prediction.

⸻

Common dataset + target
	1.	Build uniform t_uniform (65536) and interpolate f_raw = psi_minus_x onto it.
	2.	Restrict x = exp(t_uniform) to [1e5, 9e6].
	3.	Split time-ordered: 70% train, 30% test.
	4.	Forward window Δ = 0.01.
	5.	Count primes P in [x, x*exp(Δ)).
	6.	Expected E = x*(exp(Δ)-1)/log(x).
	7.	Residual R=(P−E)/sqrt(E).
	8.	Label y=1 if R > median(R_train) else 0.

Model: Fourier logistic features up to 3rd harmonic:
[1, cosφ, sinφ, cos2φ, sin2φ, cos3φ, sin3φ]
Fit on TRAIN, evaluate on TEST: AUC, accuracy@0.5, Brier.

Use the same library choices as before.

⸻

Phase extractors to compare

A) Baseline (already good)

Detrend: moving average window W=1000.
Phase: φ = angle(Hilbert(detrended_signal)) mod 2π.

B) Bandpass Hilbert (more principled instantaneous phase)
	•	First z-score normalize f_raw over the kept range.
	•	Apply FFT bandpass filter in the t-uniform index domain:
Keep only frequencies with normalized frequency f in [f_low, f_high], zero everything else.
	•	Inverse FFT to get bandpassed signal.
	•	Compute φ via Hilbert transform of the bandpassed signal.

Try three bands (in normalized cycles/sample):
	1.	[0.002, 0.01]
	2.	[0.005, 0.02]
	3.	[0.01, 0.05]

(If you prefer in rad/sample, that’s fine, just be consistent and record it.)

⸻

Output

Write phase_extractor_comparison.json with:
{
“settings”: {“x_range”:[…], “Delta”:0.01, “W_baseline”:1000, “bands”:[…]},
“baseline”: {“AUC”:…, “accuracy”:…, “Brier”:…},
“bandpass”: [
{“band”:[low,high], “AUC”:…, “accuracy”:…, “Brier”:…},
…
]
}

No plots. No extra files.

### Assistant (2026-02-20 23:48:53)

import numpy as np, json, os, math
from scipy import signal
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, accuracy_score, brier_score_loss

cheb_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"

cheb=np.load(cheb_path)
pr=np.load(primes_path)

cheb.files, pr.files[:5], pr.files

### Tool (2026-02-20 23:49:08)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'],
 ['N', 'primes', 'is_prime'],
 ['N', 'primes', 'is_prime'])

### Assistant (2026-02-20 23:49:12)

x_grid=cheb["x_grid"]
psi_minus_x=cheb["psi_minus_x"]
x_grid[:3], x_grid[-3:], np.all(np.diff(x_grid)>0), x_grid.min(), x_grid.max(), len(x_grid)

### Tool (2026-02-20 23:49:15)

(array([  1, 101, 201], dtype=int32),
 array([9999701, 9999801, 9999901], dtype=int32),
 True,
 1,
 9999901,
 100000)

### Assistant (2026-02-20 23:49:18)

t_grid=np.log(x_grid.astype(float))
t_uniform=np.linspace(t_grid.min(), t_grid.max(), 65536)
f_raw=np.interp(t_uniform, t_grid, psi_minus_x.astype(float))

x=np.exp(t_uniform)
mask=(x>=1e5) & (x<=9e6)
t_u=t_uniform[mask]
x_u=x[mask]
f_u=f_raw[mask]
len(t_u), x_u[0], x_u[-1]

### Tool (2026-02-20 23:49:23)

(18296, 100006.32008830582, 8998557.287555724)

### Assistant (2026-02-20 23:49:25)

primes=pr["primes"]
primes[:5], primes[-5:], primes.max(), len(primes)

### Tool (2026-02-20 23:49:27)

(array([ 2,  3,  5,  7, 11], dtype=int32),
 array([9999937, 9999943, 9999971, 9999973, 9999991], dtype=int32),
 9999991,
 664579)

### Assistant (2026-02-20 23:49:29)

Delta=0.01
hi=x_u*np.exp(Delta)
lo=x_u
# counts
left=np.searchsorted(primes, lo, side="left")
right=np.searchsorted(primes, hi, side="left")  # primes < hi
P=(right-left).astype(float)

E= x_u*(np.exp(Delta)-1.0)/np.log(x_u)
# avoid tiny E
R=(P-E)/np.sqrt(E)
R[:5], np.isfinite(R).all(), E.min(), E.max()

### Tool (2026-02-20 23:49:34)

(array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 True,
 87.29968001406948,
 5647.873953250908)

### Assistant (2026-02-20 23:49:36)

from dataclasses import dataclass

def fourier_features(phi):
    return np.column_stack([
        np.ones_like(phi),
        np.cos(phi), np.sin(phi),
        np.cos(2*phi), np.sin(2*phi),
        np.cos(3*phi), np.sin(3*phi)
    ])

def fit_eval(phi, y, split_idx):
    X=fourier_features(phi)
    X_train, X_test = X[:split_idx], X[split_idx:]
    y_train, y_test = y[:split_idx], y[split_idx:]
    clf=LogisticRegression(max_iter=2000, solver="lbfgs")
    clf.fit(X_train, y_train)
    p=clf.predict_proba(X_test)[:,1]
    auc=roc_auc_score(y_test, p)
    acc=accuracy_score(y_test, (p>=0.5).astype(int))
    brier=brier_score_loss(y_test, p)
    return float(auc), float(acc), float(brier)

N=len(f_u)
split_idx=int(0.7*N)
R_train=R[:split_idx]
thr=float(np.median(R_train))
y=(R>thr).astype(int)

# Baseline phase
W=1000
kernel=np.ones(W)/W
trend=np.convolve(f_u, kernel, mode="same")
detr=f_u-trend
analytic=signal.hilbert(detr)
phi_base=np.mod(np.angle(analytic), 2*np.pi)

baseline_metrics=fit_eval(phi_base, y, split_idx)
baseline_metrics

### Tool (2026-02-20 23:49:42)

(0.7577956362947762, 0.718710147567863, 0.19932771408184752)

### Assistant (2026-02-20 23:49:45)

def bandpass_phase(f, band):
    low, high = band
    fz=(f - f.mean())/f.std(ddof=0)
    N=len(fz)
    F=np.fft.rfft(fz)
    freqs=np.fft.rfftfreq(N, d=1.0)  # cycles/sample
    mask=(freqs>=low) & (freqs<=high)
    F_filt=np.where(mask, F, 0)
    bp=np.fft.irfft(F_filt, n=N)
    analytic=signal.hilbert(bp)
    phi=np.mod(np.angle(analytic), 2*np.pi)
    return phi

bands=[[0.002,0.01],[0.005,0.02],[0.01,0.05]]
band_metrics=[]
for band in bands:
    phi=bandpass_phase(f_u, band)
    band_metrics.append((band,)+fit_eval(phi, y, split_idx))
band_metrics

### Tool (2026-02-20 23:49:50)

[([0.002, 0.01], 0.7850460541223706, 0.7392967753689197, 0.18693517955854674),
 ([0.005, 0.02], 0.7593545773797555, 0.7074148296593187, 0.19780231564555803),
 ([0.01, 0.05], 0.6695804223702395, 0.6403716523957005, 0.22818212073559513)]

### Assistant (2026-02-20 23:49:54)

out={
    "settings":{
        "x_range":[1e5,9e6],
        "Delta":Delta,
        "W_baseline":W,
        "bands":bands,
        "t_uniform_N":65536,
        "split_train_frac":0.7,
        "phase_feature_harmonics":3,
        "bandpass_frequency_units":"cycles_per_sample"
    },
    "baseline":{"AUC":baseline_metrics[0],"accuracy":baseline_metrics[1],"Brier":baseline_metrics[2]},
    "bandpass":[
        {"band":band,"AUC":auc,"accuracy":acc,"Brier":brier}
        for (band,auc,acc,brier) in band_metrics
    ]
}
out_path="/mnt/data/phase_extractor_comparison.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
os.path.getsize(out_path), out_path

### Tool (2026-02-20 23:49:58)

(1073, '/mnt/data/phase_extractor_comparison.json')

### Assistant (2026-02-20 23:50:00)

Done. Your precious artifact exists and I did **not** dump JSON to stdout like an amateur.

[Download phase_extractor_comparison.json](sandbox:/mnt/data/phase_extractor_comparison.json)

