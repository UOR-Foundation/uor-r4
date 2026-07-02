# Binned state target estimation

- **Conversation ID:** `699b367a-9a68-832e-bc9f-304348941f69`
- **Date:** 2026-02-22

---

### User (2026-02-22 12:01:51)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Estimate a binned state→target “gating surface” on TRAIN and TEST and quantify stationarity (distribution shift) while controlling for sample counts.

Use canonical pipeline and existing band definitions.
Do NOT recompute anchors.

Files:
- chebyshev_psi.npz
- primes.npz
- Protocol.md

Params:
L=65536
x_min=1e5
x_max=9e6
W=2000
Delta=0.01
train_frac=0.7 time-ordered
seed=12345
FAST=[0.002,0.01]
SLOW=[0.0002,0.002]
H=[25,100]

Binning:
B_theta = 12 bins for theta_f
B_omega = 10 bins for omega_f (use quantile bins on TRAIN)
(We start with FAST only to keep it stable and interpretable)
Also compute an optional SLOW add-on:
B_theta_s = 8 bins for theta_s (uniform)
(Report both FAST-only and FAST+SLOW if cheap)

Target:
y_h(i)=1{R_{i+h}-R_i>0} for h in H.

Steps:
1) Build theta_f, omega_f, theta_s (optional), omega_s (optional) using canonical pipeline.
2) Build R and y_h with train/test valid indices for each horizon.
3) Define bins:
   - theta bins uniform on [0,2π)
   - omega_f bins using TRAIN quantiles into B_omega equal-count bins (store edges)
   - theta_s bins uniform if used
4) For each horizon h:
   - For TRAIN-valid indices, compute empirical p_train[bin] = mean(y_h | bin) and count n_train[bin]
   - For TEST-valid indices, compute p_test[bin], n_test[bin]
5) Stationarity metrics:
   - Weighted RMSE between p_train and p_test across bins with n_test weights
   - Correlation corr(p_train, p_test) across bins (only bins with enough counts, e.g. n_test>=20)
   - Calibration transfer: if you predict p̂(i)=p_train[bin(i)] on TEST, compute:
       * AUC_bin
       * Brier_bin
       * logloss_bin (add eps=1e-6)
6) Also report marginal distribution shift:
   - KL divergence between TEST and TRAIN occupancy over bins (add smoothing 1 count)

Return JSON:
{
 "params": {...},
 "per_h": {
   "25": {
     "fast2d": {
        "bin_defs": {"theta_bins":12, "omega_bins":10, "omega_edges":[...]},
        "train": {"p":[...], "n":[...]},
        "test": {"p":[...], "n":[...]},
        "stationarity": {"wrmse":..., "corr":..., "kl_occupancy":...},
        "transfer_perf": {"auc":..., "brier":..., "logloss":...}
     },
     "optional_fast_slow3d": {... if computed ...}
   },
   "100": { ... }
 },
 "summary": {
   "is_gating_surface_stationary": {"25": true/false, "100": true/false},
   "notes": "..."
 }
}

PRINT JSON ONLY.

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

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

### Tool (2026-02-22 12:01:53)

Make sure to include fileciteturn0file7 in your response to cite this file, or to surface it as a link.
{
  "ts": "YYYY-MM-DDThh:mm:ssZ",
  "leader": "<name/window>",
  "experiment_id": "A3.xx",
  "question": "<what discriminating question is tested>",
  "inputs": [
    "<artifact paths>",
    "<params>"
  ],
  "method": "<one paragraph operational definition>",
  "results": {
    "metrics": {},
    "key_numbers": {},
    "plots": []
  },
  "interpretation": "<what it means / what it rules out>",
  "decision": {
    "status": [
      "supported",
      "rejected",
      "deprioritized"
    ],
    "next_candidates": [
      "..."
    ]
  },
  "artifacts_written": [
    "<paths>"
  ]
}

### Tool (2026-02-22 12:01:53)

Make sure to include fileciteturn0file8 in your response to cite this file, or to surface it as a link.
# Prime-Structure Project Chronicle (Leader-Readable)

## North Star
We are searching for a *mathematical structure* that (i) explains why primes occur where they do, and (ii) yields a compact, portable “equation-like” model for prime propensity or prime *events* without simply re-performing trial division.

The project has been operationalized as: learn/derive a smooth field/potential on integer-space (or on residue/phase coordinates) such that primes are enriched near *minima* (or other extrema/level-set boundaries), and the phenomenon is stable across scale after appropriate transport/renormalization.

## What this chronicle is
This is the “why we did what we did” log that prevents new leaders from re-deriving old anchors (ψ-spectrum, basic phase extraction) when resuming.

## Established pillars (do not redo unless falsifying)
1. **Log-space signal pipeline exists**: analyses were repeatedly anchored to log-space transforms and band-limited phase extraction (Hilbert/FFT masking) on ψ(x)-x style residual signals.
2. **Transport is real**: phase/density patterns shift with scale; naive stationary models fail. “Comoving” alignment (rotation/transport) improves stability.
3. **Minima enrichment is real but not iff**: primes are enriched near certain minima/boundary structures of a constructed smooth field; the claim is enrichment, not an iff characterization.
4. **Residue-context matters**: moduli built from prime products (e.g., 2310, 30030) produce clean harmonic structure and stable operator bases; smaller contexts (6, 30) help but are weaker.
5. **Single-axis explanations fail**: phase-only or pressure-only models do not explain all structure; interactions matter.

## Rejected or deprioritized branches (avoid re-traversal)
- Pure digit-sum / wheel-only analyses as primary drivers (they are residue structure; not sufficient alone).
- Toroidal spiral “story-first” geometry without quantitative invariants.
- Closed-form single-harmonic U(θ) as a complete model.
- Static (non-transported) density maps across wide ranges.

## Current frontier (what remained live at last checkpoint)
- Build portable potentials/operators on residue manifolds (esp. mod 2310 → 30030) using sparse harmonic bases.
- Use transport-aligned coordinates + arithmetic “pressure” proxies to define smooth U(n) with stable minima behavior.
- Determine whether adding a *global phase* coordinate (e.g., winding-phase ψ(n)=2π frac(W(n))) materially improves out-of-sample generalization, or whether it is redundant once transport is handled.

## Canonical artifacts
See ARTIFACT_INDEX.csv for filenames saved during the A2/A3/A3_99 series. Those JSON files contain per-step summaries, fitted coefficients, stability metrics, permutation tests, operator compressions, etc.

## Important meta-lesson
Leaders must carry *trajectory state*: what was falsified, what is live, and what counts as progress. Without this, new sessions regress to safe anchors (ψ-spectrum, basic phase extraction) and burn hours.

### Tool (2026-02-22 12:01:53)

Make sure to include fileciteturn0file9 in your response to cite this file, or to surface it as a link.
# Hypothesis Tree (Current)

## H0: “Primes are random beyond PNT”
Not adopted operationally; treated as null to beat.

## H1: “Primes correspond to extrema/minima of a smooth arithmetic potential U(n)”
**Status:** Live (enrichment demonstrated; iff not established)

### H1a: U depends only on a stationary phase coordinate θ
**Status:** Rejected (transport required; phase-only insufficient)

### H1b: U depends on (transport-corrected phase θ0, arithmetic pressure P)
**Status:** Supported (stable enrichment; improved generalization vs θ alone)

### H1c: U depends on (θ0, P, global winding-phase ψ)
**Status:** Live frontier (needs decisive held-out test)

## H2: “Residue harmonic operator basis on mod 2310/30030 yields portable equation”
**Status:** Supported as representation (operator compression works); predictive sufficiency still under test.

## H3: “Geometry-first torus/helix models explain primes”
**Status:** Deprioritized unless they produce quantitative invariants beyond H1/H2.

### Tool (2026-02-22 12:01:53)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 12:01:53)

import numpy as np, math, json, os, statistics, numpy.fft as fft
from pathlib import Path

# helpers
def causal_moving_average(x, W):
    # past-only including current, window size W
    x = np.asarray(x, dtype=float)
    c = np.cumsum(np.insert(x, 0, 0.0))
    out = np.empty_like(x)
    for i in range(len(x)):
        j0 = max(0, i - W + 1)
        out[i] = (c[i+1] - c[j0]) / (i - j0 + 1)
    return out

def fft_bandpass(x, f_lo, f_hi):
    # x real, length N
    x = np.asarray(x, float)
    N = len(x)
    X = fft.rfft(x)
    freqs = fft.rfftfreq(N, d=1.0)  # cycles/sample
    mask = (freqs >= f_lo) & (freqs <= f_hi)
    X_f = X * mask
    y = fft.irfft(X_f, n=N)
    return y

def hilbert_analytic(x):
    # build analytic signal via FFT (no scipy)
    x = np.asarray(x, float)
    N = len(x)
    X = fft.fft(x)
    h = np.zeros(N)
    if N % 2 == 0:
        h[0] = 1
        h[N//2] = 1
        h[1:N//2] = 2
    else:
        h[0] = 1
        h[1:(N+1)//2] = 2
    z = fft.ifft(X * h)
    return z

def wrap_to_2pi(theta):
    return np.mod(theta, 2*np.pi)

def wrap_to_pi(dtheta):
    return (dtheta + np.pi) % (2*np.pi) - np.pi

def auc_mann_whitney(y_true, y_score):
    y_true = np.asarray(y_true, int)
    y_score = np.asarray(y_score, float)
    pos = y_true == 1
    neg = y_true == 0
    n_pos = pos.sum()
    n_neg = neg.sum()
    if n_pos == 0 or n_neg == 0:
        return None
    # ranks with average ties
    order = np.argsort(y_score)
    ranks = np.empty_like(order, dtype=float)
    ranks[order] = np.arange(1, len(y_score)+1)
    # tie handling
    # compute average ranks for tied scores
    sorted_scores = y_score[order]
    i=0
    while i < len(sorted_scores):
        j=i+1
        while j < len(sorted_scores) and sorted_scores[j]==sorted_scores[i]:
            j+=1
        if j-i>1:
            avg = (i+1 + j)/2.0
            ranks[order[i:j]] = avg
        i=j
    sum_ranks_pos = ranks[pos].sum()
    U = sum_ranks_pos - n_pos*(n_pos+1)/2.0
    auc = U / (n_pos*n_neg)
    return float(auc)

def brier_score(y, p):
    y = np.asarray(y, float)
    p = np.asarray(p, float)
    return float(np.mean((p - y)**2))

def logloss(y, p, eps=1e-6):
    y = np.asarray(y, float)
    p = np.asarray(p, float)
    p = np.clip(p, eps, 1-eps)
    return float(-np.mean(y*np.log(p) + (1-y)*np.log(1-p)))

# params
L=65536
x_min=1e5
x_max=9e6
W=2000
Delta=0.01
train_frac=0.7
seed=12345
FAST=(0.002,0.01)
SLOW=(0.0002,0.002)
H=[25,100]
B_theta=12
B_omega=10
B_theta_s=8

np.random.seed(seed)

# load files
psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
psi_npz=np.load(psi_path)
x_grid=psi_npz["x_grid"].astype(float)
psi_minus_x=psi_npz["psi_minus_x"].astype(float)
pr_npz=np.load(primes_path, allow_pickle=True)
primes=pr_npz["primes"].astype(np.int64)

# build uniform log grid
t_raw=np.log(x_grid)
t0=float(t_raw.min()); t1=float(t_raw.max())
t_uniform=np.linspace(t0,t1,L)
dt=float(t_uniform[1]-t_uniform[0])
# interpolate psi_minus_x onto uniform t grid
u_interp=np.interp(t_uniform, t_raw, psi_minus_x)
x_uniform=np.exp(t_uniform)

# restrict kept x range
kept_mask=(x_uniform>=x_min) & (x_uniform<=x_max)
idx_kept=np.nonzero(kept_mask)[0]
u_raw=u_interp[idx_kept]
x_kept=x_uniform[idx_kept]
N=len(x_kept)

train_end=int(np.floor(train_frac*N))
# detrend causal
ma=causal_moving_average(u_raw, W)
u_detr=u_raw - ma
# standardize train only
mu=u_detr[:train_end].mean()
sd=u_detr[:train_end].std(ddof=0)
if sd==0: sd=1.0
u_std=(u_detr - mu)/sd

# bandpass fast and slow
u_fast=fft_bandpass(u_std, FAST[0], FAST[1])
u_slow=fft_bandpass(u_std, SLOW[0], SLOW[1])

# Hilbert
z_fast=hilbert_analytic(u_fast)
theta_f=wrap_to_2pi(np.angle(z_fast))
z_slow=hilbert_analytic(u_slow)
theta_s=wrap_to_2pi(np.angle(z_slow))

# omega from theta (wrap to pi)
omega_f=wrap_to_pi(np.diff(theta_f))
omega_s=wrap_to_pi(np.diff(theta_s))

# features aligned to omega indices 0..N-2
theta_f_al=theta_f[:-1]
theta_s_al=theta_s[:-1]
# omega already length N-1
Nf=len(omega_f)
train_end_feat=min(train_end, Nf)  # train_end <= N, but omega drops last
train_feat_idx=np.arange(train_end_feat)

# omega quantile bin edges on TRAIN
omega_train=omega_f[train_feat_idx]
qs=np.linspace(0,1,B_omega+1)
edges=np.quantile(omega_train, qs)
# ensure strictly increasing
if np.unique(edges).size < edges.size:
    mn=float(np.min(omega_train)); mx=float(np.max(omega_train))
    if mn==mx:
        edges=np.linspace(mn-1e-6, mx+1e-6, B_omega+1)
    else:
        edges=np.linspace(mn, mx, B_omega+1)
# make last edge slightly larger to include max with digitize right=False
edges[-1]+=1e-12

def bin_theta(theta, B):
    b = np.floor(theta/(2*np.pi)*B).astype(int)
    b = np.clip(b, 0, B-1)
    return b

theta_bin_f=bin_theta(theta_f_al, B_theta)
theta_bin_s=bin_theta(theta_s_al, B_theta_s)

omega_bin=np.digitize(omega_f, edges[1:-1], right=False)  # returns 0..B_omega-1

# compute R
# prime counts in [x, x*exp(Delta))
x_hi = x_kept * math.exp(Delta)
# searchsorted on primes
lo = np.searchsorted(primes, x_kept, side="left")
hi = np.searchsorted(primes, x_hi, side="left")
P = (hi - lo).astype(float)
E = x_kept*(math.exp(Delta)-1.0)/np.log(x_kept)
R = (P - E)/np.sqrt(E)

# build per horizon results
def gating_2d_for_h(h):
    # valid i: i in [0,N-2], i+h < N
    max_i = min(N-2, N-h-1)
    idx = np.arange(max_i+1)
    y = (R[idx+h] - R[idx] > 0).astype(int)
    # split by train/test with time-ordered on original indices
    train_mask = idx < train_end
    test_mask = idx >= train_end
    idx_tr = idx[train_mask]
    idx_te = idx[test_mask]
    # bin id for each i
    bin_id = theta_bin_f[idx]*B_omega + omega_bin[idx]
    nbins = B_theta*B_omega
    
    def stats_for(indices):
        n = np.bincount(bin_id[indices], minlength=nbins).astype(int)
        s = np.bincount(bin_id[indices], weights=y[indices], minlength=nbins)
        p = np.full(nbins, np.nan)
        with np.errstate(invalid="ignore", divide="ignore"):
            p = np.where(n>0, s/n, np.nan)
        return p, n
    
    p_tr, n_tr = stats_for(idx_tr)
    p_te, n_te = stats_for(idx_te)
    
    # stationarity metrics
    # wrmse weighted by n_test over bins where both have data (n_te>0 and n_tr>0)
    m = (n_te>0) & (n_tr>0)
    if m.any():
        w = n_te[m].astype(float)
        diff = (p_te[m]-p_tr[m]).astype(float)
        wrmse = float(np.sqrt(np.sum(w*diff*diff)/np.sum(w)))
    else:
        wrmse = None
    
    # corr over bins with enough counts
    m2 = (n_te>=20) & (n_tr>=20) & np.isfinite(p_te) & np.isfinite(p_tr)
    if m2.sum() >= 3:
        a = p_tr[m2].astype(float); b = p_te[m2].astype(float)
        corr = float(np.corrcoef(a,b)[0,1])
    else:
        corr = None
    
    # KL occupancy with smoothing 1 count
    pt = (n_te + 1).astype(float); pt /= pt.sum()
    pr = (n_tr + 1).astype(float); pr /= pr.sum()
    kl = float(np.sum(pt * np.log(pt/pr)))
    
    # transfer performance on TEST: predict p_hat = p_tr[bin]
    # For bins with n_tr==0, use overall train mean
    y_tr = y[idx_tr]
    base = float(y_tr.mean()) if len(y_tr)>0 else 0.5
    p_tr_filled = np.where(np.isfinite(p_tr), p_tr, base)
    p_hat = p_tr_filled[bin_id[idx_te]]
    y_te = y[idx_te]
    auc = auc_mann_whitney(y_te, p_hat)
    brier = brier_score(y_te, p_hat) if len(y_te)>0 else None
    ll = logloss(y_te, p_hat, eps=1e-6) if len(y_te)>0 else None
    
    return {
        "bin_defs": {
            "theta_bins": B_theta,
            "omega_bins": B_omega,
            "omega_edges": edges.tolist()
        },
        "train": {"p": [None if not np.isfinite(v) else float(v) for v in p_tr.tolist()],
                  "n": n_tr.tolist()},
        "test": {"p": [None if not np.isfinite(v) else float(v) for v in p_te.tolist()],
                 "n": n_te.tolist()},
        "stationarity": {"wrmse": wrmse, "corr": corr, "kl_occupancy": kl,
                         "bins_corr_min_count": 20},
        "transfer_perf": {"auc": auc, "brier": brier, "logloss": ll,
                          "test_pos_rate": float(y_te.mean()) if len(y_te)>0 else None,
                          "test_n": int(len(y_te))}
    }

def gating_3d_for_h(h):
    # bins: theta_f(12) x omega_f(10) x theta_s(8)
    max_i = min(N-2, N-h-1)
    idx = np.arange(max_i+1)
    y = (R[idx+h] - R[idx] > 0).astype(int)
    train_mask = idx < train_end
    test_mask = idx >= train_end
    idx_tr = idx[train_mask]
    idx_te = idx[test_mask]
    nbins = B_theta*B_omega*B_theta_s
    bin_id = (theta_bin_f[idx]*B_omega + omega_bin[idx])*B_theta_s + theta_bin_s[idx]
    
    def stats_for(indices):
        n = np.bincount(bin_id[indices], minlength=nbins).astype(int)
        s = np.bincount(bin_id[indices], weights=y[indices], minlength=nbins)
        p = np.where(n>0, s/n, np.nan)
        return p, n
    
    p_tr, n_tr = stats_for(idx_tr)
    p_te, n_te = stats_for(idx_te)
    
    m = (n_te>0) & (n_tr>0)
    wrmse = float(np.sqrt(np.sum(n_te[m]*(p_te[m]-p_tr[m])**2)/np.sum(n_te[m]))) if m.any() else None
    m2 = (n_te>=20) & (n_tr>=20) & np.isfinite(p_te) & np.isfinite(p_tr)
    corr = float(np.corrcoef(p_tr[m2], p_te[m2])[0,1]) if m2.sum()>=3 else None
    
    pt = (n_te+1).astype(float); pt/=pt.sum()
    pr = (n_tr+1).astype(float); pr/=pr.sum()
    kl = float(np.sum(pt*np.log(pt/pr)))
    
    y_tr = y[idx_tr]
    base = float(y_tr.mean()) if len(y_tr)>0 else 0.5
    p_tr_filled = np.where(np.isfinite(p_tr), p_tr, base)
    p_hat = p_tr_filled[bin_id[idx_te]]
    y_te = y[idx_te]
    auc = auc_mann_whitney(y_te, p_hat)
    brier = brier_score(y_te, p_hat) if len(y_te)>0 else None
    ll = logloss(y_te, p_hat, eps=1e-6) if len(y_te)>0 else None
    
    return {
        "bin_defs": {
            "theta_bins": B_theta,
            "omega_bins": B_omega,
            "theta_s_bins": B_theta_s,
            "omega_edges": edges.tolist()
        },
        "train": {"p": [None if not np.isfinite(v) else float(v) for v in p_tr.tolist()],
                  "n": n_tr.tolist()},
        "test": {"p": [None if not np.isfinite(v) else float(v) for v in p_te.tolist()],
                 "n": n_te.tolist()},
        "stationarity": {"wrmse": wrmse, "corr": corr, "kl_occupancy": kl,
                         "bins_corr_min_count": 20},
        "transfer_perf": {"auc": auc, "brier": brier, "logloss": ll,
                          "test_pos_rate": float(y_te.mean()) if len(y_te)>0 else None,
                          "test_n": int(len(y_te))}
    }

per_h={}
summary_stationary={}
notes=[]
for h in H:
    fast2d=gating_2d_for_h(h)
    opt3d=gating_3d_for_h(h)
    per_h[str(h)]={"fast2d": fast2d, "optional_fast_slow3d": opt3d}
    # heuristic stationarity: corr>=0.3 and wrmse<=0.1 and kl<=0.05 and transfer auc>0.52
    st = fast2d["stationarity"]
    tf = fast2d["transfer_perf"]
    ok = (st["corr"] is not None and st["wrmse"] is not None and
          st["corr"]>=0.3 and st["wrmse"]<=0.1 and st["kl_occupancy"]<=0.05 and
          tf["auc"] is not None and tf["auc"]>=0.52)
    summary_stationary[str(h)]=bool(ok)

notes.append("FAST 2D bins are theta_f (uniform) x omega_f (TRAIN-quantile). Features aligned to omega indices (drop last sample).")
notes.append("R uses prime counts in multiplicative window [x, x*exp(Delta)) with E≈x(exp(Delta)-1)/log x; target is sign(R_{i+h}-R_i).")
notes.append("Correlation computed over bins with n_train>=20 and n_test>=20; wrmse weighted by n_test over bins with n_train>0,n_test>0.")
notes.append("KL occupancy is KL(test||train) on smoothed (add-1) occupancy over bins.")
notes.append("AUC uses Mann-Whitney; bins with zero train count default to global train mean for transfer prediction.")

out={
    "params":{
        "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
        "FAST":list(FAST),"SLOW":list(SLOW),"H":H,
        "B_theta":B_theta,"B_omega":B_omega,"B_theta_s":B_theta_s,
        "dt":dt,"n_kept":N,"train_end":train_end
    },
    "per_h":per_h,
    "summary":{
        "is_gating_surface_stationary":summary_stationary,
        "notes":" ".join(notes)
    }
}

print(json.dumps(out, separators=(",",":")))

### Tool (2026-02-22 12:03:41)

,1.0,0.7083333333333334,null,1.0,0.0,null,1.0,null,1.0,0.7619047619047619,null,1.0,0.1111111111111111,1.0,1.0,null,1.0,0.8666666666666667,1.0,1.0,1.0,1.0,1.0,0.0,1.0,0.5714285714285714,1.0,0.3684210526315789,1.0,0.9166666666666666,0.40425531914893614,0.9230769230769231,1.0,null,0.5,0.9545454545454546,null,null,null,0.75,1.0,null,0.25,0.9333333333333333,null,0.7368421052631579,0.6666666666666666,0.8846153846153846,1.0,0.9230769230769231,null,1.0,0.6428571428571429,null,null,0.7575757575757576,1.0,0.4782608695652174,0.85,0.9333333333333333,0.0,0.3,null,0.4,1.0,0.631578947368421,1.0,null,1.0,0.45454545454545453,0.896551724137931,0.7692307692307693,0.9285714285714286,0.9166666666666666,0.9629629629629629,null,0.5777777777777777,null,0.0,1.0,null,1.0,1.0,null,0.7419354838709677,null,0.6363636363636364,1.0,1.0,0.25,1.0,0.6666666666666666,0.0,1.0,0.7931034482758621,0.0,0.7333333333333333,0.0,0.65,1.0,null,1.0,0.59375,0.0,0.9444444444444444,0.0,null,1.0,null,0.4666666666666667,0.5,0.7586206896551724,0.6363636363636364,0.6,0.5,0.8666666666666667,null,0.7142857142857143,0.0,0.8214285714285714,0.8181818181818182,1.0,0.0,null,null,0.16666666666666666,null,0.3333333333333333,1.0,0.6666666666666666,0.5333333333333333,1.0,0.5,null,null,0.9473684210526315,0.125,0.4375,0.6285714285714286,1.0,0.36,0.3157894736842105,null,1.0,1.0,0.14285714285714285,0.0,null,1.0,0.5277777777777778,0.46875,0.15789473684210525,0.9285714285714286,0.7560975609756098,0.9411764705882353,1.0,null,null,0.8,0.6538461538461539,1.0,0.18181818181818182,0.9130434782608695,0.15789473684210525,null,null,null,0.3125,0.76,0.0,0.9130434782608695,0.34615384615384615,0.8333333333333334,null,0.6956521739130435,0.84375,0.5789473684210527,0.0,0.5555555555555556,0.8333333333333334,0.9411764705882353,1.0,0.782608695652174,0.7142857142857143,0.0,0.25,1.0,0.46153846153846156,null,1.0,0.6153846153846154,0.8421052631578947,null,null,null,0.8,null,1.0,0.0,0.625,null,0.1111111111111111,0.5,0.9444444444444444,null,0.10526315789473684,1.0,null,0.14705882352941177,0.18181818181818182,0.7142857142857143,null,0.16666666666666666,null,0.5,0.6153846153846154,0.0,0.5625,0.45161290322580644,null,0.5789473684210527,0.4782608695652174,0.0,1.0,1.0,0.42857142857142855,0.03571428571428571,null,0.0,0.03333333333333333,0.4166666666666667,0.2,1.0,null,0.76,0.7272727272727273,0.45454545454545453,null,0.2222222222222222,null,null,null,0.9032258064516129,0.36363636363636365,0.125,null,null,0.8846153846153846,0.46153846153846156,null,1.0,0.0,0.6923076923076923,null,0.5,0.0,0.06451612903225806,null,1.0,0.0,0.5882352941176471,null,0.35714285714285715,0.75,0.13043478260869565,null,0.75,0.0,0.6666666666666666,null,0.23809523809523808,0.8076923076923077,null,0.3,0.5714285714285714,0.6666666666666666,null,0.625,0.0,0.6,null,0.5555555555555556,0.5555555555555556,0.7419354838709677,null,0.3333333333333333,0.44,0.65625,0.045454545454545456,null,0.1,0.7142857142857143,0.23529411764705882,0.0,null,0.03125,0.0,0.07142857142857142,null,null,0.8,0.08333333333333333,null,0.18181818181818182,0.125,0.1111111111111111,0.4090909090909091,null,0.5714285714285714,0.10714285714285714,0.0,0.0,null,0.9166666666666666,0.9423076923076923,0.6363636363636364,null,null,1.0,null,0.0,0.5882352941176471,0.5666666666666667,0.4,null,null,0.6666666666666666,0.37777777777777777,0.3888888888888889,0.3125,0.44,0.0,null,null,null,0.14285714285714285,0.5714285714285714,0.0,0.0,0.0,0.0,null,null,0.8461538461538461,0.5714285714285714,0.0,0.0,null,0.47368421052631576,null,null,0.47368421052631576,0.2,0.0,null,0.3684210526315789,0.4444444444444444,0.2,0.0,0.0,0.7333333333333333,null,0.0,0.28888888888888886,0.7857142857142857,0.3125,0.0,0.2833333333333333,0.38461538461538464,null,0.125,null,0.4838709677419355,0.0,null,0.0,0.46153846153846156,0.2,null,null,0.42857142857142855,0.21875,null,0.0,0.0,0.5,0.35714285714285715,null,1.0,0.07142857142857142,null,0.0,0.7045454545454546,0.7843137254901961,0.5789473684210527,null,0.5135135135135135,null,null,null,0.5789473684210527,0.3170731707317073,0.8333333333333334,1.0,0.4666666666666667,null,null,null,0.5,0.0,0.8421052631578947,null,0.5454545454545454,null,null,1.0,0.1,0.0,0.0,0.0,0.5454545454545454,null,0.0,null,0.13793103448275862,null,null,0.0,1.0,null,0.0,0.0,0.23076923076923078,0.0,1.0,0.022727272727272728,0.0,0.13043478260869565,0.0,0.0,0.4074074074074074,0.0,1.0,0.0,0.6363636363636364,0.4444444444444444,0.0,0.0425531914893617,0.0,0.0,0.058823529411764705,0.0,0.42857142857142855,0.0,0.0,0.0,0.0,0.21428571428571427,0.0,null,0.0,0.5882352941176471,0.23076923076923078,null,0.18181818181818182,0.9166666666666666,0.38461538461538464,null,1.0,0.2692307692307692,0.0,0.0,0.061224489795918366,0.3157894736842105,null,0.44,1.0,0.4634146341463415,null,null,0.125,0.0,null,0.4230769230769231,null,0.13333333333333333,null,null,0.35,0.0,null,0.0,0.0,0.0,0.0,null,0.06666666666666667,0.07142857142857142,0.2564102564102564,0.0,0.0,0.0,0.0,0.0,0.0,0.1,0.08,0.0,0.0,0.0,0.0,0.1875,0.06666666666666667,null,0.7692307692307693,0.6190476190476191,0.875,0.0,null,0.0,0.0,0.0,0.5,0.0,0.9230769230769231,0.0,null,0.26785714285714285,0.1111111111111111,0.0,0.17647058823529413,0.2857142857142857,0.23529411764705882,0.0,0.46153846153846156,0.0,0.1111111111111111,0.8571428571428571,0.0,0.5555555555555556,0.375,null,0.3333333333333333,0.35714285714285715,0.75,0.8888888888888888,0.8,null,1.0,0.6923076923076923,0.25,0.0,0.36363636363636365,null,null,0.05,1.0,null,null,null,0.7037037037037037,null,0.4,0.0,null,null,null,null,0.25,0.0,0.8846153846153846,0.0,0.42424242424242425,null,0.0,0.0,0.0,0.3333333333333333,0.75,0.0625,0.09090909090909091,null,null,0.05555555555555555,0.043478260869565216,0.4666666666666667,0.0,null,null,0.0,null,0.5,0.6111111111111112,0.0,0.5,0.696969696969697,null,0.1111111111111111,0.0,0.25,1.0,0.06451612903225806,null,0.7692307692307693,0.4473684210526316,0.15789473684210525,0.0,0.5882352941176471,null,0.0,0.5384615384615384,0.25,0.0,0.14285714285714285,0.0,0.0,0.26666666666666666,0.6428571428571429,null,0.0,0.5714285714285714,null,0.6,0.10344827586206896,null,0.8,0.2727272727272727,null,0.7777777777777778,0.0,0.6111111111111112,0.0,0.3333333333333333,0.42857142857142855,null,0.6739130434782609,1.0,null,null,null,0.631578947368421,0.6153846153846154,0.6153846153846154,0.5555555555555556,1.0,null,null,null,0.3424657534246575,0.3684210526315789,0.6,0.05,0.45454545454545453,null,null,null,0.23529411764705882,1.0,0.0,0.23255813953488372,0.38095238095238093,null,null,null,0.3684210526315789,1.0,0.26666666666666666,0.3,0.2571428571428571,null,null,0.0,0.21428571428571427,0.0,0.0,0.5333333333333333,0.16666666666666666,0.6666666666666666,1.0,0.0,0.8571428571428571,0.4444444444444444,null,null,0.1,0.391304347826087,0.625,0.0,0.8,null,null,0.47058823529411764,null,1.0,0.0,0.0,0.8666666666666667,0.4,null,null,0.8571428571428571,1.0,0.6129032258064516,0.3333333333333333,null,0.46153846153846156,0.5,null,0.9375,0.0,0.6071428571428571,0.10526315789473684,0.1111111111111111,0.34615384615384615,null,null,1.0,0.0,0.02631578947368421,null,0.5428571428571428,0.7777777777777778,1.0,0.43529411764705883,0.75,0.1111111111111111,0.5454545454545454,0.0,0.0,0.625,0.5,0.8333333333333334,0.0,0.0,1.0,0.8333333333333334,0.38461538461538464,0.8823529411764706,0.4,0.0975609756097561,0.4444444444444444,null,1.0,1.0,null,1.0,0.5172413793103449,0.7391304347826086,0.5428571428571428,0.5,1.0,0.8,0.6,1.0,null,0.5625,0.7272727272727273,0.65,null,null,null,0.2,null,null,0.5833333333333334,0.42857142857142855,0.6774193548387096,0.14285714285714285,0.9310344827586207,null,null,0.17647058823529413,null,null,0.3333333333333333,0.4,0.47368421052631576,0.0,0.6153846153846154,null,0.5,0.8421052631578947,1.0,0.18518518518518517,0.09090909090909091,0.5294117647058824,0.6666666666666666,1.0,1.0,0.0,0.7083333333333334,0.0],"n":[0,0,19,28,16,33,32,0,4,26,10,60,27,0,8,0,16,19,4,32,20,8,6,0,29,3,8,19,16,11,20,0,0,11,1,46,11,32,17,28,21,18,1,11,19,27,26,16,0,23,2,4,14,0,19,0,28,1,6,18,11,0,7,38,12,33,30,0,0,17,2,0,11,7,11,12,29,14,43,0,0,0,0,0,15,17,18,19,1,47,0,42,7,17,4,6,8,34,11,13,25,38,4,3,18,0,16,0,9,8,1,2,1,0,0,24,22,11,42,14,40,0,0,21,20,20,23,20,11,2,0,0,12,0,35,14,23,33,4,33,5,9,0,24,0,45,28,9,0,21,0,0,10,9,13,5,27,11,34,5,26,16,1,0,0,0,2,2,12,48,0,1,3,0,5,0,1,21,0,12,27,14,17,0,3,15,1,20,15,30,9,1,26,7,6,19,18,24,47,26,11,0,10,22,0,0,0,4,19,0,4,15,0,19,3,52,6,26,0,20,14,0,0,33,10,46,40,15,3,30,0,5,8,19,5,0,27,11,29,13,14,12,27,0,45,0,12,16,0,7,11,0,31,0,11,6,10,44,7,6,3,32,29,3,45,1,20,26,0,19,32,5,18,1,0,15,0,15,16,29,11,5,2,45,0,21,3,28,22,9,8,0,0,24,0,6,9,21,15,1,16,0,0,19,16,32,35,10,25,19,0,7,7,14,10,0,15,36,32,19,14,41,17,36,0,0,15,26,4,22,23,19,0,0,0,16,50,19,23,26,12,0,23,32,19,4,9,12,17,12,23,14,17,4,7,13,0,9,13,38,0,0,0,25,0,3,17,32,0,9,4,18,0,19,19,0,34,11,35,0,18,0,2,26,8,16,31,0,19,46,8,13,7,14,28,0,9,30,24,10,1,0,25,11,33,0,9,0,0,0,31,22,8,0,0,52,26,0,5,16,13,0,6,3,31,0,6,35,17,0,14,4,23,0,8,10,3,0,21,26,0,10,14,12,0,8,6,65,0,9,18,31,0,24,25,32,22,0,20,14,17,10,0,32,11,14,0,0,15,36,0,11,8,18,22,0,7,28,12,13,0,12,52,22,0,0,12,0,2,17,60,15,0,0,6,45,18,16,25,5,0,0,0,14,35,11,14,3,2,0,0,13,14,7,5,0,19,0,0,19,5,9,0,19,18,5,1,34,15,0,20,45,14,16,38,60,13,0,16,0,31,15,0,30,13,5,0,0,7,32,0,5,10,28,14,0,3,28,0,26,44,51,19,0,37,0,0,0,19,41,24,18,15,0,0,0,24,23,19,0,11,0,0,4,30,7,10,1,11,0,4,0,29,0,0,26,1,0,8,2,13,4,6,44,11,23,10,19,27,15,14,9,11,9,17,47,5,9,17,4,49,7,12,42,4,28,3,0,6,17,13,0,11,12,13,0,1,26,2,17,49,19,0,50,2,41,0,0,24,16,0,26,0,15,0,0,20,10,0,4,7,7,29,0,45,14,39,15,3,9,9,8,5,10,25,25,6,9,3,16,15,0,13,21,8,22,0,1,2,18,4,4,26,1,0,56,9,17,17,7,17,17,26,18,9,14,9,9,24,0,21,28,8,9,10,0,7,13,16,7,44,0,0,20,4,0,0,0,27,0,5,5,0,0,0,0,36,7,26,9,33,0,5,8,56,9,20,32,11,0,0,18,23,15,29,0,0,3,0,6,18,3,16,33,0,18,4,24,5,31,0,13,38,19,6,17,0,4,13,4,6,28,25,17,15,14,0,5,21,0,15,29,0,10,11,0,9,7,36,9,12,21,0,46,18,0,0,0,19,13,52,27,6,0,0,0,73,38,15,20,11,0,0,0,17,5,6,43,21,0,0,0,19,6,45,10,35,0,0,6,14,4,3,45,12,21,6,27,7,18,0,0,20,23,8,9,20,0,0,17,0,15,17,8,15,15,0,0,14,1,31,24,0,13,20,0,16,8,28,19,9,26,0,0,15,20,38,0,70,9,3,85,4,9,11,2,10,8,4,6,3,1,4,6,26,17,10,41,36,0,5,6,0,14,29,23,35,4,5,25,20,12,0,32,11,20,0,0,0,10,0,0,12,14,31,7,29,0,0,17,0,0,6,15,19,9,13,0,4,19,26,27,11,17,15,7,33,10,24,1]},"test":{"p":[null,null,null,null,null,0.11538461538461539,null,null,null,1.0,null,null,null,0.0,null,null,null,0.0,null,null,null,0.3,null,null,null,0.1111111111111111,null,null,null,0.0,null,0.0,null,0.8333333333333334,null,null,null,null,0.5217391304347826,null,null,null,null,null,null,null,null,null,null,1.0,null,0.8333333333333334,null,null,null,0.8571428571428571,null,1.0,null,0.7083333333333334,null,null,null,0.13043478260869565,0.2857142857142857,null,null,0.4634146341463415,0.8,1.0,null,null,0.8,1.0,1.0,0.8235294117647058,0.3157894736842105,0.9629629629629629,1.0,null,null,1.0,null,null,null,0.625,null,null,null,1.0,null,null,null,0.25,null,null,null,null,0.6666666666666666,null,null,0.9090909090909091,null,0.0,null,null,1.0,null,null,0.8333333333333334,1.0,0.47058823529411764,null,null,null,null,null,0.5,null,0.0,null,1.0,1.0,0.8888888888888888,null,1.0,null,null,null,1.0,null,0.9285714285714286,null,0.0,null,0.21052631578947367,null,null,null,0.6666666666666666,0.7,0.8888888888888888,null,1.0,1.0,null,null,0.5714285714285714,0.4,null,null,null,0.7241379310344828,0.25,1.0,1.0,0.6923076923076923,0.926829268292683,0.35714285714285715,null,null,null,null,0.5,null,0.5714285714285714,null,null,null,null,null,1.0,null,null,null,1.0,null,null,null,1.0,1.0,null,null,1.0,null,null,0.8333333333333334,0.5833333333333334,1.0,1.0,1.0,1.0,null,null,0.7222222222222222,1.0,1.0,1.0,1.0,1.0,null,null,0.5384615384615384,1.0,1.0,1.0,1.0,1.0,1.0,null,0.625,1.0,0.2,0.42857142857142855,null,1.0,1.0,null,1.0,1.0,null,1.0,null,0.25,0.42857142857142855,null,null,0.5862068965517241,1.0,0.3888888888888889,1.0,null,0.7916666666666666,0.0,0.42857142857142855,1.0,0.9523809523809523,0.8888888888888888,0.38461538461538464,0.0,null,null,null,0.08333333333333333,null,1.0,null,null,1.0,null,null,0.75,null,1.0,null,0.7931034482758621,null,null,1.0,0.8095238095238095,1.0,0.0,null,1.0,1.0,null,1.0,0.9444444444444444,0.16666666666666666,1.0,null,1.0,0.2,null,null,1.0,null,1.0,null,null,0.8333333333333334,null,null,1.0,null,1.0,1.0,null,1.0,null,0.0,null,0.5,1.0,null,null,null,null,0.8181818181818182,null,0.1,1.0,null,0.6153846153846154,null,null,0.7333333333333333,0.625,0.9,0.59375,0.9375,0.3333333333333333,1.0,null,0.043478260869565216,0.2222222222222222,0.38461538461538464,0.6086956521739131,null,0.0,1.0,0.875,null,1.0,null,null,null,null,1.0,0.5,null,0.9772727272727273,null,0.07142857142857142,null,null,1.0,0.6,1.0,0.6666666666666666,null,0.5555555555555556,null,null,1.0,0.3333333333333333,0.045454545454545456,0.0,0.18181818181818182,1.0,null,1.0,1.0,null,null,0.0,0.0,1.0,null,0.6153846153846154,null,null,null,0.0,0.0,null,0.9523809523809523,0.0,null,null,null,0.4,null,null,null,null,null,null,null,0.2,0.23529411764705882,1.0,0.6428571428571429,null,null,null,0.75,0.5714285714285714,0.0,0.38095238095238093,1.0,0.0,0.5,0.3333333333333333,0.02631578947368421,null,0.07692307692307693,0.0,null,0.0,null,null,null,1.0,0.3157894736842105,null,null,null,null,null,null,1.0,1.0,1.0,null,null,null,null,null,0.875,1.0,1.0,null,null,null,0.0,null,1.0,0.8,1.0,null,null,1.0,0.2,0.16666666666666666,1.0,0.3,null,null,null,1.0,0.14285714285714285,0.875,1.0,0.4444444444444444,null,0.45,0.047619047619047616,0.1,0.0,null,1.0,0.5714285714285714,0.0,1.0,null,null,null,null,0.4166666666666667,0.14285714285714285,0.8235294117647058,0.8181818181818182,null,0.0,null,null,0.25,0.0,0.6,0.5555555555555556,0.07692307692307693,0.0,0.36363636363636365,0.11627906976744186,null,0.0,0.0,null,0.18181818181818182,null,null,null,null,0.9375,null,null,null,null,null,null,null,0.8,null,null,null,null,null,null,null,1.0,null,null,null,null,null,null,null,1.0,0.8,null,null,null,null,null,null,1.0,0.07692307692307693,null,null,0.0,null,null,1.0,0.2,0.0,0.2857142857142857,0.0,0.0,0.0,null,0.8125,0.32142857142857145,0.17647058823529413,1.0,0.0,0.0,0.0,null,null,0.0,null,0.058823529411764705,null,0.0,0.0,null,0.0,0.0,null,null,null,0.0,0.6153846153846154,0.0,null,0.24324324324324326,0.6666666666666666,0.1111111111111111,1.0,0.0,null,null,null,null,null,null,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,0.0,null,0.0,null,null,null,null,null,0.0,null,0.15789473684210525,null,null,null,null,0.2857142857142857,0.14285714285714285,null,null,null,null,0.0,null,null,0.25,0.0,0.0,0.125,null,0.0,0.0,null,0.0,null,null,0.20833333333333334,null,0.0,0.4,null,null,null,0.3333333333333333,1.0,0.02040816326530612,null,0.55,0.0,0.5,0.875,0.21875,null,null,null,null,null,null,null,0.0,null,null,null,null,null,null,null,0.5,null,null,null,null,null,0.0,null,0.5384615384615384,null,null,null,null,null,0.0,null,0.3076923076923077,null,null,null,null,null,0.0,null,0.5,null,null,null,null,null,0.5789473684210527,null,null,null,null,null,null,null,null,0.0,null,0.0,null,0.5,0.1875,null,0.0,0.0,null,0.8,null,0.0,0.0,null,null,null,0.5,1.0,0.27450980392156865,0.0,0.9523809523809523,0.4444444444444444,1.0,0.8571428571428571,null,null,null,null,null,0.5,null,null,null,null,null,null,null,0.0,0.0,null,null,null,null,null,null,null,0.09803921568627451,null,null,null,null,null,null,null,0.043478260869565216,null,null,null,null,null,null,null,0.0,null,0.5,0.375,null,null,null,null,null,null,null,0.0,null,null,1.0,null,null,null,null,0.0,null,1.0,0.9375,null,0.0,0.0,null,0.9285714285714286,0.041666666666666664,0.4418604651162791,0.3,null,null,null,1.0,1.0,0.5833333333333334,0.72,0.7142857142857143,1.0,1.0,1.0,null,null,null,null,null,0.9166666666666666,0.058823529411764705,null,null,null,null,null,null,null,0.0,null,null,null,null,null,null,0.0,0.1724137931034483,0.0,null,null,null,null,null,0.0,0.0,null,null,null,null,null,0.5,0.0,0.0,null,null,null,null,null,0.0,null,null,null,null,0.3,null,null,0.6363636363636364,null,null,null,null,0.0,0.5555555555555556,null,null,null,0.06666666666666667,0.4,null,0.7,0.3888888888888889,0.8627450980392157,0.4838709677419355,null,null,1.0,1.0,1.0,1.0,0.96,0.35714285714285715,1.0,0.8,null,null,null,null,null,0.8571428571428571,1.0,null,null,null,null,null,null,1.0,null,null,null,null,null,null,null,1.0,null,null,0.0,null,null,null,null,0.5,0.1,null,0.0,null,0.6666666666666666,null,null,1.0,0.07142857142857142,0.4782608695652174,0.5714285714285714,null,0.1111111111111111,null,null,null,null,null,1.0,null,null,null,null,null,null,null,1.0,null,0.5882352941176471,0.2,0.7692307692307693,null,null,null,0.35294117647058826,0.9090909090909091,1.0,null,0.7727272727272727,0.8571428571428571,1.0,null,0.75,1.0,1.0,0.95,0.7142857142857143,0.375,1.0,0.0,null],"n":[0,0,0,0,0,26,0,0,0,4,0,0,0,21,0,0,0,8,0,0,0,10,0,0,0,9,0,0,0,2,0,3,0,6,0,0,0,0,23,0,0,0,0,0,0,0,0,0,0,17,0,12,0,0,0,14,0,2,0,24,0,0,0,23,14,0,0,41,15,11,0,0,10,4,32,17,19,27,8,0,0,22,0,0,0,8,0,0,0,22,0,0,0,16,0,0,0,0,6,0,0,11,0,4,0,0,3,0,0,6,5,17,0,0,0,0,0,2,0,5,0,4,5,9,0,3,0,0,0,10,0,28,0,5,0,19,0,0,0,3,10,9,0,17,2,0,0,42,5,0,0,0,29,4,20,23,13,41,14,0,0,0,0,22,0,7,0,0,0,0,0,4,0,0,0,32,0,0,0,1,2,0,0,10,0,0,6,12,4,2,3,4,0,0,18,14,3,2,16,7,0,0,13,17,5,3,4,10,1,0,8,14,5,7,0,1,8,0,4,6,0,7,0,16,7,0,0,29,7,18,9,0,24,4,14,13,21,36,13,2,0,0,0,12,0,8,0,0,18,0,0,4,0,3,0,29,0,0,8,21,2,2,0,21,2,0,19,18,6,1,0,7,5,0,0,8,0,2,0,0,6,0,0,2,0,4,21,0,8,0,1,0,8,5,0,0,0,0,11,0,10,6,0,13,0,0,15,24,10,32,16,3,23,0,23,9,13,23,0,11,24,8,0,14,0,0,0,0,15,8,0,44,0,14,0,0,14,5,5,6,0,9,0,0,11,3,22,2,11,6,0,9,6,0,0,3,8,3,0,13,0,0,0,2,5,0,21,2,0,0,0,5,0,0,0,0,0,0,0,5,17,10,14,0,0,0,8,28,14,21,2,16,18,6,38,0,13,24,0,11,0,0,0,15,19,0,0,0,0,0,0,3,4,15,0,0,0,0,0,8,3,12,0,0,0,2,0,3,10,7,0,0,1,5,12,2,10,0,0,0,9,7,8,4,18,0,20,21,10,7,0,4,21,1,8,0,0,0,0,12,7,17,11,0,3,0,0,24,9,5,9,13,1,22,43,0,19,21,0,11,0,0,0,0,16,0,0,0,0,0,0,0,5,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,3,5,0,0,0,0,0,0,4,13,0,0,13,0,0,3,5,5,21,14,8,4,0,16,28,17,13,1,10,13,0,0,6,0,17,0,14,8,0,30,8,0,0,0,1,13,47,0,37,9,9,9,9,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,2,0,0,0,0,0,16,0,19,0,0,0,0,14,28,0,0,0,0,4,0,0,4,19,16,8,0,14,35,0,17,0,0,24,0,29,5,0,0,0,9,8,49,0,40,5,6,8,32,0,0,0,0,0,0,0,12,0,0,0,0,0,0,0,10,0,0,0,0,0,7,0,13,0,0,0,0,0,33,0,13,0,0,0,0,0,13,0,4,0,0,0,0,0,19,0,0,0,0,0,0,0,0,15,0,17,0,16,32,0,18,3,0,30,0,30,14,0,0,0,2,5,51,3,21,9,5,7,0,0,0,0,0,10,0,0,0,0,0,0,0,5,2,0,0,0,0,0,0,0,51,0,0,0,0,0,0,0,23,0,0,0,0,0,0,0,9,0,4,8,0,0,0,0,0,0,0,8,0,0,2,0,0,0,0,17,0,3,16,0,17,18,0,28,24,43,30,0,0,0,5,5,12,25,14,10,6,2,0,0,0,0,0,12,34,0,0,0,0,0,0,0,19,0,0,0,0,0,0,5,29,8,0,0,0,0,0,7,25,0,0,0,0,0,2,13,6,0,0,0,0,0,6,0,0,0,0,20,0,0,11,0,0,0,0,17,9,0,0,0,15,10,0,20,18,51,31,0,0,9,8,8,12,25,14,12,5,0,0,0,0,0,14,2,0,0,0,0,0,0,4,0,0,0,0,0,0,0,3,0,0,6,0,0,0,0,4,10,0,22,0,3,0,0,2,14,23,7,0,18,0,0,0,0,0,8,0,0,0,0,0,0,0,6,0,17,15,13,0,0,0,17,11,8,0,44,28,2,0,4,7,9,20,28,16,14,7,0]},"stationarity":{"wrmse":0.4443336678448998,"corr":0.5718309395639642,"kl_occupancy":1.3526224313428297,"bins_corr_min_count":20},"transfer_perf":{"auc":0.6180429413342986,"brier":0.29083777857567406,"logloss":2.021639612904234,"test_pos_rate":0.5004639079606606,"test_n":5389}}}},"summary":{"is_gating_surface_stationary":{"25":false,"100":false},"notes":"FAST 2D bins are theta_f (uniform) x omega_f (TRAIN-quantile). Features aligned to omega indices (drop last sample). R uses prime counts in multiplicative window [x, x*exp(Delta)) with E\u2248x(exp(Delta)-1)/log x; target is sign(R_{i+h}-R_i). Correlation computed over bins with n_train>=20 and n_test>=20; wrmse weighted by n_test over bins with n_train>0,n_test>0. KL occupancy is KL(test||train) on smoothed (add-1) occupancy over bins. AUC uses Mann-Whitney; bins with zero train count default to global train mean for transfer prediction."}}
/tmp/ipykernel_242/3931246871.py:289: RuntimeWarning: invalid value encountered in divide
  p = np.where(n>0, s/n, np.nan)

### Assistant (2026-02-22 12:03:44)

import numpy as np, math, json, numpy.fft as fft

def causal_moving_average(x, W):
    x = np.asarray(x, dtype=float)
    c = np.cumsum(np.insert(x, 0, 0.0))
    out = np.empty_like(x)
    for i in range(len(x)):
        j0 = max(0, i - W + 1)
        out[i] = (c[i+1] - c[j0]) / (i - j0 + 1)
    return out

def fft_bandpass(x, f_lo, f_hi):
    x = np.asarray(x, float)
    N = len(x)
    X = fft.rfft(x)
    freqs = fft.rfftfreq(N, d=1.0)
    mask = (freqs >= f_lo) & (freqs <= f_hi)
    return fft.irfft(X * mask, n=N)

def hilbert_analytic(x):
    x = np.asarray(x, float)
    N = len(x)
    X = fft.fft(x)
    h = np.zeros(N)
    if N % 2 == 0:
        h[0] = 1; h[N//2] = 1; h[1:N//2] = 2
    else:
        h[0] = 1; h[1:(N+1)//2] = 2
    return fft.ifft(X * h)

def wrap_to_2pi(theta): return np.mod(theta, 2*np.pi)
def wrap_to_pi(dtheta): return (dtheta + np.pi) % (2*np.pi) - np.pi

def auc_mw(y_true, y_score):
    y_true = np.asarray(y_true, int)
    y_score = np.asarray(y_score, float)
    pos = y_true == 1
    neg = y_true == 0
    n_pos = int(pos.sum()); n_neg = int(neg.sum())
    if n_pos == 0 or n_neg == 0: return None
    order = np.argsort(y_score)
    ranks = np.empty_like(order, dtype=float)
    ranks[order] = np.arange(1, len(y_score)+1)
    sorted_scores = y_score[order]
    i=0
    while i < len(sorted_scores):
        j=i+1
        while j < len(sorted_scores) and sorted_scores[j]==sorted_scores[i]:
            j+=1
        if j-i>1:
            avg = (i+1 + j)/2.0
            ranks[order[i:j]] = avg
        i=j
    sum_ranks_pos = ranks[pos].sum()
    U = sum_ranks_pos - n_pos*(n_pos+1)/2.0
    return float(U/(n_pos*n_neg))

def brier(y,p): 
    y=np.asarray(y,float); p=np.asarray(p,float); 
    return float(np.mean((p-y)**2))

def logloss(y,p,eps=1e-6):
    y=np.asarray(y,float); p=np.asarray(p,float)
    p=np.clip(p,eps,1-eps)
    return float(-np.mean(y*np.log(p)+(1-y)*np.log(1-p)))

def bin_theta(theta,B):
    b=np.floor(theta/(2*np.pi)*B).astype(int)
    return np.clip(b,0,B-1)

# params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); H=[25,100]; B_theta=12; B_omega=10
np.random.seed(seed)

# load
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"].astype(float)
psi_minus_x=psi["psi_minus_x"].astype(float)
pr=np.load("/mnt/data/primes.npz", allow_pickle=True)
primes=pr["primes"].astype(np.int64)

# uniform log grid
t_raw=np.log(x_grid)
t_uniform=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform, t_raw, psi_minus_x)
x_uniform=np.exp(t_uniform)

kept=(x_uniform>=x_min)&(x_uniform<=x_max)
idx=np.nonzero(kept)[0]
u_raw=u_interp[idx]
x_kept=x_uniform[idx]
N=len(x_kept)
train_end=int(np.floor(train_frac*N))

# detrend + standardize train
u_detr=u_raw - causal_moving_average(u_raw, W)
mu=float(u_detr[:train_end].mean())
sd=float(u_detr[:train_end].std(ddof=0)) or 1.0
u_std=(u_detr-mu)/sd

# fast band + phase
u_fast=fft_bandpass(u_std, FAST[0], FAST[1])
theta_f=wrap_to_2pi(np.angle(hilbert_analytic(u_fast)))
omega_f=wrap_to_pi(np.diff(theta_f))
theta_f_al=theta_f[:-1]
Nf=len(omega_f)
train_end_feat=min(train_end, Nf)

# omega quantile bins
omega_train=omega_f[:train_end_feat]
qs=np.linspace(0,1,B_omega+1)
edges=np.quantile(omega_train, qs)
if np.unique(edges).size < edges.size:
    mn=float(np.min(omega_train)); mx=float(np.max(omega_train))
    if mn==mx:
        edges=np.linspace(mn-1e-6, mx+1e-6, B_omega+1)
    else:
        edges=np.linspace(mn, mx, B_omega+1)
edges[-1]+=1e-12
omega_bin=np.digitize(omega_f, edges[1:-1], right=False)
theta_bin=bin_theta(theta_f_al, B_theta)

# R
x_hi=x_kept*math.exp(Delta)
lo=np.searchsorted(primes, x_kept, side="left")
hi=np.searchsorted(primes, x_hi, side="left")
P=(hi-lo).astype(float)
E=x_kept*(math.exp(Delta)-1.0)/np.log(x_kept)
R=(P-E)/np.sqrt(E)

def per_h(h):
    max_i=min(N-2, N-h-1)
    idx=np.arange(max_i+1)
    y=((R[idx+h]-R[idx])>0).astype(int)
    train_idx=idx[idx<train_end]
    test_idx=idx[idx>=train_end]
    bin_id=theta_bin[idx]*B_omega + omega_bin[idx]
    nb=B_theta*B_omega
    
    def agg(indices):
        n=np.bincount(bin_id[indices], minlength=nb).astype(int)
        s=np.bincount(bin_id[indices], weights=y[indices], minlength=nb)
        p=np.where(n>0, s/n, np.nan)
        return p,n
    
    p_tr,n_tr=agg(train_idx)
    p_te,n_te=agg(test_idx)
    
    m=(n_te>0)&(n_tr>0)
    wrmse=float(np.sqrt(np.sum(n_te[m]*(p_te[m]-p_tr[m])**2)/np.sum(n_te[m]))) if m.any() else None
    m2=(n_te>=20)&(n_tr>=20)&np.isfinite(p_te)&np.isfinite(p_tr)
    corr=float(np.corrcoef(p_tr[m2], p_te[m2])[0,1]) if m2.sum()>=3 else None
    
    pt=(n_te+1).astype(float); pt/=pt.sum()
    pr=(n_tr+1).astype(float); pr/=pr.sum()
    kl=float(np.sum(pt*np.log(pt/pr)))
    
    base=float(y[train_idx].mean()) if len(train_idx)>0 else 0.5
    p_tr_fill=np.where(np.isfinite(p_tr), p_tr, base)
    y_te=y[test_idx]
    p_hat=p_tr_fill[bin_id[test_idx]]
    auc=auc_mw(y_te,p_hat)
    return {
        "bin_defs":{"theta_bins":B_theta,"omega_bins":B_omega,"omega_edges":edges.tolist()},
        "train":{"p":[None if not np.isfinite(v) else float(v) for v in p_tr.tolist()],
                 "n":n_tr.tolist()},
        "test":{"p":[None if not np.isfinite(v) else float(v) for v in p_te.tolist()],
                "n":n_te.tolist()},
        "stationarity":{"wrmse":wrmse,"corr":corr,"kl_occupancy":kl,"bins_corr_min_count":20},
        "transfer_perf":{
            "auc":auc,
            "brier":brier(y_te,p_hat),
            "logloss":logloss(y_te,p_hat),
            "test_pos_rate":float(y_te.mean()),
            "test_n":int(len(y_te))
        }
    }

per={}
station={}
for h in H:
    per[str(h)]={"fast2d":per_h(h)}
    st=per[str(h)]["fast2d"]["stationarity"]; tf=per[str(h)]["fast2d"]["transfer_perf"]
    ok=(st["corr"] is not None and st["wrmse"] is not None and st["corr"]>=0.3 and st["wrmse"]<=0.1 and st["kl_occupancy"]<=0.05 and tf["auc"] is not None and tf["auc"]>=0.52)
    station[str(h)]=bool(ok)

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"H":H,
    "B_theta":B_theta,"B_omega":B_omega,
    "dt":dt,"n_kept":N,"train_end":train_end
 },
 "per_h":per,
 "summary":{
    "is_gating_surface_stationary":station,
    "notes":"FAST-only 2D gating surface: bins are theta_f (uniform on [0,2π)) x omega_f (TRAIN-quantile). Features aligned to omega indices (drop last sample). Target y_h=1{R_{i+h}-R_i>0}. Corr computed on bins with n_train,n_test>=20. wrmse weighted by n_test over bins with n_train>0,n_test>0. KL is KL(test||train) on add-1 smoothed occupancy. Transfer uses p_train[bin] (empty train bins filled with global train mean)."
 }
}
print(json.dumps(out, separators=(",",":")))

### Tool (2026-02-22 12:04:30)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"Delta":0.01,"train_frac":0.7,"seed":12345,"FAST":[0.002,0.01],"H":[25,100],"B_theta":12,"B_omega":10,"dt":0.00024594622340595577,"n_kept":18296,"train_end":12807},"per_h":{"25":{"fast2d":{"bin_defs":{"theta_bins":12,"omega_bins":10,"omega_edges":[-1.5097045113632008,0.007807996076745207,0.014883707312126227,0.018543755181326917,0.0215848263961413,0.023938065966597755,0.02635101125164301,0.029291129026974085,0.03238885973890709,0.037830725860585446,1.234156282848586]},"train":{"p":[0.5078125,0.5333333333333333,0.5904761904761905,0.5283018867924528,0.4178082191780822,0.60431654676259,0.6774193548387096,0.6605504587155964,0.43617021276595747,0.5905511811023622,0.4492753623188406,0.24193548387096775,0.375,0.7407407407407407,0.5350877192982456,0.5,0.4594594594594595,0.7404580152671756,0.6796116504854369,0.5350877192982456,0.2765957446808511,0.7681159420289855,0.5434782608695652,0.8191489361702128,0.5260115606936416,0.574468085106383,0.49107142857142855,0.8686868686868687,0.610738255033557,0.7053571428571429,0.2777777777777778,0.6363636363636364,0.5895522388059702,0.8378378378378378,0.723404255319149,0.7217391304347827,0.4782608695652174,0.7160493827160493,0.6805555555555556,0.6691729323308271,0.5302013422818792,0.2976190476190476,0.6216216216216216,0.8545454545454545,0.7029702970297029,0.8051948051948052,0.7971014492753623,0.5793650793650794,0.6879432624113475,0.6885245901639344,0.45569620253164556,0.48672566371681414,0.5652173913043478,0.308411214953271,0.5714285714285714,0.7130434782608696,0.6762589928057554,0.6086956521739131,0.6896551724137931,0.5740740740740741,0.6326530612244898,0.5517241379310345,0.1794871794871795,0.21794871794871795,0.359375,0.5494505494505495,0.5913461538461539,0.4857142857142857,0.5483870967741935,0.3853211009174312,0.7417218543046358,0.42735042735042733,0.49382716049382713,0.5714285714285714,0.3939393939393939,0.3384615384615385,0.40268456375838924,0.4689655172413793,0.18309859154929578,0.21951219512195122,0.8074534161490683,0.7654320987654321,0.4805194805194805,0.4225352112676056,0.29292929292929293,0.3625,0.3153153153153153,0.359375,0.2982456140350877,0.34285714285714286,0.3235294117647059,0.4864864864864865,0.2661290322580645,0.21232876712328766,0.07894736842105263,0.33620689655172414,0.31007751937984496,0.18556701030927836,0.08080808080808081,0.3170731707317073,0.7938144329896907,0.5726495726495726,0.5668789808917197,0.3804347826086957,0.4214876033057851,0.2196969696969697,0.4,0.2857142857142857,0.15,0.17307692307692307,0.2777777777777778,0.5595854922279793,0.47619047619047616,0.46808510638297873,0.34074074074074073,0.22105263157894736,0.44594594594594594,0.1044776119402985,0.3247863247863248,0.2457627118644068],"n":[128,135,105,106,146,139,62,109,94,127,69,124,136,54,114,144,74,131,103,114,47,69,92,94,173,47,112,99,149,112,126,66,134,148,94,115,69,81,144,133,149,84,185,110,101,77,69,126,141,122,79,113,69,107,91,115,139,115,87,108,98,145,78,78,64,91,208,105,62,109,151,117,81,63,66,130,149,145,71,82,161,81,77,142,99,80,111,128,114,70,68,37,124,146,76,116,129,97,99,82,97,117,157,92,121,132,85,77,100,104,108,193,42,141,135,95,74,67,117,118]},"test":{"p":[0.0,0.4,0.8947368421052632,0.7837837837837838,0.7931034482758621,null,0.6046511627906976,0.3877551020408163,0.6172839506172839,0.5555555555555556,0.43333333333333335,0.2894736842105263,0.2549019607843137,0.5161290322580645,0.5714285714285714,0.6666666666666666,0.3870967741935484,0.9487179487179487,0.46938775510204084,0.7777777777777778,0.0,0.4722222222222222,0.35294117647058826,0.6451612903225806,0.55,0.6153846153846154,0.75,0.926829268292683,0.8428571428571429,0.8661417322834646,0.25,0.2962962962962963,0.8148148148148148,0.6415094339622641,0.4666666666666667,0.5151515151515151,0.7272727272727273,0.975,0.91,0.8823529411764706,0.5434782608695652,0.5061728395061729,0.5641025641025641,0.78125,0.696969696969697,0.5666666666666667,1.0,0.6304347826086957,0.6853932584269663,0.6,0.38235294117647056,0.6363636363636364,0.8695652173913043,0.9545454545454546,0.9,0.7701149425287356,0.7647058823529411,0.8936170212765957,0.7142857142857143,0.42735042735042733,0.0,0.0,0.6666666666666666,0.75,0.4117647058823529,0.5245901639344263,0.7241379310344828,0.6086956521739131,0.5666666666666667,0.344,0.5555555555555556,0.5,null,0.0,0.16666666666666666,0.2459016393442623,0.0,0.2111111111111111,0.3448275862068966,0.4,0.46875,0.16666666666666666,0.6470588235294118,0.32608695652173914,0.5384615384615384,0.7391304347826086,0.0,0.12790697674418605,0.17567567567567569,0.24271844660194175,0.2,0.0,0.8431372549019608,0.391304347826087,0.5555555555555556,0.6666666666666666,0.2,0.09859154929577464,0.208,0.08860759493670886,0.7608695652173914,0.3684210526315789,0.21428571428571427,0.0,0.2857142857142857,1.0,0.3548387096774194,0.19607843137254902,0.32558139534883723,0.2261904761904762,0.8125,1.0,0.3333333333333333,0.4444444444444444,0.46938775510204084,0.11538461538461539,0.0,0.3387096774193548,0.4742268041237113,0.43564356435643564],"n":[26,25,19,37,29,0,43,49,81,117,30,38,51,31,7,21,62,39,49,144,29,36,34,31,60,52,36,41,70,127,20,54,54,53,15,33,22,40,100,102,46,81,39,64,33,30,5,46,89,110,34,22,23,22,30,87,51,47,63,117,16,5,3,8,17,61,87,46,60,125,9,2,0,2,18,61,27,90,58,125,32,12,17,46,26,23,15,86,74,103,10,7,51,23,9,12,10,71,125,79,46,19,42,32,21,6,31,51,129,84,16,4,9,36,49,26,6,62,97,101]},"stationarity":{"wrmse":0.20584231405381986,"corr":0.5660161172425494,"kl_occupancy":0.27079470242602915,"bins_corr_min_count":20},"transfer_perf":{"auc":0.6796688066028027,"brier":0.22873889244320594,"logloss":0.654424142794802,"test_pos_rate":0.5065885797950219,"test_n":5464}}},"100":{"fast2d":{"bin_defs":{"theta_bins":12,"omega_bins":10,"omega_edges":[-1.5097045113632008,0.007807996076745207,0.014883707312126227,0.018543755181326917,0.0215848263961413,0.023938065966597755,0.02635101125164301,0.029291129026974085,0.03238885973890709,0.037830725860585446,1.234156282848586]},"train":{"p":[0.3515625,0.5555555555555556,0.5523809523809524,0.6886792452830188,0.821917808219178,0.6906474820143885,0.7580645161290323,0.8899082568807339,0.6382978723404256,0.9291338582677166,0.5217391304347826,0.6854838709677419,0.5882352941176471,0.8518518518518519,0.7368421052631579,0.6805555555555556,0.6621621621621622,0.8473282442748091,0.6407766990291263,0.7456140350877193,0.5106382978723404,0.7536231884057971,0.6847826086956522,0.9680851063829787,0.7283236994219653,0.851063829787234,0.8571428571428571,0.8484848484848485,0.610738255033557,0.8303571428571429,0.7301587301587301,0.8181818181818182,0.6492537313432836,0.7432432432432432,0.7340425531914894,0.7652173913043478,0.4782608695652174,0.7160493827160493,0.4861111111111111,0.45864661654135336,0.8389261744966443,0.44047619047619047,0.654054054054054,0.7454545454545455,0.6237623762376238,0.5584415584415584,0.5942028985507246,0.4126984126984127,0.48936170212765956,0.22131147540983606,0.569620253164557,0.7345132743362832,0.42028985507246375,0.24299065420560748,0.4065934065934066,0.5478260869565217,0.5611510791366906,0.1565217391304348,0.20689655172413793,0.17592592592592593,0.8775510204081632,0.4896551724137931,0.32051282051282054,0.3974358974358974,0.40625,0.18681318681318682,0.27403846153846156,0.20952380952380953,0.27419354838709675,0.22018348623853212,0.6688741721854304,0.5897435897435898,0.4691358024691358,0.14285714285714285,0.07575757575757576,0.1,0.2550335570469799,0.15172413793103448,0.2676056338028169,0.3170731707317073,0.32298136645962733,0.19753086419753085,0.09090909090909091,0.09859154929577464,0.06060606060606061,0.3875,0.36936936936936937,0.171875,0.38596491228070173,0.6,0.3088235294117647,0.5675675675675675,0.3709677419354839,0.1506849315068493,0.14473684210526316,0.43103448275862066,0.3643410852713178,0.12371134020618557,0.37373737373737376,0.4878048780487805,0.6391752577319587,0.6239316239316239,0.34394904458598724,0.29347826086956524,0.30578512396694213,0.3712121212121212,0.35294117647058826,0.5064935064935064,0.59,0.4807692307692308,0.24074074074074073,0.49222797927461137,0.5,0.425531914893617,0.6814814814814815,0.6631578947368421,0.5,0.5671641791044776,0.5641025641025641,0.652542372881356],"n":[128,135,105,106,146,139,62,109,94,127,69,124,136,54,114,144,74,131,103,114,47,69,92,94,173,47,112,99,149,112,126,66,134,148,94,115,69,81,144,133,149,84,185,110,101,77,69,126,141,122,79,113,69,107,91,115,139,115,87,108,98,145,78,78,64,91,208,105,62,109,151,117,81,63,66,130,149,145,71,82,161,81,77,142,99,80,111,128,114,70,68,37,124,146,76,116,129,97,99,82,97,117,157,92,121,132,85,77,100,104,108,193,42,141,135,95,74,67,117,118]},"test":{"p":[0.11538461538461539,0.16,0.16666666666666666,0.07142857142857142,0.5862068965517241,null,0.9069767441860465,0.4489795918367347,0.5679012345679012,0.8376068376068376,0.9,0.6842105263157895,0.6666666666666666,0.6774193548387096,0.14285714285714285,0.9523809523809523,0.6451612903225806,0.8717948717948718,0.5714285714285714,0.8125,0.5172413793103449,1.0,1.0,0.8064516129032258,0.9166666666666666,0.8846153846153846,0.6944444444444444,0.7073170731707317,0.6142857142857143,0.7480314960629921,0.45,0.8703703703703703,0.8888888888888888,0.8867924528301887,0.7333333333333333,0.9696969696969697,0.7727272727272727,0.6,0.7,0.4411764705882353,0.9782608695652174,0.7777777777777778,0.7948717948717948,0.46875,0.5151515151515151,0.6666666666666666,0.4,0.5217391304347826,0.3595505617977528,0.11818181818181818,0.6176470588235294,1.0,0.9565217391304348,0.8181818181818182,0.3,0.4482758620689655,0.49019607843137253,0.6170212765957447,0.23809523809523808,0.1282051282051282,0.9375,0.8,1.0,0.875,0.29411764705882354,0.16393442622950818,0.4367816091954023,0.021739130434782608,0.0,0.264,0.0,0.0,null,0.0,0.0,0.18032786885245902,0.037037037037037035,0.011111111111111112,0.1206896551724138,0.352,0.21875,0.0,0.29411764705882354,0.15217391304347827,0.15384615384615385,0.5652173913043478,0.0,0.16279069767441862,0.32432432432432434,0.5339805825242718,0.5,0.0,0.09803921568627451,0.043478260869565216,0.0,0.4166666666666667,0.2,0.2535211267605634,0.44,0.7974683544303798,0.2826086956521739,0.0,0.11904761904761904,0.0,0.047619047619047616,0.0,0.41935483870967744,0.19607843137254902,0.689922480620155,0.8690476190476191,0.875,1.0,0.3333333333333333,0.08333333333333333,0.40816326530612246,0.38461538461538464,1.0,0.46774193548387094,0.8350515463917526,0.7425742574257426],"n":[26,25,18,14,29,0,43,49,81,117,30,38,21,31,7,21,62,39,49,144,29,36,13,31,60,52,36,41,70,127,20,54,54,53,15,33,22,40,100,102,46,81,39,64,33,30,5,46,89,110,34,22,23,22,30,87,51,47,63,117,16,5,3,8,17,61,87,46,60,125,9,2,0,2,18,61,27,90,58,125,32,12,17,46,26,23,15,86,74,103,10,7,51,23,9,12,10,71,125,79,46,19,42,32,21,6,31,51,129,84,16,4,9,36,49,26,6,62,97,101]},"stationarity":{"wrmse":0.20162398901501624,"corr":0.7109813001493706,"kl_occupancy":0.28172958664579617,"bins_corr_min_count":20},"transfer_perf":{"auc":0.7438411839471627,"brier":0.2053577175970684,"logloss":0.6017308601005579,"test_pos_rate":0.5004639079606606,"test_n":5389}}}},"summary":{"is_gating_surface_stationary":{"25":false,"100":false},"notes":"FAST-only 2D gating surface: bins are theta_f (uniform on [0,2\u03c0)) x omega_f (TRAIN-quantile). Features aligned to omega indices (drop last sample). Target y_h=1{R_{i+h}-R_i>0}. Corr computed on bins with n_train,n_test>=20. wrmse weighted by n_test over bins with n_train>0,n_test>0. KL is KL(test||train) on add-1 smoothed occupancy. Transfer uses p_train[bin] (empty train bins filled with global train mean)."}}
/tmp/ipykernel_242/4015599387.py:145: RuntimeWarning: invalid value encountered in divide
  p=np.where(n>0, s/n, np.nan)

### Assistant (2026-02-22 12:04:33)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"Delta":0.01,"train_frac":0.7,"seed":12345,"FAST":[0.002,0.01],"H":[25,100],"B_theta":12,"B_omega":10,"dt":0.00024594622340595577,"n_kept":18296,"train_end":12807},"per_h":{"25":{"fast2d":{"bin_defs":{"theta_bins":12,"omega_bins":10,"omega_edges":[-1.5097045113632008,0.007807996076745207,0.014883707312126227,0.018543755181326917,0.0215848263961413,0.023938065966597755,0.02635101125164301,0.029291129026974085,0.03238885973890709,0.037830725860585446,1.234156282848586]},"train":{"p":[0.5078125,0.5333333333333333,0.5904761904761905,0.5283018867924528,0.4178082191780822,0.60431654676259,0.6774193548387096,0.6605504587155964,0.43617021276595747,0.5905511811023622,0.4492753623188406,0.24193548387096775,0.375,0.7407407407407407,0.5350877192982456,0.5,0.4594594594594595,0.7404580152671756,0.6796116504854369,0.5350877192982456,0.2765957446808511,0.7681159420289855,0.5434782608695652,0.8191489361702128,0.5260115606936416,0.574468085106383,0.49107142857142855,0.8686868686868687,0.610738255033557,0.7053571428571429,0.2777777777777778,0.6363636363636364,0.5895522388059702,0.8378378378378378,0.723404255319149,0.7217391304347827,0.4782608695652174,0.7160493827160493,0.6805555555555556,0.6691729323308271,0.5302013422818792,0.2976190476190476,0.6216216216216216,0.8545454545454545,0.7029702970297029,0.8051948051948052,0.7971014492753623,0.5793650793650794,0.6879432624113475,0.6885245901639344,0.45569620253164556,0.48672566371681414,0.5652173913043478,0.308411214953271,0.5714285714285714,0.7130434782608696,0.6762589928057554,0.6086956521739131,0.6896551724137931,0.5740740740740741,0.6326530612244898,0.5517241379310345,0.1794871794871795,0.21794871794871795,0.359375,0.5494505494505495,0.5913461538461539,0.4857142857142857,0.5483870967741935,0.3853211009174312,0.7417218543046358,0.42735042735042733,0.49382716049382713,0.5714285714285714,0.3939393939393939,0.3384615384615385,0.40268456375838924,0.4689655172413793,0.18309859154929578,0.21951219512195122,0.8074534161490683,0.7654320987654321,0.4805194805194805,0.4225352112676056,0.29292929292929293,0.3625,0.3153153153153153,0.359375,0.2982456140350877,0.34285714285714286,0.3235294117647059,0.4864864864864865,0.2661290322580645,0.21232876712328766,0.07894736842105263,0.33620689655172414,0.31007751937984496,0.18556701030927836,0.08080808080808081,0.3170731707317073,0.7938144329896907,0.5726495726495726,0.5668789808917197,0.3804347826086957,0.4214876033057851,0.2196969696969697,0.4,0.2857142857142857,0.15,0.17307692307692307,0.2777777777777778,0.5595854922279793,0.47619047619047616,0.46808510638297873,0.34074074074074073,0.22105263157894736,0.44594594594594594,0.1044776119402985,0.3247863247863248,0.2457627118644068],"n":[128,135,105,106,146,139,62,109,94,127,69,124,136,54,114,144,74,131,103,114,47,69,92,94,173,47,112,99,149,112,126,66,134,148,94,115,69,81,144,133,149,84,185,110,101,77,69,126,141,122,79,113,69,107,91,115,139,115,87,108,98,145,78,78,64,91,208,105,62,109,151,117,81,63,66,130,149,145,71,82,161,81,77,142,99,80,111,128,114,70,68,37,124,146,76,116,129,97,99,82,97,117,157,92,121,132,85,77,100,104,108,193,42,141,135,95,74,67,117,118]},"test":{"p":[0.0,0.4,0.8947368421052632,0.7837837837837838,0.7931034482758621,null,0.6046511627906976,0.3877551020408163,0.6172839506172839,0.5555555555555556,0.43333333333333335,0.2894736842105263,0.2549019607843137,0.5161290322580645,0.5714285714285714,0.6666666666666666,0.3870967741935484,0.9487179487179487,0.46938775510204084,0.7777777777777778,0.0,0.4722222222222222,0.35294117647058826,0.6451612903225806,0.55,0.6153846153846154,0.75,0.926829268292683,0.8428571428571429,0.8661417322834646,0.25,0.2962962962962963,0.8148148148148148,0.6415094339622641,0.4666666666666667,0.5151515151515151,0.7272727272727273,0.975,0.91,0.8823529411764706,0.5434782608695652,0.5061728395061729,0.5641025641025641,0.78125,0.696969696969697,0.5666666666666667,1.0,0.6304347826086957,0.6853932584269663,0.6,0.38235294117647056,0.6363636363636364,0.8695652173913043,0.9545454545454546,0.9,0.7701149425287356,0.7647058823529411,0.8936170212765957,0.7142857142857143,0.42735042735042733,0.0,0.0,0.6666666666666666,0.75,0.4117647058823529,0.5245901639344263,0.7241379310344828,0.6086956521739131,0.5666666666666667,0.344,0.5555555555555556,0.5,null,0.0,0.16666666666666666,0.2459016393442623,0.0,0.2111111111111111,0.3448275862068966,0.4,0.46875,0.16666666666666666,0.6470588235294118,0.32608695652173914,0.5384615384615384,0.7391304347826086,0.0,0.12790697674418605,0.17567567567567569,0.24271844660194175,0.2,0.0,0.8431372549019608,0.391304347826087,0.5555555555555556,0.6666666666666666,0.2,0.09859154929577464,0.208,0.08860759493670886,0.7608695652173914,0.3684210526315789,0.21428571428571427,0.0,0.2857142857142857,1.0,0.3548387096774194,0.19607843137254902,0.32558139534883723,0.2261904761904762,0.8125,1.0,0.3333333333333333,0.4444444444444444,0.46938775510204084,0.11538461538461539,0.0,0.3387096774193548,0.4742268041237113,0.43564356435643564],"n":[26,25,19,37,29,0,43,49,81,117,30,38,51,31,7,21,62,39,49,144,29,36,34,31,60,52,36,41,70,127,20,54,54,53,15,33,22,40,100,102,46,81,39,64,33,30,5,46,89,110,34,22,23,22,30,87,51,47,63,117,16,5,3,8,17,61,87,46,60,125,9,2,0,2,18,61,27,90,58,125,32,12,17,46,26,23,15,86,74,103,10,7,51,23,9,12,10,71,125,79,46,19,42,32,21,6,31,51,129,84,16,4,9,36,49,26,6,62,97,101]},"stationarity":{"wrmse":0.20584231405381986,"corr":0.5660161172425494,"kl_occupancy":0.27079470242602915,"bins_corr_min_count":20},"transfer_perf":{"auc":0.6796688066028027,"brier":0.22873889244320594,"logloss":0.654424142794802,"test_pos_rate":0.5065885797950219,"test_n":5464}}},"100":{"fast2d":{"bin_defs":{"theta_bins":12,"omega_bins":10,"omega_edges":[-1.5097045113632008,0.007807996076745207,0.014883707312126227,0.018543755181326917,0.0215848263961413,0.023938065966597755,0.02635101125164301,0.029291129026974085,0.03238885973890709,0.037830725860585446,1.234156282848586]},"train":{"p":[0.3515625,0.5555555555555556,0.5523809523809524,0.6886792452830188,0.821917808219178,0.6906474820143885,0.7580645161290323,0.8899082568807339,0.6382978723404256,0.9291338582677166,0.5217391304347826,0.6854838709677419,0.5882352941176471,0.8518518518518519,0.7368421052631579,0.6805555555555556,0.6621621621621622,0.8473282442748091,0.6407766990291263,0.7456140350877193,0.5106382978723404,0.7536231884057971,0.6847826086956522,0.9680851063829787,0.7283236994219653,0.851063829787234,0.8571428571428571,0.8484848484848485,0.610738255033557,0.8303571428571429,0.7301587301587301,0.8181818181818182,0.6492537313432836,0.7432432432432432,0.7340425531914894,0.7652173913043478,0.4782608695652174,0.7160493827160493,0.4861111111111111,0.45864661654135336,0.8389261744966443,0.44047619047619047,0.654054054054054,0.7454545454545455,0.6237623762376238,0.5584415584415584,0.5942028985507246,0.4126984126984127,0.48936170212765956,0.22131147540983606,0.569620253164557,0.7345132743362832,0.42028985507246375,0.24299065420560748,0.4065934065934066,0.5478260869565217,0.5611510791366906,0.1565217391304348,0.20689655172413793,0.17592592592592593,0.8775510204081632,0.4896551724137931,0.32051282051282054,0.3974358974358974,0.40625,0.18681318681318682,0.27403846153846156,0.20952380952380953,0.27419354838709675,0.22018348623853212,0.6688741721854304,0.5897435897435898,0.4691358024691358,0.14285714285714285,0.07575757575757576,0.1,0.2550335570469799,0.15172413793103448,0.2676056338028169,0.3170731707317073,0.32298136645962733,0.19753086419753085,0.09090909090909091,0.09859154929577464,0.06060606060606061,0.3875,0.36936936936936937,0.171875,0.38596491228070173,0.6,0.3088235294117647,0.5675675675675675,0.3709677419354839,0.1506849315068493,0.14473684210526316,0.43103448275862066,0.3643410852713178,0.12371134020618557,0.37373737373737376,0.4878048780487805,0.6391752577319587,0.6239316239316239,0.34394904458598724,0.29347826086956524,0.30578512396694213,0.3712121212121212,0.35294117647058826,0.5064935064935064,0.59,0.4807692307692308,0.24074074074074073,0.49222797927461137,0.5,0.425531914893617,0.6814814814814815,0.6631578947368421,0.5,0.5671641791044776,0.5641025641025641,0.652542372881356],"n":[128,135,105,106,146,139,62,109,94,127,69,124,136,54,114,144,74,131,103,114,47,69,92,94,173,47,112,99,149,112,126,66,134,148,94,115,69,81,144,133,149,84,185,110,101,77,69,126,141,122,79,113,69,107,91,115,139,115,87,108,98,145,78,78,64,91,208,105,62,109,151,117,81,63,66,130,149,145,71,82,161,81,77,142,99,80,111,128,114,70,68,37,124,146,76,116,129,97,99,82,97,117,157,92,121,132,85,77,100,104,108,193,42,141,135,95,74,67,117,118]},"test":{"p":[0.11538461538461539,0.16,0.16666666666666666,0.07142857142857142,0.5862068965517241,null,0.9069767441860465,0.4489795918367347,0.5679012345679012,0.8376068376068376,0.9,0.6842105263157895,0.6666666666666666,0.6774193548387096,0.14285714285714285,0.9523809523809523,0.6451612903225806,0.8717948717948718,0.5714285714285714,0.8125,0.5172413793103449,1.0,1.0,0.8064516129032258,0.9166666666666666,0.8846153846153846,0.6944444444444444,0.7073170731707317,0.6142857142857143,0.7480314960629921,0.45,0.8703703703703703,0.8888888888888888,0.8867924528301887,0.7333333333333333,0.9696969696969697,0.7727272727272727,0.6,0.7,0.4411764705882353,0.9782608695652174,0.7777777777777778,0.7948717948717948,0.46875,0.5151515151515151,0.6666666666666666,0.4,0.5217391304347826,0.3595505617977528,0.11818181818181818,0.6176470588235294,1.0,0.9565217391304348,0.8181818181818182,0.3,0.4482758620689655,0.49019607843137253,0.6170212765957447,0.23809523809523808,0.1282051282051282,0.9375,0.8,1.0,0.875,0.29411764705882354,0.16393442622950818,0.4367816091954023,0.021739130434782608,0.0,0.264,0.0,0.0,null,0.0,0.0,0.18032786885245902,0.037037037037037035,0.011111111111111112,0.1206896551724138,0.352,0.21875,0.0,0.29411764705882354,0.15217391304347827,0.15384615384615385,0.5652173913043478,0.0,0.16279069767441862,0.32432432432432434,0.5339805825242718,0.5,0.0,0.09803921568627451,0.043478260869565216,0.0,0.4166666666666667,0.2,0.2535211267605634,0.44,0.7974683544303798,0.2826086956521739,0.0,0.11904761904761904,0.0,0.047619047619047616,0.0,0.41935483870967744,0.19607843137254902,0.689922480620155,0.8690476190476191,0.875,1.0,0.3333333333333333,0.08333333333333333,0.40816326530612246,0.38461538461538464,1.0,0.46774193548387094,0.8350515463917526,0.7425742574257426],"n":[26,25,18,14,29,0,43,49,81,117,30,38,21,31,7,21,62,39,49,144,29,36,13,31,60,52,36,41,70,127,20,54,54,53,15,33,22,40,100,102,46,81,39,64,33,30,5,46,89,110,34,22,23,22,30,87,51,47,63,117,16,5,3,8,17,61,87,46,60,125,9,2,0,2,18,61,27,90,58,125,32,12,17,46,26,23,15,86,74,103,10,7,51,23,9,12,10,71,125,79,46,19,42,32,21,6,31,51,129,84,16,4,9,36,49,26,6,62,97,101]},"stationarity":{"wrmse":0.20162398901501624,"corr":0.7109813001493706,"kl_occupancy":0.28172958664579617,"bins_corr_min_count":20},"transfer_perf":{"auc":0.7438411839471627,"brier":0.2053577175970684,"logloss":0.6017308601005579,"test_pos_rate":0.5004639079606606,"test_n":5389}}}},"summary":{"is_gating_surface_stationary":{"25":false,"100":false},"notes":"FAST-only 2D gating surface: bins are theta_f (uniform on [0,2π)) x omega_f (TRAIN-quantile). Features aligned to omega indices (drop last sample). Target y_h=1{R_{i+h}-R_i>0}. Corr computed on bins with n_train,n_test>=20. wrmse weighted by n_test over bins with n_train>0,n_test>0. KL is KL(test||train) on add-1 smoothed occupancy. Transfer uses p_train[bin] (empty train bins filled with global train mean)."}}

