# Logarithmic Spectral Analysis

- **Conversation ID:** `69991a99-bd64-8333-acc2-cb84568e59bc`
- **Date:** 2026-02-20

---

### User (2026-02-20 21:38:36)

You are a stateless compute worker.
You will receive a file upload named:

chebyshev_psi.npz

This file contains arrays:
	•	x_grid (integers)
	•	psi_minus_x (float64)

This is the ground-truth arithmetic observable:

f(x) = ψ(x) − x

built from primes and prime powers up to N = 10,000,000, sampled every 100 integers.

Your job is to perform a logarithmic spectral analysis of f(x).

⸻

Step 1 — Prepare the signal
	1.	Load x_grid and psi_minus_x
	2.	Remove the mean of psi_minus_x
	3.	Define:
t = log(x_grid)  (natural log)

We now treat f as a function f(t).

⸻

Step 2 — Interpolate onto a uniform log grid

The FFT requires uniform spacing.
	1.	Create a new grid t_uniform with 65,536 equally spaced points between:
min(t) and max(t)
	2.	Interpolate f(t) onto t_uniform using cubic interpolation.

Call the interpolated signal f_uniform.

⸻

Step 3 — Windowing

Apply a Hann window to f_uniform before FFT to reduce spectral leakage.

⸻

Step 4 — FFT

Compute the real FFT of f_uniform.

Let:
	•	freq = FFT frequency axis in cycles per unit t
	•	power = squared magnitude of FFT coefficients

Convert frequencies to angular frequencies:

gamma_est = 2π * freq

These gamma_est values correspond to candidate Riemann zero imaginary parts.

⸻

Step 5 — Detect spectral peaks

Find the 25 largest local maxima in the power spectrum (ignore the DC component).

Return a table of:

rank, gamma_est, relative_power

(sorted by power).

⸻

Output files (STRICT)

Return exactly two files:
	1.	psi_log_spectrum.json
containing:
	•	number of grid points
	•	t range
	•	the table of the 25 strongest frequencies
	2.	psi_log_spectrum.png
A single plot:
power vs gamma_est (0 ≤ γ ≤ 80)
with peaks marked.

DO NOT create any other files.

### Assistant (2026-02-20 21:38:38)

import numpy as np
import json
import matplotlib.pyplot as plt

from pathlib import Path

# Try to import scipy components; fall back gracefully if missing
try:
    from scipy.interpolate import CubicSpline
    from scipy.signal import find_peaks
    _has_scipy = True
except Exception:
    _has_scipy = False

data_path = Path("/mnt/data/chebyshev_psi.npz")
out_json = Path("/mnt/data/psi_log_spectrum.json")
out_png = Path("/mnt/data/psi_log_spectrum.png")

# -----------------------
# Step 1 — Load & prepare
# -----------------------
with np.load(data_path) as npz:
    x_grid = npz["x_grid"]
    psi_minus_x = npz["psi_minus_x"].astype(np.float64)

x_grid = x_grid.astype(np.float64)

# Remove mean
f = psi_minus_x - np.mean(psi_minus_x)

# Define t = log(x)
t = np.log(x_grid)

# Ensure monotonic t for interpolation
order = np.argsort(t)
t = t[order]
f = f[order]

# ------------------------------
# Step 2 — Uniform log grid (N)
# ------------------------------
N_uniform = 65536
t_min, t_max = float(t.min()), float(t.max())
t_uniform = np.linspace(t_min, t_max, N_uniform)
dt = float(t_uniform[1] - t_uniform[0])

# Cubic interpolation onto uniform grid
if _has_scipy:
    cs = CubicSpline(t, f, bc_type="natural", extrapolate=False)
    f_uniform = cs(t_uniform)
else:
    # Fallback: linear interpolation (should be rare)
    f_uniform = np.interp(t_uniform, t, f)

# Handle any NaNs from extrapolate=False edges (shouldn't happen with matching range)
if np.isnan(f_uniform).any():
    # Replace NaNs by nearest valid values
    valid = ~np.isnan(f_uniform)
    f_uniform[~valid] = np.interp(np.flatnonzero(~valid), np.flatnonzero(valid), f_uniform[valid])

# -------------------
# Step 3 — Windowing
# -------------------
hann = np.hanning(N_uniform)
f_win = f_uniform * hann

# ----------
# Step 4 FFT
# ----------
F = np.fft.rfft(f_win)
freq = np.fft.rfftfreq(N_uniform, d=dt)  # cycles per unit t
power = (np.abs(F) ** 2)

gamma_est = 2 * np.pi * freq  # angular frequency

# -------------------------
# Step 5 — Peak detection
# -------------------------
# Ignore DC component (index 0)
power_no_dc = power.copy()
power_no_dc[0] = 0.0

if _has_scipy:
    peaks, props = find_peaks(power_no_dc)
else:
    # Minimal local maxima detector
    peaks = np.where((power_no_dc[1:-1] > power_no_dc[:-2]) & (power_no_dc[1:-1] > power_no_dc[2:]))[0] + 1

# Sort peaks by power descending
peak_powers = power_no_dc[peaks]
sort_idx = np.argsort(peak_powers)[::-1]
peaks_sorted = peaks[sort_idx]

top_k = 25
top_peaks = peaks_sorted[:top_k]
top_peak_powers = power_no_dc[top_peaks]

# Relative power normalized to strongest peak among selected peaks
max_peak_power = float(top_peak_powers.max()) if len(top_peak_powers) else 1.0
relative_power = (top_peak_powers / max_peak_power).astype(np.float64)

# Prepare table sorted by power (already)
table = []
for i, (idx, rp) in enumerate(zip(top_peaks, relative_power), start=1):
    table.append({
        "rank": int(i),
        "gamma_est": float(gamma_est[idx]),
        "relative_power": float(rp)
    })

# -------------------
# Output JSON (STRICT)
# -------------------
payload = {
    "n_grid_points": int(N_uniform),
    "t_range": {"min": t_min, "max": t_max},
    "top_25_peaks": table
}

out_json.write_text(json.dumps(payload, indent=2))

# -------------------
# Output plot (STRICT)
# -------------------
# Plot power vs gamma for 0 <= gamma <= 80
gamma_max_plot = 80.0
mask = (gamma_est >= 0) & (gamma_est <= gamma_max_plot)
g_plot = gamma_est[mask]
p_plot = power_no_dc[mask]

plt.figure(figsize=(10, 6))
plt.plot(g_plot, p_plot)
plt.xlabel("gamma_est")
plt.ylabel("power")
plt.title("Log-domain spectrum of f(x) = ψ(x) − x")

# Mark peaks within plotting range
top_peaks_in_range = [idx for idx in top_peaks if 0 <= gamma_est[idx] <= gamma_max_plot]
if top_peaks_in_range:
    plt.scatter(gamma_est[top_peaks_in_range], power_no_dc[top_peaks_in_range], zorder=3)
    # Annotate with rank
    # Build rank mapping for annotations
    rank_map = {int(row["rank"]): row["gamma_est"] for row in table}
    # Actually annotate by rank for those in range
    for row in table:
        g = row["gamma_est"]
        if 0 <= g <= gamma_max_plot:
            # find nearest index for y
            # Use the exact peak y:
            # Locate the matching peak index among top_peaks
            # (floating compare safe because gamma derived from index)
            # We'll just compute y by nearest in g_plot
            # But better: find idx by minimizing difference on top_peaks_in_range
            idx = min(top_peaks_in_range, key=lambda k: abs(gamma_est[k] - g))
            y = power_no_dc[idx]
            plt.annotate(str(row["rank"]), (g, y), textcoords="offset points", xytext=(5, 5), fontsize=8)

plt.tight_layout()
plt.savefig(out_png, dpi=200)
plt.close()

(out_json.as_posix(), out_png.as_posix(), _has_scipy, len(peaks), len(top_peaks_in_range))

### Tool (2026-02-20 21:39:07)

Matplotlib is building the font cache; this may take a moment.
('/mnt/data/psi_log_spectrum.json',
 '/mnt/data/psi_log_spectrum.png',
 True,
 3895,
 22)

### Assistant (2026-02-20 21:39:21)

Here you go. Two files. No extra junk, because apparently we’re being *strict* today.

- [Download psi_log_spectrum.json](sandbox:/mnt/data/psi_log_spectrum.json)  
- [Download psi_log_spectrum.png](sandbox:/mnt/data/psi_log_spectrum.png)

