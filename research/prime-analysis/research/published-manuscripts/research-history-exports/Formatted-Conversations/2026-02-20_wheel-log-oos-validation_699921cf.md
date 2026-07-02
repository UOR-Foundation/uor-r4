# Wheel Log OOS Validation

- **Conversation ID:** `699921cf-5448-832e-ab38-eb6418a88d60`
- **Date:** 2026-02-20

---

### User (2026-02-20 22:09:22)

You are a stateless compute worker.

You will receive one file:
	•	chebyshev_psi.npz (x_grid, psi_minus_x)

Define:
	•	f(x)=psi_minus_x
	•	t=log(x_grid)

Goal: validate whether a wheel+log phase model generalizes out-of-sample and how it depends on modulus M.

⸻

Step 1 — analytic phase φ(t)
	1.	Interpolate f(t) onto a uniform t grid (65536 points).
	2.	Compute analytic signal via Hilbert transform:
A(t)=f+iH(f)
	3.	φ(t)=unwrap(angle(A(t)))
Work only on this uniform grid from here.

⸻

Step 2 — define θ_wheel for a modulus M

For each M in {210, 2310, 30030}:
	1.	Compute reduced residue list R={r mod M : gcd(r,M)=1} sorted ascending.
	2.	For each x in the UNIFORM GRID (map each t_uniform point back to x via exp(t) and round to nearest original x_grid value):
r = x mod M
If gcd(r,M)!=1: invalid
Else θ_wheel = 2π*idx(r)/φ(M)
Unwrap θ_wheel over valid points.

⸻

Step 3 — train/test split in log-space

Split the uniform t grid into:
	•	Train = first 70%
	•	Test = last 30%

For each M:
	1.	On TRAIN only, fit β and δ to maximize corr( sin(φ), sin(θ_wheel + β t + δ) )
Use grid search β ∈ [-5, 5] step 0.05 (or smarter optimizer if you want).
For each β, choose δ as the best phase shift (maximize correlation).
	2.	Report:
	•	corr_train_wheel_only (β=0, best δ)
	•	corr_train_best_hybrid (best β, best δ)
	•	corr_test_wheel_only (β=0, δ trained on train or re-fit δ on test; state which)
	•	corr_test_best_hybrid (apply β,δ from train to test with no refit)

Also report valid_fraction for each M.

⸻

Output (single file)

Return exactly one JSON:
	•	wheel_log_oos_validation.json

with per-modulus results:
	•	best_beta_train, best_delta_train
	•	train/test correlations listed above
	•	valid_fraction
	•	notes on any implementation choices

No plots, no extra files.

### Assistant (2026-02-20 22:09:23)

import numpy as np, json, math, os, sys
from pathlib import Path

path="/mnt/data/chebyshev_psi.npz"
data=np.load(path)
data.files

### Tool (2026-02-20 22:09:59)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-20 22:10:01)

x_grid=data['x_grid']
f=data['psi_minus_x']
x_grid[:5], x_grid[-5:], f[:5], f[-5:], x_grid.dtype, f.dtype, len(x_grid)

### Tool (2026-02-20 22:10:04)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 array([-1.        , -2.33956825,  5.14585683, -1.76766456,  2.82480716]),
 array([-1396.78466777, -1432.31246137, -1451.72215382, -1487.24987502,
        -1490.54137751]),
 dtype('int32'),
 dtype('float64'),
 100000)

### Assistant (2026-02-20 22:10:05)

import scipy, scipy.signal, scipy.fft
scipy.__version__

### Tool (2026-02-20 22:10:10)

'1.14.1'

### Assistant (2026-02-20 22:10:12)

N=int(data['N'])
step=int(data['step'])
N, step

### Tool (2026-02-20 22:10:16)

(10000000, 100)

### Assistant (2026-02-20 22:10:18)

import numpy as np, math, json
from scipy.signal import hilbert

x0=int(x_grid[0])
step=int(data['step'])
t_grid=np.log(x_grid.astype(np.float64))
t0,t1=t_grid[0],t_grid[-1]
Nu=65536
t_uniform=np.linspace(t0,t1,Nu)
# interpolate f onto t_uniform
f_uniform=np.interp(t_uniform, t_grid, f)
# analytic signal + phase
A=hilbert(f_uniform)
phi=np.unwrap(np.angle(A))
sin_phi=np.sin(phi)

# map t_uniform -> x via exp and nearest x_grid value
x_cont=np.exp(t_uniform)
idx=np.rint((x_cont - x0)/step).astype(np.int64)
idx=np.clip(idx, 0, len(x_grid)-1)
x_mapped=(x0 + idx*step).astype(np.int64)

split=int(0.7*Nu)
i=np.arange(Nu)

def pearson_corr(a,b):
    m=np.isfinite(a) & np.isfinite(b)
    if m.sum()<3:
        return float('nan')
    aa=a[m]; bb=b[m]
    aa=aa-np.mean(aa); bb=bb-np.mean(bb)
    sa=np.std(aa); sb=np.std(bb)
    if sa==0 or sb==0:
        return float('nan')
    return float(np.dot(aa,bb)/((len(aa)-1)*sa*sb))

def best_delta_and_corr(y,u):
    # y,u same length
    y0=y-np.mean(y)
    su=np.sin(u); cu=np.cos(u)
    su0=su-np.mean(su); cu0=cu-np.mean(cu)
    a=np.dot(y0, su0)
    b=np.dot(y0, cu0)
    delta=math.atan2(b,a)
    s=np.sin(u+delta)
    corr=pearson_corr(y,s)
    return delta, corr

def totient_and_map(M):
    # map residues 0..M-1 -> idx or -1
    idxmap=np.full(M, -1, dtype=np.int32)
    residues=[]
    for r in range(1,M):
        if math.gcd(r,M)==1:
            residues.append(r)
    residues=np.array(residues, dtype=np.int32)
    idxmap[residues]=np.arange(len(residues), dtype=np.int32)
    return len(residues), idxmap

results={}
notes=[]
for M in [210,2310,30030]:
    phiM, idxmap = totient_and_map(M)
    r = (x_mapped % M).astype(np.int64)
    valid = np.gcd(r, M)==1
    valid_fraction=float(valid.mean())
    theta=np.full(Nu, np.nan, dtype=np.float64)
    theta_valid = 2*math.pi*idxmap[r[valid]]/phiM
    theta[valid]=np.unwrap(theta_valid)
    # train/test masks
    train = (i < split) & valid
    test = (i >= split) & valid
    # wheel-only train delta and corr
    delta0, corr_train_wheel = best_delta_and_corr(sin_phi[train], theta[train])
    # wheel-only test corr using delta0 from train
    corr_test_wheel = pearson_corr(sin_phi[test], np.sin(theta[test]+delta0))
    # hybrid beta sweep on train
    betas=np.arange(-5, 5.0001, 0.05)
    best_corr=-1e9
    best_beta=None
    best_delta=None
    # prepare train arrays
    y_train=sin_phi[train]
    theta_train=theta[train]
    t_train=t_uniform[train]
    for beta in betas:
        u=theta_train + beta*t_train
        d,c = best_delta_and_corr(y_train,u)
        if c>best_corr:
            best_corr=c; best_beta=float(beta); best_delta=float(d)
    corr_train_hybrid=float(best_corr)
    # apply to test no refit
    u_test=theta[test] + best_beta*t_uniform[test]
    corr_test_hybrid=pearson_corr(sin_phi[test], np.sin(u_test + best_delta))
    results[str(M)] = {
        "best_beta_train": best_beta,
        "best_delta_train": best_delta,
        "corr_train_wheel_only": float(corr_train_wheel),
        "corr_train_best_hybrid": float(corr_train_hybrid),
        "corr_test_wheel_only": float(corr_test_wheel),
        "corr_test_best_hybrid": float(corr_test_hybrid),
        "valid_fraction": valid_fraction,
        "n_valid_train": int(train.sum()),
        "n_valid_test": int(test.sum()),
        "phi_M": int(phiM)
    }

notes.append("Uniform t grid: 65536 points linearly spaced in t=log(x) between log(min(x_grid)) and log(max(x_grid)). f interpolated with 1D linear interpolation in t.")
notes.append("Analytic signal computed using scipy.signal.hilbert on f(t_uniform); phase phi(t) = unwrap(angle(A)).")
notes.append("Mapping t_uniform -> x performed by x=exp(t) and rounding to nearest x_grid value using known step; equivalent to nearest neighbor in x_grid.")
notes.append("Wheel phase uses reduced residue system modulo M: residues r in [1,M-1] with gcd(r,M)=1 sorted ascending; idx(r) is 0-based index in this list; theta_wheel = 2π*idx/phi(M).")
notes.append("Theta_wheel unwrapped only over valid points (gcd(x mod M, M)=1), preserving time order; invalid points excluded from correlations.")
notes.append("Train/test split is by index on uniform t grid: first 70% train, last 30% test.")
notes.append("For each beta, delta is chosen analytically by maximizing dot product between centered sin(phi) and sin(theta+beta t + delta) using sin/cos components; correlations are Pearson on the resulting sine series.")
notes.append("Test correlations use beta,delta learned on train with no refit; wheel-only test uses delta learned on train (no refit).")

out={"per_modulus":results,"notes":notes}
outpath="/mnt/data/wheel_log_oos_validation.json"
with open(outpath,"w") as fp:
    json.dump(out, fp, indent=2)
outpath, {k: (v["corr_train_wheel_only"], v["corr_train_best_hybrid"], v["corr_test_wheel_only"], v["corr_test_best_hybrid"], v["valid_fraction"]) for k,v in results.items()}

### Tool (2026-02-20 22:11:00)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-20 22:12:02)

import numpy as np, math, json
from scipy.signal import hilbert

# reload
data=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=data['x_grid'].astype(np.int64)
f=data['psi_minus_x'].astype(np.float64)
x0=int(x_grid[0]); step=int(data['step'])
t_grid=np.log(x_grid.astype(np.float64))
Nu=65536
t_uniform=np.linspace(t_grid[0], t_grid[-1], Nu)
f_uniform=np.interp(t_uniform, t_grid, f)
A=hilbert(f_uniform)
phi=np.unwrap(np.angle(A))
sin_phi=np.sin(phi)

x_cont=np.exp(t_uniform)
idx=np.rint((x_cont - x0)/step).astype(np.int64)
idx=np.clip(idx, 0, len(x_grid)-1)
x_mapped=(x0 + idx*step).astype(np.int64)

i=np.arange(Nu)
split=int(0.7*Nu)

def pearson_corr_masked(a,b,mask):
    m=mask & np.isfinite(a) & np.isfinite(b)
    n=m.sum()
    if n<3:
        return float('nan')
    aa=a[m]; bb=b[m]
    aa=aa-np.mean(aa); bb=bb-np.mean(bb)
    sa=np.std(aa); sb=np.std(bb)
    if sa==0 or sb==0:
        return float('nan')
    return float(np.dot(aa,bb)/((n-1)*sa*sb))

def totient_and_map(M):
    idxmap=np.full(M, -1, dtype=np.int32)
    residues=[r for r in range(1,M) if math.gcd(r,M)==1]
    residues=np.array(residues, dtype=np.int32)
    idxmap[residues]=np.arange(len(residues), dtype=np.int32)
    return len(residues), idxmap

def best_beta_delta_corr(y, theta, t, betas):
    # y, theta, t are 1D arrays for TRAIN valid points
    n=len(y)
    y_mean=y.mean()
    y0=y - y_mean
    y_std=np.std(y0)
    if y_std==0 or n<3:
        return float('nan'), float('nan'), float('nan')
    sin_th=np.sin(theta); cos_th=np.cos(theta)
    best_corr=-1e9
    best_beta=0.0
    best_delta=0.0
    for beta in betas:
        bt=beta*t
        sbt=np.sin(bt); cbt=np.cos(bt)
        su = sin_th*cbt + cos_th*sbt       # sin(theta+beta t)
        cu = cos_th*cbt - sin_th*sbt       # cos(theta+beta t)
        a = float(np.dot(y0, su))
        b = float(np.dot(y0, cu))
        norm = math.hypot(a,b)
        if norm==0:
            continue
        # delta maximizing dot product
        cosd = a/norm
        sind = b/norm
        s = su*cosd + cu*sind              # sin(u+delta)
        s0 = s - s.mean()
        s_std=np.std(s0)
        if s_std==0:
            continue
        corr = float(np.dot(y0, s0)/((n-1)*y_std*s_std))
        if corr>best_corr:
            best_corr=corr
            best_beta=float(beta)
            best_delta=float(math.atan2(b,a))
    return best_beta, best_delta, float(best_corr)

results={}
notes=[]
betas=np.arange(-5, 5.0001, 0.05)

for M in [210,2310,30030]:
    phiM, idxmap = totient_and_map(M)
    r = (x_mapped % M).astype(np.int64)
    valid = np.gcd(r, M)==1
    theta=np.full(Nu, np.nan, dtype=np.float64)
    theta_vals = 2*math.pi*idxmap[r[valid]]/phiM
    theta[valid]=np.unwrap(theta_vals)
    train_mask = valid & (i < split)
    test_mask = valid & (i >= split)
    valid_fraction=float(valid.mean())
    # wheel only: beta=0 -> find delta on train
    ytr=sin_phi[train_mask]
    thtr=theta[train_mask]
    ttr=t_uniform[train_mask]
    # delta from dot products
    y0=ytr-ytr.mean()
    su=np.sin(thtr); cu=np.cos(thtr)
    a=float(np.dot(y0, su))
    b=float(np.dot(y0, cu))
    delta0=float(math.atan2(b,a))
    # correlations
    corr_train_wheel = pearson_corr_masked(sin_phi, np.sin(theta+delta0), train_mask)
    corr_test_wheel  = pearson_corr_masked(sin_phi, np.sin(theta+delta0), test_mask)
    # hybrid search
    best_beta, best_delta, corr_train_hybrid = best_beta_delta_corr(ytr, thtr, ttr, betas)
    # apply to test
    corr_test_hybrid = pearson_corr_masked(sin_phi, np.sin(theta + best_beta*t_uniform + best_delta), test_mask)
    results[str(M)] = {
        "best_beta_train": best_beta,
        "best_delta_train": best_delta,
        "corr_train_wheel_only": float(corr_train_wheel),
        "corr_train_best_hybrid": float(corr_train_hybrid),
        "corr_test_wheel_only": float(corr_test_wheel),
        "corr_test_best_hybrid": float(corr_test_hybrid),
        "valid_fraction": valid_fraction,
        "n_valid_train": int(train_mask.sum()),
        "n_valid_test": int(test_mask.sum()),
        "phi_M": int(phiM),
        "delta_wheel_only_train": delta0
    }

notes.append("Uniform t grid: 65536 points linearly spaced in t=log(x) between log(min(x_grid)) and log(max(x_grid)). f interpolated with 1D linear interpolation in t.")
notes.append("Analytic signal computed using scipy.signal.hilbert on f(t_uniform); phase phi(t) = unwrap(angle(A)).")
notes.append("Mapping t_uniform -> x performed by x=exp(t) then rounded to nearest x_grid point using the dataset's constant step (nearest neighbor on the original x_grid).")
notes.append("Wheel phase uses reduced residue system modulo M: residues r in [1,M-1] with gcd(r,M)=1 sorted ascending; idx(r) is 0-based index; theta_wheel = 2π*idx/phi(M).")
notes.append("Theta_wheel unwrapped only over valid points (gcd(x mod M, M)=1); invalid points excluded from correlations.")
notes.append("Train/test split: first 70% of the uniform t grid is train; last 30% is test.")
notes.append("Delta selection for fixed beta uses the analytic maximizer of dot(y_centered, sin(theta+beta t + delta)), equivalent to delta=atan2(<y,cos u>, <y,sin u>) with u=theta+beta t on the TRAIN valid points; Pearson correlation is then computed between centered series.")
notes.append("Test correlations apply beta,delta learned on TRAIN with no refit; wheel-only test uses delta learned on TRAIN (no refit).")

out={"per_modulus":results,"notes":notes}
outpath="/mnt/data/wheel_log_oos_validation.json"
with open(outpath,"w") as fp:
    json.dump(out, fp, indent=2)

outpath, {k:(v["corr_train_wheel_only"], v["corr_train_best_hybrid"], v["corr_test_wheel_only"], v["corr_test_best_hybrid"], v["valid_fraction"]) for k,v in results.items()}

### Tool (2026-02-20 22:12:43)

('/mnt/data/wheel_log_oos_validation.json',
 {'210': (0.579836821219842,
   0.587810572146057,
   -0.0139638222490875,
   -0.011355304185529783,
   0.678955078125),
  '2310': (0.355091749285112,
   0.6300661550765715,
   0.004806531434266348,
   -0.007970948862871731,
   0.6499176025390625),
  '30030': (0.23401751718906616,
   0.6692042386190851,
   -0.02793990093970836,
   -0.016514734811947075,
   0.6280364990234375)})

### Assistant (2026-02-20 22:13:27)

[wheellog_oos_validation.json](sandbox:/mnt/data/wheel_log_oos_validation.json)

