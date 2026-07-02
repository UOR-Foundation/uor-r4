# Phase prediction results

- **Conversation ID:** `699a5a52-2834-8333-a3c9-afe4a4285f24`
- **Date:** 2026-02-21

---

### User (2026-02-21 20:22:29)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Two-band latent-state test. Determine whether adding a second band restores forward predictability of phase.

Recompute everything (no uploads):
1) Sieve primes up to N=10_000_000.
2) Compute ψ(x) on grid x=1,101,...,9_999_901 (step=100), incl prime powers.
3) f(x)=ψ(x)−x.
4) t=log(x); interpolate f onto uniform t grid size 65536.

Define two bandpasses in cycles/sample:
Band1 = [0.002, 0.006]
Band2 = [0.006, 0.012]

For each band:
- bandpass via FFT
- Hilbert analytic signal z_j(t)
- phase θ_j(t)=unwrap(angle(z_j))
- amplitude A_j(t)=|z_j(t)|

Define:
θ1_mod = θ1 mod 2π
θ2_mod = θ2 mod 2π
Δθ_mod = (θ2 - θ1) mod 2π (wrap to [-π,π])
ω1(t)=θ1(t+1)-θ1(t)
α1(t)=ω1(t+1)-ω1(t)
L1=log(A1+1e-12), L2=log(A2+1e-12)
ΔL1=L1(t+1)-L1(t), ΔL2=L2(t+1)-L2(t)
LR = (L2-L1)  (log amplitude ratio)

Split time-ordered: 70% TRAIN, 30% TEST.

A) Fit dynamics for α1 on TRAIN
Fit model:
α1 ≈ F(θ1_mod) + G(θ1_mod)*ω1 + H(θ1_mod)*ΔL1
     + J(θ1_mod)*sin(Δθ_mod) + K(θ1_mod)*cos(Δθ_mod)
     + M(θ1_mod)*LR

Where each of F,G,H,J,K,M is a Fourier series up to K=4:
Q(θ)=cQ + Σ_{k=1..4}(aQ_k cos kθ + bQ_k sin kθ)

Fit by least squares on up to 200000 TRAIN samples.

B) Forward simulate θ1 on TEST
Initialize with true θ1_{t-1}, θ1_t, and use TRUE auxiliary inputs on TEST:
use true ΔL1(t), Δθ_mod(t), LR(t) from the extracted bands (we are testing whether these explain forcing, not predicting them yet).
Then recursively:
α_hat from fitted model
ω1_{t+1}=ω1_t+α_hat
θ1_hat_{t+1}=θ1_t+ω1_{t+1}

Compute phase prediction on TEST:
- mean_abs_circ_error between θ1_hat_mod and true θ1_mod
- circular_corr

Also compute baseline for comparison:
- naive linear extrapolation: θ1_hat=2θ1_t-θ1_{t-1}
Report its error/corr too.

Output JSON:
{
"phase_pred":{
"model":{"mean_abs_circ_error":...,"circular_corr":...},
"baseline_linear":{"mean_abs_circ_error":...,"circular_corr":...}
},
"fit_quality":{
"train_R2":...,
"test_R2_on_alpha":...
}
}

No other text.

### Assistant (2026-02-21 20:22:30)

import numpy as np, math, json, time

### Assistant (2026-02-21 20:23:52)

def sieve_primes_upto(n:int):
    # odd-only sieve
    if n < 2:
        return np.array([], dtype=np.int32)
    # size for odds up to n: number of odd numbers <= n, starting at 3
    m = (n - 1)//2  # represents odds 3,5,7,... => index i corresponds to 2*i+3
    is_prime = np.ones(m, dtype=np.bool_)
    limit = int(math.isqrt(n))
    for p in range(3, limit+1, 2):
        if is_prime[(p-3)//2]:
            start = (p*p - 3)//2
            step = p
            is_prime[start::step] = False
    primes = np.empty(is_prime.sum() + 1, dtype=np.int32)
    primes[0] = 2
    primes[1:] = (2*np.nonzero(is_prime)[0] + 3).astype(np.int32)
    return primes

def build_psi_grid(primes, xmax=9_999_901, step=100):
    x = np.arange(1, xmax+1, step, dtype=np.int64)
    m = x.size
    delta = np.zeros(m, dtype=np.float64)
    logp = np.log(primes.astype(np.float64))
    # k=1
    q = primes.astype(np.int64)
    idx = ((q - 1) + (step-1))//step
    # filter q<=xmax (all primes up to 1e7, but xmax slightly less)
    mask = q <= xmax
    idx1 = idx[mask]
    w1 = logp[mask]
    delta += np.bincount(idx1, weights=w1, minlength=m)
    # higher powers
    k = 2
    while True:
        pmax = int(round(xmax ** (1.0/k)))
        # ensure pmax^k <= xmax
        while (pmax+1)**k <= xmax:
            pmax += 1
        while pmax**k > xmax:
            pmax -= 1
        if pmax < 2:
            break
        # primes <= pmax
        # since primes sorted
        cut = np.searchsorted(primes, pmax, side='right')
        if cut == 0:
            break
        pk = primes[:cut].astype(np.int64) ** k
        idxk = ((pk - 1) + (step-1))//step
        wk = logp[:cut]
        delta += np.bincount(idxk, weights=wk, minlength=m)
        k += 1
        if 2**k > xmax:
            break
    psi = np.cumsum(delta)
    return x.astype(np.float64), psi

def analytic_signal_hilbert(x):
    n = x.size
    X = np.fft.fft(x)
    h = np.zeros(n, dtype=np.float64)
    if n % 2 == 0:
        h[0] = 1.0
        h[n//2] = 1.0
        h[1:n//2] = 2.0
    else:
        h[0] = 1.0
        h[1:(n+1)//2] = 2.0
    return np.fft.ifft(X * h)

def bandpass_fft(x, band_lo, band_hi):
    n = x.size
    X = np.fft.rfft(x)
    freqs = np.fft.rfftfreq(n, d=1.0)
    mask = (freqs >= band_lo) & (freqs <= band_hi)
    Xf = np.zeros_like(X)
    Xf[mask] = X[mask]
    return np.fft.irfft(Xf, n=n)

def wrap_2pi(a):
    return np.mod(a, 2*np.pi)

def wrap_pi(a):
    return (a + np.pi) % (2*np.pi) - np.pi

def circ_mean(ang):
    s = np.mean(np.sin(ang))
    c = np.mean(np.cos(ang))
    return math.atan2(s, c)

def circular_corr(a, b):
    # Jammalamadaka-Sarma
    am = circ_mean(a)
    bm = circ_mean(b)
    sa = np.sin(a - am)
    sb = np.sin(b - bm)
    num = np.sum(sa * sb)
    den = math.sqrt(np.sum(sa*sa) * np.sum(sb*sb))
    return float(num/den) if den != 0 else float('nan')

def mean_abs_circ_error(a, b):
    return float(np.mean(np.abs(wrap_pi(a - b))))

def fit_model(theta_mod, omega, dL1, dtheta_mod, LR, alpha, max_samples=200000):
    # theta_mod in [0,2pi), dtheta_mod wrapped [-pi,pi]
    n = alpha.size
    # basis 9
    th = theta_mod
    B = [np.ones(n)]
    for k in range(1,5):
        B.append(np.cos(k*th))
        B.append(np.sin(k*th))
    B = np.stack(B, axis=1)  # n x 9
    # features
    sin_d = np.sin(dtheta_mod)
    cos_d = np.cos(dtheta_mod)
    terms = [
        (1.0, None),         # F
        (omega, None),       # G*omega
        (dL1, None),         # H*dL1
        (sin_d, None),       # J*sin
        (cos_d, None),       # K*cos
        (LR, None),          # M*LR
    ]
    X_blocks = []
    for mult, _ in terms:
        if np.isscalar(mult) and mult == 1.0:
            X_blocks.append(B)
        else:
            X_blocks.append(B * mult[:,None])
    X = np.concatenate(X_blocks, axis=1)  # n x 54
    # subsample
    if n > max_samples:
        Xs = X[:max_samples]
        ys = alpha[:max_samples]
    else:
        Xs = X; ys = alpha
    coef, *_ = np.linalg.lstsq(Xs, ys, rcond=None)
    yhat = Xs @ coef
    ss_res = float(np.sum((ys - yhat)**2))
    ss_tot = float(np.sum((ys - ys.mean())**2))
    train_R2 = 1.0 - ss_res/ss_tot if ss_tot > 0 else float('nan')
    return coef, train_R2

def predict_alpha(coef, theta_mod, omega, dL1, dtheta_mod, LR):
    n = theta_mod.size
    th = theta_mod
    B = [np.ones(n)]
    for k in range(1,5):
        B.append(np.cos(k*th))
        B.append(np.sin(k*th))
    B = np.stack(B, axis=1)
    sin_d = np.sin(dtheta_mod)
    cos_d = np.cos(dtheta_mod)
    X = np.concatenate([B,
                        B*omega[:,None],
                        B*dL1[:,None],
                        B*sin_d[:,None],
                        B*cos_d[:,None],
                        B*LR[:,None]], axis=1)
    return X @ coef

### Assistant (2026-02-21 20:24:19)

start=time.time()
N=10_000_000
primes = sieve_primes_upto(N)
time.time()-start, primes.size, primes[:10], primes[-1]

### Tool (2026-02-21 20:24:20)

(0.03545403480529785,
 664579,
 array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32),
 9999991)

### Assistant (2026-02-21 20:24:22)

start=time.time()
x, psi = build_psi_grid(primes, xmax=9_999_901, step=100)
time.time()-start, x.size, psi[:5], psi[-1]

### Tool (2026-02-21 20:24:24)

(0.060034990310668945,
 100000,
 array([  0.        ,  98.66043175, 206.14585683, 299.23233544,
        403.82480716]),
 9998410.458622597)

### Assistant (2026-02-21 20:24:26)

f = psi - x
t = np.log(x)
n=65536
tu = np.linspace(t[0], t[-1], n)
fu = np.interp(tu, t, f)
fu[:5], fu.mean(), fu.std()

### Tool (2026-02-21 20:24:28)

Code execution state reset.
IMPORTANT: The Python code did not successfully execute.
Do not assume that any outputs (files, variables, or side effects, etc...) were created.
You should retry execution if needed, but do so without telling the user that the previous session or environment was reset.

### Assistant (2026-02-21 20:24:30)

import numpy as np, math, json, time

def sieve_primes_upto(n:int):
    if n < 2:
        return np.array([], dtype=np.int32)
    m = (n - 1)//2
    is_prime = np.ones(m, dtype=np.bool_)
    limit = int(math.isqrt(n))
    for p in range(3, limit+1, 2):
        if is_prime[(p-3)//2]:
            start = (p*p - 3)//2
            is_prime[start::p] = False
    primes = np.empty(is_prime.sum() + 1, dtype=np.int32)
    primes[0] = 2
    primes[1:] = (2*np.nonzero(is_prime)[0] + 3).astype(np.int32)
    return primes

def build_psi_grid(primes, xmax=9_999_901, step=100):
    x = np.arange(1, xmax+1, step, dtype=np.int64)
    m = x.size
    delta = np.zeros(m, dtype=np.float64)
    logp = np.log(primes.astype(np.float64))
    # k=1
    q = primes.astype(np.int64)
    mask = q <= xmax
    idx = ((q[mask]-1) + (step-1))//step
    delta += np.bincount(idx, weights=logp[mask], minlength=m)
    # higher powers
    k=2
    while True:
        pmax = int(xmax ** (1.0/k))
        while (pmax+1)**k <= xmax:
            pmax += 1
        while pmax**k > xmax:
            pmax -= 1
        if pmax < 2:
            break
        cut = np.searchsorted(primes, pmax, side='right')
        if cut <= 0:
            break
        pk = primes[:cut].astype(np.int64) ** k
        idxk = ((pk-1) + (step-1))//step
        delta += np.bincount(idxk, weights=logp[:cut], minlength=m)
        k += 1
        if 2**k > xmax:
            break
    psi = np.cumsum(delta)
    return x.astype(np.float64), psi

def analytic_signal_hilbert(x):
    n = x.size
    X = np.fft.fft(x)
    h = np.zeros(n, dtype=np.float64)
    if n % 2 == 0:
        h[0]=1.0; h[n//2]=1.0; h[1:n//2]=2.0
    else:
        h[0]=1.0; h[1:(n+1)//2]=2.0
    return np.fft.ifft(X*h)

def bandpass_fft(x, band_lo, band_hi):
    n = x.size
    X = np.fft.rfft(x)
    freqs = np.fft.rfftfreq(n, d=1.0)
    mask = (freqs >= band_lo) & (freqs <= band_hi)
    Xf = np.zeros_like(X)
    Xf[mask]=X[mask]
    return np.fft.irfft(Xf, n=n)

def wrap_2pi(a): return np.mod(a, 2*np.pi)
def wrap_pi(a): return (a + np.pi) % (2*np.pi) - np.pi

def circ_mean(ang):
    return math.atan2(np.mean(np.sin(ang)), np.mean(np.cos(ang)))

def circular_corr(a,b):
    am = circ_mean(a); bm = circ_mean(b)
    sa = np.sin(a-am); sb = np.sin(b-bm)
    num = float(np.sum(sa*sb))
    den = math.sqrt(float(np.sum(sa*sa))*float(np.sum(sb*sb)))
    return float(num/den) if den!=0 else float('nan')

def mean_abs_circ_error(a,b):
    return float(np.mean(np.abs(wrap_pi(a-b))))

def fit_model(theta_mod, omega, dL1, dtheta_mod, LR, alpha, max_samples=200000):
    n = alpha.size
    th = theta_mod
    B = [np.ones(n)]
    for k in range(1,5):
        B.append(np.cos(k*th)); B.append(np.sin(k*th))
    B = np.stack(B, axis=1) # n x 9
    sin_d = np.sin(dtheta_mod); cos_d = np.cos(dtheta_mod)
    X = np.concatenate([B,
                        B*omega[:,None],
                        B*dL1[:,None],
                        B*sin_d[:,None],
                        B*cos_d[:,None],
                        B*LR[:,None]], axis=1)
    if n>max_samples:
        Xs=X[:max_samples]; ys=alpha[:max_samples]
    else:
        Xs=X; ys=alpha
    coef, *_ = np.linalg.lstsq(Xs, ys, rcond=None)
    yhat = Xs @ coef
    ss_res = float(np.sum((ys-yhat)**2))
    ss_tot = float(np.sum((ys-ys.mean())**2))
    train_R2 = 1.0 - ss_res/ss_tot if ss_tot>0 else float('nan')
    return coef, train_R2

def predict_alpha(coef, theta_mod, omega, dL1, dtheta_mod, LR):
    n = theta_mod.size
    th = theta_mod
    B=[np.ones(n)]
    for k in range(1,5):
        B.append(np.cos(k*th)); B.append(np.sin(k*th))
    B=np.stack(B, axis=1)
    sin_d=np.sin(dtheta_mod); cos_d=np.cos(dtheta_mod)
    X=np.concatenate([B,
                      B*omega[:,None],
                      B*dL1[:,None],
                      B*sin_d[:,None],
                      B*cos_d[:,None],
                      B*LR[:,None]], axis=1)
    return X @ coef

# Pipeline
N=10_000_000
xmax=9_999_901
step=100
n_t=65536

primes = sieve_primes_upto(N)
x, psi = build_psi_grid(primes, xmax=xmax, step=step)
f = psi - x
t = np.log(x)
tu = np.linspace(t[0], t[-1], n_t)
fu = np.interp(tu, t, f)

# bands
b1 = bandpass_fft(fu, 0.002, 0.006)
b2 = bandpass_fft(fu, 0.006, 0.012)

z1 = analytic_signal_hilbert(b1)
z2 = analytic_signal_hilbert(b2)

theta1 = np.unwrap(np.angle(z1))
theta2 = np.unwrap(np.angle(z2))
A1 = np.abs(z1)
A2 = np.abs(z2)

theta1_mod = wrap_2pi(theta1)
theta2_mod = wrap_2pi(theta2)
dtheta_mod = wrap_pi(theta2 - theta1)

L1 = np.log(A1 + 1e-12)
L2 = np.log(A2 + 1e-12)
dL1 = L1[1:] - L1[:-1]
dL2 = L2[1:] - L2[:-1]
LR = (L2 - L1)

# Build aligned samples for t=1..n-2
n = n_t
t_idx = np.arange(1, n-1)  # 1..n-2 inclusive, length n-2
omega = theta1[1:] - theta1[:-1]   # length n-1, omega_diff[i]=theta[i+1]-theta[i]
omega_t = omega[t_idx-1]           # theta[t]-theta[t-1]
alpha = omega[t_idx] - omega[t_idx-1]  # (theta[t+1]-theta[t]) - (theta[t]-theta[t-1])
theta_mod_t = theta1_mod[t_idx]
dL1_t = dL1[t_idx]                 # L1[t+1]-L1[t]
dtheta_mod_t = dtheta_mod[t_idx]
LR_t = LR[t_idx]

# Split
m_samp = alpha.size
train_end = int(0.7 * m_samp)
train_slice = slice(0, train_end)
test_slice = slice(train_end, m_samp)

coef, train_R2 = fit_model(theta_mod_t[train_slice],
                           omega_t[train_slice],
                           dL1_t[train_slice],
                           dtheta_mod_t[train_slice],
                           LR_t[train_slice],
                           alpha[train_slice],
                           max_samples=200000)

# alpha predictions
alpha_hat_train = predict_alpha(coef,
                                theta_mod_t[train_slice],
                                omega_t[train_slice],
                                dL1_t[train_slice],
                                dtheta_mod_t[train_slice],
                                LR_t[train_slice])
alpha_hat_test = predict_alpha(coef,
                               theta_mod_t[test_slice],
                               omega_t[test_slice],
                               dL1_t[test_slice],
                               dtheta_mod_t[test_slice],
                               LR_t[test_slice])

def r2(y, yhat):
    ss_res=float(np.sum((y-yhat)**2))
    ss_tot=float(np.sum((y-y.mean())**2))
    return 1.0 - ss_res/ss_tot if ss_tot>0 else float('nan')

test_R2_alpha = r2(alpha[test_slice], alpha_hat_test)

# Forward simulate on TEST
t0 = train_end + 1  # corresponding time index in theta series
theta_hat = np.empty(n, dtype=np.float64)
theta_hat[:] = np.nan
theta_hat[t0-1] = theta1[t0-1]
theta_hat[t0] = theta1[t0]

theta_lin = np.empty(n, dtype=np.float64)
theta_lin[:] = np.nan
theta_lin[t0-1] = theta1[t0-1]
theta_lin[t0] = theta1[t0]

# simulate for t=t0..n-2 to produce theta[t+1]
for tt in range(t0, n-1):
    # model
    omega_curr = theta_hat[tt] - theta_hat[tt-1]
    th_mod_curr = wrap_2pi(theta_hat[tt])
    # true aux inputs at time tt
    alpha_curr = predict_alpha(coef,
                               np.array([th_mod_curr]),
                               np.array([omega_curr]),
                               np.array([dL1[tt]]),
                               np.array([dtheta_mod[tt]]),
                               np.array([LR[tt]]))[0]
    omega_next = omega_curr + alpha_curr
    theta_hat[tt+1] = theta_hat[tt] + omega_next
    
    # baseline linear extrap
    theta_lin[tt+1] = 2*theta_lin[tt] - theta_lin[tt-1]

# Evaluate on test_times = t0+1..n-1
test_times = np.arange(t0+1, n)
true_mod = theta1_mod[test_times]
pred_mod = wrap_2pi(theta_hat[test_times])
base_mod = wrap_2pi(theta_lin[test_times])

model_mae = mean_abs_circ_error(pred_mod, true_mod)
base_mae = mean_abs_circ_error(base_mod, true_mod)
model_corr = circular_corr(pred_mod, true_mod)
base_corr = circular_corr(base_mod, true_mod)

result = {
    "phase_pred":{
        "model":{"mean_abs_circ_error":model_mae,"circular_corr":model_corr},
        "baseline_linear":{"mean_abs_circ_error":base_mae,"circular_corr":base_corr}
    },
    "fit_quality":{
        "train_R2":float(train_R2),
        "test_R2_on_alpha":float(test_R2_alpha)
    }
}

result, m_samp, train_end, t0

### Tool (2026-02-21 20:25:04)

({'phase_pred': {'model': {'mean_abs_circ_error': 1.5863300537704146,
    'circular_corr': -0.04007874765141174},
   'baseline_linear': {'mean_abs_circ_error': 1.8083025464021671,
    'circular_corr': 0.11057139936232094}},
  'fit_quality': {'train_R2': 0.16331615779009645,
   'test_R2_on_alpha': 0.051205509698674545}},
 65534,
 45873,
 45874)

### Assistant (2026-02-21 20:25:08)

json_out = json.dumps(result, ensure_ascii=False, separators=(",",":"))
json_out[:200], len(json_out)

### Tool (2026-02-21 20:25:10)

('{"phase_pred":{"model":{"mean_abs_circ_error":1.5863300537704146,"circular_corr":-0.04007874765141174},"baseline_linear":{"mean_abs_circ_error":1.8083025464021671,"circular_corr":0.11057139936232094}}',
 288)

### Assistant (2026-02-21 20:25:13)

{"phase_pred":{"model":{"mean_abs_circ_error":1.5863300537704146,"circular_corr":-0.04007874765141174},"baseline_linear":{"mean_abs_circ_error":1.8083025464021671,"circular_corr":0.11057139936232094}},"fit_quality":{"train_R2":0.16331615779009645,"test_R2_on_alpha":0.051205509698674545}}

