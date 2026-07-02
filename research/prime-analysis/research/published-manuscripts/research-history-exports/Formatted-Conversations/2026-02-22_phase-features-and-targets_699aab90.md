# Phase Features and Targets

- **Conversation ID:** `699aab90-1ebc-8328-9346-de83f64e77c5`
- **Date:** 2026-02-22

---

### User (2026-02-22 02:09:07)

You are a stateless compute worker.

PRINT JSON ONLY. No files.

Goal: Remove the stickiness artifact from the slow-target test by predicting the *change* in the slow component (and a band-limited low-frequency component), and measure whether Hilbert phase adds lift beyond AR baselines.

You may import project files (preferred):
- chebyshev_psi.npz  (x_grid, psi_minus_x)
- primes.npz         (primes)

Params:
- L = 65536
- x range [1e5, 9e6]
- Delta = 0.01
- split train frac = 0.7 time-ordered
- detrend W = 2000 (causal moving average, past-only)
- phase bandpass cycles/sample = [0.002, 0.01]
- slow window WR = 400 (past-only)
- lowfreq band for R_lf cycles/sample = [0.0002, 0.002]
- horizons h in {25, 50, 100}
- seed = 12345

A) Phase features from psi_minus_x
1) Load x_grid, psi_minus_x. t_raw=log(x_grid).
2) Interpolate psi_minus_x onto uniform t grid length L across [min(t_raw), max(t_raw)].
   Record dt.
3) Restrict to indices where x=exp(t) in [1e5, 9e6]. Let f_raw.
4) Causal detrend: f = f_raw - moving_average_past_only(f_raw, W)
5) Standardize using TRAIN only: f_std=(f-mean_train)/std_train
6) FFT bandpass f_std to [0.002,0.01] -> u
7) Hilbert: z=u + i*Hilbert(u); phi=angle(z) in [0,2π); A=|z|; L_amp=log(A+1e-12)
8) PhiFeat = [1, cos(phi), sin(phi), cos(2phi), sin(2phi), cos(3phi), sin(3phi)]
   AmpFeat = [L_amp, L_amp*cos(phi), L_amp*sin(phi)]

B) Prime residual R
9) For each kept i: x_i=exp(t[i])
   P_i = #primes in [x_i, x_i*exp(Delta))
   E_i = x_i*(exp(Delta)-1)/log(x_i)
   R_i = (P_i - E_i)/sqrt(E_i)

C) Targets (de-sticky slow + innovations)
10) Slow moving average: R_slow[i] = mean(R[i-WR+1..i]) (past-only).
    Slow change: dR_slow[i] = R_slow[i] - R_slow[i-1].
    Target y_dslow(i,h) = 1 if dR_slow[i+h] > 0 else 0.

11) Low-frequency band-limited component:
    Standardize R using TRAIN only -> R_std.
    FFT bandpass R_std to lowfreq band [0.0002,0.002] -> R_lf.
    Target y_lf(i,h) = 1 if R_lf[i+h] > 0 else 0.

12) Innovation target (same as prior):
    dR[i]=R[i]-R[i-1].
    Fit AR(1) on TRAIN dR: dR[i]=phi_ar*dR[i-1]+noise
    eps[i]=dR[i]-phi_ar*dR[i-1]
    Target y_eps(i,h)=1 if eps[i+h] > 0 else 0.

Splitting rules:
13) TRAIN labels use i with (i+h) within TRAIN region.
    TEST labels use i starting at split boundary with valid (i+h).

Diagnostics for each target/h:
- positive_rate_train/test
- flip_rate_train/test
- mean_run_length_train/test
- persistence baseline accuracy (predict next label equals previous)

D) Models (logistic regression), evaluate TEST
For each target in {y_dslow, y_lf, y_eps} and each h:
Baselines:
- Base_R = [1, R[i], R[i-1]]
- Base_dslow = [1, dR_slow[i], dR_slow[i-1]]
- Base_lf = [1, R_lf[i], R_lf[i-1]]
- Base_eps = [1, eps[i]] (only for y_eps)

Models:
1) M_base: appropriate baseline
2) M_phase: PhiFeat
3) M_phase_amp: PhiFeat + AmpFeat
4) M_both: baseline + PhiFeat
5) M_both_amp: baseline + PhiFeat + AmpFeat

Metrics:
- AUC
- Brier
- accuracy@0.5

Output JSON:
{
  "params": {...},
  "ar1": {"phi_ar":..., "train_n_used":...},
  "diagnostics": {
     "y_dslow": {...}, "y_lf": {...}, "y_eps": {...}
  },
  "results": {
     "y_dslow": {"h=25": {...}, "h=50": {...}, "h=100": {...}},
     "y_lf":    {"h=25": {...}, "h=50": {...}, "h=100": {...}},
     "y_eps":   {"h=25": {...}, "h=50": {...}, "h=100": {...}}
  }
}

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

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

### Tool (2026-02-22 02:09:11)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 02:09:39)

import numpy as np, math, json, os, statistics, sys, textwrap
from pathlib import Path
import numpy.fft as fft

### Assistant (2026-02-22 02:09:46)

import numpy as np, math, json, os
from pathlib import Path
data_dir=Path("/mnt/data")
cheb_path=data_dir/"chebyshev_psi.npz"
primes_path=data_dir/"primes.npz"
cheb=np.load(cheb_path)
pr=np.load(primes_path, allow_pickle=True)
cheb.files, pr.files[:10]

### Tool (2026-02-22 02:09:48)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'],
 ['N', 'primes', 'is_prime'])

### Assistant (2026-02-22 02:09:49)

x_grid=cheb["x_grid"].astype(np.float64)
psi_minus_x=cheb["psi_minus_x"].astype(np.float64)
primes=pr["primes"].astype(np.int64)
Nmax=int(pr["N"])
len(x_grid), x_grid[:3], x_grid[-3:], len(primes), Nmax

### Tool (2026-02-22 02:09:52)

(100000,
 array([  1., 101., 201.]),
 array([9999701., 9999801., 9999901.]),
 664579,
 10000000)

### Assistant (2026-02-22 02:09:53)

import numpy as np, math
L=65536
t_raw=np.log(x_grid)
tmin=float(t_raw.min()); tmax=float(t_raw.max())
t_uniform=np.linspace(tmin, tmax, L)
dt=float(t_uniform[1]-t_uniform[0])
# interpolate psi_minus_x
f_interp=np.interp(t_uniform, t_raw, psi_minus_x)
x_uniform=np.exp(t_uniform)
# restrict x range
x_lo=1e5; x_hi=9e6
mask=(x_uniform>=x_lo)&(x_uniform<=x_hi)
idx=np.nonzero(mask)[0]
n=len(idx)
n, idx[0], idx[-1], x_uniform[idx[0]], x_uniform[idx[-1]], dt

### Tool (2026-02-22 02:09:57)

(18296,
 46811,
 65106,
 100006.32008830582,
 8998557.287555724,
 0.00024594622340595577)

### Assistant (2026-02-22 02:09:58)

def moving_average_past_only(x, W):
    x=np.asarray(x, dtype=np.float64)
    n=len(x)
    if W<=1:
        return x.copy()
    c=np.cumsum(np.insert(x,0,0.0))
    # mean over max(0,i-W+1)..i
    ma=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0=max(0, i-W+1)
        s=c[i+1]-c[j0]
        ma[i]=s/(i-j0+1)
    return ma

def fft_bandpass(signal, f_lo, f_hi):
    # cycles/sample bandpass on real signal
    x=np.asarray(signal, dtype=np.float64)
    n=len(x)
    X=np.fft.rfft(x)
    freqs=np.fft.rfftfreq(n, d=1.0)  # cycles/sample
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    X_filt=np.where(mask, X, 0.0)
    y=np.fft.irfft(X_filt, n=n)
    return y

def analytic_signal_hilbert(x):
    # Use FFT-based hilbert to avoid dependency
    x=np.asarray(x, dtype=np.float64)
    n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1
        h[1:n//2]=2
    else:
        h[0]=1
        h[1:(n+1)//2]=2
    z=np.fft.ifft(X*h)
    return z

# build f_raw restricted
f_raw=f_interp[idx]
W=2000
ma=moving_average_past_only(f_raw, W)
f=f_raw-ma

train_frac=0.7
train_end=int(np.floor(train_frac*n))
train_idx=np.arange(train_end)
test_idx=np.arange(train_end, n)

mean_train=float(np.mean(f[train_idx]))
std_train=float(np.std(f[train_idx], ddof=0))
f_std=(f-mean_train)/(std_train if std_train>0 else 1.0)

# bandpass phase band
u=fft_bandpass(f_std, 0.002, 0.01)
z=analytic_signal_hilbert(u)
phi=np.mod(np.angle(z), 2*np.pi)
A=np.abs(z)
L_amp=np.log(A+1e-12)
phi[:3], L_amp[:3], np.isfinite(L_amp).all()

### Tool (2026-02-22 02:10:10)

(array([1.85187755, 1.87159806, 1.89143317]),
 array([0.94351831, 0.94284126, 0.94209703]),
 True)

### Assistant (2026-02-22 02:10:11)

Delta=0.01
expD=math.exp(Delta)
k=(expD-1.0)
# compute R
x_kept=x_uniform[idx]  # float64
logx=np.log(x_kept)
# prime counts via searchsorted
lo=x_kept
hi=x_kept*expD
# searchsorted needs primes sorted (yes)
left=np.searchsorted(primes, lo, side='left')
right=np.searchsorted(primes, hi, side='left')  # [lo,hi)
P=(right-left).astype(np.float64)
E=(x_kept*k)/logx
R=(P-E)/np.sqrt(E)
R[:5], float(np.mean(R)), float(np.std(R))

### Tool (2026-02-22 02:10:15)

(array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 -0.025579684500115142,
 0.48854525585234265)

### Assistant (2026-02-22 02:10:16)

WR=400
# R_slow past-only moving mean
R_slow=moving_average_past_only(R, WR)
dR_slow=np.empty_like(R_slow); dR_slow[:] = np.nan
dR_slow[1:]=R_slow[1:]-R_slow[:-1]
dR=np.empty_like(R); dR[:] = np.nan
dR[1:]=R[1:]-R[:-1]

# AR(1) on TRAIN dR: dR[i]=phi*dR[i-1]+noise
# use i in train_idx where i>=2? Actually need i and i-1 valid => i>=2
train_i=np.arange(2, train_end)  # i uses dR[i], regressor dR[i-1]
x_ar=dR[train_i-1]
y_ar=dR[train_i]
# exclude any nan
mask_ar=np.isfinite(x_ar)&np.isfinite(y_ar)
x_ar=x_ar[mask_ar]; y_ar=y_ar[mask_ar]
phi_ar=float(np.dot(x_ar,y_ar)/np.dot(x_ar,x_ar)) if np.dot(x_ar,x_ar)>0 else 0.0
eps=np.empty_like(dR); eps[:]=np.nan
eps[2:]=dR[2:]-phi_ar*dR[1:-1]
phi_ar, len(y_ar)

### Tool (2026-02-22 02:10:22)

(-0.10329922154745123, 12805)

### Assistant (2026-02-22 02:10:23)

# Standardize R using TRAIN only
R_mean_train=float(np.mean(R[train_idx]))
R_std_train=float(np.std(R[train_idx], ddof=0))
R_z=(R-R_mean_train)/(R_std_train if R_std_train>0 else 1.0)
R_lf=fft_bandpass(R_z, 0.0002, 0.002)
float(np.mean(R_lf)), float(np.std(R_lf))

### Tool (2026-02-22 02:10:26)

(-3.495236457062146e-18, 0.23891600601456842)

### Assistant (2026-02-22 02:10:28)

def diagnostics_for_labels(y, region_indices):
    # region_indices is sorted array of i positions (forecast origins) used for labels
    y_seq=y[region_indices]
    # positive rate
    pos=float(np.mean(y_seq)) if len(y_seq)>0 else float('nan')
    # flip rate between consecutive i in region that are consecutive in time
    # We'll compute over adjacent positions in region_indices (they are consecutive by construction usually)
    if len(y_seq)>=2:
        flip=float(np.mean(y_seq[1:]!=y_seq[:-1]))
        # run lengths
        runs=[]
        run_len=1
        for j in range(1,len(y_seq)):
            if y_seq[j]==y_seq[j-1]:
                run_len+=1
            else:
                runs.append(run_len); run_len=1
        runs.append(run_len)
        mean_run=float(np.mean(runs))
        # persistence baseline: predict current label equals previous label
        pers_acc=float(np.mean(y_seq[1:]==y_seq[:-1]))
    else:
        flip=float('nan'); mean_run=float('nan'); pers_acc=float('nan')
    return {
        "n": int(len(y_seq)),
        "positive_rate": pos,
        "flip_rate": flip,
        "mean_run_length": mean_run,
        "persistence_baseline_accuracy": pers_acc
    }

from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score

def fit_eval_logit(X_train, y_train, X_test, y_test, seed=12345):
    # handle constant y
    out={}
    # sklearn expects 2 classes
    try:
        model=LogisticRegression(solver="lbfgs", max_iter=5000, random_state=seed)
        model.fit(X_train, y_train)
        p_test=model.predict_proba(X_test)[:,1]
    except Exception as e:
        return {"error": str(e)}
    # metrics
    # AUC requires both classes in y_test
    if len(np.unique(y_test))==2:
        auc=float(roc_auc_score(y_test, p_test))
    else:
        auc=float('nan')
    brier=float(np.mean((p_test - y_test)**2))
    acc=float(np.mean((p_test>=0.5)==y_test))
    return {"AUC": auc, "Brier": brier, "accuracy@0.5": acc}

def build_features(target_name):
    # return dict of feature matrices for each model name: function(i_indices)->X
    pass

### Assistant (2026-02-22 02:10:40)

import numpy as np

# Precompute PhiFeat and AmpFeat arrays per i
c1=np.cos(phi); s1=np.sin(phi)
c2=np.cos(2*phi); s2=np.sin(2*phi)
c3=np.cos(3*phi); s3=np.sin(3*phi)
PhiFeat=np.column_stack([np.ones(n), c1,s1,c2,s2,c3,s3])
AmpFeat=np.column_stack([L_amp, L_amp*c1, L_amp*s1])

def make_baseline_features(which, i_idx):
    i=np.asarray(i_idx)
    if which=="R":
        X=np.column_stack([np.ones(len(i)), R[i], R[i-1]])
    elif which=="dslow":
        X=np.column_stack([np.ones(len(i)), dR_slow[i], dR_slow[i-1]])
    elif which=="lf":
        X=np.column_stack([np.ones(len(i)), R_lf[i], R_lf[i-1]])
    elif which=="eps":
        X=np.column_stack([np.ones(len(i)), eps[i]])
    else:
        raise ValueError(which)
    return X

def valid_indices_for_h(h):
    train_valid=np.arange(1, train_end-h)  # i>=1 for lagged features, i+h within train
    test_valid=np.arange(max(train_end,1), n-h)
    return train_valid, test_valid

def labels_for_target(target, h):
    if target=="y_dslow":
        y=(dR_slow[h:]>0).astype(int)  # this gives y_at_time t = h..n-1; but we need y[i] refers to sign at i? easier make full array size n with nan
    # We'll construct full label arrays y_full[i]=label for i such that i+h valid
    y_full=np.full(n, np.nan)
    if target=="y_dslow":
        y_full[:-h]= (dR_slow[h:]>0).astype(float)  # for i, label looks at i+h
    elif target=="y_lf":
        y_full[:-h]= (R_lf[h:]>0).astype(float)
    elif target=="y_eps":
        y_full[:-h]= (eps[h:]>0).astype(float)
    else:
        raise ValueError
    return y_full

targets=["y_dslow","y_lf","y_eps"]
hs=[25,50,100]

diagnostics={}
results={}
seed=12345

for target in targets:
    diagnostics[target]={}
    results[target]={}
    for h in hs:
        y_full=labels_for_target(target,h)
        train_i, test_i=valid_indices_for_h(h)
        # filter indices with finite labels and finite needed predictors for baselines
        # For phase-only models, need phi/L_amp finite at i
        # Build mask per model later; diagnostics on labels only (in region)
        train_mask=np.isfinite(y_full[train_i])
        test_mask=np.isfinite(y_full[test_i])
        train_i_lab=train_i[train_mask]
        test_i_lab=test_i[test_mask]
        y_train=y_full[train_i_lab].astype(int)
        y_test=y_full[test_i_lab].astype(int)
        # diagnostics
        diag={
            "train": diagnostics_for_labels(y_full.astype(int, copy=False, casting='unsafe') if False else y_full, train_i_lab),
            "test": diagnostics_for_labels(y_full, test_i_lab)
        }
        diagnostics[target][f"h={h}"]=diag
        
        # Choose baseline type
        base_type = {"y_dslow":"dslow", "y_lf":"lf", "y_eps":"eps"}[target]
        model_metrics={}
        # build model datasets for each model
        # M_base
        for model_name, use_base, use_phase, use_amp in [
            ("M_base", True, False, False),
            ("M_phase", False, True, False),
            ("M_phase_amp", False, True, True),
            ("M_both", True, True, False),
            ("M_both_amp", True, True, True),
        ]:
            # Build X for train/test with required columns
            Xtr_list=[]
            Xte_list=[]
            # baseline
            if use_base:
                Xtr_list.append(make_baseline_features(base_type, train_i_lab))
                Xte_list.append(make_baseline_features(base_type, test_i_lab))
            if use_phase:
                Xtr_list.append(PhiFeat[train_i_lab])
                Xte_list.append(PhiFeat[test_i_lab])
            if use_amp:
                Xtr_list.append(AmpFeat[train_i_lab])
                Xte_list.append(AmpFeat[test_i_lab])
            X_train=np.column_stack(Xtr_list)
            X_test=np.column_stack(Xte_list)
            # remove rows with nan or inf
            ok_tr=np.all(np.isfinite(X_train), axis=1)
            ok_te=np.all(np.isfinite(X_test), axis=1)
            X_train2=X_train[ok_tr]; y_train2=y_train[ok_tr]
            X_test2=X_test[ok_te]; y_test2=y_test[ok_te]
            # need at least 2 classes in train to fit
            if len(np.unique(y_train2))<2 or len(y_train2)<10 or len(y_test2)<10:
                model_metrics[model_name]={"AUC": float('nan'), "Brier": float('nan'), "accuracy@0.5": float('nan'),
                                          "train_n": int(len(y_train2)), "test_n": int(len(y_test2)),
                                          "train_classes": int(len(np.unique(y_train2))), "test_classes": int(len(np.unique(y_test2)))}
            else:
                met=fit_eval_logit(X_train2, y_train2, X_test2, y_test2, seed=seed)
                met.update({"train_n": int(len(y_train2)), "test_n": int(len(y_test2)),
                            "train_classes": int(len(np.unique(y_train2))), "test_classes": int(len(np.unique(y_test2)))})
                model_metrics[model_name]=met
        results[target][f"h={h}"]=model_metrics

diagnostics.keys(), results["y_eps"]["h=25"]["M_base"]

### Tool (2026-02-22 02:11:01)

(dict_keys(['y_dslow', 'y_lf', 'y_eps']),
 {'AUC': 0.4978093665940363,
  'Brier': 0.2500414190543522,
  'accuracy@0.5': 0.49890190336749635,
  'train_n': 12780,
  'test_n': 5464,
  'train_classes': 2,
  'test_classes': 2})

### Assistant (2026-02-22 02:12:05)

def diagnostics_for_labels_full(y_full, region_indices):
    i=np.asarray(region_indices, dtype=int)
    y_seq=y_full[i].astype(int)
    pos=float(np.mean(y_seq)) if len(y_seq)>0 else float('nan')
    if len(y_seq)>=2:
        flip=float(np.mean(y_seq[1:]!=y_seq[:-1]))
        runs=[]
        run_len=1
        for j in range(1,len(y_seq)):
            if y_seq[j]==y_seq[j-1]:
                run_len+=1
            else:
                runs.append(run_len); run_len=1
        runs.append(run_len)
        mean_run=float(np.mean(runs))
        pers_acc=float(np.mean(y_seq[1:]==y_seq[:-1]))
    else:
        flip=mean_run=pers_acc=float('nan')
    return {"n": int(len(y_seq)), "positive_rate": pos, "flip_rate": flip,
            "mean_run_length": mean_run, "persistence_baseline_accuracy": pers_acc}

diagnostics={}
results_out=results  # already computed
for target in targets:
    diagnostics[target]={}
    for h in hs:
        y_full=labels_for_target(target,h)
        train_i, test_i=valid_indices_for_h(h)
        train_i_lab=train_i[np.isfinite(y_full[train_i])]
        test_i_lab=test_i[np.isfinite(y_full[test_i])]
        diagnostics[target][f"h={h}"]={
            "train": diagnostics_for_labels_full(y_full, train_i_lab),
            "test": diagnostics_for_labels_full(y_full, test_i_lab)
        }
diagnostics["y_dslow"]["h=25"]

### Tool (2026-02-22 02:12:11)

{'train': {'n': 12781,
  'positive_rate': 0.4988655034817307,
  'flip_rate': 0.11087636932707355,
  'mean_run_length': 9.013399153737659,
  'persistence_baseline_accuracy': 0.8891236306729264},
 'test': {'n': 5464,
  'positive_rate': 0.4780380673499268,
  'flip_rate': 0.10873146622734761,
  'mean_run_length': 9.183193277310924,
  'persistence_baseline_accuracy': 0.8912685337726524}}

### Assistant (2026-02-22 02:12:15)

def replace_nan(obj):
    if isinstance(obj, float):
        if math.isnan(obj) or math.isinf(obj):
            return None
        return obj
    if isinstance(obj, dict):
        return {k: replace_nan(v) for k,v in obj.items()}
    if isinstance(obj, list):
        return [replace_nan(v) for v in obj]
    if isinstance(obj, tuple):
        return [replace_nan(v) for v in obj]
    return obj

out={
  "params":{
      "L": L,
      "x_range":[x_lo,x_hi],
      "Delta": Delta,
      "train_frac": train_frac,
      "detrend_W": W,
      "phase_bandpass_cycles_per_sample":[0.002,0.01],
      "slow_window_WR": WR,
      "lowfreq_band_cycles_per_sample":[0.0002,0.002],
      "horizons": hs,
      "seed": seed,
      "dt": dt,
      "n_kept": n,
      "train_end": train_end,
      "N_max": Nmax
  },
  "ar1":{"phi_ar": phi_ar, "train_n_used": int(len(y_ar))},
  "diagnostics": diagnostics,
  "results": results_out
}
out_clean=replace_nan(out)
# ensure no NaN
def contains_nan(o):
    if isinstance(o,float):
        return math.isnan(o) or math.isinf(o)
    if isinstance(o,dict):
        return any(contains_nan(v) for v in o.values())
    if isinstance(o,list):
        return any(contains_nan(v) for v in o)
    return False
contains_nan(out_clean)

### Tool (2026-02-22 02:12:22)

False

### Assistant (2026-02-22 02:12:23)

# peek some key results for sanity
out_clean["results"]["y_lf"]["h=100"]["M_both_amp"]

### Tool (2026-02-22 02:12:26)

{'AUC': 0.799780632773554,
 'Brier': 0.19167462664138427,
 'accuracy@0.5': 0.7305622564483206,
 'train_n': 12706,
 'test_n': 5389,
 'train_classes': 2,
 'test_classes': 2}

### Assistant (2026-02-22 02:12:28)

{"params":{"L":65536,"x_range":[100000.0,9000000.0],"Delta":0.01,"train_frac":0.7,"detrend_W":2000,"phase_bandpass_cycles_per_sample":[0.002,0.01],"slow_window_WR":400,"lowfreq_band_cycles_per_sample":[0.0002,0.002],"horizons":[25,50,100],"seed":12345,"dt":0.00024594622340595577,"n_kept":18296,"train_end":12807,"N_max":10000000},"ar1":{"phi_ar":-0.10329922154745123,"train_n_used":12805},"diagnostics":{"y_dslow":{"h=25":{"train":{"n":12781,"positive_rate":0.4988655034817307,"flip_rate":0.11087636932707355,"mean_run_length":9.013399153737659,"persistence_baseline_accuracy":0.8891236306729264},"test":{"n":5464,"positive_rate":0.4780380673499268,"flip_rate":0.10873146622734761,"mean_run_length":9.183193277310924,"persistence_baseline_accuracy":0.8912685337726524}},"h=50":{"train":{"n":12756,"positive_rate":0.4980401379730317,"flip_rate":0.11759176758917675,"mean_run_length":8.512324493014343,"persistence_baseline_accuracy":0.8824082324108232},"test":{"n":5439,"positive_rate":0.47894833682662255,"flip_rate":0.10867769918955632,"mean_run_length":9.190862068965517,"persistence_baseline_accuracy":0.8913223008104436}},"h=100":{"train":{"n":12706,"positive_rate":0.4983469227149378,"flip_rate":0.11628730449822814,"mean_run_length":8.601626016260163,"persistence_baseline_accuracy":0.8837126955017719},"test":{"n":5389,"positive_rate":0.47968194470254126,"flip_rate":0.1082000371391722,"mean_run_length":9.231115879828326,"persistence_baseline_accuracy":0.8917999628608278}}},"y_lf":{"h=25":{"train":{"n":12781,"positive_rate":0.7503329942876145,"flip_rate":0.07801251956181534,"mean_run_length":12.835341365461847,"persistence_baseline_accuracy":0.9219874804381846},"test":{"n":5464,"positive_rate":0.421119326500366,"flip_rate":0.0900604063704924,"mean_run_length":11.16530612244898,"persistence_baseline_accuracy":0.9099395936295076}},"h=50":{"train":{"n":12756,"positive_rate":0.7504703675133277,"flip_rate":0.07924085623211511,"mean_run_length":12.61133069828722,"persistence_baseline_accuracy":0.9207591437678849},"test":{"n":5439,"positive_rate":0.4225040448619224,"flip_rate":0.09009009009009009,"mean_run_length":11.161522633744856,"persistence_baseline_accuracy":0.9099099099099099}},"h=100":{"train":{"n":12706,"positive_rate":0.7497245395883834,"flip_rate":0.07947106965000393,"mean_run_length":12.577227722772277,"persistence_baseline_accuracy":0.9205289303499961},"test":{"n":5389,"positive_rate":0.42531193171608683,"flip_rate":0.08981332456338528,"mean_run_length":11.174534161490683,"persistence_baseline_accuracy":0.9101866754366147}}},"y_eps":{"h=25":{"train":{"n":12780,"positive_rate":0.4991392801251956,"flip_rate":0.4942475930825493,"mean_run_length":2.0200638244227664,"persistence_baseline_accuracy":0.5057524069174507},"test":{"n":5464,"positive_rate":0.4769403378473642,"flip_rate":0.4938652754896588,"mean_run_length":2.0327635327635327,"persistence_baseline_accuracy":0.5061347245103412}},"h=50":{"train":{"n":12755,"positive_rate":0.4991767934143473,"flip_rate":0.49462984462984465,"mean_run_length":2.0185394456289977,"persistence_baseline_accuracy":0.5053701553701553},"test":{"n":5439,"positive_rate":0.47802868247839677,"flip_rate":0.49328870811180583,"mean_run_length":2.0352101910828026,"persistence_baseline_accuracy":0.5067112918881941}},"h=100":{"train":{"n":12705,"positive_rate":0.5007489177489177,"flip_rate":0.49452812696710564,"mean_run_length":2.0222602739726026,"persistence_baseline_accuracy":0.5054718730328944},"test":{"n":5389,"positive_rate":0.4783826396362969,"flip_rate":0.4928536376790605,"mean_run_length":2.0362661207513416,"persistence_baseline_accuracy":0.5071463623209395}}}},"results":{"y_dslow":{"h=25":{"M_base":{"AUC":0.5494241856765528,"Brier":0.24648226080192677,"accuracy@0.5":0.5448396775267218,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.6173502703057238,"Brier":0.2337428763142972,"accuracy@0.5":0.6099926793557833,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.6181352978781206,"Brier":0.2335063474167176,"accuracy@0.5":0.6110911420204978,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.6292057037133186,"Brier":0.22985225856831795,"accuracy@0.5":0.6206046859469256,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.6296781944909068,"Brier":0.2297050162386267,"accuracy@0.5":0.6207877013173499,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2}},"h=50":{"M_base":{"AUC":0.5467214658243486,"Brier":0.2466404325418991,"accuracy@0.5":0.5399999999999999,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.6167857872741184,"Brier":0.23384817004354162,"accuracy@0.5":0.6067299135863218,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.6174457033164523,"Brier":0.2336376510190808,"accuracy@0.5":0.6078323228534657,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.6279942684472376,"Brier":0.23044381915974774,"accuracy@0.5":0.6179444743511675,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.6283045664562134,"Brier":0.23031111870669943,"accuracy@0.5":0.618679168229454,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2}},"h=100":{"M_base":{"AUC":0.542777072624924,"Brier":0.24685247916840712,"accuracy@0.5":0.5364622378910744,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.6150014640337326,"Brier":0.23418306570909108,"accuracy@0.5":0.6047504175171609,"train_n":12706,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.615588370586695,"Brier":0.23397126472140257,"accuracy@0.5":0.6053074781963266,"train_n":12706,"test_n":5389,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.6262686368028037,"Brier":0.23113081634894104,"accuracy@0.5":0.6164392280571532,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.6265916668267423,"Brier":0.23100945399606867,"accuracy@0.5":0.616810540915568,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2}}},"y_lf":{"h=25":{"M_base":{"AUC":0.7851269838393038,"Brier":0.19696902726897735,"accuracy@0.5":0.7262071749633972,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.7758239884003162,"Brier":0.20230902709047426,"accuracy@0.5":0.7165087847730604,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.7766298131978014,"Brier":0.20177326377483545,"accuracy@0.5":0.7174245973645681,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.799238661361366,"Brier":0.19212407676368423,"accuracy@0.5":0.7316972906295755,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.7996107931021392,"Brier":0.19193345150894758,"accuracy@0.5":0.7316972906295755,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2}},"h=50":{"M_base":{"AUC":0.7846460739231401,"Brier":0.1978178086387841,"accuracy@0.5":0.7236624386835815,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.7753500434837993,"Brier":0.20248463534984966,"accuracy@0.5":0.7161242875535944,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.7761282060946047,"Brier":0.2019958376823402,"accuracy@0.5":0.717227431512997,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.7985438414463242,"Brier":0.19257514550971716,"accuracy@0.5":0.7302813024453024,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.7988792430038582,"Brier":0.19236209088322197,"accuracy@0.5":0.7297308321382607,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2}},"h=100":{"M_base":{"AUC":0.7828064601486488,"Brier":0.20033023864394248,"accuracy@0.5":0.7179447025412863,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.7729772396832553,"Brier":0.20385952290976114,"accuracy@0.5":0.7136752644271675,"train_n":12706,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.7739239107644188,"Brier":0.2031417076795024,"accuracy@0.5":0.7153466320282052,"train_n":12706,"test_n":5389,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.7993983670462191,"Brier":0.19188982291301832,"accuracy@0.5":0.7300055668951568,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.799780632773554,"Brier":0.19167462664138427,"accuracy@0.5":0.7305622564483206,"train_n":12706,"test_n":5389,"train_classes":2,"test_classes":2}}},"y_eps":{"h=25":{"M_base":{"AUC":0.4978093665940363,"Brier":0.2500414190543522,"accuracy@0.5":0.49890190336749635,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.501052783988327,"Brier":0.25012275514945217,"accuracy@0.5":0.49615556369033675,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.5015146497048635,"Brier":0.2500857057809831,"accuracy@0.5":0.49725695315153776,"train_n":12781,"test_n":5464,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.4993920714708492,"Brier":0.2500795254319202,"accuracy@0.5":0.4983526717557252,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.4995856139372037,"Brier":0.2500584147838379,"accuracy@0.5":0.4990841865756542,"train_n":12780,"test_n":5464,"train_classes":2,"test_classes":2}},"h=50":{"M_base":{"AUC":0.4971805409002972,"Brier":0.2501010966783644,"accuracy@0.5":0.4951292517006803,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.5003929793140951,"Brier":0.2500733335049924,"accuracy@0.5":0.5017466446037875,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.5007242499756143,"Brier":0.2500452236714243,"accuracy@0.5":0.5032175032175032,"train_n":12756,"test_n":5439,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.4980962280277271,"Brier":0.2500792764494718,"accuracy@0.5":0.4962316594962317,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.4982879043915355,"Brier":0.2500584494594474,"accuracy@0.5":0.4967838911567839,"train_n":12755,"test_n":5439,"train_classes":2,"test_classes":2}},"h=100":{"M_base":{"AUC":0.49802157978698326,"Brier":0.25011598055301566,"accuracy@0.5":0.4984227136752644,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase":{"AUC":0.4996556661685207,"Brier":0.2500966064841698,"accuracy@0.5":0.49229949990721915,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_phase_amp":{"AUC":0.49950221135486476,"Brier":0.250087741117256,"accuracy@0.5":0.49322731452959734,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_both":{"AUC":0.49863300997857655,"Brier":0.25010044843171296,"accuracy@0.5":0.49582482742622935,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2},"M_both_amp":{"AUC":0.4987490912956547,"Brier":0.25008898478331476,"accuracy@0.5":0.49582482742622935,"train_n":12705,"test_n":5389,"train_classes":2,"test_classes":2}}}}}

