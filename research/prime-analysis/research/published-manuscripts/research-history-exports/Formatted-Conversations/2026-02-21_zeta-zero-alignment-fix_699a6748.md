# Zeta-zero alignment fix

- **Conversation ID:** `699a6748-b0fc-8330-a734-1babfb026a9d`
- **Date:** 2026-02-21

---

### User (2026-02-21 21:17:47)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal: Fix the zeta-zero alignment controls (range bug) and re-test significance using spacing-normalized distances.

Recompute signal u(t):
- sieve primes up to N=10,000,000
- compute ψ(x) on step-100 grid including prime powers
- f(x)=ψ(x)-x
- t=log(x_grid), interpolate f onto uniform t grid length L=65536
- standardize u(t)
- Welch PSD (Hann win=4096, 50% overlap)
- extract top M=120 peaks in f in [0.002,0.05] cycles/sample, min separation 2 bins
- convert each peak f_k to gamma_est = 2*pi*f_k/dt

Let gamma_min = min(gamma_est), gamma_max = max(gamma_est).

Download true zeta zeros up to at least gamma_max + 200 from a reputable source (Odlyzko table).
Parse all gamma_true <= gamma_max + 200.

For each gamma_est:
- d = min_j |gamma_est - gamma_true[j]|
- find nearest true zero index j*
- local_spacing = (gamma_true[j*+1]-gamma_true[j*-1])/2  (handle edges safely)
- nd = d / local_spacing

Compute summary for REAL:
nd_mean, nd_median, d_mean, d_median.

Controls (FIXED):
1) Random-gamma control (range-matched):
For each rep (200 reps), sample M values gamma_rand uniform on [gamma_min, gamma_max].
Compute nd_mean_rand for that rep using same gamma_true list and same nd definition.
Report mean/std over reps and p-value for REAL nd_mean being smaller.

2) Phase-randomized surrogate (50 reps):
Randomize FFT phases of u(t) (keep magnitudes), inverse FFT -> u_surr.
Repeat peak extraction -> gamma_est_surr, compute nd_mean_surr.
Report mean/std and p-value for REAL nd_mean being smaller.

Output JSON:
{
 "grid":{"dt":...},
 "real":{"M":...,"gamma_min":...,"gamma_max":...,"d_mean":...,"d_median":...,"nd_mean":...,"nd_median":...},
 "zeros":{"num_zeros":...,"gamma_max_used":...},
 "control_random_gamma":{"reps":200,"nd_mean_mean":...,"nd_mean_std":...,"p_value":...},
 "control_phase_surr":{"reps":50,"nd_mean_mean":...,"nd_mean_std":...,"p_value":...}
}

No other text.

### Assistant (2026-02-21 21:17:48)

{"search_query":[{"q":"Odlyzko table zeta zeros imaginary parts first 100000 plain text","recency":30},{"q":"Odlyzko zeta zeros 0 to 1000 file","recency":3650}],"response_length":"short"}

### Assistant (2026-02-21 21:18:21)

{"open":[{"ref_id":"turn0search0"}],"response_length":"short"}

### Assistant (2026-02-21 21:18:22)

{"click":[{"ref_id":"turn1view0","id":0}],"response_length":"short"}

### Assistant (2026-02-21 21:18:25)

{"open":[{"ref_id":"turn2view0","lineno":0}],"response_length":"long"}

### Assistant (2026-02-21 21:18:30)

{"open":[{"ref_id":"turn0search4"}],"response_length":"short"}

### Assistant (2026-02-21 21:18:39)

{"click":[{"ref_id":"turn4view0","id":70}],"response_length":"short"}

### Assistant (2026-02-21 21:18:41)

{"find":[{"ref_id":"turn4view0","pattern":"plain text"},{"ref_id":"turn4view0","pattern":"download"}],"response_length":"short"}

### Assistant (2026-02-21 21:18:43)

{"open":[{"ref_id":"turn4view0","lineno":64}],"response_length":"short"}

### Assistant (2026-02-21 21:18:47)

{"click":[{"ref_id":"turn1view0","id":1}],"response_length":"short"}

### Assistant (2026-02-21 21:18:56)

{"open":[{"ref_id":"turn0search12"}],"response_length":"short"}

### Assistant (2026-02-21 21:18:58)

{"click":[{"ref_id":"turn9view0","id":9}],"response_length":"short"}

### Assistant (2026-02-21 21:19:00)

{"click":[{"ref_id":"turn9view0","id":10}],"response_length":"short"}

### Assistant (2026-02-21 21:19:03)

import numpy as np, math, statistics, time, sys, os, json, random

### Assistant (2026-02-21 21:19:12)

import numpy as np, math
from math import log

### Assistant (2026-02-21 21:19:15)

def sieve_primes(n:int):
    sieve = np.ones(n+1, dtype=bool)
    sieve[:2]=False
    r = int(n**0.5)
    for p in range(2, r+1):
        if sieve[p]:
            sieve[p*p:n+1:p]=False
    return np.nonzero(sieve)[0]

t0=time.time()
N=10_000_000
primes = sieve_primes(N)
len(primes), primes[:5], primes[-1], time.time()-t0

### Tool (2026-02-21 21:19:21)

(664579, array([ 2,  3,  5,  7, 11]), 9999991, 0.05114555358886719)

### Assistant (2026-02-21 21:19:23)

t0=time.time()
contrib = np.zeros(N+1, dtype=np.float32)
contrib[primes] = np.log(primes).astype(np.float32)
sqrtN = int(N**0.5)
small_primes = primes[primes<=sqrtN]
for p in small_primes:
    lp = float(math.log(int(p)))
    q = int(p)*int(p)
    while q<=N:
        contrib[q] += lp
        q *= int(p)
psi_full = np.cumsum(contrib, dtype=np.float64)
time.time()-t0, psi_full[-1]

### Tool (2026-02-21 21:19:26)

(0.18525123596191406, 9998539.403653145)

### Assistant (2026-02-21 21:19:28)

x_grid = np.arange(1, N+1, 100, dtype=np.int32)
psi_grid = psi_full[x_grid]
f_grid = psi_grid - x_grid.astype(np.float64)
t_grid = np.log(x_grid.astype(np.float64))
L=65536
t_uniform = np.linspace(t_grid[0], t_grid[-1], L, dtype=np.float64)
dt = float(t_uniform[1]-t_uniform[0])
u = np.interp(t_uniform, t_grid, f_grid).astype(np.float64)
u = (u - u.mean())/u.std()
dt, u.mean(), u.std(), u.shape

### Tool (2026-02-21 21:19:31)

(0.00024594622340595577, 1.8214596497756474e-17, 1.0, (65536,))

### Assistant (2026-02-21 21:19:35)

import numpy as np
from scipy.signal import welch, get_window, find_peaks

### Assistant (2026-02-21 21:19:38)

def extract_gamma_peaks(signal, dt, M=120, fmin=0.002, fmax=0.05, nperseg=4096, noverlap=2048, min_sep_bins=2):
    freqs, psd = welch(signal, fs=1.0, window='hann', nperseg=nperseg, noverlap=noverlap, detrend=False, scaling='density', return_onesided=True)
    mask = (freqs>=fmin) & (freqs<=fmax)
    fseg = freqs[mask]
    pseg = psd[mask]
    # peaks with minimum separation in bins
    peaks, props = find_peaks(pseg, distance=min_sep_bins)
    if peaks.size == 0:
        return np.array([]), freqs, psd
    # take top by height
    peak_powers = pseg[peaks]
    order = np.argsort(peak_powers)[::-1]
    peaks_sorted = peaks[order][:M]
    # sort by power descending (already), but return gamma_est aligned
    f_peaks = fseg[peaks_sorted]
    p_peaks = peak_powers[order][:M]
    gamma = 2*math.pi*f_peaks/dt
    return gamma.astype(np.float64), f_peaks.astype(np.float64), p_peaks.astype(np.float64), freqs, psd

gamma_est, f_peaks, p_peaks, freqs_full, psd_full = extract_gamma_peaks(u, dt, M=120)
gamma_est[:5], len(gamma_est), gamma_est.min(), gamma_est.max()

### Tool (2026-02-21 21:19:46)

(array([ 68.60763476, 112.2670387 ,  99.79292329, 124.74115411,
        168.40055805]),
 52,
 68.60763476286597,
 1259.885656554448)

### Assistant (2026-02-21 21:19:48)

import mpmath as mp, time, math, numpy as np

### Assistant (2026-02-21 21:19:52)

gamma_min=float(gamma_est.min())
gamma_max=float(gamma_est.max())
gamma_max_used=gamma_max+200
mp.mp.dps = 50
t0=time.time()
zeros=[]
n=1
while True:
    z = mp.zetazero(n)
    g = float(mp.im(z))
    zeros.append(g)
    if g>=gamma_max_used:
        break
    n += 1
    if n>5000:
        break
len(zeros), zeros[-1], time.time()-t0

### Tool (2026-02-21 21:19:55)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[9], line 9
      7 n=1
      8 while True:
----> 9     z = mp.zetazero(n)
     10     g = float(mp.im(z))
     11     zeros.append(g)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zetazeros.py:421, in zetazero(ctx, n, info, round)
    419         pattern = pattern_construct(ctx,block,T,V)
    420     prec = max(wpinitial, wpz)
--> 421     t = separate_my_zero(ctx, my_zero_number, zero_number_block,T,V,prec)
    422     v = ctx.mpc(0.5,t)
    423 finally:

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zetazeros.py:162, in separate_my_zero(ctx, my_zero_number, zero_number_block, T, V, prec)
    160     precs = [precs[0] // 2 +3+2*index] + precs
    161 ctx.prec = precs[0] + guard
--> 162 r = ctx.findroot(lambda x:ctx.siegelz(x), (t0,t1), solver ='illinois', verbose=False)
    163 #print "first step at", ctx.dps, "digits"
    164 z=ctx.mpc(0.5,r)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/calculus/optimization.py:969, in findroot(ctx, f, x0, solver, tol, verbose, verify, **kwargs)
    967     maxsteps = iterations.maxsteps
    968 i = 0
--> 969 for x, error in iterations:
    970     if verbose:
    971         print('x:    ', x)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/calculus/optimization.py:407, in Illinois.__iter__(self)
    405 a = self.a
    406 b = self.b
--> 407 fa = f(a)
    408 fb = f(b)
    409 m = None

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zetazeros.py:162, in separate_my_zero.<locals>.<lambda>(x)
    160     precs = [precs[0] // 2 +3+2*index] + precs
    161 ctx.prec = precs[0] + guard
--> 162 r = ctx.findroot(lambda x:ctx.siegelz(x), (t0,t1), solver ='illinois', verbose=False)
    163 #print "first step at", ctx.dps, "digits"
    164 z=ctx.mpc(0.5,r)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/ctx_mp_python.py:1023, in PythonMPContext._wrap_specfun.<locals>.f_wrapped(ctx, *args, **kwargs)
   1021 try:
   1022     ctx.prec += 10
-> 1023     retval = f(ctx, *args, **kwargs)
   1024 finally:
   1025     ctx.prec = prec

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zeta.py:103, in siegelz(ctx, t, **kwargs)
    101 ctx.prec += 21
    102 e1 = ctx.expj(ctx.siegeltheta(t))
--> 103 z = ctx.zeta(0.5+ctx.j*t)
    104 if d == 0:
    105     v = e1*z

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zeta.py:580, in zeta(ctx, s, a, derivative, method, **kwargs)
    578 if ctx.re(s) > 2*ctx.prec and a == 1 and not derivative:
    579     return ctx.one + ctx.power(2, -s)
--> 580 return +ctx._hurwitz(s, a, d, **kwargs)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zeta.py:604, in _hurwitz(ctx, s, a, d, **kwargs)
    602 while 1:
    603     ctx.prec = prec + extraprec
--> 604     T1, T2 = _hurwitz_em(ctx, s, a, d, prec+10, verbose)
    605     cancellation = ctx.mag(T1) - ctx.mag(T1+T2)
    606     if verbose:

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/functions/zeta.py:712, in _hurwitz_em(ctx, s, a, d, prec, verbose)
    710     U = Un
    711     r *= rM2a
--> 712 t = ctx.fdot(U, logs) * r * ctx.bernoulli(j2)/(-fact)
    713 tailsum += t
    714 if ctx.mag(t) < tol:

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/ctx_mp_python.py:974, in PythonMPContext.fdot(ctx, A, B, conjugate)
    972 s = mpf_sum(real, prec, rnd)
    973 if imag:
--> 974     s = ctx.make_mpc((s, mpf_sum(imag, prec, rnd)))
    975 else:
    976     s = ctx.make_mpf(s)

File /opt/pyvenv/lib/python3.11/site-packages/mpmath/ctx_mp_python.py:602, in PythonMPContext.make_mpc(ctx, v)
    599     a._mpf_ = v
    600     return a
--> 602 def make_mpc(ctx, v):
    603     a = new(ctx.mpc)
    604     a._mpc_ = v

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-21 21:20:57)

{"open":[{"ref_id":"turn4view0","lineno":68}],"response_length":"long"}

### Assistant (2026-02-21 21:21:05)

{"click":[{"ref_id":"turn4view0","id":36}],"response_length":"short"}

### Assistant (2026-02-21 21:21:11)

{"search_query":[{"q":"zeros6 file 2001052 zeros zeta function plain text download","recency":3650},{"q":"database_odlyzko_zeta zeros6 raw github","recency":3650},{"q":"first 2000 zeros of riemann zeta function text file","recency":3650}],"response_length":"short"}

### Assistant (2026-02-21 21:21:15)

{"open":[{"ref_id":"turn14search1"}],"response_length":"short"}

### Assistant (2026-02-21 21:21:17)

{"open":[{"ref_id":"https://raw.githubusercontent.com/sagemath/sage/develop/src/sage/databases/odlyzko.py"}],"response_length":"short"}

### Tool (2026-02-21 21:21:34)

url='https://www-users.cse.umn.edu/~odlyzko/zeta_tables/zeros1'
content_length='1800000 bytes'
response.status=200
content_type='text/plain'
content_encoding='utf-8'
derived_content_type='text/plain'
Saved file to /mnt/data/zeros1.txt
SUCCESS

### Assistant (2026-02-21 21:21:34)

import numpy as np, re, math, json, random, statistics, time, os, pathlib
zeros_text = open('/mnt/data/zeros1.txt','r',encoding='utf-8',errors='ignore').read()
len(zeros_text), zeros_text[:100]

### Tool (2026-02-21 21:21:36)

(1800000,
 '     14.134725142\n     21.022039639\n     25.010857580\n     30.424876126\n     32.935061588\n     37.58')

### Assistant (2026-02-21 21:21:39)

gamma_true_all = np.array([float(s) for s in zeros_text.split()], dtype=np.float64)
gamma_true_all[:5], gamma_true_all.shape, gamma_true_all[-1]

### Tool (2026-02-21 21:21:41)

(array([14.13472514, 21.02203964, 25.01085758, 30.42487613, 32.93506159]),
 (100000,),
 74920.827498994)

### Assistant (2026-02-21 21:21:43)

gamma_limit = gamma_max + 200.0
gamma_true = gamma_true_all[gamma_true_all <= gamma_limit]
len(gamma_true), gamma_true[-1], gamma_limit

### Tool (2026-02-21 21:21:45)

(1034, 1458.393777369, 1459.885656554448)

### Assistant (2026-02-21 21:21:46)

def nd_stats(gamma_est, gamma_true):
    gt = gamma_true
    # for each gamma_est find insertion index
    idx = np.searchsorted(gt, gamma_est)
    # nearest index among idx-1 and idx
    j = np.clip(idx, 0, len(gt)-1)
    j0 = np.clip(idx-1, 0, len(gt)-1)
    # choose nearer
    d0 = np.abs(gamma_est - gt[j0])
    d1 = np.abs(gamma_est - gt[j])
    jstar = np.where(d0<=d1, j0, j)
    d = np.minimum(d0,d1)
    # local spacing
    jm1 = np.clip(jstar-1, 0, len(gt)-1)
    jp1 = np.clip(jstar+1, 0, len(gt)-1)
    # if edge, use forward/backward spacing
    spacing = np.empty_like(d)
    for i, js in enumerate(jstar):
        if js==0:
            spacing[i] = gt[1]-gt[0]
        elif js==len(gt)-1:
            spacing[i] = gt[-1]-gt[-2]
        else:
            spacing[i] = (gt[js+1]-gt[js-1])/2.0
    nd = d/spacing
    return d, nd

d_real, nd_real = nd_stats(gamma_est, gamma_true)
real_summary = {
    "M": int(len(gamma_est)),
    "gamma_min": float(gamma_min),
    "gamma_max": float(gamma_max),
    "d_mean": float(d_real.mean()),
    "d_median": float(np.median(d_real)),
    "nd_mean": float(nd_real.mean()),
    "nd_median": float(np.median(nd_real))
}
real_summary

### Tool (2026-02-21 21:21:53)

{'M': 52,
 'gamma_min': 68.60763476286597,
 'gamma_max': 1259.885656554448,
 'd_mean': 0.42324884964424325,
 'd_median': 0.39113462870713533,
 'nd_mean': 0.2813244641268938,
 'nd_median': 0.2732804172574424}

### Assistant (2026-02-21 21:21:55)

rng = np.random.default_rng(12345)
def nd_mean_for_gamma_list(glist, gamma_true):
    _, nd = nd_stats(np.asarray(glist, dtype=np.float64), gamma_true)
    return float(nd.mean())

reps_rg=200
nd_means_rg = np.empty(reps_rg, dtype=np.float64)
for r in range(reps_rg):
    gamma_rand = rng.uniform(gamma_min, gamma_max, size=len(gamma_est))
    nd_means_rg[r] = nd_mean_for_gamma_list(gamma_rand, gamma_true)
rg_mean = float(nd_means_rg.mean())
rg_std = float(nd_means_rg.std(ddof=1))
p_rg = float((np.sum(nd_means_rg <= real_summary["nd_mean"]) + 1) / (reps_rg + 1))
rg_mean, rg_std, p_rg, nd_means_rg.min(), nd_means_rg.max()

### Tool (2026-02-21 21:21:59)

(0.2751824708542268,
 0.025343369593594227,
 0.6218905472636815,
 0.20083298310217065,
 0.35817021182386816)

### Assistant (2026-02-21 21:22:01)

def phase_randomize_real(x, rng):
    X = np.fft.rfft(x)
    mag = np.abs(X)
    phase = np.angle(X)
    n = X.shape[0]
    # random phases for bins 1..n-2 (exclude DC and Nyquist if present)
    rand_phase = rng.uniform(0, 2*np.pi, size=n)
    rand_phase[0]=0.0
    if n>1 and (len(x)%2==0):
        # Nyquist bin is last
        rand_phase[-1]=0.0
    Xr = mag * np.exp(1j*rand_phase)
    xr = np.fft.irfft(Xr, n=len(x))
    return xr

def nd_mean_from_signal(sig, dt, gamma_true):
    sig = (sig - sig.mean())/sig.std()
    gamma_s, *_ = extract_gamma_peaks(sig, dt, M=120)
    if len(gamma_s)==0:
        return float('nan')
    d, nd = nd_stats(gamma_s, gamma_true)
    return float(nd.mean())

reps_surr=50
nd_means_surr=[]
t0=time.time()
for r in range(reps_surr):
    us = phase_randomize_real(u, rng)
    ndm = nd_mean_from_signal(us, dt, gamma_true)
    nd_means_surr.append(ndm)
time.time()-t0, np.isnan(nd_means_surr).sum(), (min(nd_means_surr), max(nd_means_surr), float(np.mean(nd_means_surr)), float(np.std(nd_means_surr, ddof=1)))

### Tool (2026-02-21 21:22:12)

(0.2803363800048828,
 0,
 (0.2783757457536953,
  0.3515860963916437,
  0.31595896002495405,
  0.013975193430664381))

### Assistant (2026-02-21 21:22:14)

nd_means_surr = np.array(nd_means_surr, dtype=np.float64)
p_surr = float((np.sum(nd_means_surr <= real_summary["nd_mean"]) + 1) / (reps_surr + 1))
float(nd_means_surr.mean()), float(nd_means_surr.std(ddof=1)), p_surr

### Tool (2026-02-21 21:22:19)

(0.31595896002495405, 0.013975193430664381, 0.0392156862745098)

### Assistant (2026-02-21 21:22:20)

output = {
 "grid":{"dt": dt},
 "real": {
     "M": int(len(gamma_est)),
     "gamma_min": float(gamma_min),
     "gamma_max": float(gamma_max),
     "d_mean": float(d_real.mean()),
     "d_median": float(np.median(d_real)),
     "nd_mean": float(nd_real.mean()),
     "nd_median": float(np.median(nd_real)),
 },
 "zeros": {
     "num_zeros": int(len(gamma_true)),
     "gamma_max_used": float(gamma_true[-1]) if len(gamma_true)>0 else None
 },
 "control_random_gamma":{
     "reps": reps_rg,
     "nd_mean_mean": rg_mean,
     "nd_mean_std": rg_std,
     "p_value": p_rg
 },
 "control_phase_surr":{
     "reps": reps_surr,
     "nd_mean_mean": float(nd_means_surr.mean()),
     "nd_mean_std": float(nd_means_surr.std(ddof=1)),
     "p_value": p_surr
 }
}
json_text = json.dumps(output, separators=(',',':'))
len(json_text), json_text[:200]

### Tool (2026-02-21 21:22:30)

(559,
 '{"grid":{"dt":0.00024594622340595577},"real":{"M":52,"gamma_min":68.60763476286597,"gamma_max":1259.885656554448,"d_mean":0.42324884964424325,"d_median":0.39113462870713533,"nd_mean":0.281324464126893')

### Assistant (2026-02-21 21:22:32)

{"grid":{"dt":0.00024594622340595577},"real":{"M":52,"gamma_min":68.60763476286597,"gamma_max":1259.885656554448,"d_mean":0.42324884964424325,"d_median":0.39113462870713533,"nd_mean":0.2813244641268938,"nd_median":0.2732804172574424},"zeros":{"num_zeros":1034,"gamma_max_used":1458.393777369},"control_random_gamma":{"reps":200,"nd_mean_mean":0.2751824708542268,"nd_mean_std":0.025343369593594227,"p_value":0.6218905472636815},"control_phase_surr":{"reps":50,"nd_mean_mean":0.31595896002495405,"nd_mean_std":0.013975193430664381,"p_value":0.0392156862745098}}

