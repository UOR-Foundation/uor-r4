# Phase predictability test

- **Conversation ID:** `699a481a-3d88-8325-8e62-c18362969a96`
- **Date:** 2026-02-21

---

### User (2026-02-21 19:04:54)

You are a stateless compute worker.

You MUST write:
phase_dynamics.json

Do NOT print JSON.

Goal: Determine whether the low-frequency Hilbert phase θ(t) of ψ(x)−x has forward predictability (temporal persistence).

⸻

Build phase
	1.	Interpolate ψ(x)−x onto uniform log grid t_uniform (65536).
	2.	Bandpass filter frequencies in [0.002, 0.01] cycles/sample.
	3.	Compute analytic signal via Hilbert transform.
	4.	θ(t)=unwrap(angle(A(t))).

⸻

Predictability test

For lag L in:
{50,100,200,400,800,1600}

Predict future phase using linear extrapolation:
\hat{\theta}(t+L)=2\theta(t)-\theta(t-L)

Compute circular error:
err=\text{angle\_diff}(\hat{\theta}(t+L),\theta(t+L))

For each L compute:
	•	mean absolute circular error
	•	circular correlation between predicted and true phase

⸻

Output

Write phase_dynamics.json containing:
{
“lags”:[…],
“mean_abs_error”:[…],
“circular_corr”:[…]
}

No plots. No extra files.

### Assistant (2026-02-21 19:04:56)

import numpy as np, json, math, os, pathlib, statistics
from numpy.fft import fft, ifft, fftfreq

path="/mnt/data/chebyshev_psi.npz"
data=np.load(path)
x_grid=data["x_grid"]
psi_minus_x=data["psi_minus_x"]
t=np.log(x_grid.astype(float))
# ensure increasing
order=np.argsort(t)
t=t[order]
psi_minus_x=psi_minus_x[order]

n=65536
t_uniform=np.linspace(t.min(), t.max(), n)
f_u=np.interp(t_uniform, t, psi_minus_x)

# Bandpass in cycles/sample [0.002, 0.01]
fhat=fft(f_u)
freq=fftfreq(n, d=1.0)  # cycles/sample
mask=(np.abs(freq)>=0.002) & (np.abs(freq)<=0.01)
fhat_bp=np.where(mask, fhat, 0)
f_bp=np.real(ifft(fhat_bp))

# Analytic signal via FFT Hilbert
X=fft(f_bp)
h=np.zeros(n)
if n%2==0:
    h[0]=1
    h[n//2]=1
    h[1:n//2]=2
else:
    h[0]=1
    h[1:(n+1)//2]=2
A=ifft(X*h)
theta=np.unwrap(np.angle(A))

def angle_diff(a,b):
    # return wrapped difference a-b in (-pi,pi]
    d=a-b
    return (d + np.pi)%(2*np.pi)-np.pi

def circ_mean(ang):
    s=np.mean(np.sin(ang))
    c=np.mean(np.cos(ang))
    return math.atan2(s,c)

def circ_corr(alpha,beta):
    # Jammalamadaka-Sarma circular correlation
    am=circ_mean(alpha)
    bm=circ_mean(beta)
    sa=np.sin(alpha-am)
    sb=np.sin(beta-bm)
    num=np.sum(sa*sb)
    den=np.sqrt(np.sum(sa*sa)*np.sum(sb*sb))
    return float(num/den) if den>0 else float("nan")

lags=[50,100,200,400,800,1600]
mean_abs=[]
corrs=[]
for L in lags:
    i0=L
    i1=n-L-1  # inclusive? use i where i+L < n and i-L >=0
    idx=np.arange(i0, n-L)
    pred=2*theta[idx]-theta[idx-L]
    true=theta[idx+L]
    diff=angle_diff(pred,true)
    mean_abs.append(float(np.mean(np.abs(diff))))
    corrs.append(circ_corr(np.mod(pred,2*np.pi), np.mod(true,2*np.pi)))

out={"lags":lags,"mean_abs_error":mean_abs,"circular_corr":corrs}
out_path="/mnt/data/phase_dynamics.json"
with open(out_path,"w") as f:
    json.dump(out,f,indent=2)
out_path, out

### Tool (2026-02-21 19:05:26)

('/mnt/data/phase_dynamics.json',
 {'lags': [50, 100, 200, 400, 800, 1600],
  'mean_abs_error': [0.5280034743938374,
   0.7346202182206523,
   0.9305263990700454,
   0.8263149223551634,
   1.031290711077717,
   0.9411773629715716],
  'circular_corr': [-0.3385240555388136,
   0.5875327238470369,
   -0.3030691969400448,
   0.5144301627039001,
   0.2840700764372085,
   0.11881310195922501]})

### Assistant (2026-02-21 19:05:28)

Done. Your fragile little phase has been tested for whether it remembers its own past.

I wrote the required file (and did **not** print JSON to stdout).

[Download phase_dynamics.json](sandbox:/mnt/data/phase_dynamics.json)

