# Phase model comparison

- **Conversation ID:** `69991e88-b0cc-8325-ac46-27f7fbff2717`
- **Date:** 2026-02-20

---

### User (2026-02-20 21:55:22)

You are a stateless compute worker.

You will receive one file:

chebyshev_psi.npz

It contains x_grid and psi_minus_x.

Define:

f(x) = ψ(x) − x

t = log(x_grid)

⸻

Step 1 — extract analytic phase

We want the instantaneous phase of the prime fluctuation signal.
	1.	Interpolate f(t) onto a uniform t grid (same method as before).
	2.	Compute the Hilbert transform to obtain analytic signal:
A(t) = f(t) + i·H(f(t))
	3.	Compute instantaneous phase:
φ(t) = unwrap(angle(A(t)))

Normalize φ to mean 0 and unit variance.

⸻

Step 2 — simple arithmetic phase model

Construct a naive arithmetic phase:

θ(n) = 2π·fractional_part(√n)

Evaluate θ at x_grid.

Convert to function of t.

Unwrap θ to a continuous phase.

⸻

Step 3 — compare phases

Compute:
	1.	Circular correlation between φ(t) and θ(t)
	2.	Linear correlation between sin(φ) and sin(θ)
	3.	Best phase offset (allow constant shift δ)

⸻

Output

Return exactly one file:

phase_model_comparison.json

containing:
	•	circular correlation
	•	sin-correlation
	•	optimal phase shift δ

Do not create plots.

### Assistant (2026-02-20 21:55:23)

import json, numpy as np, math
from pathlib import Path

path = Path("/mnt/data/chebyshev_psi.npz")
data = np.load(path)
x_grid = data["x_grid"]
psi_minus_x = data["psi_minus_x"].astype(float)

# t = log(x_grid)
t = np.log(x_grid.astype(float))

# --- Step 1: interpolate f(t) onto uniform grid ---
N_uniform = 65536
t_uniform = np.linspace(t.min(), t.max(), N_uniform)

# cubic interpolation (fall back to linear if scipy unavailable)
f = psi_minus_x.copy()
f = f - f.mean()

try:
    from scipy.interpolate import CubicSpline
    cs = CubicSpline(t, f, bc_type="natural")
    f_uniform = cs(t_uniform)
except Exception:
    f_uniform = np.interp(t_uniform, t, f)

# Hilbert transform analytic signal
try:
    from scipy.signal import hilbert
    analytic = hilbert(f_uniform)
except Exception as e:
    # Minimal FFT-based Hilbert as fallback
    n = f_uniform.size
    F = np.fft.fft(f_uniform)
    h = np.zeros(n)
    if n % 2 == 0:
        h[0] = 1
        h[n//2] = 1
        h[1:n//2] = 2
    else:
        h[0] = 1
        h[1:(n+1)//2] = 2
    analytic = np.fft.ifft(F * h)

phi = np.unwrap(np.angle(analytic))

# Normalize φ to mean 0 and unit variance (as requested)
phi_norm = (phi - phi.mean()) / (phi.std(ddof=0) if phi.std(ddof=0) != 0 else 1.0)

# --- Step 2: naive arithmetic phase model θ(n) = 2π frac(sqrt(n)) evaluated at x_grid ---
sqrt_x = np.sqrt(x_grid.astype(float))
frac = sqrt_x - np.floor(sqrt_x)
theta = 2*np.pi*frac  # in [0, 2π)

# Interpolate θ as complex unit phasor to uniform grid for stability
z = np.exp(1j * theta)
try:
    from scipy.interpolate import CubicSpline
    cs_re = CubicSpline(t, np.real(z), bc_type="natural")
    cs_im = CubicSpline(t, np.imag(z), bc_type="natural")
    z_uniform = cs_re(t_uniform) + 1j*cs_im(t_uniform)
except Exception:
    z_uniform = np.interp(t_uniform, t, np.real(z)) + 1j*np.interp(t_uniform, t, np.imag(z))

theta_uniform = np.unwrap(np.angle(z_uniform))

# --- helpers ---
def wrap_pi(a):
    return (a + np.pi) % (2*np.pi) - np.pi

# Circular correlation (Jammalamadaka-Sengupta)
def circular_corr(alpha, beta):
    alpha = wrap_pi(alpha)
    beta = wrap_pi(beta)
    # mean directions
    mu_a = np.angle(np.mean(np.exp(1j*alpha)))
    mu_b = np.angle(np.mean(np.exp(1j*beta)))
    sa = np.sin(alpha - mu_a)
    sb = np.sin(beta - mu_b)
    denom = np.sqrt(np.sum(sa**2) * np.sum(sb**2))
    if denom == 0:
        return float("nan")
    return float(np.sum(sa*sb) / denom)

# --- Step 3: compare phases ---
# Optimal constant phase shift δ (align theta to phi_norm) via circular mean of difference
diff = wrap_pi(phi_norm - theta_uniform)
delta = float(np.angle(np.mean(np.exp(1j*diff))))  # in (-π, π]

theta_shifted = theta_uniform + delta

circ = circular_corr(phi_norm, theta_shifted)

# Linear correlation between sin(φ) and sin(θ)
sin_phi = np.sin(phi_norm)
sin_theta = np.sin(theta_shifted)
# Pearson corr
sin_corr = float(np.corrcoef(sin_phi, sin_theta)[0,1])

out = {
    "circular_correlation": circ,
    "sin_correlation": sin_corr,
    "optimal_phase_shift_delta": delta
}

out_path = Path("/mnt/data/phase_model_comparison.json")
out_path.write_text(json.dumps(out, indent=2))
str(out_path), out

### Tool (2026-02-20 21:56:12)

('/mnt/data/phase_model_comparison.json',
 {'circular_correlation': 0.0004342771129214345,
  'sin_correlation': 0.14766230393250912,
  'optimal_phase_shift_delta': -0.23119239230243033})

### Assistant (2026-02-20 21:56:15)

Your precious phase comparison artifact has been produced.

[Download **phase_model_comparison.json**](sandbox:/mnt/data/phase_model_comparison.json)

