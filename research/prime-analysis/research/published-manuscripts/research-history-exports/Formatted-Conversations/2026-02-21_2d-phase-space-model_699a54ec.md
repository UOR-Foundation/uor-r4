# 2D Phase-Space Model

- **Conversation ID:** `699a54ec-f41c-8325-a773-bf90dc3e0597`
- **Date:** 2026-02-21

---

### User (2026-02-21 19:59:28)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Fit a 2D phase-space dynamics model using (θ, ω) and test forward predictability + prime prediction.

Recompute everything (no uploads):
1) primes up to N=10_000_000
2) ψ(x) on step-100 grid including prime powers
3) f(x)=ψ(x)−x
4) uniform t grid 65536
5) bandpass [0.002,0.01]
6) Hilbert analytic signal -> unwrap θ(t)
Define θ_mod = θ mod 2π
Define ω(t)=θ(t+1)-θ(t)
Define α(t)=ω(t+1)-ω(t)

Split time-ordered: 70% TRAIN, 30% TEST.

A) Fit dynamics on TRAIN
Fit model:
α ≈ F(θ_mod) + G(θ_mod) * ω

where F and G are Fourier series up to K=6:
F(θ)=cF + Σ_{k=1..6}(aF_k cos kθ + bF_k sin kθ)
G(θ)=cG + Σ_{k=1..6}(aG_k cos kθ + bG_k sin kθ)

Fit by least squares on up to 200000 TRAIN samples.

B) Forward simulate θ on TEST
Initialize at start of TEST with true θ_{t-1}, θ_t (so true ω_t).
Then recursively:
α_hat = F(θ_mod_t) + G(θ_mod_t)*ω_t
ω_{t+1} = ω_t + α_hat
θ_{t+1} = θ_t + ω_{t+1}

Compute on TEST:
- mean_abs_circ_error between predicted θ_mod and true θ_mod
- circular_corr

C) Prime residual prediction
Same as before:
Δ=0.01, compute residual R using primes and expected E.
Label y using TRAIN median residual.

Fit gating model g(θ) on TRAIN (Fourier logistic up to 3rd harmonic).
Evaluate on TEST:
1) using true θ_mod
2) using predicted θ_hat_mod

Report AUC and Brier for both.

Output JSON:
{
"fit":{"K":6, "F_coeffs":{...}, "G_coeffs":{...}},
"phase_pred":{"mean_abs_circ_error":..., "circular_corr":...},
"prime_pred":{"AUC_true_theta":..., "Brier_true_theta":..., "AUC_pred_theta":..., "Brier_pred_theta":...}
}

No other text.

### Assistant (2026-02-21 19:59:28)

import numpy as np, math, time, sys, os, statistics

### Assistant (2026-02-21 20:00:27)

def sieve_primes(n:int):
    # returns numpy array of primes <= n
    is_prime = np.ones(n+1, dtype=np.bool_)
    is_prime[:2] = False
    # evens
    is_prime[4::2] = False
    limit = int(n**0.5)
    for p in range(3, limit+1, 2):
        if is_prime[p]:
            step = p*2
            start = p*p
            is_prime[start::step] = False
    primes = np.concatenate(([2], np.nonzero(is_prime[3:])[0]*1 + 3))
    # Note: nonzero gives indices, offset 3
    return primes

t0=time.time()
N=10_000_000
primes = sieve_primes(N)
len(primes), primes[:10], primes[-1], time.time()-t0

### Tool (2026-02-21 20:00:31)

(664579,
 array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29]),
 9999991,
 0.04638242721557617)

### Assistant (2026-02-21 20:00:32)

def chebyshev_psi_grid(N:int, step:int, primes:np.ndarray):
    M = N//step  # max index for x = M*step
    diff = np.zeros(M+2, dtype=np.float64)
    ln = np.log
    sqrtN = int(N**0.5)
    for p in primes:
        lp = math.log(int(p))
        q = int(p)
        # iterate over prime powers
        while q <= N:
            start = (q + step - 1)//step
            diff[start] += lp
            if q > N//p:
                break
            q *= int(p)
    psi = np.cumsum(diff[:-1])
    x = np.arange(M+1, dtype=np.float64) * step
    return x, psi

t0=time.time()
x, psi = chebyshev_psi_grid(N, 100, primes)
psi.shape, x[-1], time.time()-t0, psi[-1]

### Tool (2026-02-21 20:00:37)

((100001,), 10000000.0, 0.3027322292327881, 9998539.403345881)

### Assistant (2026-02-21 20:00:40)

import importlib, numpy.fft as fft
has_scipy = importlib.util.find_spec("scipy") is not None
has_scipy

### Tool (2026-02-21 20:00:42)

True

### Assistant (2026-02-21 20:00:44)

from scipy.signal import hilbert

### Assistant (2026-02-21 20:00:47)

T=65536
f = psi - x
# resample to uniform t grid in x
x_t = np.linspace(0, N, T)
f_t = np.interp(x_t, x, f)

# bandpass [0.002,0.01] cycles/sample
freqs = np.fft.rfftfreq(T, d=1.0)
F = np.fft.rfft(f_t)
band = (freqs >= 0.002) & (freqs <= 0.01)
F_filt = np.zeros_like(F)
F_filt[band] = F[band]
f_bp = np.fft.irfft(F_filt, n=T)

analytic = hilbert(f_bp)
theta = np.unwrap(np.angle(analytic))
two_pi = 2*np.pi
theta_mod = np.mod(theta, two_pi)
omega = np.diff(theta)  # length T-1
alpha = np.diff(omega)  # length T-2
theta_mod_feat = theta_mod[:-2]
omega_feat = omega[:-1]  # wait omega length T-1, want first T-2
omega_feat = omega[:T-2]
alpha_target = alpha
len(alpha_target), len(theta_mod_feat), len(omega_feat)

### Tool (2026-02-21 20:00:52)

(65534, 65534, 65534)

### Assistant (2026-02-21 20:00:53)

K=6
def fourier_basis(theta_arr, K):
    # returns (n, 1+2K)
    n = theta_arr.shape[0]
    cols = [np.ones(n)]
    for k in range(1, K+1):
        cols.append(np.cos(k*theta_arr))
        cols.append(np.sin(k*theta_arr))
    return np.column_stack(cols)

n_samples = len(alpha_target)
train_end = int(0.7*n_samples)
# use up to 200000
train_n = min(200000, train_end)
theta_tr = theta_mod_feat[:train_n]
omega_tr = omega_feat[:train_n]
alpha_tr = alpha_target[:train_n]

B = fourier_basis(theta_tr, K)  # (train_n, 13)
X = np.hstack([B, B * omega_tr[:,None]])  # (train_n, 26)
coef, *_ = np.linalg.lstsq(X, alpha_tr, rcond=None)
coef.shape

### Tool (2026-02-21 20:00:58)

(26,)

### Assistant (2026-02-21 20:01:00)

coef_F = coef[:1+2*K]
coef_G = coef[1+2*K:]
coef_F, coef_G[:5]

### Tool (2026-02-21 20:01:01)

(array([ 0.00291475, -0.0040888 , -0.0018531 ,  0.00204992,  0.00526987,
         0.00077431, -0.00109905,  0.00168198, -0.00047453, -0.0027634 ,
         0.00483414,  0.0044583 , -0.00042797]),
 array([-0.11052808,  0.14869157,  0.06623296, -0.08054075, -0.18749451]))

### Assistant (2026-02-21 20:01:02)

def eval_fourier(theta, coef, K):
    # coef length 1+2K, ordering [c, cos1, sin1, cos2, sin2,...]
    res = coef[0] * np.ones_like(theta)
    idx=1
    for k in range(1, K+1):
        res += coef[idx]*np.cos(k*theta); idx+=1
        res += coef[idx]*np.sin(k*theta); idx+=1
    return res

def simulate(theta_true, t0, coef_F, coef_G, K):
    m = len(theta_true) - t0
    theta_hat = np.empty(m, dtype=np.float64)
    theta_hat[0] = theta_true[t0]
    omega_hat = theta_true[t0] - theta_true[t0-1]
    for i in range(1, m):
        th_mod = theta_hat[i-1] % (2*np.pi)
        # scalar eval
        # compute basis quickly:
        b = [1.0]
        for k in range(1, K+1):
            b.append(math.cos(k*th_mod))
            b.append(math.sin(k*th_mod))
        b = np.array(b)
        Fv = float(np.dot(coef_F, b))
        Gv = float(np.dot(coef_G, b))
        alpha_hat = Fv + Gv * omega_hat
        omega_hat = omega_hat + alpha_hat
        theta_hat[i] = theta_hat[i-1] + omega_hat
    return theta_hat

t0 = train_end  # start of test in feature index corresponding to theta index t0
theta_hat = simulate(theta, t0, coef_F, coef_G, K)
theta_true_test = theta[t0:]
theta_mod_true = np.mod(theta_true_test, two_pi)
theta_mod_hat = np.mod(theta_hat, two_pi)

# mean abs circular error
diff = np.angle(np.exp(1j*(theta_mod_hat - theta_mod_true)))
mean_abs_circ_error = float(np.mean(np.abs(diff)))

# circular correlation (Jammalamadaka-Sengupta)
def circ_mean(a):
    return math.atan2(np.mean(np.sin(a)), np.mean(np.cos(a)))
mu_hat = circ_mean(theta_mod_hat)
mu_true = circ_mean(theta_mod_true)
num = np.sum(np.sin(theta_mod_hat - mu_hat) * np.sin(theta_mod_true - mu_true))
den = math.sqrt(np.sum(np.sin(theta_mod_hat - mu_hat)**2) * np.sum(np.sin(theta_mod_true - mu_true)**2))
circular_corr = float(num/den) if den!=0 else float('nan')
mean_abs_circ_error, circular_corr, len(theta_mod_true)

### Tool (2026-02-21 20:01:13)

(1.5808416268767285, 0.03046002719700912, 19663)

### Assistant (2026-02-21 20:01:14)

Delta=0.01
nbins = int(math.ceil(two_pi/Delta))
# map prime -> t index
t_idx = ((primes.astype(np.float64) / N) * (T-1)).astype(np.int64)
# clip
t_idx = np.clip(t_idx, 0, T-1)
prime_theta = theta_mod[t_idx]
bin_idx = np.floor(prime_theta / Delta).astype(np.int64)
bin_idx = np.clip(bin_idx, 0, nbins-1)
counts = np.bincount(bin_idx, minlength=nbins).astype(np.float64)
E = counts.sum() * (Delta / two_pi)
# standardized residual
R_bins = (counts - E) / math.sqrt(E) if E>0 else np.zeros(nbins)
# assign residual to each t via its bin
t_bins_true = np.floor(theta_mod / Delta).astype(np.int64)
t_bins_true = np.clip(t_bins_true,0,nbins-1)
R_t = R_bins[t_bins_true]

# define train/test on time indices for gating: use same split on theta series length T
train_end_t = int(0.7*T)
R_train = R_t[:train_end_t]
median_R = float(np.median(R_train))
y = (R_t > median_R).astype(np.int8)
y_train = y[:train_end_t]
theta_train = theta_mod[:train_end_t]
theta_test_true = theta_mod[train_end_t:]
y_test = y[train_end_t:]
len(y_train), y_train.mean(), len(y_test)

### Tool (2026-02-21 20:01:25)

(45875, 0.4966757493188011, 19661)

### Assistant (2026-02-21 20:01:26)

K_log=3
def logistic_fit_fourier(theta_arr, y_arr, K, max_iter=50, tol=1e-8, ridge=1e-6):
    B = fourier_basis(theta_arr, K)  # (n, 1+2K)
    n,p = B.shape
    beta = np.zeros(p, dtype=np.float64)
    for it in range(max_iter):
        z = B @ beta
        # avoid overflow
        z = np.clip(z, -30, 30)
        p_hat = 1/(1+np.exp(-z))
        W = p_hat*(1-p_hat)
        # prevent zeros
        W = np.maximum(W, 1e-6)
        # IRLS update
        # compute weighted least squares: (B^T W B + ridge I) beta = B^T (W z + (y-p))
        # using vectorized with sqrtW
        sqrtW = np.sqrt(W)
        BW = B * sqrtW[:,None]
        y_work = z + (y_arr - p_hat)/W
        yW = y_work * sqrtW
        A = BW.T @ BW + ridge*np.eye(p)
        b = BW.T @ yW
        beta_new = np.linalg.solve(A, b)
        if np.max(np.abs(beta_new - beta)) < tol:
            beta = beta_new
            break
        beta = beta_new
    return beta

beta_gate = logistic_fit_fourier(theta_train, y_train, K_log)
beta_gate

### Tool (2026-02-21 20:01:33)

array([-0.02104827,  0.07545443, -0.2783541 ,  0.25479174,  0.16659565,
       -0.26181878, -0.0733668 ])

### Assistant (2026-02-21 20:01:35)

def sigmoid(z):
    z = np.clip(z, -30, 30)
    return 1/(1+np.exp(-z))

def predict_prob(theta_arr, beta, K):
    B = fourier_basis(theta_arr, K)
    return sigmoid(B @ beta)

p_true = predict_prob(theta_test_true, beta_gate, K_log)

# predicted theta for same indices
start_offset = train_end_t - t0
if start_offset < 0:
    # need simulate earlier; but should not
    start_offset = 0
theta_test_pred = theta_mod_hat[start_offset:]  # theta_hat_mod corresponds to indices t0..end
# ensure length matches y_test
theta_test_pred = theta_test_pred[:len(y_test)]
p_pred = predict_prob(theta_test_pred, beta_gate, K_log)

def auc_score(y_true, y_score):
    # Mann-Whitney U / rank method, handling ties by average ranks
    y_true = np.asarray(y_true)
    y_score = np.asarray(y_score)
    n = len(y_true)
    order = np.argsort(y_score)
    ranks = np.empty(n, dtype=np.float64)
    ranks[order] = np.arange(1, n+1)
    # tie correction: average ranks for ties
    sorted_scores = y_score[order]
    i=0
    while i<n:
        j=i+1
        while j<n and sorted_scores[j]==sorted_scores[i]:
            j+=1
        if j-i>1:
            avg = (i+1 + j)/2.0
            ranks[order[i:j]] = avg
        i=j
    pos = y_true==1
    n_pos = np.sum(pos)
    n_neg = n - n_pos
    if n_pos==0 or n_neg==0:
        return float('nan')
    sum_ranks_pos = np.sum(ranks[pos])
    U = sum_ranks_pos - n_pos*(n_pos+1)/2.0
    return float(U/(n_pos*n_neg))

def brier(y_true, p):
    y_true = np.asarray(y_true, dtype=np.float64)
    p = np.asarray(p, dtype=np.float64)
    return float(np.mean((p - y_true)**2))

auc_true = auc_score(y_test, p_true)
brier_true = brier(y_test, p_true)
auc_pred = auc_score(y_test, p_pred)
brier_pred = brier(y_test, p_pred)
auc_true, brier_true, auc_pred, brier_pred, len(y_test), len(p_pred)

### Tool (2026-02-21 20:01:49)

(0.6164077631715144,
 0.2408431297570687,
 0.5014794845254115,
 0.2845095861119034,
 19661,
 19661)

### Assistant (2026-02-21 20:01:52)

def coeffs_to_dict(coef, K, prefix):
    d = {f"c{prefix}": float(coef[0])}
    a=[]
    b=[]
    idx=1
    for k in range(1,K+1):
        a.append(float(coef[idx])); idx+=1
        b.append(float(coef[idx])); idx+=1
    d[f"a{prefix}"] = a
    d[f"b{prefix}"] = b
    return d

F_dict = coeffs_to_dict(coef_F, K, "F")
G_dict = coeffs_to_dict(coef_G, K, "G")
F_dict, list(G_dict.keys())

### Tool (2026-02-21 20:01:54)

({'cF': 0.0029147511343746617,
  'aF': [-0.004088798531196286,
   0.0020499213794158457,
   0.0007743121060525956,
   0.0016819837327725663,
   -0.0027633988395865724,
   0.004458300987332733],
  'bF': [-0.001853104532113444,
   0.005269870595815022,
   -0.0010990468789432613,
   -0.00047452561246179997,
   0.0048341396060132,
   -0.0004279675895105268]},
 ['cG', 'aG', 'bG'])

### Assistant (2026-02-21 20:01:57)

{"fit":{"K":6,"F_coeffs":{"c":0.0029147511343746617,"a":[-0.004088798531196286,0.0020499213794158457,0.0007743121060525956,0.0016819837327725663,-0.0027633988395865724,0.004458300987332733],"b":[-0.001853104532113444,0.005269870595815022,-0.0010990468789432613,-0.00047452561246179997,0.0048341396060132,-0.0004279675895105268]},"G_coeffs":{"c":-0.11052808125623671,"a":[0.14869156821136448,-0.08054075339024243,0.06966862820953188,0.12060779399663494,0.2796739097557285,0.19717171994515088],"b":[0.06623296050334706,-0.18749450586517607,-0.07034496842177957,-0.023071081518171245,-0.01829925881998358,-0.04899905045619507]}},"phase_pred":{"mean_abs_circ_error":1.5808416268767285,"circular_corr":0.03046002719700912},"prime_pred":{"AUC_true_theta":0.6164077631715144,"Brier_true_theta":0.2408431297570687,"AUC_pred_theta":0.5014794845254115,"Brier_pred_theta":0.2845095861119034}}

