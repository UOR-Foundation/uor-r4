# Spectral Phase Sign Prediction

- **Conversation ID:** `6999248a-4b04-832f-b275-f32a086f5e3a`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:20:46)

You are a stateless compute worker.

Uploads:
	1.	chebyshev_psi.npz (x_grid, psi_minus_x)
	2.	psi_log_spectrum.json (gamma_est peaks)

Goal: confirm that spectral-phase sign prediction generalizes when amplitudes are fit on TRAIN ONLY, and compare against a random-frequency control.

⸻

Step 1 — uniform log grid

Load data.
t = log(x_grid)
Interpolate f(t)=psi_minus_x onto t_uniform with n=65536 points.

Define s_true(t)=sign(f_uniform), forward-fill zeros.

Split index at 70%:
train = [0:idx_split)
test = [idx_split:end)

⸻

Step 2 — choose frequencies

From psi_log_spectrum.json, take first K=20 gamma_est with gamma >= 2.
Call this GAMMA_REAL.

Construct a control frequency set GAMMA_CTRL of same size:
	•	sample uniformly from [min(GAMMA_REAL), max(GAMMA_REAL)]
	•	enforce minimum separation 0.5 between sampled gammas (to avoid duplicates)
Set random seed = 12345 so it’s reproducible.

⸻

Step 3 — fit amplitudes on TRAIN ONLY

For each gamma set (REAL and CTRL):
Fit a_k, b_k by least squares on TRAIN ONLY to approximate f_uniform(train):
f ≈ Σ a_k cos(gamma_k t) + b_k sin(gamma_k t)

Then define complex field on ALL t:
Z(t)=Σ (a_k - i b_k) * exp(i gamma_k t)
φ(t)=unwrap(angle(Z(t)))

⸻

Step 4 — phase sign predictor

Predict sign using phase only:
pred(t)=+1 if cos(φ(t))>0 else -1

Compute accuracy on TRAIN and TEST for REAL and CTRL.

Also compute the rolling-memory predictor accuracy on TRAIN and TEST:
pred_roll(t)=sign(rolling_mean(s_true, window=200))

⸻

Output (single JSON)

Return exactly one file: spectral_phase_trainfit_control.json

Include:
	•	gammas_real, gammas_ctrl
	•	accuracy_real_train/test
	•	accuracy_ctrl_train/test
	•	accuracy_roll_train/test
	•	optional: R² of reconstruction on train/test for both real/ctrl

No plots, no extra files.

### Tool (2026-02-20 22:20:49)

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

### Tool (2026-02-20 22:20:49)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-20 22:20:48)

import numpy as np, json, os, math, pandas as pd
npz_path="/mnt/data/chebyshev_psi.npz"
json_path="/mnt/data/psi_log_spectrum.json"
data=np.load(npz_path)
x_grid=data["x_grid"]
psi_minus_x=data["psi_minus_x"]
with open(json_path,"r") as f:
    spec=json.load(f)
# Step1
t=np.log(x_grid.astype(float))
n=65536
tmin=float(np.min(t)); tmax=float(np.max(t))
t_uniform=np.linspace(tmin,tmax,n)
# interpolate f(t)
# ensure t is increasing
order=np.argsort(t)
t_sorted=t[order]; f_sorted=psi_minus_x[order].astype(float)
f_uniform=np.interp(t_uniform,t_sorted,f_sorted)
# s_true sign with forward-fill zeros
s=np.sign(f_uniform)
# forward fill zeros
s_true=s.copy()
last=1.0
# choose initial last as first non-zero or 1
nz=np.nonzero(s_true)[0]
if len(nz)>0:
    last=float(s_true[nz[0]])
else:
    last=1.0
for i in range(len(s_true)):
    if s_true[i]==0:
        s_true[i]=last
    else:
        last=float(s_true[i])
s_true=s_true.astype(int)
idx_split=int(0.7*n)
train_slice=slice(0,idx_split)
test_slice=slice(idx_split,n)
# Step2 frequencies
peaks=spec.get("top_25_peaks", spec.get("peaks", []))
gammas=[p["gamma_est"] for p in peaks]
gammas_real=[g for g in gammas if g>=2.0][:20]
K=20
assert len(gammas_real)==K, len(gammas_real)
gmin=min(gammas_real); gmax=max(gammas_real)
rng=np.random.default_rng(12345)
gammas_ctrl=[]
tries=0
while len(gammas_ctrl)<K and tries<200000:
    g=float(rng.uniform(gmin,gmax))
    if all(abs(g-h)>=0.5 for h in gammas_ctrl):
        gammas_ctrl.append(g)
    tries+=1
if len(gammas_ctrl)<K:
    raise RuntimeError("Could not sample control frequencies with separation constraint")
# Step3 fit amplitudes
def fit_and_eval(gammas):
    gammas=np.array(gammas,float)
    tt=t_uniform
    # build design matrix on train
    cos_terms=np.cos(np.outer(tt[train_slice],gammas))
    sin_terms=np.sin(np.outer(tt[train_slice],gammas))
    X=np.hstack([cos_terms,sin_terms])
    y=f_uniform[train_slice]
    coeffs, *_=np.linalg.lstsq(X,y,rcond=None)
    a=coeffs[:len(gammas)]
    b=coeffs[len(gammas):]
    # reconstruction
    cos_all=np.cos(np.outer(tt,gammas))
    sin_all=np.sin(np.outer(tt,gammas))
    fhat=cos_all@a + sin_all@b
    # R2
    def r2(ytrue, ypred):
        ss_res=np.sum((ytrue-ypred)**2)
        ss_tot=np.sum((ytrue-np.mean(ytrue))**2)
        return 1.0 - ss_res/ss_tot if ss_tot>0 else float("nan")
    r2_train=r2(f_uniform[train_slice], fhat[train_slice])
    r2_test=r2(f_uniform[test_slice], fhat[test_slice])
    # field
    Z=np.sum((a-1j*b)*np.exp(1j*np.outer(tt,gammas)), axis=1)
    phi=np.unwrap(np.angle(Z))
    pred=np.where(np.cos(phi)>0, 1, -1).astype(int)
    acc_train=float(np.mean(pred[train_slice]==s_true[train_slice]))
    acc_test=float(np.mean(pred[test_slice]==s_true[test_slice]))
    return dict(a=a,b=b,fhat=fhat,phi=phi,pred=pred,acc_train=acc_train,acc_test=acc_test,r2_train=r2_train,r2_test=r2_test)
real_res=fit_and_eval(gammas_real)
ctrl_res=fit_and_eval(gammas_ctrl)
# Step4 rolling predictor
window=200
# causal rolling mean via cumulative sum
cs=np.cumsum(s_true, dtype=float)
roll_mean=np.empty(n,float)
for i in range(n):
    j0=max(0,i-window+1)
    total=cs[i] - (cs[j0-1] if j0>0 else 0.0)
    roll_mean[i]= total / (i-j0+1)
pred_roll=np.sign(roll_mean)
# forward fill zeros (rare if mean=0)
last=int(s_true[0])
for i in range(n):
    if pred_roll[i]==0:
        pred_roll[i]=last
    else:
        last=int(pred_roll[i])
pred_roll=pred_roll.astype(int)
acc_roll_train=float(np.mean(pred_roll[train_slice]==s_true[train_slice]))
acc_roll_test=float(np.mean(pred_roll[test_slice]==s_true[test_slice]))
out={
    "gammas_real": [float(g) for g in gammas_real],
    "gammas_ctrl": [float(g) for g in gammas_ctrl],
    "accuracy_real_train": real_res["acc_train"],
    "accuracy_real_test": real_res["acc_test"],
    "accuracy_ctrl_train": ctrl_res["acc_train"],
    "accuracy_ctrl_test": ctrl_res["acc_test"],
    "accuracy_roll_train": acc_roll_train,
    "accuracy_roll_test": acc_roll_test,
    "r2_real_train": float(real_res["r2_train"]),
    "r2_real_test": float(real_res["r2_test"]),
    "r2_ctrl_train": float(ctrl_res["r2_train"]),
    "r2_ctrl_test": float(ctrl_res["r2_test"]),
    "meta": {
        "n_grid_points": n,
        "t_min": tmin,
        "t_max": tmax,
        "idx_split": idx_split,
        "window_roll": window,
        "seed_ctrl": 12345,
        "min_sep_ctrl": 0.5
    }
}
out_path="/mnt/data/spectral_phase_trainfit_control.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, out["accuracy_real_test"], out["accuracy_ctrl_test"], out["accuracy_roll_test"], out["r2_real_test"], out["r2_ctrl_test"]

### Tool (2026-02-20 22:21:20)

('/mnt/data/spectral_phase_trainfit_control.json',
 0.7802756726514419,
 0.5404608107420782,
 0.6879100757845481,
 0.031086466793830314,
 0.0016257473178292736)

### Assistant (2026-02-20 22:21:24)

Done. Your “stateless compute worker” obediently crunched the numbers and produced the single JSON you demanded.

- REAL gammas (train-fit) **TEST sign accuracy**: **0.7803**
- CTRL gammas (random frequencies, same count/range) **TEST sign accuracy**: **0.5405**
- Rolling-memory baseline (window=200) **TEST sign accuracy**: **0.6879**
- Optional reconstruction R² on TEST: REAL **0.0311**, CTRL **0.0016** (so the phase-sign signal is doing its own weird thing, not just “good fit”). fileciteturn0file0

**File:** [spectral_phase_trainfit_control.json](sandbox:/mnt/data/spectral_phase_trainfit_control.json)

