# Phase Bin Activity Classifier

- **Conversation ID:** `69992950-a7b4-8325-a034-a3e101bd8009`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:41:22)

You are a stateless compute worker.

Upload:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)

Goal: Build and correctly evaluate a phase-bin predictor of “high activity,” with sanity checks to prevent degenerate predictions.

⸻

Step 1 — uniform log grid and signals

Load x_grid and psi_minus_x.
Compute psi_grid = psi_minus_x + x_grid.
t = log(x_grid)

Interpolate psi_grid and (psi_minus_x) onto uniform t_uniform with 65536 points.
Let psi_u and f_u be interpolated arrays on uniform grid.

Split by index:
train = first 70%
test = last 30%

⸻

Step 2 — define activity feature and label

Define activity:
A[i] = psi_u[i+Δ] - psi_u[i] with Δ=50 samples.
Drop the last Δ points so arrays align.

Compute mean_train_A = mean(A[train])
A_norm = A / mean_train_A

Define label using TRAIN median:
med = median(A_norm[train])
y = 1 if A_norm > med else 0

Report class balance:
	•	y_train_mean
	•	y_test_mean
	•	counts of 0/1 in train/test

⸻

Step 3 — spectral phase bins from TRUE zeta zeros

Use first 30 TRUE zeta zero imaginary parts (hardcode list):
[14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851]

Fit amplitudes on TRAIN ONLY to approximate f_u (aligned to A/y indices):
f_train ≈ Σ a_k cos(γ_k t) + b_k sin(γ_k t)

Build Z[i]=Σ (a_k - i b_k) exp(i γ_k t_uniform[i])
φ = angle(Z) (unwrap optional), then φ_mod = φ mod 2π

Bin φ_mod into 12 equal bins:
bin = floor(12*φ_mod/(2π)) in {0..11}

Align bins to y (drop last Δ points same as A).

⸻

Step 4 — predictor: empirical bin probabilities (no sklearn needed)

Compute per-bin probability on TRAIN:
p_bin[j] = mean(y_train where bin=j)

On TEST, predicted probability is p_bin[bin].

Compute:
	•	AUC on TEST using these probabilities (use roc_auc_score if available; else implement AUC manually)
	•	Accuracy on TEST using threshold 0.5
	•	ALSO choose a threshold that matches TRAIN positive rate (calibration threshold) and report that accuracy too:
thresh_cal = quantile(pred_prob_train, 1 - y_train_mean)

Compute confusion matrices for both thresholds.

Sanity checks:
	•	Confirm test has both classes; if not, say so and still report accuracy but AUC should be marked null.
	•	Confirm predicted probabilities are not all equal; if they are, flag it.

⸻

Output

Return exactly one JSON file: phase_bin_activity_classifier_fixed.json
Include:
	•	class balances
	•	p_bin (12 numbers)
	•	AUC_test (or null)
	•	accuracy_test_thresh_0.5
	•	accuracy_test_thresh_cal
	•	confusion matrices (both thresholds)
	•	any sanity-check warnings

No plots. No extra files.

### Assistant (2026-02-20 22:41:23)

import numpy as np, json, math, os, statistics, sys, textwrap, pandas as pd
path="/mnt/data/chebyshev_psi.npz"
data=np.load(path)
data.files

### Tool (2026-02-20 22:41:57)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-20 22:41:59)

x_grid=data["x_grid"]
psi_minus_x=data["psi_minus_x"]
psi_grid = psi_minus_x + x_grid
x_grid[:5], x_grid[-5:], psi_minus_x.shape

### Tool (2026-02-20 22:42:02)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 (100000,))

### Assistant (2026-02-20 22:42:05)

t=np.log(x_grid.astype(np.float64))
N=65536
t_uniform=np.linspace(t.min(), t.max(), N)
psi_u=np.interp(t_uniform, t, psi_grid.astype(np.float64))
f_u=np.interp(t_uniform, t, psi_minus_x.astype(np.float64))
train_end=int(0.7*N)
Delta=50
A = psi_u[Delta:] - psi_u[:-Delta]  # length N-Delta
# drop last Delta already via this form (equivalent to i=0..N-Delta-1)
mean_train_A = A[:train_end].mean()
A_norm = A / mean_train_A
med = np.median(A_norm[:train_end])
y = (A_norm > med).astype(int)
y_train = y[:train_end]
y_test = y[train_end:]
balances={
    "y_train_mean": float(y_train.mean()),
    "y_test_mean": float(y_test.mean()),
    "train_counts": {"0": int((y_train==0).sum()), "1": int((y_train==1).sum())},
    "test_counts": {"0": int((y_test==0).sum()), "1": int((y_test==1).sum())},
}
balances

### Tool (2026-02-20 22:42:10)

{'y_train_mean': 0.49968392370572207,
 'y_test_mean': 1.0,
 'train_counts': {'0': 22952, '1': 22923},
 'test_counts': {'0': 0, '1': 19611}}

### Assistant (2026-02-20 22:42:12)

zeros=[14.134725, 21.022040, 25.010858, 30.424876, 32.935062, 37.586178, 40.918719, 43.327073, 48.005151, 49.773832,
52.970321, 56.446248, 59.347044, 60.831779, 65.112544, 67.079811, 69.546402, 72.067158, 75.704691, 77.144840,
79.337375, 82.910381, 84.735493, 87.425275, 88.809111, 92.491899, 94.651344, 95.870634, 98.831194, 101.317851]
gam=np.array(zeros, dtype=np.float64)
t_aligned=t_uniform[:-Delta]
f_aligned=f_u[:-Delta]
X_cos=np.cos(np.outer(t_aligned[:train_end], gam))
X_sin=np.sin(np.outer(t_aligned[:train_end], gam))
X=np.hstack([X_cos, X_sin])  # (train_end, 60)
target=f_aligned[:train_end]
coef, *_ = np.linalg.lstsq(X, target, rcond=None)
a=coef[:len(gam)]
b=coef[len(gam):]
# compute Z on all points
Z = np.zeros_like(t_aligned, dtype=np.complex128)
# vectorized: exp(i gamma t) matrix maybe memory huge (65500x30) ok ~ 15MB complex? Actually 65k*30=2M, complex128 16 bytes => 32MB, acceptable
E = np.exp(1j*np.outer(t_aligned, gam))
Z = E.dot(a - 1j*b)
phi = np.angle(Z)
phi_mod = np.mod(phi, 2*np.pi)
bins = np.floor(12*phi_mod/(2*np.pi)).astype(int)
bins.min(), bins.max(), len(bins), len(y)

### Tool (2026-02-20 22:42:23)

(0, 11, 65486, 65486)

### Assistant (2026-02-20 22:42:25)

warnings=[]
p_bin=[]
overall=float(y_train.mean())
for j in range(12):
    mask = (bins[:train_end]==j)
    if mask.sum()==0:
        p=overall
        warnings.append(f"train_bin_{j}_empty_used_overall_rate")
    else:
        p=float(y_train[mask].mean())
    p_bin.append(p)
p_bin

### Tool (2026-02-20 22:42:30)

[0.5,
 0.48712636874815035,
 0.41037735849056606,
 0.5121423419091128,
 0.5293644408688657,
 0.44220572640509015,
 0.5012512512512513,
 0.5251686559418786,
 0.5450711649973643,
 0.5586387434554974,
 0.5408388520971302,
 0.45950748921045953]

### Assistant (2026-02-20 22:42:31)

pred_prob = np.array([p_bin[j] for j in bins], dtype=np.float64)
pred_prob_train=pred_prob[:train_end]
pred_prob_test=pred_prob[train_end:]
# sanity: all equal?
if np.allclose(pred_prob, pred_prob[0]):
    warnings.append("predicted_probabilities_all_equal")
if np.allclose(pred_prob_test, pred_prob_test[0]):
    warnings.append("predicted_probabilities_test_all_equal")
# AUC
def auc_manual(y_true, y_score):
    y_true=np.asarray(y_true)
    y_score=np.asarray(y_score)
    # rank method with ties: use average ranks
    order=np.argsort(y_score)
    y_sorted=y_true[order]
    scores_sorted=y_score[order]
    n=len(y_true)
    ranks=np.empty(n, dtype=np.float64)
    i=0
    rank=1
    while i<n:
        j=i
        while j+1<n and scores_sorted[j+1]==scores_sorted[i]:
            j+=1
        avg=(rank + rank+(j-i))/2.0
        ranks[i:j+1]=avg
        rank += (j-i+1)
        i=j+1
    # sum ranks for positive
    pos = y_sorted==1
    n_pos=pos.sum()
    n_neg=n-n_pos
    if n_pos==0 or n_neg==0:
        return None
    sum_ranks_pos=ranks[pos].sum()
    auc=(sum_ranks_pos - n_pos*(n_pos+1)/2) / (n_pos*n_neg)
    return float(auc)
auc_test=None
if (y_test==0).sum()==0 or (y_test==1).sum()==0:
    warnings.append("test_set_single_class_auc_null")
else:
    auc_test=auc_manual(y_test, pred_prob_test)
# accuracies
def confusion(y_true, y_pred):
    y_true=np.asarray(y_true)
    y_pred=np.asarray(y_pred)
    tn=int(((y_true==0)&(y_pred==0)).sum())
    fp=int(((y_true==0)&(y_pred==1)).sum())
    fn=int(((y_true==1)&(y_pred==0)).sum())
    tp=int(((y_true==1)&(y_pred==1)).sum())
    return {"tn":tn,"fp":fp,"fn":fn,"tp":tp}
# thresh 0.5
pred_label_05=(pred_prob_test>=0.5).astype(int)
acc_05=float((pred_label_05==y_test).mean())
cm_05=confusion(y_test, pred_label_05)
# cal threshold
y_train_mean=float(y_train.mean())
thresh_cal=float(np.quantile(pred_prob_train, 1 - y_train_mean))
pred_label_cal=(pred_prob_test>=thresh_cal).astype(int)
acc_cal=float((pred_label_cal==y_test).mean())
cm_cal=confusion(y_test, pred_label_cal)
auc_test, acc_05, acc_cal, thresh_cal, cm_05, cm_cal, warnings[:5], len(warnings)

### Tool (2026-02-20 22:42:43)

(None,
 0.6774769262148794,
 0.5844169088776707,
 0.5012512512512513,
 {'tn': 0, 'fp': 0, 'fn': 6325, 'tp': 13286},
 {'tn': 0, 'fp': 0, 'fn': 8150, 'tp': 11461},
 ['test_set_single_class_auc_null'],
 1)

### Assistant (2026-02-20 22:42:45)

out={
    "class_balances": balances,
    "p_bin": p_bin,
    "AUC_test": auc_test,
    "accuracy_test_thresh_0.5": acc_05,
    "accuracy_test_thresh_cal": acc_cal,
    "confusion_thresh_0.5": cm_05,
    "confusion_thresh_cal": cm_cal,
    "sanity_warnings": warnings
}
out_path="/mnt/data/phase_bin_activity_classifier_fixed.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2,sort_keys=False)
out_path, os.path.getsize(out_path)

### Tool (2026-02-20 22:42:48)

('/mnt/data/phase_bin_activity_classifier_fixed.json', 877)

### Assistant (2026-02-20 22:42:50)

[phase_bin_activity_classifier_fixed.json](sandbox:/mnt/data/phase_bin_activity_classifier_fixed.json)

