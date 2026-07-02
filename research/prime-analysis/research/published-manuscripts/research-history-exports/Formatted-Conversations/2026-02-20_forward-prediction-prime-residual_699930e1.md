# Forward prediction prime residual

- **Conversation ID:** `699930e1-ca84-832c-8cc5-89cc20b88163`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:13:38)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (primes array)

Goal: Strict forward prediction: does spectral phase at window start predict scale-normalized prime-density residual in the next multiplicative interval?

One JSON output only. No plots.

⸻

Step 1 — uniform log grid + split (avoid tiny-x regime)

Load x_grid and psi_minus_x.
t = log(x_grid)
Interpolate f=psi_minus_x onto uniform t_uniform with 65536 points.

To avoid tiny-x artifacts, restrict to x >= 1e5:
keep indices where exp(t_uniform) >= 1e5 and exp(t_uniform) <= 9e6.

From the remaining indices, split:
TRAIN = first 70% (in t order)
TEST = last 30%

⸻

Step 2 — spectral phase from TRUE zeros (train-only amplitude fit)

Use first 30 true zeta zero imaginary parts (same list as before).

Fit a_k,b_k on TRAIN ONLY:
f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Compute on ALL kept indices:
Z(t)=Σ (a_k - i b_k) exp(i γ_k t)
φ_mod = angle(Z) mod 2π
bin = floor(12*φ_mod/(2π)) in {0..11}

⸻

Step 3 — forward prime count and expected count

Let Δ = 0.01.

For each index i in kept set:
x = exp(t_uniform[i])
interval = [x, x*exp(Δ))

Count primes:
P = #{p in primes: x <= p < x*exp(Δ)} (binary search)

Expected count (logarithmic integral approximation):
E = x*(exp(Δ)-1) / log(x)

Define residual score:
R = (P - E) / sqrt(E)

⸻

Step 4 — label and predictor

Define TRAIN threshold using TRAIN median of R:
med_R = median(R_train)
y = 1 if R > med_R else 0

Train phase-bin probabilities on TRAIN:
p_bin[j] = mean(y_train | bin=j)

Predict on TEST:
pred_prob = p_bin[bin]

Compute TEST metrics:
	•	AUC
	•	accuracy at threshold 0.5
	•	Brier score
	•	per-bin mean(R_test) and per-bin positive rate

Report TRAIN/TEST class balance for y.

⸻

Output (single JSON)

Return exactly one JSON: forward_phase_prime_residual_prediction.json

Include:
	•	x_range used
	•	Δ used
	•	med_R
	•	class balance train/test
	•	p_bin (12)
	•	AUC_test, accuracy_test, brier_test
	•	per-bin test stats: n, mean_R, positive_rate

No plots. No extra files.

### Assistant (2026-02-20 23:13:39)

import numpy as np, json, math, os, statistics, pandas as pd
psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
psi=np.load(psi_path)
pr=np.load(primes_path)
psi.files, pr.files[:10], pr.files[-1]

### Tool (2026-02-20 23:13:58)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'],
 ['N', 'primes', 'is_prime'],
 'is_prime')

### Assistant (2026-02-20 23:14:01)

x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
primes=pr["primes"]
x_grid.shape, psi_minus_x.shape, primes.shape, x_grid[0], x_grid[-1], primes[0], primes[-1]

### Tool (2026-02-20 23:14:02)

((100000,), (100000,), (664579,), 1, 9999901, 2, 9999991)

### Assistant (2026-02-20 23:14:05)

t=np.log(x_grid.astype(float))
t_uniform=np.linspace(t.min(), t.max(), 65536)
f_uniform=np.interp(t_uniform, t, psi_minus_x.astype(float))
x_uniform=np.exp(t_uniform)
mask=(x_uniform>=1e5) & (x_uniform<=9e6)
t_keep=t_uniform[mask]
f_keep=f_uniform[mask]
x_keep=x_uniform[mask]
len(t_uniform), len(t_keep), (x_keep.min(), x_keep.max())

### Tool (2026-02-20 23:14:11)

(65536, 18296, (100006.32008830582, 8998557.287555724))

### Assistant (2026-02-20 23:14:14)

n=len(t_keep)
n_train=int(np.floor(0.70*n))
train_idx=np.arange(n_train)
test_idx=np.arange(n_train,n)
n_train, n-n_train

### Tool (2026-02-20 23:14:16)

(12807, 5489)

### Assistant (2026-02-20 23:14:30)

gammas=np.array([
14.134725141734693,21.022039638771555,25.010857580145689,30.424876125859513,
32.935061587739190,37.586178158825671,40.918719012147495,43.327073280914999,
48.005150881167160,49.773832477672302,52.970321477714461,56.446247697063395,
59.347044002602353,60.831778524609810,65.112544048081607,67.079810529494174,
69.546401711173979,72.067157674481908,75.704690699083933,77.144840068874805,
79.337375020249368,82.910380854086030,84.735492980517050,87.425274613125229,
88.809111207634465,92.491899270558484,94.651344040519887,95.870634228245310,
98.831194218193692,101.317851005731391
],dtype=float)
k=len(gammas)
X=np.empty((n_train,2*k),dtype=float)
t_tr=t_keep[:n_train]
for idx,g in enumerate(gammas):
    X[:,2*idx]=np.cos(g*t_tr)
    X[:,2*idx+1]=np.sin(g*t_tr)
y_tr=f_keep[:n_train]
coef,resid,rank,s=np.linalg.lstsq(X,y_tr,rcond=None)
a=coef[0::2]; b=coef[1::2]
c=a-1j*b
# compute Z on all
Z=np.exp(1j*np.outer(t_keep,gammas)).dot(c)
phi=np.angle(Z)
phi_mod=np.mod(phi,2*np.pi)
bins=np.floor(12*phi_mod/(2*np.pi)).astype(int)
bins=np.clip(bins,0,11)
bins[:5], bins.min(), bins.max()

### Tool (2026-02-20 23:14:36)

(array([2, 2, 2, 2, 2]), 0, 11)

### Assistant (2026-02-20 23:14:38)

Delta=0.01
expD=math.exp(Delta)
x=x_keep
hi=x*expD
lo_idx=np.searchsorted(primes, x, side='left')
hi_idx=np.searchsorted(primes, hi, side='left')
P=(hi_idx-lo_idx).astype(float)
E = x*(expD-1.0)/np.log(x)
R = (P - E)/np.sqrt(E)
R_train=R[:n_train]; R_test=R[n_train:]
med_R=float(np.median(R_train))
y=(R>med_R).astype(int)
y_train=y[:n_train]; y_test=y[n_train:]
class_balance_train=float(y_train.mean())
class_balance_test=float(y_test.mean())
med_R, class_balance_train, class_balance_test, (R_train.min(), R_train.max())

### Tool (2026-02-20 23:14:42)

(-0.021281541370115658,
 0.4999609588506286,
 0.4796866460193113,
 (-1.806644999812491, 1.7698642128176687))

### Assistant (2026-02-20 23:14:44)

p_bin=[]
for j in range(12):
    m=(bins[:n_train]==j)
    if m.any():
        p=float(y_train[m].mean())
    else:
        p=float(class_balance_train)
    p_bin.append(p)
p_bin

### Tool (2026-02-20 23:14:47)

[0.3985239852398524,
 0.3251063829787234,
 0.36531365313653136,
 0.31473684210526315,
 0.36344314558979807,
 0.514689880304679,
 0.6199804113614104,
 0.6511627906976745,
 0.685243328100471,
 0.598939929328622,
 0.6436507936507937,
 0.47335140018066846]

### Assistant (2026-02-20 23:14:51)

pred_prob=np.array([p_bin[j] for j in bins[n_train:]],dtype=float)
# accuracy threshold 0.5
pred_label=(pred_prob>=0.5).astype(int)
accuracy=float((pred_label==y_test).mean())
brier=float(np.mean((pred_prob - y_test)**2))
# AUC via rank (Mann-Whitney)
def auc_score(y_true, y_score):
    y_true=np.asarray(y_true)
    y_score=np.asarray(y_score)
    # handle ties via average ranks
    order=np.argsort(y_score)
    ranks=np.empty_like(order,dtype=float)
    ranks[order]=np.arange(1,len(y_score)+1)
    # average ranks for ties
    # find tie groups in sorted scores
    sorted_scores=y_score[order]
    i=0
    while i<len(sorted_scores):
        j=i
        while j<len(sorted_scores) and sorted_scores[j]==sorted_scores[i]:
            j+=1
        if j-i>1:
            avg=(i+1 + j)/2.0
            ranks[order[i:j]]=avg
        i=j
    pos=ranks[y_true==1]
    n_pos=len(pos); n_neg=len(y_true)-n_pos
    if n_pos==0 or n_neg==0:
        return float('nan')
    U=pos.sum() - n_pos*(n_pos+1)/2.0
    return float(U/(n_pos*n_neg))
auc=float(auc_score(y_test,pred_prob))
auc, accuracy, brier

### Tool (2026-02-20 23:14:57)

(0.5963177048259486, 0.5767899435234105, 0.2417069686648332)

### Assistant (2026-02-20 23:14:59)

per_bin=[]
for j in range(12):
    m=(bins[n_train:]==j)
    n_j=int(m.sum())
    if n_j>0:
        mean_R=float(R_test[m].mean())
        pos_rate=float(y_test[m].mean())
    else:
        mean_R=float('nan'); pos_rate=float('nan')
    per_bin.append({"bin":j,"n":n_j,"mean_R":mean_R,"positive_rate":pos_rate})
per_bin[:3], sum(d["n"] for d in per_bin)

### Tool (2026-02-20 23:15:04)

([{'bin': 0, 'n': 374, 'mean_R': -0.026585989589444272, 'positive_rate': 0.5},
  {'bin': 1,
   'n': 397,
   'mean_R': -0.381843587903906,
   'positive_rate': 0.16624685138539042},
  {'bin': 2,
   'n': 510,
   'mean_R': -0.2128502796533479,
   'positive_rate': 0.3235294117647059}],
 5489)

### Assistant (2026-02-20 23:15:06)

out={
 "filename":"forward_phase_prime_residual_prediction.json",
 "x_range_used":[float(x_keep.min()), float(x_keep.max())],
 "Delta":Delta,
 "med_R":med_R,
 "class_balance":{
     "train":{"n":int(n_train),"positive_rate":class_balance_train},
     "test":{"n":int(n-n_train),"positive_rate":class_balance_test}
 },
 "p_bin":p_bin,
 "metrics_test":{
     "AUC":auc,
     "accuracy_at_0.5":accuracy,
     "brier":brier
 },
 "per_bin_test_stats":[{"n":d["n"],"mean_R":d["mean_R"],"positive_rate":d["positive_rate"]} for d in per_bin]
}
out_path="/mnt/data/forward_phase_prime_residual_prediction.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, os.path.getsize(out_path)

### Tool (2026-02-20 23:15:14)

('/mnt/data/forward_phase_prime_residual_prediction.json', 2128)

### Assistant (2026-02-20 23:15:17)

with open(out_path) as f:
    s=f.read()
s[:300]

### Tool (2026-02-20 23:15:19)

'{\n  "filename": "forward_phase_prime_residual_prediction.json",\n  "x_range_used": [\n    100006.32008830582,\n    8998557.287555724\n  ],\n  "Delta": 0.01,\n  "med_R": -0.021281541370115658,\n  "class_balance": {\n    "train": {\n      "n": 12807,\n      "positive_rate": 0.4999609588506286\n    },\n    "test":'

### Assistant (2026-02-20 23:15:21)

{
  "filename": "forward_phase_prime_residual_prediction.json",
  "x_range_used": [
    100006.32008830582,
    8998557.287555724
  ],
  "Delta": 0.01,
  "med_R": -0.021281541370115658,
  "class_balance": {
    "train": {
      "n": 12807,
      "positive_rate": 0.4999609588506286
    },
    "test": {
      "n": 5489,
      "positive_rate": 0.4796866460193113
    }
  },
  "p_bin": [
    0.3985239852398524,
    0.3251063829787234,
    0.36531365313653136,
    0.31473684210526315,
    0.36344314558979807,
    0.514689880304679,
    0.6199804113614104,
    0.6511627906976745,
    0.685243328100471,
    0.598939929328622,
    0.6436507936507937,
    0.47335140018066846
  ],
  "metrics_test": {
    "AUC": 0.5963177048259486,
    "accuracy_at_0.5": 0.5767899435234105,
    "brier": 0.2417069686648332
  },
  "per_bin_test_stats": [
    {
      "n": 374,
      "mean_R": -0.026585989589444272,
      "positive_rate": 0.5
    },
    {
      "n": 397,
      "mean_R": -0.381843587903906,
      "positive_rate": 0.16624685138539042
    },
    {
      "n": 510,
      "mean_R": -0.2128502796533479,
      "positive_rate": 0.3235294117647059
    },
    {
      "n": 426,
      "mean_R": -0.38406824821682967,
      "positive_rate": 0.1784037558685446
    },
    {
      "n": 490,
      "mean_R": -0.12355997730707146,
      "positive_rate": 0.3653061224489796
    },
    {
      "n": 478,
      "mean_R": 0.04107787277862361,
      "positive_rate": 0.5209205020920502
    },
    {
      "n": 463,
      "mean_R": 0.25118616952017843,
      "positive_rate": 0.6976241900647948
    },
    {
      "n": 387,
      "mean_R": 0.36110812078520845,
      "positive_rate": 0.7364341085271318
    },
    {
      "n": 450,
      "mean_R": 0.3198573424163126,
      "positive_rate": 0.7311111111111112
    },
    {
      "n": 566,
      "mean_R": 0.26534672223685437,
      "positive_rate": 0.6183745583038869
    },
    {
      "n": 469,
      "mean_R": 0.2274622229259158,
      "positive_rate": 0.7132196162046908
    },
    {
      "n": 479,
      "mean_R": 0.07564535137412208,
      "positive_rate": 0.5365344467640919
    }
  ]
}

