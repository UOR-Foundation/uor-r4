# Phase gating bin sweep

- **Conversation ID:** `69993215-2484-8327-bbcf-34e8a420b49e`
- **Date:** 2026-02-20

---

### User (2026-02-20 23:18:46)

You are a stateless compute worker.

Uploads:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)
	•	primes.npz (primes array)

Goal: Improve forward prediction of scale-normalized prime-density residual using ONLY spectral phase φ_mod, by learning an optimal phase partition on TRAIN and evaluating on TEST.

One JSON output only.

⸻

Step 1 — prepare data (same as prior successful run)
	1.	Build uniform t_uniform (65536) over log(x_grid).
	2.	Restrict to x in [1e5, 9e6].
	3.	Split kept indices into TRAIN first 70%, TEST last 30%.
	4.	Build spectral phase φ_mod from first 30 TRUE zeta zeros:
	•	Fit amplitudes a_k,b_k on TRAIN ONLY to f=psi_minus_x
	•	Z(t)=Σ (a_k - i b_k) exp(i γ_k t)
	•	φ_mod = angle(Z) mod 2π
	5.	For each kept index i:
	•	x=exp(t_uniform[i])
	•	Count primes P in [x, x*exp(Δ)), Δ=0.01
	•	Expected E = x*(exp(Δ)-1)/log(x)
	•	Residual R=(P-E)/sqrt(E)
	6.	Define label threshold from TRAIN:
med_R = median(R_train)
y=1 if R > med_R else 0

⸻

Step 2 — learn phase gating on TRAIN

We will model p(y=1 | φ_mod) as piecewise-constant in φ.

Try bin counts B in {6, 12, 24, 36, 48}.
For each B:
	•	compute empirical p_bin on TRAIN (mean y in each phase bin)
	•	predict prob on TRAIN/TEST accordingly
	•	compute AUC on TRAIN and TEST
	•	compute Brier on TRAIN and TEST

Select B* that maximizes TEST AUC (or if you want to avoid peeking, use a 10% validation split inside TRAIN: TRAIN→(fit/val) and pick B using val, then report test).

To avoid overfitting: do the selection using a validation split inside TRAIN:
	•	fit bins on first 90% of TRAIN
	•	choose B by AUC on last 10% of TRAIN
	•	then refit p_bin using full TRAIN and evaluate on TEST

⸻

Step 3 — output

Return exactly one JSON: phase_gating_bin_sweep.json
Include:
	•	chosen B
	•	AUC/Brier on TEST for each B tried
	•	final TEST AUC, accuracy@0.5, Brier
	•	p_bin for chosen B (list length B)
	•	class balance train/test
	•	x_range used, Δ, med_R

No plots. No extra files.

### Assistant (2026-02-20 23:18:47)

import numpy as np, math, json, os, statistics
from bisect import bisect_left, bisect_right
import mpmath as mp

psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"

psi_npz=np.load(psi_path)
psi_npz.files

### Tool (2026-02-20 23:19:03)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-20 23:19:05)

x_grid=psi_npz['x_grid']
psi_minus_x=psi_npz['psi_minus_x']
x_grid[:5], x_grid[-5:], len(x_grid), psi_minus_x[:5], psi_minus_x[-5:]

### Tool (2026-02-20 23:19:06)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 100000,
 array([-1.        , -2.33956825,  5.14585683, -1.76766456,  2.82480716]),
 array([-1396.78466777, -1432.31246137, -1451.72215382, -1487.24987502,
        -1490.54137751]))

### Assistant (2026-02-20 23:19:07)

pr_npz=np.load(primes_path)
pr_npz.files, pr_npz['primes'][:10], pr_npz['primes'][-10:], len(pr_npz['primes'])

### Tool (2026-02-20 23:19:10)

(['N', 'primes', 'is_prime'],
 array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32),
 array([9999889, 9999901, 9999907, 9999929, 9999931, 9999937, 9999943,
        9999971, 9999973, 9999991], dtype=int32),
 664579)

### Assistant (2026-02-20 23:19:13)

t_grid=np.log(x_grid.astype(float))
t_uniform=np.linspace(t_grid.min(), t_grid.max(), 65536)
# interpolate psi_minus_x to uniform t
f_uniform=np.interp(t_uniform, t_grid, psi_minus_x)
# restrict x in [1e5,9e6]
x_uniform=np.exp(t_uniform)
mask=(x_uniform>=1e5) & (x_uniform<=9e6)
idx=np.nonzero(mask)[0]
len(idx), idx[0], idx[-1], x_uniform[idx[0]], x_uniform[idx[-1]]

### Tool (2026-02-20 23:19:16)

(18296, 46811, 65106, 100006.32008830582, 8998557.287555724)

### Assistant (2026-02-20 23:19:18)

kept_idx=idx
n=len(kept_idx)
n_train=int(math.floor(0.7*n))
train_idx=kept_idx[:n_train]
test_idx=kept_idx[n_train:]
len(train_idx), len(test_idx)

### Tool (2026-02-20 23:19:24)

(12807, 5489)

### Assistant (2026-02-20 23:19:25)

mp.mp.dps=50
gammas=[float(mp.im(mp.zetazero(k))) for k in range(1,31)]
gammas[:5], gammas[-1]

### Tool (2026-02-20 23:19:27)

([14.134725141734695,
  21.022039638771556,
  25.01085758014569,
  30.424876125859512,
  32.93506158773919],
 101.31785100573138)

### Assistant (2026-02-20 23:19:29)

t_train=t_uniform[train_idx]
y_train_f=f_uniform[train_idx]
K=len(gammas)
# build design matrix: [cos(gamma t), sin(gamma t)] for each gamma
X=np.empty((len(t_train), 2*K), dtype=float)
for j,g in enumerate(gammas):
    X[:,2*j]=np.cos(g*t_train)
    X[:,2*j+1]=np.sin(g*t_train)
# least squares
coef, *_ = np.linalg.lstsq(X, y_train_f, rcond=None)
a=coef[0::2]
b=coef[1::2]
a[:3], b[:3]

### Tool (2026-02-20 23:19:33)

(array([ 1.24177647, -4.69607211, -1.82110675]),
 array([-109.9362509 ,  -76.60950482,  -68.14038217]))

### Assistant (2026-02-20 23:19:38)

t_kept=t_uniform[kept_idx]
# compute exp(i g t) matrix maybe big 18296x30 ~ 548k ok
G=np.array(gammas)
E=np.exp(1j*np.outer(t_kept, G))  # shape (n_kept,K)
c=(a - 1j*b)  # complex coeff
Z=E @ c
phi_mod=np.mod(np.angle(Z), 2*math.pi)
phi_mod[:5], phi_mod.min(), phi_mod.max()

### Tool (2026-02-20 23:19:42)

(array([1.29063903, 1.2834853 , 1.27572257, 1.26733566, 1.2583103 ]),
 0.0001114151264859486,
 6.282071686787273)

### Assistant (2026-02-20 23:19:47)

primes=pr_npz['primes'].astype(np.int64)
Delta=0.01
factor=math.exp(Delta)
x_kept=np.exp(t_kept)
low=x_kept
high=x_kept*factor
# searchsorted
lo_idx=np.searchsorted(primes, low, side='left')
hi_idx=np.searchsorted(primes, high, side='left')  # high exclusive
P=(hi_idx - lo_idx).astype(float)
# expected
Eexp = x_kept*(factor-1.0)/np.log(x_kept)
R=(P - Eexp)/np.sqrt(Eexp)
R[:5], np.median(R[:len(train_idx)])

### Tool (2026-02-20 23:19:52)

(array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 -0.021281541370115658)

### Assistant (2026-02-20 23:19:53)

R_train=R[:n_train]
R_test=R[n_train:]
phi_train=phi_mod[:n_train]
phi_test=phi_mod[n_train:]
med_R=float(np.median(R_train))
y_train=(R_train>med_R).astype(int)
y_test=(R_test>med_R).astype(int)
y_train.mean(), y_test.mean(), med_R

### Tool (2026-02-20 23:19:57)

(0.4999609588506286, 0.4796866460193113, -0.021281541370115658)

### Assistant (2026-02-20 23:19:59)

n_fit=int(math.floor(0.9*n_train))
phi_fit=phi_train[:n_fit]
y_fit=y_train[:n_fit]
phi_val=phi_train[n_fit:]
y_val=y_train[n_fit:]
len(phi_fit), len(phi_val)

### Tool (2026-02-20 23:20:01)

(11526, 1281)

### Assistant (2026-02-20 23:20:03)

def roc_auc_score(y_true, y_score):
    y_true=np.asarray(y_true)
    y_score=np.asarray(y_score)
    # Only binary
    pos = y_true==1
    neg = y_true==0
    n_pos=pos.sum()
    n_neg=neg.sum()
    if n_pos==0 or n_neg==0:
        return float('nan')
    # rank scores with average ranks for ties
    order=np.argsort(y_score)
    scores_sorted=y_score[order]
    y_sorted=y_true[order]
    ranks=np.empty_like(scores_sorted, dtype=float)
    i=0
    r=1
    while i<len(scores_sorted):
        j=i
        while j+1<len(scores_sorted) and scores_sorted[j+1]==scores_sorted[i]:
            j+=1
        avg_rank=(r + r+(j-i))/2.0
        ranks[i:j+1]=avg_rank
        r += (j-i+1)
        i=j+1
    # sum ranks of positive
    sum_ranks_pos = ranks[y_sorted==1].sum()
    U = sum_ranks_pos - n_pos*(n_pos+1)/2.0
    auc = U/(n_pos*n_neg)
    return float(auc)

def brier(y_true, y_prob):
    y_true=np.asarray(y_true, dtype=float)
    y_prob=np.asarray(y_prob, dtype=float)
    return float(np.mean((y_prob - y_true)**2))

def accuracy_at_05(y_true, y_prob):
    y_pred=(np.asarray(y_prob)>=0.5).astype(int)
    return float(np.mean(y_pred==np.asarray(y_true)))

def phase_bin_probs(phi, y, B):
    # bins [0,2pi), equal width
    edges=np.linspace(0, 2*math.pi, B+1)
    # bin index 0..B-1
    bin_idx=np.minimum(np.searchsorted(edges, phi, side='right')-1, B-1)
    # compute mean y per bin; handle empty bins with global mean
    p=np.zeros(B, dtype=float)
    global_mean=float(np.mean(y)) if len(y)>0 else 0.5
    for b in range(B):
        m = (bin_idx==b)
        if m.any():
            p[b]=float(np.mean(y[m]))
        else:
            p[b]=global_mean
    return p, bin_idx

def predict_from_bins(phi, p, B):
    edges=np.linspace(0, 2*math.pi, B+1)
    bin_idx=np.minimum(np.searchsorted(edges, phi, side='right')-1, B-1)
    return p[bin_idx]

### Assistant (2026-02-20 23:20:16)

Bs=[6,12,24,36,48]
val_results={}
for B in Bs:
    p_bin,_=phase_bin_probs(phi_fit,y_fit,B)
    p_val=predict_from_bins(phi_val,p_bin,B)
    auc_val=roc_auc_score(y_val,p_val)
    val_results[B]=auc_val
val_results

### Tool (2026-02-20 23:20:18)

{6: 0.6397291983764587,
 12: 0.6815873434024119,
 24: 0.6665495843578035,
 36: 0.6372107091285173,
 48: 0.6387901006907856}

### Assistant (2026-02-20 23:20:19)

sweep=[]
for B in Bs:
    p_bin_train,_=phase_bin_probs(phi_train,y_train,B)
    p_train=predict_from_bins(phi_train,p_bin_train,B)
    p_test=predict_from_bins(phi_test,p_bin_train,B)
    sweep.append({
        "B":B,
        "train_auc":roc_auc_score(y_train,p_train),
        "train_brier":brier(y_train,p_train),
        "test_auc":roc_auc_score(y_test,p_test),
        "test_brier":brier(y_test,p_test),
    })
sweep

### Tool (2026-02-20 23:20:24)

[{'B': 6,
  'train_auc': 0.6369460369675637,
  'train_brier': 0.2346899905491223,
  'test_auc': 0.6280893576572293,
  'test_brier': 0.23739381318747882},
 {'B': 12,
  'train_auc': 0.6523887611044284,
  'train_brier': 0.23180262516020114,
  'test_auc': 0.5963177048259486,
  'test_brier': 0.2417069686648332},
 {'B': 24,
  'train_auc': 0.6592035222597777,
  'train_brier': 0.23071360881902445,
  'test_auc': 0.6093244171956668,
  'test_brier': 0.24264631603826045},
 {'B': 36,
  'train_auc': 0.6617667702024825,
  'train_brier': 0.2300826544547837,
  'test_auc': 0.6156184273937452,
  'test_brier': 0.24260790825910422},
 {'B': 48,
  'train_auc': 0.6659672772063923,
  'train_brier': 0.228844105280505,
  'test_auc': 0.6193306034909216,
  'test_brier': 0.24421610046479347}]

### Assistant (2026-02-20 23:20:26)

B_star=12
p_bin_star,_=phase_bin_probs(phi_train,y_train,B_star)
p_test_star=predict_from_bins(phi_test,p_bin_star,B_star)
final={
    "test_auc":roc_auc_score(y_test,p_test_star),
    "test_brier":brier(y_test,p_test_star),
    "test_accuracy_at_0_5":accuracy_at_05(y_test,p_test_star),
}
final, p_bin_star

### Tool (2026-02-20 23:20:32)

({'test_auc': 0.5963177048259486,
  'test_brier': 0.2417069686648332,
  'test_accuracy_at_0_5': 0.5767899435234105},
 array([0.39852399, 0.32510638, 0.36531365, 0.31473684, 0.36344315,
        0.51468988, 0.61998041, 0.65116279, 0.68524333, 0.59893993,
        0.64365079, 0.4733514 ]))

### Assistant (2026-02-20 23:20:34)

out={
 "file":"phase_gating_bin_sweep.json",
 "x_range_used":[float(x_uniform[kept_idx[0]]), float(x_uniform[kept_idx[-1]])],
 "Delta":Delta,
 "med_R":med_R,
 "class_balance":{
     "train_pos_rate": float(y_train.mean()),
     "test_pos_rate": float(y_test.mean()),
     "train_n": int(len(y_train)),
     "test_n": int(len(y_test)),
     "fit_n": int(len(y_fit)),
     "val_n": int(len(y_val))
 },
 "selection":{
     "method":"train_internal_validation_first90_last10_time_order",
     "val_auc_by_B": {str(B): float(val_results[B]) for B in Bs},
     "chosen_B": int(B_star)
 },
 "bin_sweep": [
     {"B": int(d["B"]),
      "test_auc": float(d["test_auc"]),
      "test_brier": float(d["test_brier"]),
      "train_auc": float(d["train_auc"]),
      "train_brier": float(d["train_brier"])} for d in sweep
 ],
 "chosen_model":{
     "B": int(B_star),
     "p_bin": [float(x) for x in p_bin_star.tolist()],
     "test_auc": final["test_auc"],
     "test_brier": final["test_brier"],
     "test_accuracy_at_0_5": final["test_accuracy_at_0_5"]
 }
}
json_str=json.dumps(out, indent=2)
len(json_str.splitlines()), json_str[:300]

### Tool (2026-02-20 23:20:42)

(85,
 '{\n  "file": "phase_gating_bin_sweep.json",\n  "x_range_used": [\n    100006.32008830582,\n    8998557.287555724\n  ],\n  "Delta": 0.01,\n  "med_R": -0.021281541370115658,\n  "class_balance": {\n    "train_pos_rate": 0.4999609588506286,\n    "test_pos_rate": 0.4796866460193113,\n    "train_n": 12807,\n    "test')

### Assistant (2026-02-20 23:20:45)

```json
{
  "file": "phase_gating_bin_sweep.json",
  "x_range_used": [
    100006.32008830582,
    8998557.287555724
  ],
  "Delta": 0.01,
  "med_R": -0.021281541370115658,
  "class_balance": {
    "train_pos_rate": 0.4999609588506286,
    "test_pos_rate": 0.4796866460193113,
    "train_n": 12807,
    "test_n": 5489,
    "fit_n": 11526,
    "val_n": 1281
  },
  "selection": {
    "method": "train_internal_validation_first90_last10_time_order",
    "val_auc_by_B": {
      "6": 0.6397291983764587,
      "12": 0.6815873434024119,
      "24": 0.6665495843578035,
      "36": 0.6372107091285173,
      "48": 0.6387901006907856
    },
    "chosen_B": 12
  },
  "bin_sweep": [
    {
      "B": 6,
      "test_auc": 0.6280893576572293,
      "test_brier": 0.23739381318747882,
      "train_auc": 0.6369460369675637,
      "train_brier": 0.2346899905491223
    },
    {
      "B": 12,
      "test_auc": 0.5963177048259486,
      "test_brier": 0.2417069686648332,
      "train_auc": 0.6523887611044284,
      "train_brier": 0.23180262516020114
    },
    {
      "B": 24,
      "test_auc": 0.6093244171956668,
      "test_brier": 0.24264631603826045,
      "train_auc": 0.6592035222597777,
      "train_brier": 0.23071360881902445
    },
    {
      "B": 36,
      "test_auc": 0.6156184273937452,
      "test_brier": 0.24260790825910422,
      "train_auc": 0.6617667702024825,
      "train_brier": 0.2300826544547837
    },
    {
      "B": 48,
      "test_auc": 0.6193306034909216,
      "test_brier": 0.24421610046479347,
      "train_auc": 0.6659672772063923,
      "train_brier": 0.228844105280505
    }
  ],
  "chosen_model": {
    "B": 12,
    "p_bin": [
      0.3985239852398524,
      0.3251063829787234,
      0.36531365313653136,
      0.31473684210526315,
      0.363443145649624,
      0.5146898824292727,
      0.6199804113614104,
      0.6511627906976745,
      0.685243328100471,
      0.5989399293286219,
      0.6436507936507937,
      0.4733514002339181
    ],
    "test_auc": 0.5963177048259486,
    "test_brier": 0.2417069686648332,
    "test_accuracy_at_0_5": 0.5767899435234105
  }
}
```

