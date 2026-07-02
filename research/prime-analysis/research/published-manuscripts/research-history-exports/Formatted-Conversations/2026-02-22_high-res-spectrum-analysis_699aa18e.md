# High-Res Spectrum Analysis

- **Conversation ID:** `699aa18e-0ff4-8330-9520-3bca359dedad`
- **Date:** 2026-02-22

---

### User (2026-02-22 01:26:26)

You are a stateless compute worker.
PRINT JSON ONLY.
No files.
Seed = 12345.

Goal

Determine whether the “peak ladder / phase bands” persist when we remove Welch bin-quantization by using high-resolution spectral estimation, and whether ψ differs from Cox null and log-factorial under that higher resolution.

0) Fixed params
	•	N_max = 10_000_000
	•	x_grid_step = 100 on x = 1,101,…,9_999_901
	•	uniform log grid length L = 65536
	•	compute dt from uniform t grid and report it
	•	detrend: causal moving average window W = 2000 on the interpolated uniform-t signal
	•	standardize each signal (mean 0, std 1) after detrend
	•	spectrum scan range: cycles/sample f ∈ [0.0, 0.05]

1) Build the same three signals on the same uniform t grid

(A) ψ residual
	•	sieve primes up to N_max
	•	compute ψ(x) including prime powers on step-100 grid
	•	fψ(x)=ψ(x)-x
	•	interpolate to uniform t grid length L
	•	detrend W=2000 (causal MA), standardize
Call this u_psi.

(B) log-factorial residual
	•	S(x)=gammaln(floor(x)+1)
	•	fL(x)=S(x) - x*log(x) + x
	•	interpolate to same uniform t grid length L
	•	detrend W=2000, standardize
Call this u_logfact.

(C) Cox null (inhomogeneous Poisson on x with λ=1/log x)
	•	Implement per-step Poisson counts on each [x, x+step) with mean ≈ step / log(midpoint), then points uniform in interval.
	•	No prime powers.
	•	Build ψ̃(x) on same grid by adding log(p) at each simulated “prime” p.
	•	fC(x)=ψ̃(x)-x
	•	interpolate to uniform t grid, detrend W=2000, standardize
Call this u_cox.

Report counts: number of events in Cox; number of primes in real.

2) High-resolution spectrum methods (do BOTH)

Method 1: Full-length FFT periodogram

For each u(t):
	•	compute FFT of full length L (no Welch)
	•	compute one-sided power spectrum
	•	search local maxima in f ∈ [0.0, 0.05]
	•	extract top M=60 local maxima by power, minimum separation = 4 FFT bins
Return each peak: {f, power_rel} (relative to max peak for that signal)

Method 2: Welch with larger segments (reduces bin quantization)

Repeat Welch peak extraction for each signal with:
	•	nperseg = 32768
	•	noverlap = 16384
	•	Hann window
Extract top M=60 local maxima, min separation 2 bins (Welch bins).
Return {f, power_rel}.

3) Quantization diagnostics (the whole point)

For each method and each signal, compute:
	•	bin_spacing (FFT: 1/L; Welch: 1/nperseg)
	•	For each extracted peak frequency f, compute f / bin_spacing
	•	Report how close each is to an integer:
	•	mean_abs_frac = mean(|frac(f/bin_spacing) - 0| or |frac - 1|) i.e. distance to nearest integer
	•	max_abs_frac
If peaks are “locked” to bins, these numbers will be near 0.

4) Band test near 0.008

For each signal and method:
	•	consider band f ∈ [0.0075, 0.0085]
	•	report strongest peak in band if any: {has_peak, f, power_rel}

5) Output JSON

{
  "params": {...},
  "counts": {"num_primes_real":..., "num_events_cox":...},
  "dt": ...,
  "spectra": {
    "fft": {"psi":[...], "logfact":[...], "cox":[...]},
    "welch32768": {"psi":[...], "logfact":[...], "cox":[...]}
  },
  "quantization": {
    "fft": {"bin_spacing":..., "psi":{...}, "logfact":{...}, "cox":{...}},
    "welch32768": {"bin_spacing":..., "psi":{...}, "logfact":{...}, "cox":{...}}
  },
  "band_0p008": {
    "fft": {...},
    "welch32768": {...}
  }
}

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file0 in your response to cite this file, or to surface it as a link.
# Prime Phase / Chebyshev Psi Spectral Program — Handoff Pack

This folder is a self-contained handoff for the prime/ψ(x) spectral-phase research line.

## Goal
Empirically test whether arithmetic observables derived from primes (primarily f(x)=ψ(x)-x) contain recoverable spectral/phase structure related to zeta-zero frequencies, and whether phase variables predict *forward* prime-density residual activity out-of-sample.

## Canonical data (N ≤ 10,000,000)
- `primes.npz` — primes array + is_prime mask
- `chebyshev_psi.npz` — x_grid (step=100), ψ(x) including prime powers, and ψ(x)-x
- `pi.npz` — π(x) on same grid
- `gaps.npz` — consecutive prime gaps

## Major artifacts
- `wheel_baseline.json` (+ plots) — primorial wheel baseline; residue-class density nearly uniform, but gap distribution depends on residue.
- `psi_log_spectrum.json` (+ png) — spectral peaks of f(t)=ψ(e^t)-e^t on uniform log grid.
- `hilbert_phase_prediction.json` / `hilbert_phase_gating_model.json` / `hilbert_robustness_sweep.json` — Hilbert-phase based prime residual prediction sweeps.
- `gamma_permutation_control.json` — control experiment around true zeta-zero phase construction.
- `spectral_phase_trainfit_control.json` — showed strong phase-sign accuracy vs controls in one design.
- `forward_phase_prime_prediction_fixed.json` — strict no-leakage forward prediction with warnings.
- `project_handoff_state.json` — **authoritative state**: purpose, history, key results, gotchas, and ranked next steps.

## Worker protocol (stateless)
Assume every compute worker has zero context.
Default: **PRINT JSON ONLY. No files.**
Give full pipeline steps in every prompt.

## Latest key diagnosis
The corrected zeta-alignment experiment shows REAL peak-gammas are **not better than** uniform random gammas in the same range (p≈0.62), though they beat phase-randomized surrogates (p≈0.039). So: peak extraction is real, but “zeta-uniqueness” is not isolated by that test alone.

## Recommended next step
Redesign the prediction target to reduce label stickiness:
- e.g., y = sign(R_{t+h} - R_{t}) or y = 1 if R_{t+h} is in the top decile (TRAIN-defined)
Then re-test phase predictors vs persistence baselines.

---
Generated: 2026-02-22T02:25:45.879630+00:00

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file1 in your response to cite this file, or to surface it as a link.
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

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file2 in your response to cite this file, or to surface it as a link.
{
  "snapshot_name": "Prime Phase Gating Project Snapshot",
  "date": "2026-02-21",
  "scope": "Empirical evidence that local prime-density residuals in multiplicative intervals are modulated by a latent log-scale phase extracted from \u03c8(x)\u2212x; the phase exhibits a stable low-frequency mode in log(x).",
  "core_objects": {
    "chebyshev_residual": "f(x)=\u03c8(x)\u2212x built from primes and prime powers",
    "log_time": "t=log(x)",
    "multiplicative_window": "[x, x\u00b7exp(\u0394)) with \u0394 typically 0.01",
    "expected_prime_count": "E(x)\u2248x\u00b7(exp(\u0394)\u22121)/log(x)",
    "normalized_residual": "R(x)=(P(x)\u2212E(x))/sqrt(E(x))",
    "label": "y=1 if R(x)>median(R_train) else 0",
    "phase_extractor": "Hilbert analytic phase of detrended/bandpassed f(t) on a uniform t grid"
  },
  "key_findings": [
    {
      "finding": "Residue-class densities for small primorial moduli are near-uniform; gap distributions carry more congruence information than raw residue densities.",
      "evidence": "wheel_baseline.json (mutual information increases with M; density near-uniform for M=30,210)."
    },
    {
      "finding": "Prime-square neighborhoods show no strong 'flip' in simple local metrics after multiple testing correction (within N\u22641e7, windows \u00b11000/\u00b15000).",
      "evidence": "prime_square_summary.json (BH-corrected q\u22480.23 at best; effects small)."
    },
    {
      "finding": "Using true zeta-zero-peak gammas as phase coordinates yields above-chance forward prediction of prime-density residual, but gamma-permutation control shows this is not unique to the true gammas.",
      "evidence": {
        "gamma_permutation_control": {
          "AUC_true": 0.6195743804481356,
          "mean_perm_auc": 0.6197846046041416,
          "std_perm_auc": 0.017507382290900653,
          "max_perm_auc": 0.641118930503994
        }
      }
    },
    {
      "finding": "Direct Hilbert phase of the detrended \u03c8-residual gives strong out-of-sample prediction of future prime-density residual in multiplicative windows.",
      "evidence": {
        "hilbert_phase_prediction": {
          "AUC_test": 0.7434718095365757,
          "accuracy_test": 0.687921297139734,
          "Brier_test": 0.20573705402050957
        }
      }
    },
    {
      "finding": "Phase\u2192probability gating is well-captured by a low-order circular model (Fourier logistic up to 3rd harmonic), with performance \u2248 binning baseline.",
      "evidence": {
        "hilbert_phase_gating_model": {
          "AUC_test": 0.79521069374096,
          "accuracy_test": 0.748792024820711,
          "Brier_test": 0.18166773529911875,
          "coeffs": [
            0.05227898460310678,
            -1.2410509359752318,
            -0.9389255380287798,
            0.004377384932597117,
            -0.05651392225339423,
            0.030567718077650873,
            0.005433456745270413
          ],
          "bin12_AUC": 0.7942247257800745
        }
      }
    },
    {
      "finding": "Robustness: high predictive performance persists across (W,\u0394), peaking near \u0394=0.01 and W\u2248500\u20131000.",
      "evidence": {
        "best_AUC_run": {
          "W": 1000,
          "Delta": 0.01,
          "AUC": 0.8500142458776063,
          "accuracy": 0.7829205025176745,
          "Brier": 0.15674233189255302
        },
        "best_Brier_run": {
          "W": 500,
          "Delta": 0.01,
          "AUC": 0.8469358880767717,
          "accuracy": 0.7875998168963939,
          "Brier": 0.1560112655751158
        }
      }
    },
    {
      "finding": "Bandpass filtering improves phase predictiveness; signal is concentrated in low frequencies of the \u03c8-residual in log-space.",
      "evidence": {
        "phase_extractor_comparison": {
          "baseline": {
            "AUC": 0.7577956362947762,
            "Brier": 0.19932771408184752
          },
          "bandpass_[0.002,0.01]": {
            "AUC": 0.7850460541223706,
            "Brier": 0.18693517955854674
          },
          "bandpass_[0.01,0.05]": {
            "AUC": 0.6695804223702395,
            "Brier": 0.22818212073559513
          }
        }
      }
    },
    {
      "finding": "A stable log-frequency mode exists in the phase-velocity spectrum near 0.007996 cycles/sample, stable across grid resolutions (N=32768/65536/131072).",
      "evidence": {
        "peak_frequency_cycles_per_sample": 0.0079957,
        "grid_stability": [
          {
            "N": 32768,
            "peak_frequency": 0.007995849482711264
          },
          {
            "N": 65536,
            "peak_frequency": 0.007995727473868924
          },
          {
            "N": 131072,
            "peak_frequency": 0.007995666470844046
          }
        ],
        "implied_period_samples": 125.06679389312978
      }
    }
  ],
  "current_hypothesis": {
    "statement": "In log-scale t=log(x), the Chebyshev residual f(t)=\u03c8(e^t)\u2212e^t contains a low-frequency oscillatory mode. A phase coordinate \u03b8(t) extracted from the analytic signal (Hilbert phase) modulates local prime intensity in multiplicative intervals: \u03bb(x)\u2248(1/log x)\u00b7g(\u03b8(log x)).",
    "model_class": "Phase-modulated Cox process / latent-state intensity model",
    "what_is_not_claimed": [
      "This does not prove RH.",
      "This does not provide an exact closed-form for primes.",
      "This does not imply literal periodicity of primes in x; it implies log-scale statistical modulation."
    ]
  },
  "open_questions_next": [
    "Nonlinearity test: does phase acceleration \u03b1 depend on phase \u03b8 (suggesting a nonlinear oscillator) or is it flat (linear/harmonic superposition)?",
    "Stability under different bandpass choices and detrend methods beyond moving-average (already partially tested).",
    "Cross-N validation: replicate the whole pipeline for larger N (e.g., 1e8) using public datasets to test scale persistence.",
    "Connect the detected dominant log-frequency to known low-lying zero contributions and/or to an emergent envelope phenomenon."
  ],
  "recommended_minimal_protocol": [
    "Compute \u03c8(x)\u2212x on a grid up to N (include prime powers).",
    "Convert to uniform t=log(x) grid; bandpass low frequencies [0.002,0.01] cycles/sample.",
    "Compute Hilbert phase \u03b8(t).",
    "Define multiplicative forward window \u0394\u22480.01; compute prime-count residual R=(P\u2212E)/sqrt(E).",
    "Fit circular gating model p(y=1|\u03b8) on first 70% of t; evaluate on last 30%."
  ]
}

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file3 in your response to cite this file, or to surface it as a link.
{
  "timestamp_utc": "2026-02-22T01:03:51.806126Z",
  "dataset": {
    "N": 10000000,
    "grid_step": 100,
    "pipeline": [
      "primes up to N",
      "Chebyshev psi with prime powers",
      "f(x)=psi(x)-x",
      "uniform log grid",
      "bandpass [0.002,0.01]",
      "Hilbert analytic signal",
      "unwrap theta",
      "omega=theta(t+1)-theta(t)",
      "alpha=omega(t+1)-omega(t)"
    ]
  },
  "key_findings": [
    "Phase theta predicts prime residual activity (AUC ~0.61-0.76 across tests).",
    "1D phase dynamics alpha=F(theta) fails to forward-predict theta.",
    "2D phase-space model alpha=F(theta)+G(theta)*omega also fails to forward-predict theta (near-random).",
    "Therefore theta is predictive but not Markovian in (theta, omega); implies hidden driver or higher-dimensional state."
  ],
  "latest_results": {
    "phase_space_model": {
      "K": 6,
      "phase_prediction": {
        "mean_abs_circ_error": 1.5808416268767285,
        "circular_corr": 0.03046002719700912
      },
      "prime_prediction": {
        "AUC_true_theta": 0.6164077631715144,
        "Brier_true_theta": 0.2408431297570687,
        "AUC_pred_theta": 0.5014794845254115,
        "Brier_pred_theta": 0.2845095861119034
      }
    }
  },
  "interpretation": {
    "status": "phase predictive but not forward-simulatable",
    "implication": "theta depends on hidden state beyond (theta, omega)",
    "next_direction": "introduce amplitude envelope or multi-frequency forcing state"
  }
}

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file4 in your response to cite this file, or to surface it as a link.
{
  "project_name": "Prime Phase / Chebyshev Psi Spectral Program",
  "date_created_utc": "2026-02-22T02:25:22.129859+00:00",
  "scope": {
    "goal": "Empirically probe whether arithmetic observables derived from primes (primarily f(x)=\u03c8(x)-x on log scale) contain recoverable spectral/phase structure related to zeta-zero frequencies, and whether phase variables predict forward prime-density residual activity out-of-sample.",
    "N_max": 10000000,
    "primary_observable": "Chebyshev psi including prime powers; f(x)=\u03c8(x)-x sampled on step-100 x-grid then interpolated to uniform log grid."
  },
  "core_datasets": {
    "primes_n": 664579,
    "files": {
      "primes.npz": "primes array (int32), is_prime boolean mask, N",
      "chebyshev_psi.npz": "x_grid (step=100), psi_grid, psi_minus_x",
      "pi.npz": "\u03c0(x) on same grid",
      "gaps.npz": "prime gaps sequence"
    }
  },
  "key_results_so_far": [
    {
      "topic": "Wheel / primorial baseline",
      "finding": "Prime counts across reduced residue classes modulo small primorials are extremely close to uniform (M=30,210); however, gap distribution carries measurable mutual information with residue class, increasing with modulus.",
      "artifact": "wheel_baseline.json"
    },
    {
      "topic": "Prime-square neighborhood test",
      "finding": "No robust evidence of a strong 'prime-square flip' effect in basic local metrics; slight deficit at w=1000 does not survive multiple-testing correction.",
      "artifacts": [
        "prime_square_summary.json",
        "prime_square_flip_test.parquet"
      ]
    },
    {
      "topic": "Dominant log-frequency / phase dynamics of bandpassed \u03c8 residual",
      "finding": "A stable peak around cycles/sample \u22480.0079957 shows up across grid sizes; Hilbert-phase acceleration \u03b1 exhibits phase dependence with multiple stable/unstable fixed points under harmonic fit (K up to 8).",
      "artifacts": [
        "psi_log_spectrum.json",
        "phase_persistence.json",
        "harmonic-fit JSON snippets pasted in chat (also summarized in snapshots)"
      ]
    },
    {
      "topic": "Forward prediction of prime-density residual via phase bins",
      "finding": "Phase-based binning models (Hilbert or reconstructed spectral phase) achieved modest out-of-sample AUC (~0.59\u20130.62) under strict no-leakage setups in some runs; however label stickiness is very high, so persistence baselines can dominate unless targets are de-biased.",
      "artifacts": [
        "hilbert_phase_prediction.json",
        "hilbert_phase_gating_model.json",
        "hilbert_robustness_sweep.json",
        "phase_extractor_comparison.json",
        "phase_vs_prime_density.json",
        "forward_phase_prime_prediction_fixed.json"
      ]
    },
    {
      "topic": "Zeta-zero alignment of peak-derived gammas (properly range-matched controls)",
      "finding": "Spacing-normalized nearest-zero distance for REAL peaks nd_mean\u22480.281 is NOT better than uniform random gamma in the same range (p\u22480.62). REAL is better than phase-randomized surrogates (p\u22480.039), suggesting peak extraction depends on time-domain structure beyond |FFT| alone, but does not isolate zeta-uniqueness beyond density effects.",
      "artifact_inline": {
        "grid": {
          "dt": 0.00024594622340595577
        },
        "real": {
          "M": 52,
          "gamma_min": 68.60763476286597,
          "gamma_max": 1259.885656554448,
          "d_mean": 0.42324884964424325,
          "d_median": 0.39113462870713533,
          "nd_mean": 0.2813244641268938,
          "nd_median": 0.2732804172574424
        },
        "zeros": {
          "num_zeros": 1034,
          "gamma_max_used": 1458.393777369
        },
        "control_random_gamma": {
          "reps": 200,
          "nd_mean_mean": 0.2751824708542268,
          "nd_mean_std": 0.025343369593594227,
          "p_value": 0.6218905472636815
        },
        "control_phase_surr": {
          "reps": 50,
          "nd_mean_mean": 0.31595896002495405,
          "nd_mean_std": 0.013975193430664381,
          "p_value": 0.0392156862745098
        }
      }
    }
  ],
  "interpretation": {
    "what_is_solid": [
      "We can robustly extract spectral peaks from f(t)=\u03c8(e^t)-e^t on a uniform log grid; a dominant peak near 0.008 cycles/sample is stable to grid size.",
      "Hilbert-phase acceleration \u03b1(\u03b8) is non-uniform, and harmonic regression yields multiple stable/unstable phase fixed points (suggestive of a driven nonlinear oscillator viewpoint)."
    ],
    "what_is_not_yet_proven": [
      "That extracted peak frequencies correspond specifically to true zeta zeros beyond what zero-density alone would predict.",
      "That phase variables provide predictive signal beyond trivial label persistence / long runs in the target."
    ],
    "most_likely_failure_mode": "Apparent zeta alignment can be an artifact of comparing to an incomplete zero list or of using a statistic that ignores density variation; after fixing controls, evidence for special alignment is weak."
  },
  "worker_protocol": {
    "rule": "Assume every worker is stateless. Provide full context, insist on PRINT JSON ONLY unless a file is absolutely necessary.",
    "default_pipeline": [
      "Load or reconstruct primes up to 1e7",
      "Compute \u03c8(x) on x-grid step=100 including prime powers",
      "Form f(x)=\u03c8(x)-x",
      "Convert t=log(x), interpolate to uniform t grid (typically L=65536)",
      "Standardize, then Welch PSD / FFT bandpass / Hilbert phase as needed",
      "Out-of-sample splits must be time-ordered; any thresholds computed on TRAIN only"
    ],
    "common_gotchas": [
      "Control ranges must match REAL ranges (gamma ranges, x ranges).",
      "If using AUC, ensure TEST has both classes; otherwise report properly and switch target definition.",
      "Beware label stickiness dominating results; compare against persistence baselines."
    ]
  },
  "next_steps_ranked": [
    {
      "priority": 1,
      "title": "Design a target that is NOT trivially persistent",
      "why": "Current y labels from forward windows show long run lengths; a persistence baseline can exceed learned models.",
      "ideas": [
        "Use residual increments: y = sign(R_{t+h}-R_t) or y = (R_{t+h} > R_t)",
        "Use z-scored residual after local detrending in t",
        "Predict rare events: y = 1 if R in top decile on TRAIN"
      ]
    },
    {
      "priority": 2,
      "title": "Spectral-statistics test in the Hilbert\u2013P\u00f3lya direction",
      "why": "If there's a genuine zeta operator signature, it should appear in spacing statistics (GUE) rather than raw peak matching.",
      "ideas": [
        "Unfold gamma_est using local density and compare nearest-neighbor spacing distribution to GUE vs Poisson",
        "Compare spectral rigidity \u03943 / number variance"
      ]
    },
    {
      "priority": 3,
      "title": "Two-band or multi-band phase state model",
      "why": "Earlier attempts suggest single-phase dynamics are hard to forecast; multi-band phases may recover latent state.",
      "ideas": [
        "Compute phases in two low-frequency bands and model \u0394\u03b8 dynamics; test whether predicts residual-change targets"
      ]
    }
  ],
  "file_manifest": [
    {
      "file": "Egap_M30.png",
      "bytes": 31627,
      "sha256": "136be413b4e9fd30b60a83430873f9ee6dcc289c04ce3a8d69fde49cc70585e9"
    },
    {
      "file": "Egap_M30030.png",
      "bytes": 161194,
      "sha256": "0b3a4829c536b4adff38dcfd78a9cdabd82dff2e23e9afda846117a87e4f20b5"
    },
    {
      "file": "MI_vs_M.png",
      "bytes": 37373,
      "sha256": "1c52ce226a355ec1f1df275b4281373710aea3359ae6387d45f2622b11669155"
    },
    {
      "file": "chebyshev_psi.npz",
      "bytes": 1401106,
      "sha256": "f221075b611d065c7e3560ae36b76b689649efe6eba762aaf093a527b025727e"
    },
    {
      "file": "density_M210.png",
      "bytes": 45028,
      "sha256": "c4f72c87cf5718312532e5d87179215eb9eeedee38acf0543ef6406fdd4d9369"
    },
    {
      "file": "density_M2310.png",
      "bytes": 47170,
      "sha256": "e0a4b6c2befaef77991cd77afb415ce26637a4d7f794ddd4e3e9c8b6191c8520"
    },
    {
      "file": "density_M30.png",
      "bytes": 45565,
      "sha256": "9ab29511543519f52be3f892521ee5d1ad0d9234c2c65d2341975bcf143ba5d3"
    },
    {
      "file": "density_M30030.png",
      "bytes": 49453,
      "sha256": "1b24599deae7809496ecd27b804473a003696b213a2ecb95ff8e3beb48058c63"
    },
    {
      "file": "effect_hist_gap_frac_ge40_w5000.png",
      "bytes": 35328,
      "sha256": "28671ebe9082187e8d5dab859427261e201d0b2c083b65a58821d5685449402d"
    },
    {
      "file": "effect_hist_gap_mean_w5000.png",
      "bytes": 33868,
      "sha256": "1a5e6795c73e5a785bad5a9016113c06e688de968cbc439d5dfe45fb5d28dbbf"
    },
    {
      "file": "effect_hist_gap_median_w5000.png",
      "bytes": 36066,
      "sha256": "78cf873f12be26ca0a0f26d948f3a6c043bb0d2bc60ff2a2f55ceb30f3ad100d"
    },
    {
      "file": "effect_hist_prime_count_w5000.png",
      "bytes": 30156,
      "sha256": "1701fc9c376f9862429939ed2097961a7ed65e363e8e535b3f3eb2d855581b6f"
    },
    {
      "file": "effect_hist_prime_density_w5000.png",
      "bytes": 32032,
      "sha256": "b60f877d339418d3a11ca64b61726dd1a666ff5d1b81c33d45226ace075b704f"
    },
    {
      "file": "forward_phase_prime_prediction.json",
      "bytes": 2990,
      "sha256": "9e9c5d8fc3e15480254440eae794f78b9b933316e89bdb99a817394cf2f506a0"
    },
    {
      "file": "forward_phase_prime_prediction_fixed.json",
      "bytes": 2866,
      "sha256": "c4624acb0b6b7055d9eb5dde8d56dd9ebc89506f2d3eaac5c5fd0a14c8abaa3d"
    },
    {
      "file": "gamma_permutation_control.json",
      "bytes": 557,
      "sha256": "75b9d28185963ed410eecb1e407fb8ba39afa8c6f47fae4f849a6c752f55853e"
    },
    {
      "file": "gamma_phase_k_sweep.json",
      "bytes": 24179,
      "sha256": "4c0d035c51a2e76da4e1d4adab3e429339e48bf422258d82133dce15772ee15b"
    },
    {
      "file": "gaps.npz",
      "bytes": 520271,
      "sha256": "bc7157fb4150051f612ef7ad329b5cdeb3c58bf84155e45df9849915463df252"
    },
    {
      "file": "hilbert_phase_gating_model.json",
      "bytes": 535,
      "sha256": "cffebed2014432cb7f42d3c6e54ad384c83c45324857172a50828dfeb0ffe8d6"
    },
    {
      "file": "hilbert_phase_prediction.json",
      "bytes": 190,
      "sha256": "3c8f010716f5cbac65b0464a9848fb7a0b4d0345d30ef86fe10546aec5c85231"
    },
    {
      "file": "hilbert_robustness_sweep.json",
      "bytes": 3599,
      "sha256": "a639a82f8f53c954cb1dc9559805917a7e06b11e283302a39b0f743d330e77b1"
    },
    {
      "file": "phase_bin_activity_classifier.json",
      "bytes": 529,
      "sha256": "48ffe61f62c83ce894d3cbfbfe65cbf55c050a50562f3f4acf7cbd0d611ab4b6"
    },
    {
      "file": "phase_bin_activity_classifier_fixed.json",
      "bytes": 877,
      "sha256": "2a84871e48312813461f12002e2e56616de9498a4cf803d040017ce7c972ae89"
    },
    {
      "file": "phase_bin_activity_residual_classifier.json",
      "bytes": 632,
      "sha256": "808a57b08962455b7773a7a9fffbb68b595be54d7c2efeefb7e38b2dfff17e8a"
    },
    {
      "file": "phase_extractor_comparison.json",
      "bytes": 1073,
      "sha256": "21ebb66b783b60251a472c2776328ca3effc068cc149e929316f06bb5b3739cb"
    },
    {
      "file": "phase_model_comparison.json",
      "bytes": 146,
      "sha256": "a44de7b1a72218bb1dc9af26a82de42baab0b62b6c1fdfd65e9ca32d375838d4"
    },
    {
      "file": "phase_persistence.json",
      "bytes": 183,
      "sha256": "016f9b405d10a00b652c76c11555df1e795543092c53c8fff986ee5ebbc234e0"
    },
    {
      "file": "phase_vs_activity.json",
      "bytes": 1484,
      "sha256": "e6fbb4a44746e2cd2d1559e32c4592f6b6d06ed81f401dfae86bba96622eb1d9"
    },
    {
      "file": "phase_vs_prime_density.json",
      "bytes": 488,
      "sha256": "38f191edfaaa4a21cc10a1745f412c9366361461fe49d2c7780f3f53ae64a696"
    },
    {
      "file": "pi.npz",
      "bytes": 308437,
      "sha256": "a68ed52af50ae0c0c55b0eb1020fb9d653b8d7ca6910df972c86319e612ab60a"
    },
    {
      "file": "prime_phase_research_snapshot.json",
      "bytes": 6577,
      "sha256": "2551b559831393f6ce21baa1c69e3a1b2c320b41283bdbb00d164aa8acff707a"
    },
    {
      "file": "prime_phase_research_snapshot_2.json",
      "bytes": 1498,
      "sha256": "1d426cb0e4937cb781aaf3d0429d8d0003bec7a9b695853356689d6bb6f9f207"
    },
    {
      "file": "prime_phase_research_snapshot_3.json",
      "bytes": 4756,
      "sha256": "32b25b23456034bd1adc98782e78f70a2784765fb553bcb28e7d9c90bdef943a"
    },
    {
      "file": "prime_square_flip_test.parquet",
      "bytes": 140080,
      "sha256": "bcd0bfd4aedc8241d2482d8e1fa0750d1e7aec94030fe5669a3ad32e96e45025"
    },
    {
      "file": "prime_square_summary.json",
      "bytes": 3194,
      "sha256": "40a478cd11b7218916c68c01a5161e46359378b93173fd9bd39000259e900e12"
    },
    {
      "file": "primes.npz",
      "bytes": 1531045,
      "sha256": "cba570db1f09db856aada958cb0ae78f6dd5d7b8ce0778f3ab1a7944f5c46494"
    },
    {
      "file": "psi_log_spectrum.json",
      "bytes": 2929,
      "sha256": "60c6264305ca618b00e3868903103d9b5c3f1e9fa187b1dd678aca5b3a13464e"
    },
    {
      "file": "psi_log_spectrum.png",
      "bytes": 122844,
      "sha256": "0e9cf967b5238ac756ace5f0454122213563215e3bc8fc9c8285c76dc4ca2283"
    },
    {
      "file": "spectral_phase_oos.json",
      "bytes": 966,
      "sha256": "cec9ffd7c0001d059dc407bc11594b60dae2b2b0fa7cdd12722b9d4e3d304558"
    },
    {
      "file": "spectral_phase_trainfit_control.json",
      "bytes": 1595,
      "sha256": "9293f7f1095ecfa626a0221cc1caabc666905ef2323bd3e824abc5243007dcb4"
    },
    {
      "file": "spectral_reconstruction.json",
      "bytes": 679,
      "sha256": "9ee5e582d36e5461af8fb409beddcad7121c33b36034a9962c1b015cec6f17b4"
    },
    {
      "file": "spectral_reconstruction.png",
      "bytes": 193935,
      "sha256": "f561bf84872c85d7b48c4b687bba98af11f8d12a4c41736f8f91b121abed4029"
    },
    {
      "file": "wheel_baseline.json",
      "bytes": 690123,
      "sha256": "b8f452a5a2ebd2120b6eb15eddf584b406503c8884c99d3ae8491a9fcb701b69"
    },
    {
      "file": "wheel_log_oos_validation.json",
      "bytes": 2727,
      "sha256": "39ee08c76ed2ca020a955cf25eee72cd57d28ff6b64103f3754946a3a2d31976"
    },
    {
      "file": "wheel_log_phase_fit.json",
      "bytes": 300,
      "sha256": "12479e569cfc4b53d9343ce7f5640588de16cb0daa2d4661cd05fa7a7628cc78"
    }
  ]
}

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file5 in your response to cite this file, or to surface it as a link.
{
  "date": "2026-02-21",
  "project": "Prime spectral-field / zeta-zero alignment via \u03c8(x)-x on log grid",
  "data_substrate": {
    "N": 10000000,
    "artifacts_available": [
      "primes.npz",
      "chebyshev_psi.npz",
      "pi.npz",
      "gaps.npz",
      "wheel_baseline.json",
      "prime_square_summary.json",
      "prime_square_flip_test.parquet",
      "psi_log_spectrum.json",
      "spectral_reconstruction.json",
      "phase_persistence.json",
      "phase_model_comparison.json",
      "wheel_log_phase_fit.json",
      "wheel_log_oos_validation.json",
      "spectral_phase_oos.json",
      "spectral_phase_trainfit_control.json",
      "gamma_phase_k_sweep.json",
      "phase_vs_activity.json",
      "phase_bin_activity_classifier.json",
      "phase_bin_activity_classifier_fixed.json",
      "phase_bin_activity_residual_classifier.json",
      "phase_vs_prime_density.json",
      "forward_phase_prime_prediction.json",
      "forward_phase_prime_prediction_fixed.json",
      "gamma_permutation_control.json",
      "hilbert_phase_prediction.json",
      "hilbert_phase_gating_model.json",
      "hilbert_robustness_sweep.json",
      "phase_extractor_comparison.json",
      "prime_phase_research_snapshot.json",
      "prime_phase_research_snapshot_2.json"
    ],
    "grid_note": "\u03c8 computed on x=1,101,...,9,999,901 (step=100); then interpolated onto uniform t=log(x) grid (typically L=65536)."
  },
  "latest_result_interpretation": {
    "input_json": {
      "dt": 0.00024594622340595577,
      "zeros_num": 340,
      "gamma_range_real": [
        51.45572607214948,
        399.1716931657657
      ],
      "nd_mean_real": 0.2774852420252272,
      "nd_mean_phase_surr": 0.2804817841090656,
      "p_value_phase_surr": 0.39215686274509803,
      "nd_mean_random_control_reported": 176.64004194554633,
      "p_value_random_reported": 0.004975124378109453
    },
    "diagnosis": "The random-control statistic is invalid because the control sampled frequencies in [0.002,0.05], which map to gamma up to ~1277 given dt, but the zero table was only downloaded up to ~599 (based on REAL peak gamma_max + 200). This pushes many control gammas out-of-range, exploding normalized distances and producing a misleadingly tiny p-value.",
    "meaning": "Real vs phase-randomized surrogate are statistically indistinguishable at the level of normalized nearest-zero distance. That suggests the observed alignment is largely explained by second-order structure (power spectrum / |FFT|) rather than phase-specific coupling."
  },
  "key_findings": [
    "Wheel/primorial residue classes are nearly uniform in prime density after conditioning on gcd(a,M)=1; major structure appears in gap distributions (MI rises with primorial M).",
    "Prime-square neighborhood test (\u22641e7) shows no robust local 'flip' after multiple-testing correction.",
    "\u03c8(x)-x in log-space shows strong low-frequency structure; regime labels are highly persistent at short horizons.",
    "Hilbert phase can correlate with current activity but learned phase dynamics have weak forward predictability out-of-sample.",
    "Spectral phase sign predictors using peak-derived gammas showed strong test sign accuracy vs random-frequency controls and persistence baselines in earlier runs.",
    "Phase-acceleration \u03b1(\u03b8) shows weak but structured phase dependence; harmonic fits produce stable/unstable fixed points."
  ],
  "next_steps": [
    {
      "name": "Fix zeta-alignment controls",
      "actions": [
        "Sample random controls in the SAME gamma range as REAL peaks (or same discrete frequency bins between min(top_f) and max(top_f)).",
        "Alternatively, download zeros up to gamma corresponding to f_max=0.05 (gamma_max\u22482\u03c0*0.05/dt) so controls are in-range.",
        "Report nd_mean for REAL, RANDOM, and PHASE-SURR with valid overlapping ranges."
      ]
    },
    {
      "name": "Zeta-specificity beyond density",
      "actions": [
        "Create density-matched fake zero sets (unfolded spacings randomized) and compare nd_mean against TRUE zeros.",
        "Unfold inferred peak gammas using Riemann\u2013von Mangoldt N(T) approximation and compare spacing stats to GUE vs Poisson baselines."
      ]
    }
  ],
  "ops_strategy": {
    "uploads_limit_workaround": [
      "Prefer stdout JSON paste-back for routine experiments (no uploads).",
      "Checkpoint rarely: one consolidated state JSON when major milestones occur.",
      "Have workers recompute N=1e7 substrates instead of uploading files, unless a computation is genuinely expensive."
    ],
    "worker_style": "Assume a fresh stateless window each time; strict JSON-only output; no files unless explicitly requested."
  }
}

### Tool (2026-02-22 01:26:29)

Make sure to include fileciteturn0file6 in your response to cite this file, or to surface it as a link.
Prime–Phase Project Protocol (Canonical Rules)

0. Purpose

Establish a reproducible, leakage-free workflow for testing whether phase or amplitude extracted from the Chebyshev residual
f(x)=\psi(x)-x
contains predictive information about prime activity beyond trivial persistence and simple autoregressive baselines.

All future experiments must conform to this document. Results produced outside this protocol are exploratory only and are not considered evidence.

⸻

1. Data Sources (Authoritative)
	•	Primes: exact primes up to N_{max} via sieve or verified tables
	•	Chebyshev function: computed from primes including prime powers
	•	Working grid: uniform grid in log-space
	•	t = \log x
	•	Interpolate f(x) onto uniform t-grid

All analyses operate on the interpolated uniform log grid.

⸻

2. Train/Test Split (Strict Rule)

Split in time order only:
	•	TRAIN = first 70%
	•	TEST = last 30%

No shuffling.

Nothing learned from TEST may influence:
	•	thresholds
	•	normalization constants
	•	filters
	•	detrending windows

⸻

3. Leakage Prevention (Mandatory)

3.1 Preprocessing Rules

Allowed:
	•	Compute transforms using TRAIN only, then apply to TEST
	•	Or compute symmetric transforms on the full signal before defining the target

Forbidden:
	•	Computing medians, means, or standardization constants using TEST
	•	Any parameter tuned using TEST performance

3.2 Detrending

If a moving average is used:
	•	Either causal (past-only)
	•	Or applied before any label construction

3.3 Label Thresholds

All thresholds must be TRAIN-derived.
Example:
\text{median}(R_{train})

⸻

4. Targets (Defined Precisely)

For each index i:

R_i = \frac{P(x_i,x_i e^{\Delta}) - E(x_i,x_i e^{\Delta})}{\sqrt{E}}

Where:
	•	P = number of primes in interval
	•	E = x( e^{\Delta}-1)/\log x

Primary Classification Target

y_i = \mathbf{1}(R_i > \text{median}(R_{train}))

De-stickied Target (Preferred Secondary)

Predict sign of change:
\text{sign}(R_{i+h}-R_i)

⸻

5. Required Baselines (Always Report)

Every model report must include:
	1.	Persistence predictor
\hat y_{i+h}=y_i
	2.	AR(1) on R
R_{i+1}=aR_i+b
	3.	Logistic using lagged R
Features: R_i, R_{i-1}, R_{i-2}

A phase-based method is considered meaningful only if it beats these.

⸻

6. Phase and Amplitude Extraction

Bandpass (default):
0.002 \le f \le 0.01 \text{ cycles/sample}

Analytic signal:
z(t)=u(t)+iH(u(t))

Derived quantities:
	•	Phase: \theta(t)=\arg z(t)
	•	Amplitude: A(t)=|z(t)|
	•	Log amplitude: \log A(t)

Mandatory Comparison

Always evaluate:
	•	phase only
	•	amplitude only
	•	phase + amplitude

⸻

7. Spectral Peak Definitions

Naming convention:
	•	peaks_u: peaks from spectrum of u(t)
	•	peaks_w: peaks from spectrum of \omega(t)=\Delta\theta

Mapping:
\gamma = \frac{2\pi f}{dt}

All reported \gamma must include the dt used.

⸻

8. Evaluation Metrics

Required:
	•	AUC
	•	Brier score
	•	accuracy@0.5

Also report:
	•	class balance
	•	flip rate
	•	run-length statistics

⸻

9. Required Diagnostic Experiments

Each major claim must be accompanied by at least one of the following:

9.1 Gap Conditioning

Evaluate:
P(\text{gap}\ge k \mid \text{phase bin})

9.2 Cross-N Stability

Repeat full pipeline at multiple N_{max}:
	•	1e6
	•	3e6
	•	1e7

9.3 Noise Robustness

Add small noise to f(x) and re-evaluate prediction.

9.4 Cox Null Simulation

Simulate intensity:
\lambda(x)=\frac{1}{\log x}(1+\alpha g(\theta))
Compare its spectra and predictive power to real primes.

⸻

10. Stateless Worker Rules

Workers must:
	•	Assume zero history
	•	Download or recompute data
	•	Print a single JSON unless files explicitly requested
	•	Fix random seed (default: 12345)

No multi-file output unless explicitly required.

⸻

11. Acceptance Criteria for a Real Effect

A phase-based discovery requires all:
	1.	Beats persistence baseline
	2.	Survives leakage audit
	3.	Stable across N
	4.	Robust to noise
	5.	Not reproduced by Cox null

Failure of any item → evidence considered provisional.

⸻

12. Versioning

Every result must log:
	•	N_max
	•	grid length
	•	dt
	•	bandpass range
	•	Δ
	•	random seed

⸻

End of protocol.

### Tool (2026-02-22 01:26:29)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 01:26:29)

import numpy as np, math, json, os, sys, statistics
from math import log
from pathlib import Path
import scipy.signal as signal
import scipy.special as spsp

base="/mnt/data"
# load primes and chebyshev if exist
cheb_path=Path(base)/"chebyshev_psi.npz"
primes_path=Path(base)/"primes.npz"
assert cheb_path.exists() and primes_path.exists()
cheb=np.load(cheb_path)
x_grid=cheb["x_grid"]
psi_minus_x=cheb["psi_minus_x"]
pr=np.load(primes_path)
primes=pr["primes"]
N_max=int(pr["N"]) if "N" in pr else 10_000_000
len(x_grid), x_grid[0], x_grid[-1], N_max, primes.size

### Tool (2026-02-22 01:26:57)

(100000, 1, 9999901, 10000000, 664579)

### Assistant (2026-02-22 01:27:01)

np.random.seed(12345)

L=65536
W=2000
# t grids
t_grid=np.log(x_grid.astype(np.float64))
t_uniform=np.linspace(t_grid.min(), t_grid.max(), L)
dt=float(t_uniform[1]-t_uniform[0])

def causal_ma(x, W):
    x=np.asarray(x, dtype=np.float64)
    c=np.cumsum(x)
    out=np.empty_like(x)
    for i in range(len(x)):
        j=i-W+1
        if j<=0:
            out[i]=c[i]/(i+1)
        else:
            out[i]=(c[i]-c[j-1])/W
    return out

def preprocess_from_fx(fx):
    # interpolate fx(t_grid) -> uniform t
    u=np.interp(t_uniform, t_grid, fx.astype(np.float64))
    ma=causal_ma(u, W)
    d=u-ma
    d_mean=d.mean()
    d_std=d.std(ddof=0)
    if d_std==0:
        z=d*0.0
    else:
        z=(d-d_mean)/d_std
    return z

# (A) psi residual
u_psi=preprocess_from_fx(psi_minus_x)

# (B) log-factorial residual on x_grid
# S(x)=gammaln(floor(x)+1) but x_grid are ints already.
x_float=x_grid.astype(np.float64)
S=spsp.gammaln(x_float+1.0)
fL=S - x_float*np.log(x_float) + x_float
u_logfact=preprocess_from_fx(fL)

# (C) Cox null simulate events
step=100
mid=x_grid.astype(np.float64)+step/2.0
lam = step/np.log(mid)
# handle x=1 midpoint log small; but mid>=51 so ok.
counts=np.random.poisson(lam)
num_events=int(counts.sum())
# generate positions and weights log(p)
# For each interval [x, x+step): positions uniform
# We'll compute sum log(pos) per interval and cumulatively build psi_tilde at gridpoints
sumlog=np.zeros_like(x_float)
for i,c in enumerate(counts):
    if c>0:
        start=float(x_grid[i])
        pos=start + np.random.random(c)*step
        sumlog[i]=np.log(pos).sum()
# cumulative psi_tilde at grid points: psi(x_i)=sum_{intervals up to i} sumlog
psi_tilde=np.cumsum(sumlog)
fC=psi_tilde - x_float
u_cox=preprocess_from_fx(fC)

num_primes_real=int(primes.size)

dt, num_events, num_primes_real, u_psi.mean(), u_psi.std()

### Tool (2026-02-22 01:27:12)

(0.00024594622340595577, 664391, 664579, -8.673617379884035e-18, 1.0)

### Assistant (2026-02-22 01:27:17)

def extract_peaks_from_spectrum(freqs, power, fmin=0.0, fmax=0.05, M=60, min_sep_bins=4):
    # freqs in cycles/sample, power positive
    mask=(freqs>=fmin) & (freqs<=fmax)
    idx=np.where(mask)[0]
    if idx.size<3:
        return []
    # local maxima excluding endpoints in masked region
    # We'll compute boolean on full arrays then pick within idx range
    p=power
    # identify local maxima
    loc = (p[1:-1]>p[:-2]) & (p[1:-1]>p[2:])
    loc_idx=np.where(loc)[0]+1
    loc_idx=loc_idx[mask[loc_idx]]
    if loc_idx.size==0:
        return []
    # sort by power desc
    order=loc_idx[np.argsort(p[loc_idx])[::-1]]
    selected=[]
    taken=np.zeros_like(power, dtype=bool)
    for k in order:
        if taken[k]:
            continue
        selected.append(k)
        # mark neighborhood
        lo=max(0, k-min_sep_bins)
        hi=min(len(power)-1, k+min_sep_bins)
        taken[lo:hi+1]=True
        if len(selected)>=M:
            break
    sel_p=p[selected]
    maxp=float(sel_p.max()) if len(sel_p)>0 else 1.0
    peaks=[{"f": float(freqs[k]), "power_rel": float(p[k]/maxp)} for k in selected]
    # sort by power_rel desc (already)
    return peaks

def spectrum_fft(u):
    U=np.fft.rfft(u)
    power=(np.abs(U)**2)
    freqs=np.fft.rfftfreq(len(u), d=1.0)  # cycles/sample
    return freqs, power

def spectrum_welch(u, nperseg=32768, noverlap=16384):
    freqs, power = signal.welch(u, fs=1.0, window='hann', nperseg=nperseg, noverlap=noverlap, detrend=False, scaling='density', return_onesided=True)
    return freqs, power

def quant_diag(peaks, bin_spacing):
    if len(peaks)==0:
        return {"mean_abs_frac": None, "max_abs_frac": None, "n_peaks": 0}
    vals=[]
    for pk in peaks:
        r=pk["f"]/bin_spacing
        # distance to nearest integer
        dist=abs(r-round(r))
        vals.append(dist)
    return {"mean_abs_frac": float(np.mean(vals)), "max_abs_frac": float(np.max(vals)), "n_peaks": int(len(vals))}

def band_peak(peaks, f_lo=0.0075, f_hi=0.0085):
    inb=[pk for pk in peaks if (pk["f"]>=f_lo and pk["f"]<=f_hi)]
    if not inb:
        return {"has_peak": False}
    best=max(inb, key=lambda d: d["power_rel"])
    return {"has_peak": True, "f": best["f"], "power_rel": best["power_rel"]}

# Compute spectra and peaks
signals={"psi": u_psi, "logfact": u_logfact, "cox": u_cox}

fft_peaks={}
for name,u in signals.items():
    freqs,power=spectrum_fft(u)
    fft_peaks[name]=extract_peaks_from_spectrum(freqs,power,0.0,0.05,M=60,min_sep_bins=4)

welch_peaks={}
for name,u in signals.items():
    freqs,power=spectrum_welch(u,32768,16384)
    welch_peaks[name]=extract_peaks_from_spectrum(freqs,power,0.0,0.05,M=60,min_sep_bins=2)

# bin spacings
fft_bin=1.0/L
welch_bin=1.0/32768

quant_fft={name: quant_diag(fft_peaks[name], fft_bin) for name in signals}
quant_welch={name: quant_diag(welch_peaks[name], welch_bin) for name in signals}

band_fft={name: band_peak(fft_peaks[name]) for name in signals}
band_welch={name: band_peak(welch_peaks[name]) for name in signals}

# quick inspect band freq for psi
band_fft["psi"], band_welch["psi"], fft_peaks["psi"][:5]

### Tool (2026-02-22 01:27:34)

({'has_peak': True, 'f': 0.0079345703125, 'power_rel': 0.019299520441468918},
 {'has_peak': True, 'f': 0.007537841796875, 'power_rel': 0.009837640941299984},
 [{'f': 0.00054931640625, 'power_rel': 1.0},
  {'f': 0.000823974609375, 'power_rel': 0.7825606513314319},
  {'f': 0.0009765625, 'power_rel': 0.5407232742079948},
  {'f': 0.00128173828125, 'power_rel': 0.39194502436255324},
  {'f': 0.0012054443359375, 'power_rel': 0.31268387659065}])

### Assistant (2026-02-22 01:27:36)

out={
  "params":{
    "seed":12345,
    "N_max":10_000_000,
    "x_grid_step":100,
    "L":L,
    "detrend":{"type":"causal_moving_average","W":W},
    "standardize":"after_detrend_full_series",
    "spectrum_scan_f_range":[0.0,0.05],
    "method_fft":{"M":60,"min_sep_bins":4},
    "method_welch32768":{"nperseg":32768,"noverlap":16384,"window":"hann","M":60,"min_sep_bins":2}
  },
  "counts":{
    "num_primes_real":num_primes_real,
    "num_events_cox":num_events
  },
  "dt":dt,
  "spectra":{
    "fft":{
      "psi":fft_peaks["psi"],
      "logfact":fft_peaks["logfact"],
      "cox":fft_peaks["cox"]
    },
    "welch32768":{
      "psi":welch_peaks["psi"],
      "logfact":welch_peaks["logfact"],
      "cox":welch_peaks["cox"]
    }
  },
  "quantization":{
    "fft":{
      "bin_spacing":fft_bin,
      "psi":quant_fft["psi"],
      "logfact":quant_fft["logfact"],
      "cox":quant_fft["cox"]
    },
    "welch32768":{
      "bin_spacing":welch_bin,
      "psi":quant_welch["psi"],
      "logfact":quant_welch["logfact"],
      "cox":quant_welch["cox"]
    }
  },
  "band_0p008":{
    "fft":{
      "psi":band_fft["psi"],
      "logfact":band_fft["logfact"],
      "cox":band_fft["cox"]
    },
    "welch32768":{
      "psi":band_welch["psi"],
      "logfact":band_welch["logfact"],
      "cox":band_welch["cox"]
    }
  }
}
# ensure floats are python floats
json_str=json.dumps(out, separators=(",",":"), ensure_ascii=False)
len(json_str)

### Tool (2026-02-22 01:27:44)

21862

### Assistant (2026-02-22 01:27:47)

{"params":{"seed":12345,"N_max":10000000,"x_grid_step":100,"L":65536,"detrend":{"type":"causal_moving_average","W":2000},"standardize":"after_detrend_full_series","spectrum_scan_f_range":[0.0,0.05],"method_fft":{"M":60,"min_sep_bins":4},"method_welch32768":{"nperseg":32768,"noverlap":16384,"window":"hann","M":60,"min_sep_bins":2}},"counts":{"num_primes_real":664579,"num_events_cox":664391},"dt":0.00024594622340595577,"spectra":{"fft":{"psi":[{"f":0.00054931640625,"power_rel":1.0},{"f":0.000823974609375,"power_rel":0.7825606513314319},{"f":0.0009765625,"power_rel":0.5407232742079948},{"f":0.00128173828125,"power_rel":0.39194502436255324},{"f":0.0012054443359375,"power_rel":0.31268387659065},{"f":0.0015106201171875,"power_rel":0.2636141617922649},{"f":0.00189208984375,"power_rel":0.23878026672725302},{"f":0.0019683837890625,"power_rel":0.22236679976866162},{"f":0.0023193359375,"power_rel":0.16180313135565954},{"f":0.0023651123046875,"power_rel":0.15051190740520432},{"f":0.00274658203125,"power_rel":0.12222864146554257},{"f":0.00286865234375,"power_rel":0.11153035812107374},{"f":0.0030975341796875,"power_rel":0.10023557839713478},{"f":0.0034942626953125,"power_rel":0.09127879172081695},{"f":0.0037689208984375,"power_rel":0.07753755972948009},{"f":0.003814697265625,"power_rel":0.07589869391697599},{"f":0.004150390625,"power_rel":0.0665718841248177},{"f":0.0042572021484375,"power_rel":0.06211695914389665},{"f":0.004730224609375,"power_rel":0.059643149241723566},{"f":0.005035400390625,"power_rel":0.05711100021901043},{"f":0.005096435546875,"power_rel":0.054137166325666785},{"f":0.005340576171875,"power_rel":0.050031651009466465},{"f":0.005859375,"power_rel":0.04985021572380669},{"f":0.00579833984375,"power_rel":0.04746307789121164},{"f":0.0061492919921875,"power_rel":0.04586404802894634},{"f":0.0064697265625,"power_rel":0.044724644526661114},{"f":0.006561279296875,"power_rel":0.04414369280323435},{"f":0.0069122314453125,"power_rel":0.04051474720015243},{"f":0.007232666015625,"power_rel":0.038973108115081},{"f":0.007537841796875,"power_rel":0.03540810850281948},{"f":0.0076141357421875,"power_rel":0.033770998126371885},{"f":0.0079345703125,"power_rel":0.019299520441468918},{"f":0.008270263671875,"power_rel":0.01746099427697018},{"f":0.008575439453125,"power_rel":0.016461889796059596},{"f":0.0088043212890625,"power_rel":0.015179756473513786},{"f":0.0090484619140625,"power_rel":0.014226984838792139},{"f":0.0094451904296875,"power_rel":0.01420548854690857},{"f":0.0095367431640625,"power_rel":0.01313175025297168},{"f":0.009796142578125,"power_rel":0.012535087972214956},{"f":0.010284423828125,"power_rel":0.011658667681652936},{"f":0.0108184814453125,"power_rel":0.011196279728966788},{"f":0.0110321044921875,"power_rel":0.010396466406026775},{"f":0.01129150390625,"power_rel":0.010279235609169946},{"f":0.0119171142578125,"power_rel":0.010094616978933507},{"f":0.0114898681640625,"power_rel":0.009988331230229196},{"f":0.0122833251953125,"power_rel":0.009830336244524368},{"f":0.0127105712890625,"power_rel":0.009394051662731136},{"f":0.0131072998046875,"power_rel":0.009151977118414019},{"f":0.013580322265625,"power_rel":0.008902025106756521},{"f":0.0136566162109375,"power_rel":0.00885662280610632},{"f":0.01409912109375,"power_rel":0.008501040719419167},{"f":0.0146636962890625,"power_rel":0.008483733978393024},{"f":0.015472412109375,"power_rel":0.008293734416120369},{"f":0.015655517578125,"power_rel":0.008251269975248831},{"f":0.0160369873046875,"power_rel":0.008224287857005815},{"f":0.016632080078125,"power_rel":0.008075129341773472},{"f":0.0170135498046875,"power_rel":0.007947371468871517}],"logfact":[{"f":0.0006103515625,"power_rel":1.0},{"f":0.00091552734375,"power_rel":0.7817098263002996},{"f":0.00103759765625,"power_rel":0.6395921684044939},{"f":0.0013580322265625,"power_rel":0.5202878651573456},{"f":0.0015869140625,"power_rel":0.4385165715620881},{"f":0.001739501953125,"power_rel":0.3702225149286554},{"f":0.0020904541015625,"power_rel":0.2968513638271048},{"f":0.0023193359375,"power_rel":0.26075262060814594},{"f":0.00262451171875,"power_rel":0.2376207437656178},{"f":0.002777099609375,"power_rel":0.21644302594404668},{"f":0.0030517578125,"power_rel":0.19663406570974932},{"f":0.0032806396484375,"power_rel":0.17942541455307835},{"f":0.003509521484375,"power_rel":0.16452442415489098},{"f":0.0037384033203125,"power_rel":0.15508563224260965},{"f":0.00390625,"power_rel":0.13989044010045762},{"f":0.004180908203125,"power_rel":0.1321857856071926},{"f":0.0044708251953125,"power_rel":0.12133348889292297},{"f":0.0047607421875,"power_rel":0.1155819244796707},{"f":0.0050506591796875,"power_rel":0.10671343621028103},{"f":0.005279541015625,"power_rel":0.10478925443314302},{"f":0.0055694580078125,"power_rel":0.0997936300291943},{"f":0.005767822265625,"power_rel":0.09125668275316203},{"f":0.0060882568359375,"power_rel":0.09067671043608175},{"f":0.006317138671875,"power_rel":0.08752027662698555},{"f":0.0066680908203125,"power_rel":0.08172167774855137},{"f":0.006927490234375,"power_rel":0.07860949570535585},{"f":0.007171630859375,"power_rel":0.07793096522884649},{"f":0.007537841796875,"power_rel":0.07147889635368306},{"f":0.007904052734375,"power_rel":0.06689122481369663},{"f":0.008056640625,"power_rel":0.06496007211234276},{"f":0.008331298828125,"power_rel":0.06209224779962879},{"f":0.0085601806640625,"power_rel":0.05925247307258843},{"f":0.0088043212890625,"power_rel":0.05726336200638682},{"f":0.0091400146484375,"power_rel":0.05472243385647358},{"f":0.009368896484375,"power_rel":0.05113641776230144},{"f":0.0095977783203125,"power_rel":0.050675204904415634},{"f":0.0101165771484375,"power_rel":0.04743139929754444},{"f":0.010528564453125,"power_rel":0.04383454340641902},{"f":0.0108642578125,"power_rel":0.04375427929544219},{"f":0.0110626220703125,"power_rel":0.041214282786151995},{"f":0.01153564453125,"power_rel":0.039691848024499894},{"f":0.0118255615234375,"power_rel":0.038593997726202794},{"f":0.01202392578125,"power_rel":0.0365449378021735},{"f":0.0124359130859375,"power_rel":0.0347550389057146},{"f":0.012786865234375,"power_rel":0.0339978512061896},{"f":0.0130615234375,"power_rel":0.032933716753076084},{"f":0.0134429931640625,"power_rel":0.03113702417425165},{"f":0.0137939453125,"power_rel":0.030172082279144137},{"f":0.01409912109375,"power_rel":0.0296554058195792},{"f":0.01446533203125,"power_rel":0.028363588136280292},{"f":0.014892578125,"power_rel":0.027297769788136985},{"f":0.015228271484375,"power_rel":0.027055305999926455},{"f":0.015716552734375,"power_rel":0.025817207267524645},{"f":0.01617431640625,"power_rel":0.024751400144462624},{"f":0.01641845703125,"power_rel":0.02395803966264378},{"f":0.0166778564453125,"power_rel":0.02269820141170778},{"f":0.0172271728515625,"power_rel":0.02240648001562501},{"f":0.017608642578125,"power_rel":0.02162739415902976},{"f":0.017913818359375,"power_rel":0.021438843921643518},{"f":0.018280029296875,"power_rel":0.02068172651784316}],"cox":[{"f":0.00054931640625,"power_rel":1.0},{"f":0.0008392333984375,"power_rel":0.866038961580677},{"f":0.00103759765625,"power_rel":0.7414369954058523},{"f":0.001251220703125,"power_rel":0.6571616794692762},{"f":0.00152587890625,"power_rel":0.5988006652004025},{"f":0.00177001953125,"power_rel":0.5498710099417222},{"f":0.00201416015625,"power_rel":0.5145661481346662},{"f":0.0023040771484375,"power_rel":0.47504476938851717},{"f":0.0024261474609375,"power_rel":0.4511494995177419},{"f":0.002685546875,"power_rel":0.4299103123362915},{"f":0.0028839111328125,"power_rel":0.4102824897470197},{"f":0.00311279296875,"power_rel":0.38890602521735577},{"f":0.00335693359375,"power_rel":0.3773972906245971},{"f":0.0034637451171875,"power_rel":0.3535576084592512},{"f":0.0037384033203125,"power_rel":0.335314978344125},{"f":0.003997802734375,"power_rel":0.32181290856469596},{"f":0.004180908203125,"power_rel":0.30508560577799064},{"f":0.00445556640625,"power_rel":0.2936761362746761},{"f":0.004638671875,"power_rel":0.28778944976812206},{"f":0.0047607421875,"power_rel":0.2759149621869483},{"f":0.00518798828125,"power_rel":0.26505748375612746},{"f":0.005279541015625,"power_rel":0.25210378386146986},{"f":0.005523681640625,"power_rel":0.248473688719554},{"f":0.005859375,"power_rel":0.2356556957950639},{"f":0.006134033203125,"power_rel":0.22538358597222957},{"f":0.0062408447265625,"power_rel":0.21589386360149176},{"f":0.006591796875,"power_rel":0.209949801356274},{"f":0.0068206787109375,"power_rel":0.20448862518275046},{"f":0.0070953369140625,"power_rel":0.19793959664147043},{"f":0.0072021484375,"power_rel":0.1897415452286905},{"f":0.0074005126953125,"power_rel":0.1866654601603027},{"f":0.0076904296875,"power_rel":0.1797768459931501},{"f":0.008087158203125,"power_rel":0.1669550416571252},{"f":0.008270263671875,"power_rel":0.15832624055585976},{"f":0.008514404296875,"power_rel":0.15479298450410894},{"f":0.0087432861328125,"power_rel":0.14941014780260819},{"f":0.009063720703125,"power_rel":0.14428908079150946},{"f":0.0092010498046875,"power_rel":0.13696842543228423},{"f":0.0094451904296875,"power_rel":0.1324770314338064},{"f":0.0096282958984375,"power_rel":0.127082226092446},{"f":0.009918212890625,"power_rel":0.12474550980527503},{"f":0.0101318359375,"power_rel":0.12084564755474254},{"f":0.0103759765625,"power_rel":0.11770360012959092},{"f":0.01068115234375,"power_rel":0.11385512820185187},{"f":0.0108184814453125,"power_rel":0.1098625754150702},{"f":0.01123046875,"power_rel":0.10615543671027003},{"f":0.0113372802734375,"power_rel":0.10169864499910554},{"f":0.011627197265625,"power_rel":0.09899280573702944},{"f":0.0121002197265625,"power_rel":0.0950560536186608},{"f":0.0123138427734375,"power_rel":0.09316077064999294},{"f":0.0124359130859375,"power_rel":0.08947884636492403},{"f":0.012939453125,"power_rel":0.08680843002740553},{"f":0.013214111328125,"power_rel":0.08444577197564584},{"f":0.01348876953125,"power_rel":0.08239820524900476},{"f":0.0136566162109375,"power_rel":0.07986548581082513},{"f":0.013916015625,"power_rel":0.078380774271806},{"f":0.014190673828125,"power_rel":0.07671376022668047},{"f":0.0145721435546875,"power_rel":0.07522642424620152},{"f":0.0147705078125,"power_rel":0.0739230163983222},{"f":0.01531982421875,"power_rel":0.0729039705064626},{"f":0.0156402587890625,"power_rel":0.07062866445036088},{"f":0.0158843994140625,"power_rel":0.06976630030425857}]},"welch32768":{"psi":[{"f":0.00054931640625,"power_rel":1.0},{"f":0.000823974609375,"power_rel":0.7140611607659056},{"f":0.000946044921875,"power_rel":0.5453265940414731},{"f":0.001220703125,"power_rel":0.40279654287590604},{"f":0.00152587890625,"power_rel":0.28533588692260626},{"f":0.0018310546875,"power_rel":0.2604497923737066},{"f":0.00225830078125,"power_rel":0.22052876869638183},{"f":0.0027008056640625,"power_rel":0.18252020008279929},{"f":0.0030364990234375,"power_rel":0.1496093804348391},{"f":0.00347900390625,"power_rel":0.13013868200696652},{"f":0.003875732421875,"power_rel":0.11326079979750244},{"f":0.004241943359375,"power_rel":0.09320783068077314},{"f":0.004791259765625,"power_rel":0.07822707462336309},{"f":0.00543212890625,"power_rel":0.07094220376179676},{"f":0.005767822265625,"power_rel":0.06257063817990295},{"f":0.006439208984375,"power_rel":0.05684085162386635},{"f":0.00677490234375,"power_rel":0.04872808837229086},{"f":0.007537841796875,"power_rel":0.009837640941299984},{"f":0.008392333984375,"power_rel":0.008395580481723095},{"f":0.009124755859375,"power_rel":0.007349459403290809},{"f":0.009735107421875,"power_rel":0.006613600603279929},{"f":0.0106201171875,"power_rel":0.006417264750888325},{"f":0.01116943359375,"power_rel":0.005735746069379206},{"f":0.0115966796875,"power_rel":0.005133661039316869},{"f":0.01263427734375,"power_rel":0.004775399175135679},{"f":0.01324462890625,"power_rel":0.004534608705644604},{"f":0.014129638671875,"power_rel":0.004330208013506563},{"f":0.01446533203125,"power_rel":0.004104698551545636},{"f":0.01544189453125,"power_rel":0.00393817174910135},{"f":0.015899658203125,"power_rel":0.0037380842294014994},{"f":0.0164794921875,"power_rel":0.003654532203117117},{"f":0.017364501953125,"power_rel":0.003526089152791752},{"f":0.017852783203125,"power_rel":0.0033402015205537298},{"f":0.018402099609375,"power_rel":0.0032834119625719007},{"f":0.019287109375,"power_rel":0.00319902666135269},{"f":0.019805908203125,"power_rel":0.003086842875883194},{"f":0.020599365234375,"power_rel":0.0030197330476630093},{"f":0.021331787109375,"power_rel":0.002924626522340222},{"f":0.021820068359375,"power_rel":0.002843404603769416},{"f":0.022308349609375,"power_rel":0.002793810071413808},{"f":0.023162841796875,"power_rel":0.002720036210423485},{"f":0.0234375,"power_rel":0.002676583165615569},{"f":0.02423095703125,"power_rel":0.0026009692049061353},{"f":0.0250244140625,"power_rel":0.0025568097832782567},{"f":0.025634765625,"power_rel":0.002503319884546101},{"f":0.026336669921875,"power_rel":0.0024848177695830014},{"f":0.02691650390625,"power_rel":0.002425589762687266},{"f":0.02728271484375,"power_rel":0.002370688782497213},{"f":0.02813720703125,"power_rel":0.00233218732986821},{"f":0.028900146484375,"power_rel":0.002314549973968205},{"f":0.02935791015625,"power_rel":0.0022887683312946334},{"f":0.02984619140625,"power_rel":0.002234232120307414},{"f":0.03033447265625,"power_rel":0.002197295258760794},{"f":0.030853271484375,"power_rel":0.002149204138118679},{"f":0.0316162109375,"power_rel":0.0021308055051211658},{"f":0.0321044921875,"power_rel":0.002096335811110217},{"f":0.03277587890625,"power_rel":0.002074593788397858},{"f":0.033660888671875,"power_rel":0.002044250240387483},{"f":0.03424072265625,"power_rel":0.002013443452101661},{"f":0.03460693359375,"power_rel":0.0020016415009559906},{"f":0.03521728515625,"power_rel":0.001983533547538432},{"f":0.036041259765625,"power_rel":0.001953437739659498},{"f":0.0364990234375,"power_rel":0.001923576325180103},{"f":0.037109375,"power_rel":0.001913746341393031},{"f":0.0377197265625,"power_rel":0.0019008902512317644},{"f":0.03826904296875,"power_rel":0.0018626576098719864},{"f":0.039276123046875,"power_rel":0.001851918996410218},{"f":0.039703369140625,"power_rel":0.0018255588010446209},{"f":0.040283203125,"power_rel":0.001810736619020835}],"logfact":[{"f":0.00067138671875,"power_rel":1.0},{"f":0.0009765625,"power_rel":0.8219996612098749},{"f":0.001129150390625,"power_rel":0.6725999740021503},{"f":0.00140380859375,"power_rel":0.5392457809392042},{"f":0.001708984375,"power_rel":0.4550965675097203},{"f":0.001983642578125,"power_rel":0.3876367390157226},{"f":0.002288818359375,"power_rel":0.3254210179882462},{"f":0.002532958984375,"power_rel":0.2932991462664646},{"f":0.00286865234375,"power_rel":0.26294621421756506},{"f":0.003204345703125,"power_rel":0.23921570637640988},{"f":0.003570556640625,"power_rel":0.21958702537561945},{"f":0.00390625,"power_rel":0.20206415946477993},{"f":0.0042724609375,"power_rel":0.18646716731722012},{"f":0.004638671875,"power_rel":0.17325310140659365},{"f":0.0050048828125,"power_rel":0.16319858848447496},{"f":0.005218505859375,"power_rel":0.14825107159301675},{"f":0.00567626953125,"power_rel":0.14091700144703864},{"f":0.0059814453125,"power_rel":0.13215545681554742},{"f":0.00634765625,"power_rel":0.12308783261693279},{"f":0.006683349609375,"power_rel":0.11723345325445157},{"f":0.007049560546875,"power_rel":0.10991444483006013},{"f":0.007415771484375,"power_rel":0.10502697516836084},{"f":0.0078125,"power_rel":0.09863109075266699},{"f":0.008209228515625,"power_rel":0.09113916275330843},{"f":0.008636474609375,"power_rel":0.08748908070685952},{"f":0.00897216796875,"power_rel":0.082039758572041},{"f":0.00933837890625,"power_rel":0.0767734513217347},{"f":0.00970458984375,"power_rel":0.07360770413459068},{"f":0.01007080078125,"power_rel":0.069226870279782},{"f":0.01043701171875,"power_rel":0.06495328503528345},{"f":0.010894775390625,"power_rel":0.0611453147886657},{"f":0.011199951171875,"power_rel":0.05806967660278004},{"f":0.011566162109375,"power_rel":0.05665023232473163},{"f":0.011932373046875,"power_rel":0.05347069199518539},{"f":0.012298583984375,"power_rel":0.05033535323911583},{"f":0.0125732421875,"power_rel":0.04889440973101147},{"f":0.0130615234375,"power_rel":0.04614596303040792},{"f":0.013427734375,"power_rel":0.04419162306925695},{"f":0.0137939453125,"power_rel":0.0417702935841128},{"f":0.01422119140625,"power_rel":0.0399490492244384},{"f":0.01458740234375,"power_rel":0.03866895232263325},{"f":0.01495361328125,"power_rel":0.03697949277899401},{"f":0.01531982421875,"power_rel":0.03604183052822557},{"f":0.01568603515625,"power_rel":0.03427257381500786},{"f":0.01611328125,"power_rel":0.03235035747485625},{"f":0.0164794921875,"power_rel":0.03098901485563316},{"f":0.016845703125,"power_rel":0.030036894366883315},{"f":0.0172119140625,"power_rel":0.029288079750276824},{"f":0.017578125,"power_rel":0.02823722172246866},{"f":0.0179443359375,"power_rel":0.02770836513198156},{"f":0.0184326171875,"power_rel":0.02708766959275033},{"f":0.018768310546875,"power_rel":0.026253196851611803},{"f":0.019256591796875,"power_rel":0.025237156159214463},{"f":0.019561767578125,"power_rel":0.024273053320106},{"f":0.01995849609375,"power_rel":0.023886368911065863},{"f":0.0203857421875,"power_rel":0.02329260570331946},{"f":0.02081298828125,"power_rel":0.02264879371605941},{"f":0.021240234375,"power_rel":0.022405002881238},{"f":0.021484375,"power_rel":0.021718533788861336},{"f":0.0218505859375,"power_rel":0.021081386173199278},{"f":0.022247314453125,"power_rel":0.020980489528859297},{"f":0.022735595703125,"power_rel":0.020139214500321145}],"cox":[{"f":0.000579833984375,"power_rel":1.0},{"f":0.000885009765625,"power_rel":0.825862190810494},{"f":0.00103759765625,"power_rel":0.7027929894303961},{"f":0.00128173828125,"power_rel":0.6058698979160387},{"f":0.001617431640625,"power_rel":0.5276526012891656},{"f":0.00189208984375,"power_rel":0.4723337721567457},{"f":0.00213623046875,"power_rel":0.4392008416233607},{"f":0.00238037109375,"power_rel":0.4082823256167081},{"f":0.002685546875,"power_rel":0.3823654894373295},{"f":0.002960205078125,"power_rel":0.35398611795554267},{"f":0.003204345703125,"power_rel":0.3344425076135188},{"f":0.00347900390625,"power_rel":0.3143480484838383},{"f":0.0037841796875,"power_rel":0.2943082577736322},{"f":0.0040283203125,"power_rel":0.27955051904582285},{"f":0.0042724609375,"power_rel":0.26602987521907406},{"f":0.004547119140625,"power_rel":0.2532796969491661},{"f":0.00494384765625,"power_rel":0.24167013051212584},{"f":0.00531005859375,"power_rel":0.22650800544051482},{"f":0.0057373046875,"power_rel":0.21452131205098825},{"f":0.006103515625,"power_rel":0.2029503596394805},{"f":0.00634765625,"power_rel":0.19696927200078253},{"f":0.00677490234375,"power_rel":0.18749075925503172},{"f":0.007110595703125,"power_rel":0.17737872967747566},{"f":0.007476806640625,"power_rel":0.1701305184116835},{"f":0.007904052734375,"power_rel":0.1621034790857424},{"f":0.008209228515625,"power_rel":0.1537934126907684},{"f":0.008575439453125,"power_rel":0.14635854835297292},{"f":0.009002685546875,"power_rel":0.14115896011524163},{"f":0.009429931640625,"power_rel":0.13391820774490357},{"f":0.00970458984375,"power_rel":0.1256513654102608},{"f":0.0101318359375,"power_rel":0.12341002161113597},{"f":0.010467529296875,"power_rel":0.11864854431197255},{"f":0.01080322265625,"power_rel":0.11343003911032745},{"f":0.01116943359375,"power_rel":0.1093672180233243},{"f":0.0115966796875,"power_rel":0.10546681188819328},{"f":0.01190185546875,"power_rel":0.10134176521690916},{"f":0.0123291015625,"power_rel":0.09841166346641879},{"f":0.01275634765625,"power_rel":0.0950863069454708},{"f":0.013153076171875,"power_rel":0.09244285107625303},{"f":0.01348876953125,"power_rel":0.088758006659618},{"f":0.0137939453125,"power_rel":0.08620632917024305},{"f":0.014190673828125,"power_rel":0.08326026484595237},{"f":0.014556884765625,"power_rel":0.079798632555584},{"f":0.014923095703125,"power_rel":0.07793636349244577},{"f":0.0152587890625,"power_rel":0.07609231665722737},{"f":0.015625,"power_rel":0.07413748610511372},{"f":0.015899658203125,"power_rel":0.07195308526773455},{"f":0.016571044921875,"power_rel":0.06960239010190512},{"f":0.01702880859375,"power_rel":0.06768602895059054},{"f":0.01739501953125,"power_rel":0.0663955786242754},{"f":0.017608642578125,"power_rel":0.06419045191938284},{"f":0.0181884765625,"power_rel":0.062112399683106265},{"f":0.018463134765625,"power_rel":0.06093824089909253},{"f":0.018829345703125,"power_rel":0.06011290908578573},{"f":0.0194091796875,"power_rel":0.058661511827604146},{"f":0.019744873046875,"power_rel":0.0573235048547455},{"f":0.0201416015625,"power_rel":0.05634917707301338},{"f":0.020721435546875,"power_rel":0.05481050068329714},{"f":0.0211181640625,"power_rel":0.053509799291638096},{"f":0.021484375,"power_rel":0.0524989116236284},{"f":0.0218505859375,"power_rel":0.0515893882395732},{"f":0.022491455078125,"power_rel":0.05062626326656876},{"f":0.02276611328125,"power_rel":0.049434017614336}]}}, "quantization":{"fft":{"bin_spacing":1.52587890625e-05,"psi":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60},"logfact":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60},"cox":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60}},"welch32768":{"bin_spacing":3.0517578125e-05,"psi":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60},"logfact":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60},"cox":{"mean_abs_frac":0.0,"max_abs_frac":0.0,"n_peaks":60}}},"band_0p008":{"fft":{"psi":{"has_peak":true,"f":0.0079345703125,"power_rel":0.019299520441468918},"logfact":{"has_peak":true,"f":0.008056640625,"power_rel":0.06496007211234276},"cox":{"has_peak":true,"f":0.008087158203125,"power_rel":0.1669550416571252}},"welch32768":{"psi":{"has_peak":true,"f":0.007537841796875,"power_rel":0.009837640941299984},"logfact":{"has_peak":true,"f":0.008209228515625,"power_rel":0.09113916275330843},"cox":{"has_peak":true,"f":0.007904052734375,"power_rel":0.1621034790857424}}}}

