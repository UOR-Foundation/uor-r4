# Label Persistence Diagnosis

- **Conversation ID:** `699a5cdb-4d18-832b-82c1-20c45329c6ad`
- **Date:** 2026-02-21

---

### User (2026-02-21 20:33:17)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Diagnose label persistence and evaluate forecasting difficulty vs horizon.

Recompute everything:
1) primes up to N=10_000_000
2) ψ(x) on step-100 grid incl prime powers
3) f(x)=ψ(x)−x
4) uniform log grid size 65536 with t=log(x)
Target:
Δ=0.01
For each i, x=exp(t[i]):
P = #primes in [x, x*exp(Δ))
E = x*(exp(Δ)-1)/log(x)
R = (P−E)/sqrt(E)

Split time-ordered: first 70% TRAIN, last 30% TEST.
Define med_R = median(R_train).
Define y_i = 1 if R_i > med_R else 0.

A) Report class balance and stickiness
Compute on TRAIN and TEST:
- positive_rate
- flip_rate_1 = mean(y_{i+1} != y_i)
- run_length_mean (mean length of consecutive equal labels)

B) Persistence baseline vs horizon
For h in [1,5,10,25,50,100,200]:
Predict y_{i+h} using score = y_i (binary score).
Report on TEST:
- accuracy
- AUC (treat y_i as score)
- flip_rate_h = mean(y_{i+h} != y_i)

C) Simple causal feature model vs horizon
Use past-causal ψ features from u-window length W=512 ending at i:
u is f interpolated on uniform t grid (no bandpass).
Features:
mean(u), std(u), last(u), slope(u), mean(abs(diff(u))),
FFT energy in [0,0.005], [0.005,0.01], [0.01,0.02] cycles/sample.

For each horizon h in [1,5,10,25,50,100,200]:
Train logistic regression on TRAIN to predict y_{i+h} from features at i.
Evaluate on TEST:
- AUC
- Brier
- accuracy

Output JSON:
{
"balance":{
"train_pos_rate":...,"test_pos_rate":...,
"train_flip_rate_1":...,"test_flip_rate_1":...,
"train_runlen_mean":...,"test_runlen_mean":...
},
"persistence_by_h":[{"h":...,"acc":...,"auc":...,"flip_rate_h":...},...],
"feature_model_by_h":[{"h":...,"auc":...,"brier":...,"acc":...},...]
}

No other text.

### Assistant (2026-02-21 20:33:18)

import numpy as np, math, json, sys, os, statistics

### Assistant (2026-02-21 20:34:11)

N=10_000_000
# sieve
limit=N
is_prime = np.ones(limit+1, dtype=np.bool_)
is_prime[:2]=False
# simple optimized sieve
for p in range(2, int(limit**0.5)+1):
    if is_prime[p]:
        step=p
        start=p*p
        is_prime[start:limit+1:step]=False
primes = np.nonzero(is_prime)[0]
len(primes), primes[:5], primes[-1]

### Tool (2026-02-21 20:34:14)

(664579, array([ 2,  3,  5,  7, 11]), 9999991)

### Assistant (2026-02-21 20:34:16)

# pi array cumulative counts
pi = np.zeros(limit+1, dtype=np.int32)
pi[0]=0
pi[1]=0
pi[2:]=np.cumsum(is_prime[2:], dtype=np.int32)
pi[:10], pi[10], pi[100]

### Tool (2026-02-21 20:34:18)

(array([0, 0, 1, 2, 2, 3, 3, 4, 4, 4], dtype=int32), 4, 25)

### Assistant (2026-02-21 20:34:20)

# psi on step-100 grid
step=100
grid_x = np.arange(0, limit+1, step, dtype=np.int64)
m = grid_x.size
delta = np.zeros(m, dtype=np.float64)
logp = np.log(primes.astype(np.float64))
# add contributions for prime powers
for p, lp in zip(primes, logp):
    q = int(p)
    while q <= limit:
        idx = (q + step - 1)//step  # ceil(q/step)
        if idx < m:
            delta[idx] += lp
        else:
            break
        # next power
        if q > limit//p:
            break
        q *= p
# cumulative
psi_grid = np.cumsum(delta)
# f on grid (exclude x=0)
xgrid2 = grid_x[1:]
psi2 = psi_grid[1:]
f_linear = psi2 - xgrid2.astype(np.float64)
t_linear = np.log(xgrid2.astype(np.float64))
t_linear[0], t_linear[-1], f_linear[:3], f_linear[-3:]

### Tool (2026-02-21 20:34:24)

(4.605170185988091,
 16.11809565095832,
 array([-5.95468877,  6.14585683, -0.76766456]),
 array([-1486.24987503, -1505.65946328, -1460.59665412]))

### Assistant (2026-02-21 20:34:30)

n_t=65536
t = np.linspace(t_linear[0], math.log(limit), n_t, dtype=np.float64)
x = np.exp(t)
Delta=0.01
x2 = x*np.exp(Delta)
# compute P using pi
lower = np.ceil(x).astype(np.int64)
upper = np.ceil(x2).astype(np.int64)-1
lower = np.clip(lower, 1, limit+1)
upper = np.clip(upper, 0, limit)
# P: if lower>limit or upper<lower =>0
P = np.zeros(n_t, dtype=np.int32)
mask = (lower<=limit) & (upper>=lower)
lo = lower[mask]
up = upper[mask]
P[mask] = pi[up] - pi[lo-1]
# expected
E = x*(np.exp(Delta)-1.0)/t
R = (P - E)/np.sqrt(E)
# interpolate f onto t grid
u = np.interp(t, t_linear, f_linear)  # causal feature input
# split
train_end = int(0.7*n_t)
R_train = R[:train_end]
med_R = np.median(R_train)
y = (R > med_R).astype(np.int8)
y_train = y[:train_end]
y_test = y[train_end:]
y_train.mean(), y_test.mean(), med_R

### Tool (2026-02-21 20:34:36)

(0.4999891008174387, 0.5233202787243783, -0.06303334072198186)

### Assistant (2026-02-21 20:34:38)

def flip_rate(arr):
    if len(arr)<2: return float('nan')
    return float(np.mean(arr[1:] != arr[:-1]))
def run_length_mean(arr):
    # mean length of consecutive equal labels
    if len(arr)==0: return float('nan')
    diffs = np.nonzero(arr[1:] != arr[:-1])[0] + 1
    # run boundaries
    boundaries = np.concatenate(([0], diffs, [len(arr)]))
    lengths = np.diff(boundaries)
    return float(lengths.mean())

balance = {
    "train_pos_rate": float(y_train.mean()),
    "test_pos_rate": float(y_test.mean()),
    "train_flip_rate_1": flip_rate(y_train),
    "test_flip_rate_1": flip_rate(y_test),
    "train_runlen_mean": run_length_mean(y_train),
    "test_runlen_mean": run_length_mean(y_test),
}
balance

### Tool (2026-02-21 20:34:42)

{'train_pos_rate': 0.4999891008174387,
 'test_pos_rate': 0.5233202787243783,
 'train_flip_rate_1': 0.05231721672407028,
 'test_flip_rate_1': 0.09150559511698882,
 'train_runlen_mean': 19.106622240733028,
 'test_runlen_mean': 10.922777777777778}

### Assistant (2026-02-21 20:34:44)

from sklearn.metrics import roc_auc_score, accuracy_score, brier_score_loss

### Assistant (2026-02-21 20:34:47)

hs=[1,5,10,25,50,100,200]
persistence=[]
for h in hs:
    # test indices i in [train_end, n_t-h)
    i0=train_end
    i1=n_t-h
    y_true = y[i0+h:i1+h]
    y_score = y[i0:i1]  # 0/1
    y_pred = y_score
    acc = float(accuracy_score(y_true, y_pred))
    # AUC
    try:
        auc = float(roc_auc_score(y_true, y_score))
    except Exception:
        auc = float('nan')
    flip_h = float(np.mean(y[i0+h:i1+h] != y[i0:i1]))
    persistence.append({"h":h,"acc":acc,"auc":auc,"flip_rate_h":flip_h})
persistence

### Tool (2026-02-21 20:34:50)

[{'h': 1,
  'acc': 0.9084944048830111,
  'auc': 0.9082929554564769,
  'flip_rate_h': 0.09150559511698882},
 {'h': 5,
  'acc': 0.8152726902726902,
  'auc': 0.8148623273177484,
  'flip_rate_h': 0.18472730972730972},
 {'h': 10,
  'acc': 0.7564500534323954,
  'auc': 0.755906568248222,
  'flip_rate_h': 0.2435499465676047},
 {'h': 25,
  'acc': 0.6406090853534324,
  'auc': 0.6398079601632563,
  'flip_rate_h': 0.3593909146465675},
 {'h': 50,
  'acc': 0.49650706236295955,
  'auc': 0.49540811356342696,
  'flip_rate_h': 0.5034929376370404},
 {'h': 100,
  'acc': 0.46597822197229183,
  'auc': 0.4647538022396498,
  'flip_rate_h': 0.5340217780277082},
 {'h': 200,
  'acc': 0.4751554390832948,
  'auc': 0.4739959904700192,
  'flip_rate_h': 0.5248445609167052}]

### Assistant (2026-02-21 20:34:53)

W=512
u_arr = u.astype(np.float64)
n=len(u_arr)
start=W-1
m_feat = n-start  # count of i from W-1..n-1
# rolling sums for mean/std
cs = np.concatenate(([0.0], np.cumsum(u_arr)))
cs2 = np.concatenate(([0.0], np.cumsum(u_arr*u_arr)))
# sum over window ending at i: [i-W+1, i+1)
idx_end = np.arange(start+1, n+1)
idx_start = idx_end - W
sum_u = cs[idx_end]-cs[idx_start]
sum_u2 = cs2[idx_end]-cs2[idx_start]
mean_u = sum_u/W
var_u = sum_u2/W - mean_u*mean_u
var_u = np.maximum(var_u, 0.0)
std_u = np.sqrt(var_u)
last_u = u_arr[start:]
# slope via weighted sum with centered indices
j = np.arange(W, dtype=np.float64)
j_mean = (W-1)/2.0
w = j - j_mean
den = np.sum(w*w)
# convolution to get weighted sum for each window
# np.convolve full; need valid for windows of length W
weighted_sum = np.convolve(u_arr, w[::-1], mode='valid')  # length n-W+1 equals m_feat
# slope = (Σ w*u - mean_u*Σ w)/den ; but Σ w =0 by centering
slope = weighted_sum/den
# mean abs diff
absdiff = np.abs(np.diff(u_arr))
csad = np.concatenate(([0.0], np.cumsum(absdiff)))
# for window ending at i, diffs from i-W+1..i-1 inclusive length W-1 => indices in absdiff [i-W+1 : i]
idx_end_d = np.arange(start, n)  # corresponds to i
# end exclusive in csad is i (since absdiff index up to i-1)
end_d = idx_end_d
start_d = end_d-(W-1)
mad = (csad[end_d]-csad[start_d])/(W-1)
# FFT energies in chunks
freqs = np.fft.rfftfreq(W, d=1.0)  # cycles/sample
bands=[(0.0,0.005),(0.005,0.01),(0.01,0.02)]
band_bins=[np.where((freqs>=lo) & (freqs<hi))[0] for lo,hi in bands]
[ (b[0], b[-1], len(b)) for b in band_bins ]

### Tool (2026-02-21 20:35:07)

[(0, 2, 3), (3, 5, 3), (6, 10, 5)]

### Assistant (2026-02-21 20:35:09)

# compute FFT energies for each window, chunked
energy = np.zeros((m_feat, 3), dtype=np.float32)
batch=2048
for s in range(0, m_feat, batch):
    e = min(s+batch, m_feat)
    # windows: u_arr[s:s+W]?? careful: valid convolution gives windows starting at s. 
    # We need windows ending at i=start+s => window start = (start+s)-W+1 = s
    # so indeed window is u_arr[s:s+W]
    win = np.lib.stride_tricks.sliding_window_view(u_arr, W)[s:e]  # shape (e-s,W)
    F = np.fft.rfft(win, axis=1)
    powspec = (F.real*F.real + F.imag*F.imag)
    for bi,bins in enumerate(band_bins):
        energy[s:e, bi] = powspec[:, bins].sum(axis=1).astype(np.float32)
# assemble features matrix X for i indices start..n-1
X = np.column_stack([mean_u, std_u, last_u, slope, mad, energy])
X.shape, X[:2]

### Tool (2026-02-21 20:35:14)

((65025, 8),
 array([[-5.17110978e+00,  4.53284001e-01, -4.38753078e+00,
          3.06684538e-03,  3.06684538e-03,  7.03029450e+06,
          3.49787476e+03,  1.41159094e+03],
        [-5.16804293e+00,  4.53284001e-01, -4.38446394e+00,
          3.06684538e-03,  3.06684538e-03,  7.02198250e+06,
          3.49787476e+03,  1.41159094e+03]]))

### Assistant (2026-02-21 20:35:17)

from sklearn.preprocessing import StandardScaler
from sklearn.linear_model import LogisticRegression
from sklearn.pipeline import make_pipeline

feature_model=[]
for h in hs:
    # determine row ranges
    train_last_i = train_end - h - 1
    if train_last_i < start:
        continue
    r_train_end = (train_end - h) - start  # exclusive
    X_train = X[:r_train_end]
    y_train_h = y[start+h:train_end]  # y_{i+h} for i in [start, train_end-h)
    # test
    r_test_start = train_end - start
    r_test_end = (n_t - h) - start
    X_test = X[r_test_start:r_test_end]
    y_test_h = y[train_end+h:n_t]  # for i in [train_end, n-h)
    # model
    clf = make_pipeline(StandardScaler(with_mean=True, with_std=True),
                        LogisticRegression(max_iter=200, solver='lbfgs'))
    clf.fit(X_train, y_train_h)
    prob = clf.predict_proba(X_test)[:,1]
    pred = (prob>=0.5).astype(np.int8)
    acc=float(accuracy_score(y_test_h, pred))
    auc=float(roc_auc_score(y_test_h, prob))
    brier=float(brier_score_loss(y_test_h, prob))
    feature_model.append({"h":h,"auc":auc,"brier":brier,"acc":acc})
feature_model[:2], len(feature_model)

### Tool (2026-02-21 20:35:26)

([{'h': 1,
   'auc': 0.5843170177339072,
   'brier': 0.42742464457739987,
   'acc': 0.5207019328585961},
  {'h': 5,
   'auc': 0.5797550380130458,
   'brier': 0.42996836172784864,
   'acc': 0.5156186406186406}],
 7)

### Assistant (2026-02-21 20:35:40)

feature_model

### Tool (2026-02-21 20:35:41)

[{'h': 1,
  'auc': 0.5843170177339072,
  'brier': 0.42742464457739987,
  'acc': 0.5207019328585961},
 {'h': 5,
  'auc': 0.5797550380130458,
  'brier': 0.42996836172784864,
  'acc': 0.5156186406186406},
 {'h': 10,
  'auc': 0.5769616994434036,
  'brier': 0.42929584239086205,
  'acc': 0.5146302987125337},
 {'h': 25,
  'auc': 0.574951934736029,
  'brier': 0.42447489023350976,
  'acc': 0.5135974740272968},
 {'h': 50,
  'auc': 0.5687566311547526,
  'brier': 0.4118552495297918,
  'acc': 0.513079394217531},
 {'h': 100,
  'auc': 0.5512738899871465,
  'brier': 0.41329231446363995,
  'acc': 0.5128060937579878},
 {'h': 200,
  'auc': 0.5069150248178874,
  'brier': 0.3709129256188818,
  'acc': 0.5013103129335594}]

### Assistant (2026-02-21 20:35:42)

out = {"balance": balance, "persistence_by_h": persistence, "feature_model_by_h": feature_model}
json_str = json.dumps(out, separators=(',',':'))
len(json_str), json_str[:200]

### Tool (2026-02-21 20:35:44)

(1537,
 '{"balance":{"train_pos_rate":0.4999891008174387,"test_pos_rate":0.5233202787243783,"train_flip_rate_1":0.05231721672407028,"test_flip_rate_1":0.09150559511698882,"train_runlen_mean":19.106622240733028')

### Assistant (2026-02-21 20:35:46)

{"balance":{"train_pos_rate":0.4999891008174387,"test_pos_rate":0.5233202787243783,"train_flip_rate_1":0.05231721672407028,"test_flip_rate_1":0.09150559511698882,"train_runlen_mean":19.106622240733028,"test_runlen_mean":10.922777777777778},"persistence_by_h":[{"h":1,"acc":0.9084944048830111,"auc":0.9082929554564769,"flip_rate_h":0.09150559511698882},{"h":5,"acc":0.8152726902726902,"auc":0.8148623273177484,"flip_rate_h":0.18472730972730972},{"h":10,"acc":0.7564500534323954,"auc":0.755906568248222,"flip_rate_h":0.2435499465676047},{"h":25,"acc":0.6406090853534324,"auc":0.6398079601632563,"flip_rate_h":0.3593909146465675},{"h":50,"acc":0.49650706236295955,"auc":0.49540811356342696,"flip_rate_h":0.5034929376370404},{"h":100,"acc":0.46597822197229183,"auc":0.4647538022396498,"flip_rate_h":0.5340217780277082},{"h":200,"acc":0.4751554390832948,"auc":0.4739959904700192,"flip_rate_h":0.5248445609167052}],"feature_model_by_h":[{"h":1,"auc":0.5843170177339072,"brier":0.42742464457739987,"acc":0.5207019328585961},{"h":5,"auc":0.5797550380130458,"brier":0.42996836172784864,"acc":0.5156186406186406},{"h":10,"auc":0.5769616994434036,"brier":0.42929584239086205,"acc":0.5146302987125337},{"h":25,"auc":0.574951934736029,"brier":0.42447489023350976,"acc":0.5135974740272968},{"h":50,"auc":0.5687566311547526,"brier":0.4118552495297918,"acc":0.513079394217531},{"h":100,"auc":0.5512738899871465,"brier":0.41329231446363995,"acc":0.5128060937579878},{"h":200,"auc":0.5069150248178874,"brier":0.3709129256188818,"acc":0.5013103129335594}]}

