# Phase Model Comparison

- **Conversation ID:** `699af4ff-cfe0-8325-94c3-2945c43fb2e5`
- **Date:** 2026-02-22

---

### User (2026-02-22 07:22:30)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Test whether a deterministic global winding phase adds incremental predictive value beyond transport-corrected Hilbert phase + pressure terms.

Do NOT redesign the canonical pipeline.
Do NOT recompute historical peak anchors.
Use the existing dominant frequency stored in prime_phase_research_snapshot.json.

Available project files:
- chebyshev_psi.npz
- primes.npz
- prime_phase_research_snapshot.json
- Protocol.md

Canonical parameters:
L=65536
x_min=1e5
x_max=9e6
W=2000 (causal detrend)
bandpass=[0.002,0.01] cycles/sample
Delta=0.01
train_frac=0.7 (time ordered)
horizons H=[25,50,100]
seed=12345

STEP 1 — Build canonical phase features (do once)
- Load chebyshev_psi.npz (x_grid, psi_minus_x)
- Build uniform log grid length L
- Restrict x to [x_min,x_max]
- Causal detrend W
- Standardize using TRAIN only
- FFT bandpass [0.002,0.01]
- Hilbert analytic signal → theta ∈ [0,2π), logA

STEP 2 — Build prime residual targets
- Load primes.npz
- Compute P_i and expected E_i using multiplicative window Delta
- R_i = (P_i - E_i)/sqrt(E_i)
- De-stickied target:
    y(h) = 1{ R_{i+h} - R_i > 0 }
Ensure TRAIN-valid thresholding and time ordering.

STEP 3 — Build deterministic global winding phase
- Load prime_phase_research_snapshot.json
- Extract peak_frequency_cycles_per_sample = f0
- Define psi_wind(i) = 2π * frac(f0 * i)

Do NOT re-estimate f0.

STEP 4 — Model comparison (logistic regression)

Baselines:
- Persistence baseline
- Logistic on [R_i, R_{i-1}, R_{i-2}]

Phase models:
M0: Fourier logistic K=3 on theta only
M1: pressure only (logA + R lags)
M2: theta + pressure
M3: theta + pressure + psi_wind (sin/cos terms K=2)

Use TRAIN to fit.
Evaluate on TEST only.

For each horizon h in H compute:
- AUC
- Brier
- Accuracy@0.5
- Positive rate
- Flip rate
- Mean run length

STEP 5 — Incremental lift
Compute:
- ΔAUC_M3_vs_M2
- ΔBrier_M3_vs_M2
- ΔAUC_M2_vs_laggedR_baseline

Return JSON structured exactly as:

{
  "params": {...},
  "horizon_results": {
     "25": {
        "baselines": {...},
        "M0": {...},
        "M1": {...},
        "M2": {...},
        "M3": {...},
        "delta": {
            "auc_M3_minus_M2": ...,
            "brier_M3_minus_M2": ...
        }
     },
     ...
  },
  "summary": {
      "mean_delta_auc": ...,
      "mean_delta_brier": ...,
      "evidence_for_global_winding": true/false
  }
}

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

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

### Tool (2026-02-22 07:22:32)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 07:22:32)

import json, numpy as np, math, os, pandas as pd
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score

# Load data
cheb = np.load('/mnt/data/chebyshev_psi.npz')
x_grid = cheb['x_grid']
psi_minus_x = cheb['psi_minus_x']

pr = np.load('/mnt/data/primes.npz')
primes = pr['primes']

with open('/mnt/data/prime_phase_research_snapshot.json','r') as f:
    snapshot = json.load(f)
f0 = snapshot['key_findings'][-1]['evidence']['peak_frequency_cycles_per_sample'] if 'key_findings' in snapshot else snapshot.get('peak_frequency_cycles_per_sample')
f0

### Tool (2026-02-22 07:23:30)

0.0079957

### Assistant (2026-02-22 07:23:34)

import numpy as np, math
L=65536
x_min=1e5
x_max=9e6
W=2000
bandpass=(0.002,0.01)
Delta=0.01
train_frac=0.7
H=[25,50,100]
seed=12345
np.random.seed(seed)

t_raw = np.log(x_grid.astype(float))
t_uniform = np.linspace(t_raw.min(), t_raw.max(), L)
dt = t_uniform[1]-t_uniform[0]
# interpolate psi_minus_x onto uniform t
u_uniform = np.interp(t_uniform, t_raw, psi_minus_x.astype(float))
x_uniform = np.exp(t_uniform)
kept = np.where((x_uniform>=x_min) & (x_uniform<=x_max))[0]
t = t_uniform[kept]
x = x_uniform[kept]
u_raw = u_uniform[kept]
N = len(kept)
train_end = int(np.floor(train_frac*N))
N, train_end, dt

### Tool (2026-02-22 07:23:39)

(18296, 12807, 0.00024594622340595577)

### Assistant (2026-02-22 07:23:41)

def causal_moving_average(x, W):
    # past-only including current, variable window near start
    c = np.cumsum(np.insert(x,0,0.0))
    out = np.empty_like(x, dtype=float)
    for i in range(len(x)):
        j0 = max(0, i-W+1)
        out[i] = (c[i+1]-c[j0])/(i-j0+1)
    return out

u_ma = causal_moving_average(u_raw, W)
u_det = u_raw - u_ma

mu = u_det[:train_end].mean()
sd = u_det[:train_end].std(ddof=0)
u_std = (u_det - mu) / (sd + 1e-12)

# FFT bandpass on full kept
def fft_bandpass_real(x, f_lo, f_hi):
    n=len(x)
    X=np.fft.rfft(x)
    freqs=np.fft.rfftfreq(n, d=1.0)  # cycles/sample since sample spacing=1
    mask = (freqs>=f_lo) & (freqs<=f_hi)
    X_f = X*mask
    y = np.fft.irfft(X_f, n=n)
    return y

u_bp = fft_bandpass_real(u_std, bandpass[0], bandpass[1])

# Hilbert analytic signal via FFT (no scipy)
def analytic_signal(x):
    n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1
        h[n//2]=1
        h[1:n//2]=2
    else:
        h[0]=1
        h[1:(n+1)//2]=2
    z=np.fft.ifft(X*h)
    return z

z = analytic_signal(u_bp)
theta = np.mod(np.angle(z), 2*np.pi)
A = np.abs(z)
logA = np.log(A + 1e-12)

# global winding phase
i_idx = np.arange(N)
psi_wind = 2*np.pi * np.mod(f0 * i_idx, 1.0)  # frac
psi_wind[:5], theta[:5], logA[:5]

### Tool (2026-02-22 07:23:54)

(array([0.        , 0.05023846, 0.10047693, 0.15071539, 0.20095386]),
 array([1.85187755, 1.87159806, 1.89143317, 1.91138184, 1.93144286]),
 array([0.94351831, 0.94284126, 0.94209703, 0.94128082, 0.94038786]))

### Assistant (2026-02-22 07:23:55)

expDelta = math.exp(Delta)
hi = x*expDelta
# counts
lo_idx = np.searchsorted(primes, x, side='left')
hi_idx = np.searchsorted(primes, hi, side='left')
P = (hi_idx - lo_idx).astype(float)
E = x*(expDelta-1.0)/np.log(x)
R = (P - E)/np.sqrt(E + 1e-12)
R[:5], P[:5], E[:5]

### Tool (2026-02-22 07:24:00)

(array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 array([81., 82., 81., 81., 82.]),
 array([87.29968001, 87.31928832, 87.33890108, 87.35851827, 87.37813992]))

### Assistant (2026-02-22 07:24:01)

def build_fourier(theta, K):
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*theta))
        cols.append(np.cos(k*theta))
    return np.column_stack(cols) if cols else np.zeros((len(theta),0))

def build_wind(psi, K):
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*psi))
        cols.append(np.cos(k*psi))
    return np.column_stack(cols)

def model_fit_predict(X_train, y_train, X_test):
    # reasonably strong regularization to avoid blowups
    clf = LogisticRegression(penalty='l2', C=1.0, solver='lbfgs', max_iter=2000)
    clf.fit(X_train, y_train)
    p = clf.predict_proba(X_test)[:,1]
    return p

def metrics(y_true, p_pred):
    auc = roc_auc_score(y_true, p_pred) if (len(np.unique(y_true))>1) else float('nan')
    brier = brier_score_loss(y_true, p_pred)
    y_hat = (p_pred>=0.5).astype(int)
    acc = accuracy_score(y_true, y_hat)
    pos_rate = float(y_hat.mean())
    if len(y_hat)>=2:
        flip_rate = float(np.mean(y_hat[1:] != y_hat[:-1]))
        # mean run length
        runs=[]
        run=1
        for i in range(1,len(y_hat)):
            if y_hat[i]==y_hat[i-1]:
                run+=1
            else:
                runs.append(run)
                run=1
        runs.append(run)
        mean_run=float(np.mean(runs))
    else:
        flip_rate=float('nan'); mean_run=float('nan')
    return dict(auc=float(auc), brier=float(brier), accuracy=float(acc), 
                positive_rate=pos_rate, flip_rate=flip_rate, mean_run_length=mean_run)

def persistence_predict(y_past):
    # deterministic 0/1 labels already; treat as proba 0/1
    return y_past.astype(float)

results={}
for h in H:
    i0 = max(2, h)  # to define R_{i-h} and R_{i-2}
    train_idx = np.arange(i0, train_end - h)
    test_idx  = np.arange(max(i0, train_end), N - h)
    y_train = (R[train_idx + h] - R[train_idx] > 0).astype(int)
    y_test  = (R[test_idx  + h] - R[test_idx ] > 0).astype(int)
    
    # Baseline persistence: use past change over same horizon
    y_persist_test = (R[test_idx] - R[test_idx - h] > 0).astype(int)
    p_persist = persistence_predict(y_persist_test)
    base_persist = metrics(y_test, p_persist)
    
    # Lagged R logistic baseline
    Xr_train = np.column_stack([R[train_idx], R[train_idx-1], R[train_idx-2]])
    Xr_test  = np.column_stack([R[test_idx],  R[test_idx-1],  R[test_idx-2]])
    p_lr = model_fit_predict(Xr_train, y_train, Xr_test)
    base_lagR = metrics(y_test, p_lr)
    
    # M0 theta only Fourier K=3
    Xt_train = build_fourier(theta[train_idx], 3)
    Xt_test = build_fourier(theta[test_idx], 3)
    p_M0 = model_fit_predict(Xt_train, y_train, Xt_test)
    M0 = metrics(y_test, p_M0)
    
    # M1 pressure only: logA + R lags
    Xp_train = np.column_stack([logA[train_idx], R[train_idx], R[train_idx-1], R[train_idx-2]])
    Xp_test  = np.column_stack([logA[test_idx],  R[test_idx],  R[test_idx-1],  R[test_idx-2]])
    p_M1 = model_fit_predict(Xp_train, y_train, Xp_test)
    M1 = metrics(y_test, p_M1)
    
    # M2 theta + pressure
    Xtp_train = np.column_stack([Xt_train, Xp_train])
    Xtp_test  = np.column_stack([Xt_test,  Xp_test])
    p_M2 = model_fit_predict(Xtp_train, y_train, Xtp_test)
    M2 = metrics(y_test, p_M2)
    
    # M3 add wind K=2
    Xw_train = build_wind(psi_wind[train_idx], 2)
    Xw_test  = build_wind(psi_wind[test_idx], 2)
    Xtpw_train = np.column_stack([Xtp_train, Xw_train])
    Xtpw_test  = np.column_stack([Xtp_test,  Xw_test])
    p_M3 = model_fit_predict(Xtpw_train, y_train, Xtpw_test)
    M3 = metrics(y_test, p_M3)
    
    results[str(h)] = {
        "baselines": {
            "persistence": base_persist,
            "laggedR_logistic": base_lagR
        },
        "M0": M0,
        "M1": M1,
        "M2": M2,
        "M3": M3,
        "delta": {
            "auc_M3_minus_M2": float(M3["auc"] - M2["auc"]),
            "brier_M3_minus_M2": float(M3["brier"] - M2["brier"])
        }
    }

results

### Tool (2026-02-22 07:24:31)

{'25': {'baselines': {'persistence': {'auc': 0.453028785955644,
    'brier': 0.5468521229868228,
    'accuracy': 0.45314787701317716,
    'positive_rate': 0.5084187408491947,
    'flip_rate': 0.11641954969796815,
    'mean_run_length': 8.577708006279435},
   'laggedR_logistic': {'auc': 0.7938681101096036,
    'brier': 0.18494271835162396,
    'accuracy': 0.7088213762811127,
    'positive_rate': 0.5129941434846267,
    'flip_rate': 0.08676551345414607,
    'mean_run_length': 11.503157894736843}},
  'M0': {'auc': 0.6736010906759747,
   'brier': 0.22959085985115876,
   'accuracy': 0.6469619326500732,
   'positive_rate': 0.546303074670571,
   'flip_rate': 0.010616877173714076,
   'mean_run_length': 92.61016949152543},
  'M1': {'auc': 0.7940559821015076,
   'brier': 0.18484206630442618,
   'accuracy': 0.7101024890190337,
   'positive_rate': 0.5157393850658858,
   'flip_rate': 0.08676551345414607,
   'mean_run_length': 11.503157894736843},
  'M2': {'auc': 0.830346901210957,
   'brier': 0.1683608169863161,
   'accuracy': 0.75201317715959,
   'positive_rate': 0.5155563689604685,
   'flip_rate': 0.09079260479589969,
   'mean_run_length': 10.993963782696177},
  'M3': {'auc': 0.8286271086688051,
   'brier': 0.16913519592029974,
   'accuracy': 0.753294289897511,
   'positive_rate': 0.5212298682284041,
   'flip_rate': 0.09189090243455976,
   'mean_run_length': 10.862823061630218},
  'delta': {'auc_M3_minus_M2': -0.0017197925421519322,
   'brier_M3_minus_M2': 0.0007743789339836327}},
 '50': {'baselines': {'persistence': {'auc': 0.29454306212914133,
    'brier': 0.7054605626034197,
    'accuracy': 0.29453943739658023,
    'positive_rate': 0.5019305019305019,
    'flip_rate': 0.09783008458992276,
    'mean_run_length': 10.204502814258912},
   'laggedR_logistic': {'auc': 0.8776983553692879,
    'brier': 0.14387653115982965,
    'accuracy': 0.7859900717043574,
    'positive_rate': 0.5113072255929398,
    'flip_rate': 0.11438028687017286,
    'mean_run_length': 8.730337078651685}},
  'M0': {'auc': 0.7142264491494505,
   'brier': 0.21736444833381874,
   'accuracy': 0.6988416988416989,
   'positive_rate': 0.540724397867255,
   'flip_rate': 0.011033468186833395,
   'mean_run_length': 89.1639344262295},
  'M1': {'auc': 0.8769497756559607,
   'brier': 0.1442674879387708,
   'accuracy': 0.7869093583379297,
   'positive_rate': 0.49862107004964146,
   'flip_rate': 0.11438028687017286,
   'mean_run_length': 8.730337078651685},
  'M2': {'auc': 0.8861604432414043,
   'brier': 0.1384450839672228,
   'accuracy': 0.7951829380400809,
   'positive_rate': 0.5065269350983637,
   'flip_rate': 0.11107024641412284,
   'mean_run_length': 8.9900826446281},
  'M3': {'auc': 0.8857985936545429,
   'brier': 0.1384683820204329,
   'accuracy': 0.7946313660599374,
   'positive_rate': 0.5103879389593675,
   'flip_rate': 0.10702464141228393,
   'mean_run_length': 9.329331046312179},
  'delta': {'auc_M3_minus_M2': -0.0003618495868613625,
   'brier_M3_minus_M2': 2.3298053210107916e-05}},
 '100': {'baselines': {'persistence': {'auc': 0.3319592211036312,
    'brier': 0.6680274633512711,
    'accuracy': 0.3319725366487289,
    'positive_rate': 0.5141955835962145,
    'flip_rate': 0.1004083147735709,
    'mean_run_length': 9.94280442804428},
   'laggedR_logistic': {'auc': 0.8358290897210647,
    'brier': 0.16592945548207494,
    'accuracy': 0.7491185748747449,
    'positive_rate': 0.5227314900723696,
    'flip_rate': 0.09651076466221233,
    'mean_run_length': 10.343570057581575}},
  'M0': {'auc': 0.7046976140458745,
   'brier': 0.21852152465965766,
   'accuracy': 0.659491556875116,
   'positive_rate': 0.5073297457784376,
   'flip_rate': 0.011135857461024499,
   'mean_run_length': 88.34426229508196},
  'M1': {'auc': 0.8357635279086718,
   'brier': 0.1660021531669635,
   'accuracy': 0.7494897012432733,
   'positive_rate': 0.5242159955464836,
   'flip_rate': 0.09539717891610987,
   'mean_run_length': 10.464077669902913},
  'M2': {'auc': 0.8353496345342163,
   'brier': 0.16634923938355656,
   'accuracy': 0.7544999072184079,
   'positive_rate': 0.5258860642048617,
   'flip_rate': 0.10096510764662213,
   'mean_run_length': 9.888073394495413},
  'M3': {'auc': 0.8325596764001166,
   'brier': 0.16786864650231967,
   'accuracy': 0.7474485062163667,
   'positive_rate': 0.5303395806272035,
   'flip_rate': 0.09613956941351151,
   'mean_run_length': 10.383429672447013},
  'delta': {'auc_M3_minus_M2': -0.002789958134099657,
   'brier_M3_minus_M2': 0.0015194071187631109}}}

### Assistant (2026-02-22 07:25:01)

for h in H:
    hstr=str(h)
    results[hstr]["delta"]["auc_M2_minus_laggedR"] = float(results[hstr]["M2"]["auc"] - results[hstr]["baselines"]["laggedR_logistic"]["auc"])
    results[hstr]["delta"]["brier_M2_minus_laggedR"] = float(results[hstr]["M2"]["brier"] - results[hstr]["baselines"]["laggedR_logistic"]["brier"])
    
mean_delta_auc = float(np.mean([results[str(h)]["delta"]["auc_M3_minus_M2"] for h in H]))
mean_delta_brier = float(np.mean([results[str(h)]["delta"]["brier_M3_minus_M2"] for h in H]))

summary = {
    "mean_delta_auc": mean_delta_auc,
    "mean_delta_brier": mean_delta_brier,
    "evidence_for_global_winding": bool((mean_delta_auc > 0.002) and (mean_delta_brier < -1e-4))
}
params = dict(L=L, x_min=x_min, x_max=x_max, W=W, bandpass=list(bandpass), Delta=Delta, train_frac=train_frac, horizons=H, seed=seed,
              dt=dt, n_kept=N, train_end=train_end, f0=f0)

out = {"params": params, "horizon_results": results, "summary": summary}
out["summary"]

### Tool (2026-02-22 07:25:11)

{'mean_delta_auc': -0.001623866754370984,
 'mean_delta_brier': 0.0007723613686522838,
 'evidence_for_global_winding': False}

### Assistant (2026-02-22 07:25:13)

json_str = json.dumps(out, indent=2, sort_keys=False)
len(json_str), json_str[:500]

### Tool (2026-02-22 07:25:16)

(6386,
 '{\n  "params": {\n    "L": 65536,\n    "x_min": 100000.0,\n    "x_max": 9000000.0,\n    "W": 2000,\n    "bandpass": [\n      0.002,\n      0.01\n    ],\n    "Delta": 0.01,\n    "train_frac": 0.7,\n    "horizons": [\n      25,\n      50,\n      100\n    ],\n    "seed": 12345,\n    "dt": 0.00024594622340595577,\n    "n_kept": 18296,\n    "train_end": 12807,\n    "f0": 0.0079957\n  },\n  "horizon_results": {\n    "25": {\n      "baselines": {\n        "persistence": {\n          "auc": 0.453028785955644,\n          "brier": 0')

### Assistant (2026-02-22 07:25:17)

{
  "params": {
    "L": 65536,
    "x_min": 100000.0,
    "x_max": 9000000.0,
    "W": 2000,
    "bandpass": [
      0.002,
      0.01
    ],
    "Delta": 0.01,
    "train_frac": 0.7,
    "horizons": [
      25,
      50,
      100
    ],
    "seed": 12345,
    "dt": 0.00024594622340595577,
    "n_kept": 18296,
    "train_end": 12807,
    "f0": 0.0079957
  },
  "horizon_results": {
    "25": {
      "baselines": {
        "persistence": {
          "auc": 0.453028785955644,
          "brier": 0.5468521229868228,
          "accuracy": 0.45314787701317716,
          "positive_rate": 0.5084187408491947,
          "flip_rate": 0.11641954969796815,
          "mean_run_length": 8.577708006279435
        },
        "laggedR_logistic": {
          "auc": 0.7938681101096036,
          "brier": 0.18494271835162396,
          "accuracy": 0.7088213762811127,
          "positive_rate": 0.5129941434846267,
          "flip_rate": 0.08676551345414607,
          "mean_run_length": 11.503157894736843
        }
      },
      "M0": {
        "auc": 0.6736010906759747,
        "brier": 0.22959085985115876,
        "accuracy": 0.6469619326500732,
        "positive_rate": 0.546303074670571,
        "flip_rate": 0.010616877173714076,
        "mean_run_length": 92.61016949152543
      },
      "M1": {
        "auc": 0.7940559821015076,
        "brier": 0.18484206630442618,
        "accuracy": 0.7101024890190337,
        "positive_rate": 0.5157393850658858,
        "flip_rate": 0.08676551345414607,
        "mean_run_length": 11.503157894736843
      },
      "M2": {
        "auc": 0.830346901210957,
        "brier": 0.1683608169863161,
        "accuracy": 0.75201317715959,
        "positive_rate": 0.5155563689604685,
        "flip_rate": 0.09079260479589969,
        "mean_run_length": 10.993963782696177
      },
      "M3": {
        "auc": 0.8286271086688051,
        "brier": 0.16913519592029974,
        "accuracy": 0.753294289897511,
        "positive_rate": 0.5212298682284041,
        "flip_rate": 0.09189090243455976,
        "mean_run_length": 10.862823061630218
      },
      "delta": {
        "auc_M3_minus_M2": -0.0017197925421519322,
        "brier_M3_minus_M2": 0.0007743789339836327,
        "auc_M2_minus_laggedR": 0.03647879110135343,
        "brier_M2_minus_laggedR": -0.016581901365307857
      }
    },
    "50": {
      "baselines": {
        "persistence": {
          "auc": 0.29454306212914133,
          "brier": 0.7054605626034197,
          "accuracy": 0.29453943739658023,
          "positive_rate": 0.5019305019305019,
          "flip_rate": 0.09783008458992276,
          "mean_run_length": 10.204502814258912
        },
        "laggedR_logistic": {
          "auc": 0.8776983553692879,
          "brier": 0.14387653115982965,
          "accuracy": 0.7859900717043574,
          "positive_rate": 0.5113072255929398,
          "flip_rate": 0.11438028687017286,
          "mean_run_length": 8.730337078651685
        }
      },
      "M0": {
        "auc": 0.7142264491494505,
        "brier": 0.21736444833381874,
        "accuracy": 0.6988416988416989,
        "positive_rate": 0.540724397867255,
        "flip_rate": 0.011033468186833395,
        "mean_run_length": 89.1639344262295
      },
      "M1": {
        "auc": 0.8769497756559607,
        "brier": 0.1442674879387708,
        "accuracy": 0.7869093583379297,
        "positive_rate": 0.49862107004964146,
        "flip_rate": 0.11438028687017286,
        "mean_run_length": 8.730337078651685
      },
      "M2": {
        "auc": 0.8861604432414043,
        "brier": 0.1384450839672228,
        "accuracy": 0.7951829380400809,
        "positive_rate": 0.5065269350983637,
        "flip_rate": 0.11107024641412284,
        "mean_run_length": 8.9900826446281
      },
      "M3": {
        "auc": 0.8857985936545429,
        "brier": 0.1384683820204329,
        "accuracy": 0.7946313660599374,
        "positive_rate": 0.5103879389593675,
        "flip_rate": 0.10702464141228393,
        "mean_run_length": 9.329331046312179
      },
      "delta": {
        "auc_M3_minus_M2": -0.0003618495868613625,
        "brier_M3_minus_M2": 2.3298053210107916e-05,
        "auc_M2_minus_laggedR": 0.008462087872116436,
        "brier_M2_minus_laggedR": -0.005431447192606842
      }
    },
    "100": {
      "baselines": {
        "persistence": {
          "auc": 0.3319592211036312,
          "brier": 0.6680274633512711,
          "accuracy": 0.3319725366487289,
          "positive_rate": 0.5141955835962145,
          "flip_rate": 0.1004083147735709,
          "mean_run_length": 9.94280442804428
        },
        "laggedR_logistic": {
          "auc": 0.8358290897210647,
          "brier": 0.16592945548207494,
          "accuracy": 0.7491185748747449,
          "positive_rate": 0.5227314900723696,
          "flip_rate": 0.09651076466221233,
          "mean_run_length": 10.343570057581575
        }
      },
      "M0": {
        "auc": 0.7046976140458745,
        "brier": 0.21852152465965766,
        "accuracy": 0.659491556875116,
        "positive_rate": 0.5073297457784376,
        "flip_rate": 0.011135857461024499,
        "mean_run_length": 88.34426229508196
      },
      "M1": {
        "auc": 0.8357635279086718,
        "brier": 0.1660021531669635,
        "accuracy": 0.7494897012432733,
        "positive_rate": 0.5242159955464836,
        "flip_rate": 0.09539717891610987,
        "mean_run_length": 10.464077669902913
      },
      "M2": {
        "auc": 0.8353496345342163,
        "brier": 0.16634923938355656,
        "accuracy": 0.7544999072184079,
        "positive_rate": 0.5258860642048617,
        "flip_rate": 0.10096510764662213,
        "mean_run_length": 9.888073394495413
      },
      "M3": {
        "auc": 0.8325596764001166,
        "brier": 0.16786864650231967,
        "accuracy": 0.7474485062163667,
        "positive_rate": 0.5303395806272035,
        "flip_rate": 0.09613956941351151,
        "mean_run_length": 10.383429672447013
      },
      "delta": {
        "auc_M3_minus_M2": -0.002789958134099657,
        "brier_M3_minus_M2": 0.0015194071187631109,
        "auc_M2_minus_laggedR": -0.00047945518684832133,
        "brier_M2_minus_laggedR": 0.0004197839014816147
      }
    }
  },
  "summary": {
    "mean_delta_auc": -0.001623866754370984,
    "mean_delta_brier": 0.0007723613686522838,
    "evidence_for_global_winding": false
  }
}

