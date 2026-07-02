# Rolling AUC Shift Analysis

- **Conversation ID:** `699b66ff-85dc-8325-86ac-2dcb20e8ae04`
- **Date:** 2026-02-22

---

### User (2026-02-22 15:28:53)

{
  "role": "stateless compute worker",
  "output": "PRINT JSON ONLY. No files.",
  "goal": "Determine whether the large h=100 improvement from shifting theta_zero200 earlier is stable over time (rolling), and identify what extra features/components are required to recover h=25 performance.",
  "files": {
    "required": ["chebyshev_psi.npz", "primes.npz", "zeros1.gz"]
  },
  "params": {
    "L": 65536,
    "x_min": 100000.0,
    "x_max": 9000000.0,
    "W": 2000,
    "train_frac": 0.7,
    "Delta": 0.01,
    "bandpass_cycles_per_sample": [0.002, 0.01],
    "horizons": [25, 100],
    "seed": 12345,
    "Gamma": 200,
    "shift_candidates": [-120, -100, -80, -60, -44, -30, -20, -10, 0, 10, 20],
    "rolling": { "train_window": 4096, "test_window": 1024, "step": 1024, "min_valid": 800 }
  },
  "pipeline": [
    "Rebuild canonical u_bp_real, theta_real, logA_real, omega_real, and R=z exactly as before (TRAIN-only standardization).",
    "Rebuild u_bp_zero200 and theta_zero200 from zeros1.gz truncated at Gamma=200, applying the SAME detrend/standardize/bandpass as REAL."
  ],
  "tasks": [
    {
      "name": "Rolling AUC vs shift for h=100",
      "steps": [
        "For each rolling window k (train=[s,s+train_window), test=[s+train_window, s+train_window+test_window)), for each shift q in shift_candidates:",
        "  - Build theta_zero200_shifted_q by shifting theta_zero200 by q samples (theta[i-q]) and trimming edges to keep valid indices for both train and test within that window.",
        "  - Train phase-only Fourier logistic K=3 on TRAIN indices valid for horizon h=100 within that window (desticky y=1{R_{i+100}-R_i>0}).",
        "  - Evaluate AUC on TEST indices valid within that window.",
        "Also compute the REAL theta phase-only AUC in the same rolling window with the same trimming rules (reference)."
      ],
      "return": [
        "For each shift q: mean/std AUC over windows, and fraction of windows where shifted-zero beats REAL.",
        "Also return the per-window AUC series for q in {-80,0} and REAL for sanity."
      ]
    },
    {
      "name": "Does h=25 deficit vanish with extra phase-adjacent features?",
      "steps": [
        "Using full TRAIN/TEST (time split), compare models for h=25 and h=100:",
        "  M1: phase-only Fourier K=3 (theta).",
        "  M2: theta + logA.",
        "  M3: theta + omega.",
        "  M4: theta + logA + omega.",
        "Run each for REAL and for zero200 (using q=0 and q=-80 for h=100; for h=25 use q in {0,-44} as the best-found).",
        "All scalars standardized using TRAIN only. Fourier terms unscaled.",
        "Report AUC/Brier/LogLoss and calibration slope/intercept on TEST."
      ]
    },
    {
      "name": "Component test: do higher zeros restore h=25?",
      "steps": [
        "Repeat M1 (phase-only) at h=25 for Gamma in {200,250,300,400} using q=0 (no shift), and report AUC_real vs AUC_zeroGamma.",
        "This isolates whether the missing h=25 lift is simply higher-γ content."
      ]
    }
  ],
  "return_schema": {
    "meta": ["dt","N_kept","train_end","Gamma","shift_candidates","rolling"],
    "rolling_h100": {
      "by_shift_summary": "map shift-> {mean,std,frac_win_zero_gt_real,n_windows}",
      "series": {
        "real": "AUC per window",
        "zero_shift_-80": "AUC per window",
        "zero_shift_0": "AUC per window"
      }
    },
    "feature_ablation": {
      "h=25": "table-like dict of REAL vs ZERO200 with models M1..M4",
      "h=100": "table-like dict of REAL vs ZERO200 with models M1..M4"
    },
    "gamma_sweep_h25": "Gamma->{AUC_zeroGamma, delta_vs_real}",
    "interpretation": [
      "If shift -80 wins in most rolling windows, treat it as a robust alignment effect; otherwise it is a single-split artifact.",
      "If adding logA/omega restores h=25 for zero200, conclude truncation keeps phase but loses fast local coupling carried by amplitude/velocity.",
      "If increasing Gamma restores h=25, conclude missing lift is higher-γ content rather than auxiliary features."
    ]
  }
}

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

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

### Tool (2026-02-22 15:28:56)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 15:29:13)

import numpy as np, math, gzip, json, os, pandas as pd
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, log_loss

### Assistant (2026-02-22 15:29:27)

import numpy as np, gzip, os, math
psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
zeros_path="/mnt/data/zeros1.gz"
psi=np.load(psi_path)
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load(primes_path, allow_pickle=True)
primes=pr["primes"]
(len(x_grid), len(psi_minus_x), primes[:5], primes[-1])

### Tool (2026-02-22 15:29:30)

(100000, 100000, array([ 2,  3,  5,  7, 11], dtype=int32), 9999991)

### Assistant (2026-02-22 15:29:33)

# load zeros
with gzip.open(zeros_path,'rt') as f:
    txt=f.read().split()
zeros=np.array([float(z) for z in txt])
zeros[:5], zeros[-1], len(zeros)

### Tool (2026-02-22 15:29:35)

(array([14.13472514, 21.02203964, 25.01085758, 30.42487613, 32.93506159]),
 74920.827498994,
 100000)

### Assistant (2026-02-22 15:29:38)

# parameters
L=65536
x_min=1e5; x_max=9e6
W=2000
train_frac=0.7
Delta=0.01
band_lo, band_hi = 0.002, 0.01
horizons=[25,100]
seed=12345
Gamma=200.0
shift_candidates=[-120,-100,-80,-60,-44,-30,-20,-10,0,10,20]
roll_train=4096; roll_test=1024; step=1024; min_valid=800
np.random.seed(seed)

### Assistant (2026-02-22 15:29:44)

# interpolate onto uniform t grid length L spanning full raw t range
t_raw=np.log(x_grid.astype(float))
t_uniform=np.linspace(t_raw.min(), t_raw.max(), L)
dt=t_uniform[1]-t_uniform[0]
# interp psi_minus_x onto t_uniform
u_interp=np.interp(t_uniform, t_raw, psi_minus_x.astype(float))
x_uniform=np.exp(t_uniform)
# restrict kept indices based on x range
kept_mask=(x_uniform>=x_min)&(x_uniform<=x_max)
idx=np.where(kept_mask)[0]
t=t_uniform[idx]; x=x_uniform[idx]; u_raw=u_interp[idx]
N_kept=len(u_raw)
train_end=int(math.floor(train_frac*N_kept))
dt, N_kept, train_end, (x[0], x[-1])

### Tool (2026-02-22 15:29:48)

(0.00024594622340595577, 18296, 12807, (100006.32008830582, 8998557.287555724))

### Assistant (2026-02-22 15:29:51)

def causal_moving_average(a, W):
    # past-only including current, window size W
    n=len(a)
    c=np.cumsum(np.insert(a,0,0.0))
    out=np.empty(n)
    for i in range(n):
        j0=max(0, i-W+1)
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def train_standardize(u, train_end):
    mu=u[:train_end].mean()
    sd=u[:train_end].std(ddof=0)
    if sd==0: sd=1.0
    return (u-mu)/sd, mu, sd

def fft_bandpass_real(u, lo, hi):
    n=len(u)
    U=rfft(u)
    freqs=rfftfreq(n, d=1.0)  # cycles/sample
    mask=(freqs>=lo)&(freqs<=hi)
    U_bp=np.zeros_like(U)
    U_bp[mask]=U[mask]
    return irfft(U_bp, n=n)

def hilbert_analytic(u):
    # return analytic signal via FFT
    n=len(u)
    U=np.fft.fft(u)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1
        h[1:n//2]=2
    else:
        h[0]=1
        h[1:(n+1)//2]=2
    z=np.fft.ifft(U*h)
    return z

def build_phase_from_u(u_raw, train_end, W, lo, hi):
    detr=u_raw - causal_moving_average(u_raw, W)
    detr_std, mu, sd = train_standardize(detr, train_end)
    u_bp=fft_bandpass_real(detr_std, lo, hi)
    z=hilbert_analytic(u_bp)
    theta=np.mod(np.angle(z), 2*np.pi)
    A=np.abs(z)
    logA=np.log(A+1e-12)
    # omega: wrapped phase diff in (-pi,pi]
    dtheta=np.diff(theta)
    omega=(dtheta + np.pi)%(2*np.pi)-np.pi
    return {
        "detr_mu": float(mu), "detr_sd": float(sd),
        "u_bp": u_bp, "theta": theta, "A": A, "logA": logA, "omega": omega
    }

real=build_phase_from_u(u_raw, train_end, W, band_lo, band_hi)
(len(real["theta"]), len(real["omega"]))

### Tool (2026-02-22 15:30:01)

(18296, 18295)

### Assistant (2026-02-22 15:30:04)

# Build R residuals from primes in multiplicative window
# We need P_i count primes in [x_i, x_i*exp(Delta))
expD=math.exp(Delta)
# primes array sorted
primes_int=primes.astype(np.int64)
def prime_counts_intervals(x, expD, primes):
    # returns counts for each x_i
    # use searchsorted
    lo = x
    hi = x*expD
    left=np.searchsorted(primes, lo, side='left')
    right=np.searchsorted(primes, hi, side='left')  # primes < hi
    return (right-left).astype(np.int32)

P=prime_counts_intervals(x, expD, primes_int)
E= x*(expD-1.0)/np.log(x)
R=(P - E)/np.sqrt(E)
R[:5], P[:5], E[:5]

### Tool (2026-02-22 15:30:13)

(array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 array([81, 82, 81, 81, 82], dtype=int32),
 array([87.29968001, 87.31928832, 87.33890108, 87.35851827, 87.37813992]))

### Assistant (2026-02-22 15:30:15)

# Build zero-based reconstruction u_raw_zeroGamma on same t grid (kept)
def build_zero_recon(t, zeros, Gamma):
    gam=zeros[zeros<=Gamma]
    # simple cosine sum
    # normalize by sqrt(0.25+gamma^2) as in explicit formula? use 1/gamma for taper to avoid dominance
    w=1.0/np.sqrt(0.25+gam*gam)
    # compute sum cos(gam*t) * w
    # vectorize with broadcasting might be heavy: len(t)=18296, len(gam) ~? up to 200 -> ~? 30 zeros
    # do matrix mult in chunks
    out=np.zeros_like(t)
    for g,ww in zip(gam,w):
        out += ww*np.cos(g*t)
    return out, gam

u_zero200_raw, gam200 = build_zero_recon(t, zeros, Gamma)
len(gam200), gam200[:5], gam200[-1]

### Tool (2026-02-22 15:30:22)

(79,
 array([14.13472514, 21.02203964, 25.01085758, 30.42487613, 32.93506159]),
 198.015309676)

### Assistant (2026-02-22 15:30:24)

zero200=build_phase_from_u(u_zero200_raw, train_end, W, band_lo, band_hi)

### Assistant (2026-02-22 15:30:28)

# logistic Fourier features
def fourier_design(theta, K=3):
    cols=[np.ones_like(theta)]
    for k in range(1,K+1):
        cols.append(np.cos(k*theta))
        cols.append(np.sin(k*theta))
    return np.column_stack(cols)

def fit_predict_auc(theta, y, train_idx, test_idx, extra=None, extra_train_stats=None):
    # theta is array length N; extra is dict name->array length N (or N-1)
    X=fourier_design(theta, K=3)
    if extra:
        # extra arrays aligned to same indices as theta
        extra_cols=[]
        for name,arr in extra.items():
            extra_cols.append(arr.reshape(-1,1))
        X=np.hstack([X]+extra_cols)
    X_train=X[train_idx]
    y_train=y[train_idx]
    X_test=X[test_idx]
    y_test=y[test_idx]
    if len(np.unique(y_test))<2 or len(np.unique(y_train))<2:
        return None
    clf=LogisticRegression(max_iter=1000, solver="lbfgs")
    clf.fit(X_train, y_train)
    p=clf.predict_proba(X_test)[:,1]
    auc=roc_auc_score(y_test, p)
    return auc, p, clf

def calib_slope_intercept(y, p):
    eps=1e-12
    p=np.clip(p, eps, 1-eps)
    logit=np.log(p/(1-p)).reshape(-1,1)
    lr=LogisticRegression(max_iter=1000, solver="lbfgs")
    lr.fit(logit, y)
    slope=float(lr.coef_[0,0])
    intercept=float(lr.intercept_[0])
    return intercept, slope

def eval_metrics(y, p):
    auc=float(roc_auc_score(y,p)) if len(np.unique(y))==2 else float("nan")
    brier=float(brier_score_loss(y,p))
    ll=float(log_loss(y,p, labels=[0,1]))
    ci, cs=calib_slope_intercept(y,p)
    return {"AUC":auc,"Brier":brier,"LogLoss":ll,"calib_intercept":ci,"calib_slope":cs}

### Assistant (2026-02-22 15:30:39)

# Rolling AUC vs shift for h=100
h=100
# define valid indices for label: i+h < N_kept
valid_base=np.arange(0,N_kept-h)
y_desticky = (R[h:]-R[:N_kept-h] > 0).astype(int)
# align theta arrays to length N_kept; labels length N_kept-h correspond to i in [0,N_kept-h)
theta_real=real["theta"][:N_kept]
theta_zero=zero200["theta"][:N_kept]

def rolling_windows(N, train_w, test_w, step):
    s=0
    while s+train_w+test_w <= N:
        yield s, s+train_w, s+train_w+test_w
        s += step

# because label length is N_kept-h, we can operate on index i for y, with N_y = N_kept-h
N_y=N_kept-h
windows=list(rolling_windows(N_y, roll_train, roll_test, step))
len(windows), windows[:2], windows[-1]

### Tool (2026-02-22 15:30:45)

(13, [(0, 4096, 5120), (1024, 5120, 6144)], (12288, 16384, 17408))

### Assistant (2026-02-22 15:30:47)

def auc_in_window(theta_source, shift_q, s, a, b, min_valid=800):
    # theta_source length N_kept; we need theta aligned with y indices 0..N_y-1 (i)
    # apply shift: theta_shift[i]=theta_source[i - shift_q]? spec says theta[i-q] (shift earlier for negative q?) 
    # if q=-80, theta_shift[i]=theta[i+80] (earlier? actually using theta[i - (-80)]=theta[i+80], which moves phase earlier)
    q=shift_q
    # index mapping: use base i indices in [s,b)
    idx_win=np.arange(s,b)
    src_idx=idx_win - q
    # keep those within [0, N_kept-1] and also within [0,N_y-1] label space already
    ok=(src_idx>=0)&(src_idx<N_kept)
    idx_ok=idx_win[ok]
    src_ok=src_idx[ok].astype(int)
    # now split train/test within window: train [s,a), test [a,b)
    train_mask=(idx_ok>=s)&(idx_ok<a)
    test_mask=(idx_ok>=a)&(idx_ok<b)
    train_idx=idx_ok[train_mask]
    test_idx=idx_ok[test_mask]
    if len(train_idx)<min_valid or len(test_idx)<min_valid:
        return None
    # ensure labels exist (they do for idx_ok in 0..N_y-1)
    y_train=y_desticky[train_idx]
    y_test=y_desticky[test_idx]
    if len(np.unique(y_train))<2 or len(np.unique(y_test))<2:
        return None
    # build features using theta at shifted source positions
    theta_used=theta_source[src_ok]
    # build design for all idx_ok rows
    X=fourier_design(theta_used, K=3)
    # map train/test indices to row positions in idx_ok
    # create dict from idx_ok to row
    pos = {int(ii):k for k,ii in enumerate(idx_ok)}
    tr_rows=np.array([pos[int(ii)] for ii in train_idx])
    te_rows=np.array([pos[int(ii)] for ii in test_idx])
    clf=LogisticRegression(max_iter=1000, solver="lbfgs")
    clf.fit(X[tr_rows], y_train)
    p=clf.predict_proba(X[te_rows])[:,1]
    auc=float(roc_auc_score(y_test,p))
    return auc

# compute series
auc_series_real=[]
auc_series_zero_shift0=[]
auc_series_zero_shiftm80=[]
by_shift={q:[] for q in shift_candidates}
for (s,a,b) in windows:
    # real baseline q=0
    ar=auc_in_window(theta_real, 0, s,a,b, min_valid=min_valid)
    auc_series_real.append(ar)
    a0=auc_in_window(theta_zero, 0, s,a,b, min_valid=min_valid)
    auc_series_zero_shift0.append(a0)
    am80=auc_in_window(theta_zero, -80, s,a,b, min_valid=min_valid)
    auc_series_zero_shiftm80.append(am80)
    for q in shift_candidates:
        aq=auc_in_window(theta_zero, q, s,a,b, min_valid=min_valid)
        if aq is not None and ar is not None:
            by_shift[q].append((aq, ar))
# summarize
summary={}
for q, pairs in by_shift.items():
    if len(pairs)==0:
        summary[q]={"mean":None,"std":None,"frac_win_zero_gt_real":None,"n_windows":0}
        continue
    aucs=np.array([p[0] for p in pairs])
    ars=np.array([p[1] for p in pairs])
    summary[q]={
        "mean": float(aucs.mean()),
        "std": float(aucs.std(ddof=0)),
        "frac_win_zero_gt_real": float((aucs>ars).mean()),
        "n_windows": int(len(pairs))
    }
summary[-80], summary[0], len(auc_series_real), auc_series_real

### Tool (2026-02-22 15:31:09)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[12], line 47
     44 by_shift={q:[] for q in shift_candidates}
     45 for (s,a,b) in windows:
     46     # real baseline q=0
---> 47     ar=auc_in_window(theta_real, 0, s,a,b, min_valid=min_valid)
     48     auc_series_real.append(ar)
     49     a0=auc_in_window(theta_zero, 0, s,a,b, min_valid=min_valid)

Cell In[12], line 35, in auc_in_window(theta_source, shift_q, s, a, b, min_valid)
     33 te_rows=np.array([pos[int(ii)] for ii in test_idx])
     34 clf=LogisticRegression(max_iter=1000, solver="lbfgs")
---> 35 clf.fit(X[tr_rows], y_train)
     36 p=clf.predict_proba(X[te_rows])[:,1]
     37 auc=float(roc_auc_score(y_test,p))

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/base.py:1474, in _fit_context.<locals>.decorator.<locals>.wrapper(estimator, *args, **kwargs)
   1467     estimator._validate_params()
   1469 with config_context(
   1470     skip_parameter_validation=(
   1471         prefer_skip_nested_validation or global_skip_validation
   1472     )
   1473 ):
-> 1474     return fit_method(estimator, *args, **kwargs)

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/linear_model/_logistic.py:1296, in LogisticRegression.fit(self, X, y, sample_weight)
   1293 else:
   1294     n_threads = 1
-> 1296 fold_coefs_ = Parallel(n_jobs=self.n_jobs, verbose=self.verbose, prefer=prefer)(
   1297     path_func(
   1298         X,
   1299         y,
   1300         pos_class=class_,
   1301         Cs=[C_],
   1302         l1_ratio=self.l1_ratio,
   1303         fit_intercept=self.fit_intercept,
   1304         tol=self.tol,
   1305         verbose=self.verbose,
   1306         solver=solver,
   1307         multi_class=multi_class,
   1308         max_iter=self.max_iter,
   1309         class_weight=self.class_weight,
   1310         check_input=False,
   1311         random_state=self.random_state,
   1312         coef=warm_start_coef_,
   1313         penalty=penalty,
   1314         max_squared_sum=max_squared_sum,
   1315         sample_weight=sample_weight,
   1316         n_threads=n_threads,
   1317     )
   1318     for class_, warm_start_coef_ in zip(classes_, warm_start_coef)
   1319 )
   1321 fold_coefs_, _, n_iter_ = zip(*fold_coefs_)
   1322 self.n_iter_ = np.asarray(n_iter_, dtype=np.int32)[:, 0]

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/parallel.py:67, in Parallel.__call__(self, iterable)
     62 config = get_config()
     63 iterable_with_config = (
     64     (_with_config(delayed_func, config), args, kwargs)
     65     for delayed_func, args, kwargs in iterable
     66 )
---> 67 return super().__call__(iterable_with_config)

File /opt/pyvenv/lib/python3.11/site-packages/joblib/parallel.py:1977, in Parallel.__call__(self, iterable)
   1974 self._start_time = time.time()
   1976 if not self._managed_backend:
-> 1977     n_jobs = self._initialize_backend()
   1978 else:
   1979     n_jobs = self._effective_n_jobs()

File /opt/pyvenv/lib/python3.11/site-packages/joblib/parallel.py:1379, in Parallel._initialize_backend(self)
   1377 """Build a process or thread pool and return the number of workers"""
   1378 try:
-> 1379     n_jobs = self._backend.configure(
   1380         n_jobs=self.n_jobs, parallel=self, **self._backend_kwargs
   1381     )
   1382     if self.timeout is not None and not self._backend.supports_timeout:
   1383         warnings.warn(
   1384             "The backend class {!r} does not support timeout. "
   1385             "You have set 'timeout={}' in Parallel but "
   (...)   1388             )
   1389         )

File /opt/pyvenv/lib/python3.11/site-packages/joblib/_parallel_backends.py:610, in LokyBackend.configure(self, n_jobs, parallel, prefer, require, idle_worker_timeout, **memmapping_executor_kwargs)
    607 supports_retrieve_callback = True
    608 supports_inner_max_num_threads = True
--> 610 def configure(
    611     self,
    612     n_jobs=1,
    613     parallel=None,
    614     prefer=None,
    615     require=None,
    616     idle_worker_timeout=None,
    617     **memmapping_executor_kwargs,
    618 ):
    619     """Build a process executor and return the number of workers"""
    620     n_jobs = self.effective_n_jobs(n_jobs)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 15:32:14)

import numpy as np, math, gzip
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, log_loss
import warnings, os
warnings.filterwarnings("ignore")

# reload data quickly
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"]; psi_minus_x=psi["psi_minus_x"]
pr=np.load("/mnt/data/primes.npz", allow_pickle=True); primes=pr["primes"].astype(np.int64)
with gzip.open("/mnt/data/zeros1.gz",'rt') as f:
    zeros=np.fromstring(f.read(), sep=' ')
# params
L=65536; x_min=1e5; x_max=9e6; W=2000; train_frac=0.7; Delta=0.01
band_lo, band_hi=0.002,0.01
seed=12345
Gamma=200.0
shift_candidates=np.array([-120,-100,-80,-60,-44,-30,-20,-10,0,10,20], dtype=int)
roll_train=4096; roll_test=1024; step=1024; min_valid=800
h25=25; h100=100
expD=math.exp(Delta)

# helpers
def causal_moving_average(a, W):
    n=len(a)
    c=np.cumsum(np.insert(a,0,0.0))
    out=np.empty(n)
    for i in range(n):
        j0=i-W+1
        if j0<=0:
            out[i]=c[i+1]/(i+1)
        else:
            out[i]=(c[i+1]-c[j0])/(W)
    return out

def train_standardize(u, train_end):
    mu=u[:train_end].mean()
    sd=u[:train_end].std(ddof=0)
    if sd==0: sd=1.0
    return (u-mu)/sd, mu, sd

def fft_bandpass_real(u, lo, hi):
    n=len(u)
    U=rfft(u)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=lo)&(freqs<=hi)
    U_bp=np.zeros_like(U); U_bp[mask]=U[mask]
    return irfft(U_bp, n=n)

def hilbert_analytic(u):
    n=len(u)
    U=np.fft.fft(u)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1; h[1:n//2]=2
    else:
        h[0]=1; h[1:(n+1)//2]=2
    return np.fft.ifft(U*h)

def build_phase(u_raw, train_end):
    detr=u_raw - causal_moving_average(u_raw, W)
    detr_std, mu, sd = train_standardize(detr, train_end)
    u_bp=fft_bandpass_real(detr_std, band_lo, band_hi)
    z=hilbert_analytic(u_bp)
    theta=np.mod(np.angle(z), 2*np.pi)
    A=np.abs(z)
    logA=np.log(A+1e-12)
    omega=((np.diff(theta)+np.pi)%(2*np.pi))-np.pi
    return {"theta":theta, "logA":logA, "omega":omega, "mu":float(mu), "sd":float(sd), "u_bp":u_bp}

def build_zero_u(t, Gamma):
    gam=zeros[zeros<=Gamma]
    w=1.0/np.sqrt(0.25+gam*gam)
    out=np.zeros_like(t)
    # loop ok: gam count modest
    for g,ww in zip(gam,w):
        out += ww*np.cos(g*t)
    return out

# build uniform grid and kept
t_raw=np.log(x_grid.astype(float))
t_uniform=np.linspace(t_raw.min(), t_raw.max(), L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform, t_raw, psi_minus_x.astype(float))
x_uniform=np.exp(t_uniform)
kept=(x_uniform>=x_min)&(x_uniform<=x_max)
idx=np.where(kept)[0]
t=t_uniform[idx]; x=x_uniform[idx]; u_raw=u_interp[idx]
N_kept=len(u_raw); train_end=int(math.floor(train_frac*N_kept))

# real phase
real=build_phase(u_raw, train_end)

# residual R
left=np.searchsorted(primes, x, side='left')
right=np.searchsorted(primes, x*expD, side='left')
P=(right-left).astype(np.int32)
E=x*(expD-1.0)/np.log(x)
R=(P-E)/np.sqrt(E)

# zero200 phase
u_zero200_raw=build_zero_u(t, Gamma)
zero200=build_phase(u_zero200_raw, train_end)

dt, N_kept, train_end, len(zeros)

### Tool (2026-02-22 15:32:41)

(0.00024594622340595577, 18296, 12807, 100000)

### Assistant (2026-02-22 15:32:44)

def fourier_terms(theta, K=3):
    n=len(theta)
    X=np.ones((n, 1+2*K))
    col=1
    for k in range(1,K+1):
        X[:,col]=np.cos(k*theta); col+=1
        X[:,col]=np.sin(k*theta); col+=1
    return X

X_real=fourier_terms(real["theta"],3)
X_zero=fourier_terms(zero200["theta"],3)

def rolling_windows(N, train_w, test_w, step):
    s=0
    while s+train_w+test_w <= N:
        yield s, s+train_w, s+train_w+test_w
        s += step

def auc_window_preX(X_source, theta_source_len, q, s,a,b, y, min_valid=800):
    # y indexed by i in [0, N_y)
    idx=np.arange(s,b)
    src=idx - q
    ok=(src>=0)&(src<theta_source_len)
    idx=idx[ok]; src=src[ok].astype(int)
    # split train/test
    tr=(idx<s+a-s)  # idx<a
    tr= idx < a
    te= idx >= a
    tr_idx=idx[tr]; te_idx=idx[te]
    if len(tr_idx)<min_valid or len(te_idx)<min_valid:
        return None
    y_tr=y[tr_idx]; y_te=y[te_idx]
    if len(np.unique(y_tr))<2 or len(np.unique(y_te))<2:
        return None
    X_tr=X_source[src[tr]]
    X_te=X_source[src[te]]
    clf=LogisticRegression(max_iter=200, solver="liblinear", n_jobs=1)
    clf.fit(X_tr, y_tr)
    p=clf.predict_proba(X_te)[:,1]
    return float(roc_auc_score(y_te,p))

# rolling for h=100
h=h100
N_y=N_kept-h
y100=(R[h:]-R[:N_kept-h]>0).astype(int)
windows=list(rolling_windows(N_y, roll_train, roll_test, step))

auc_series_real=[]; auc_series_zero0=[]; auc_series_zero_m80=[]
by_shift={int(q):[] for q in shift_candidates}
for (s,a,b) in windows:
    ar=auc_window_preX(X_real, N_kept, 0, s,a,b, y100, min_valid)
    a0=auc_window_preX(X_zero, N_kept, 0, s,a,b, y100, min_valid)
    am80=auc_window_preX(X_zero, N_kept, -80, s,a,b, y100, min_valid)
    auc_series_real.append(ar); auc_series_zero0.append(a0); auc_series_zero_m80.append(am80)
    if ar is None: 
        continue
    for q in shift_candidates:
        aq=auc_window_preX(X_zero, N_kept, int(q), s,a,b, y100, min_valid)
        if aq is not None:
            by_shift[int(q)].append((aq, ar))
summary={}
for q,pairs in by_shift.items():
    if len(pairs)==0:
        summary[q]={"mean":None,"std":None,"frac_win_zero_gt_real":None,"n_windows":0}
    else:
        aucs=np.array([p[0] for p in pairs]); ars=np.array([p[1] for p in pairs])
        summary[q]={
            "mean":float(aucs.mean()),
            "std":float(aucs.std(ddof=0)),
            "frac_win_zero_gt_real":float((aucs>ars).mean()),
            "n_windows":int(len(pairs))
        }
summary[-80], summary[0], auc_series_real

### Tool (2026-02-22 15:32:58)

({'mean': 0.7772646207021554,
  'std': 0.07305219276255105,
  'frac_win_zero_gt_real': 1.0,
  'n_windows': 13},
 {'mean': 0.7068401974278391,
  'std': 0.0558624278800934,
  'frac_win_zero_gt_real': 0.6923076923076923,
  'n_windows': 13},
 [0.7048112030318436,
  0.5331511693321547,
  0.7404693481106621,
  0.6824427480916031,
  0.7583951138638638,
  0.7547383375105392,
  0.6953550658096113,
  0.6202452934556621,
  0.6847645748521392,
  0.7066908396946566,
  0.7089190977583264,
  0.7444696213226824,
  0.6894104720191676])

### Assistant (2026-02-22 15:33:02)

auc_series_zero_m80, auc_series_zero0

### Tool (2026-02-22 15:33:04)

([0.7828988923294974,
  0.6670876338962813,
  0.8328696335817849,
  0.7318664122137404,
  0.8125977540040039,
  0.9153663428850826,
  0.7977808386899296,
  0.6719400309001088,
  0.718542130732007,
  0.8997595419847327,
  0.7450304345975396,
  0.7503533297488286,
  0.7783470935644848],
 [0.6841367410762428,
  0.5720370604241053,
  0.7728138369532446,
  0.6850381679389314,
  0.7281031031031031,
  0.7782577393808496,
  0.6893633302724211,
  0.6400061037251799,
  0.6766418311698287,
  0.7632595419847328,
  0.7371054434009356,
  0.7475382133804439,
  0.7146214537518886])

### Assistant (2026-02-22 15:33:06)

def zscore_train(arr, train_idx):
    mu=arr[train_idx].mean()
    sd=arr[train_idx].std(ddof=0)
    if sd==0: sd=1.0
    return (arr-mu)/sd, float(mu), float(sd)

def make_shifted_theta(theta, q):
    # returns theta_shift[i]=theta[i-q]; invalid where i-q out of bounds => nan
    n=len(theta)
    out=np.full(n, np.nan)
    idx=np.arange(n)
    src=idx - q
    ok=(src>=0)&(src<n)
    out[idx[ok]]=theta[src[ok].astype(int)]
    return out

def eval_models(theta, logA, omega, R, h, train_end, q=0, use_shift=True):
    # returns metrics for M1..M4 on TEST
    n=len(theta)
    # y defined for i in [0, n-h)
    N_y=n-h
    y=(R[h:]-R[:n-h]>0).astype(int)
    # build base indices
    idx_all=np.arange(N_y)
    # apply shift to theta/logA/omega by q if requested
    if use_shift and q!=0:
        theta_s=make_shifted_theta(theta, q)
        logA_s=np.full_like(logA, np.nan); 
        # shift logA similarly if we want alignment: it's derived from same analytic signal so should shift too
        # for fairness, shift all zero-based features; real has q=0 always.
        tmp=make_shifted_theta(logA, q); logA_s=tmp
        # omega length n-1: shift with same mapping on its index i (omega_i between i and i+1). We'll shift omega array by q too.
        omega_s=np.full_like(omega, np.nan)
        idxo=np.arange(len(omega)); src=idxo - q
        ok=(src>=0)&(src<len(omega))
        omega_s[idxo[ok]]=omega[src[ok].astype(int)]
    else:
        theta_s=theta.copy()
        logA_s=logA.copy()
        omega_s=omega.copy()
    # models: build design matrices, standardize scalars with TRAIN only
    results={}
    # common train/test split by time on idx_all: train indices < train_end and valid i+h in range (already) so ok
    train_idx=idx_all[idx_all < train_end]
    test_idx=idx_all[idx_all >= train_end]
    # define helper to filter rows with finite features
    def run_model(include_logA=False, include_omega=False):
        # determine max index based on omega availability
        idx=idx_all.copy()
        feat_cols=[]
        # theta terms
        th=theta_s[:N_y]
        # drop nan
        mask=np.isfinite(th)
        if include_logA:
            la=logA_s[:N_y]
            mask &= np.isfinite(la)
        if include_omega:
            # omega defined up to n-2; for idx in [0,N_y) require idx < len(omega_s) = n-1
            mask &= (idx < len(omega_s))
            om=omega_s[:N_y]  # last element exists if N_y<=n-1
            mask &= np.isfinite(om)
        idx=idx[mask]
        y_use=y[mask]
        # split
        tr=idx[idx < train_end]; te=idx[idx >= train_end]
        if len(tr)==0 or len(te)==0: 
            return None
        if len(np.unique(y_use[idx < train_end]))<2 or len(np.unique(y_use[idx >= train_end]))<2:
            return None
        X=fourier_terms(th[mask],3)
        if include_logA:
            la=logA_s[:N_y][mask]
            la_z, mu_la, sd_la = zscore_train(la, np.where(idx < train_end)[0])  # positions in masked array
            X=np.hstack([X, la_z.reshape(-1,1)])
        if include_omega:
            om=omega_s[:N_y][mask]
            om_z, mu_om, sd_om = zscore_train(om, np.where(idx < train_end)[0])
            X=np.hstack([X, om_z.reshape(-1,1)])
        # train/test rows by boolean in masked array
        tr_rows=np.where(idx < train_end)[0]
        te_rows=np.where(idx >= train_end)[0]
        clf=LogisticRegression(max_iter=500, solver="liblinear", n_jobs=1)
        clf.fit(X[tr_rows], y_use[tr_rows])
        p=clf.predict_proba(X[te_rows])[:,1]
        return eval_metrics(y_use[te_rows], p)
    results["M1"]=run_model(False, False)
    results["M2"]=run_model(True, False)
    results["M3"]=run_model(False, True)
    results["M4"]=run_model(True, True)
    return results

# evaluate REAL and zero200 for h=25 and 100
feat_ablation={"h=25":{}, "h=100":{}}

# REAL q=0
feat_ablation["h=25"]["REAL"]=eval_models(real["theta"], real["logA"], real["omega"], R, h25, train_end, q=0, use_shift=False)
feat_ablation["h=100"]["REAL"]=eval_models(real["theta"], real["logA"], real["omega"], R, h100, train_end, q=0, use_shift=False)

# ZERO200
feat_ablation["h=25"]["ZERO200_q0"]=eval_models(zero200["theta"], zero200["logA"], zero200["omega"], R, h25, train_end, q=0, use_shift=False)
feat_ablation["h=25"]["ZERO200_q-44"]=eval_models(zero200["theta"], zero200["logA"], zero200["omega"], R, h25, train_end, q=-44, use_shift=True)

feat_ablation["h=100"]["ZERO200_q0"]=eval_models(zero200["theta"], zero200["logA"], zero200["omega"], R, h100, train_end, q=0, use_shift=False)
feat_ablation["h=100"]["ZERO200_q-80"]=eval_models(zero200["theta"], zero200["logA"], zero200["omega"], R, h100, train_end, q=-80, use_shift=True)

feat_ablation["h=100"]["REAL"], feat_ablation["h=100"]["ZERO200_q-80"]["M1"]

### Tool (2026-02-22 15:33:42)

({'M1': {'AUC': 0.7040840050664405,
   'Brier': 0.21866747684233326,
   'LogLoss': 0.6284377193975194,
   'calib_intercept': -0.03028905479754264,
   'calib_slope': 1.091780238099885},
  'M2': {'AUC': 0.7040501222810442,
   'Brier': 0.21867663136043128,
   'LogLoss': 0.6284562901215507,
   'calib_intercept': -0.031582549331347784,
   'calib_slope': 1.0925950043254287},
  'M3': {'AUC': 0.7039021949984602,
   'Brier': 0.21867350619875273,
   'LogLoss': 0.6284638307448681,
   'calib_intercept': -0.02897533591408438,
   'calib_slope': 1.0911030795783352},
  'M4': {'AUC': 0.7038769895117628,
   'Brier': 0.2186807301633296,
   'LogLoss': 0.6284784447084804,
   'calib_intercept': -0.030131407255116767,
   'calib_slope': 1.0910781378656105}},
 {'AUC': 0.787472707829568,
  'Brier': 0.18705087868576467,
  'LogLoss': 0.5575334599801995,
  'calib_intercept': -0.03125460450770408,
  'calib_slope': 0.9871901598247016})

### Assistant (2026-02-22 15:33:49)

feat_ablation["h=25"]["REAL"]["M1"], feat_ablation["h=25"]["ZERO200_q0"]["M1"], feat_ablation["h=25"]["ZERO200_q-44"]["M1"]

### Tool (2026-02-22 15:33:53)

({'AUC': 0.6737429996912574,
  'Brier': 0.22952517552647142,
  'LogLoss': 0.6515122138615618,
  'calib_intercept': 0.006310961847709158,
  'calib_slope': 1.3797969439937754},
 {'AUC': 0.6300935822284351,
  'Brier': 0.2365876504080204,
  'LogLoss': 0.666078495782691,
  'calib_intercept': 0.03502619971343482,
  'calib_slope': 1.0391197795204137},
 {'AUC': 0.6318069849721323,
  'Brier': 0.2373009216426529,
  'LogLoss': 0.6673447066126502,
  'calib_intercept': 0.0333592594608487,
  'calib_slope': 1.0767095548340764})

### Assistant (2026-02-22 15:33:57)

feat_ablation["h=25"]["ZERO200_q-44"]

### Tool (2026-02-22 15:34:00)

{'M1': {'AUC': 0.6318069849721323,
  'Brier': 0.2373009216426529,
  'LogLoss': 0.6673447066126502,
  'calib_intercept': 0.0333592594608487,
  'calib_slope': 1.0767095548340764},
 'M2': {'AUC': 0.6302371569771317,
  'Brier': 0.23749936807863947,
  'LogLoss': 0.6678029804988949,
  'calib_intercept': 0.03311598519693389,
  'calib_slope': 1.0572642416076967},
 'M3': {'AUC': 0.631890099941911,
  'Brier': 0.23728146150485666,
  'LogLoss': 0.6673052991163184,
  'calib_intercept': 0.03380651781071,
  'calib_slope': 1.077604174044004},
 'M4': {'AUC': 0.6303364259666724,
  'Brier': 0.2374817383515946,
  'LogLoss': 0.6677670122136664,
  'calib_intercept': 0.033554950452073615,
  'calib_slope': 1.0580344130079415}}

### Assistant (2026-02-22 15:34:04)

feat_ablation["h=25"]["REAL"]

### Tool (2026-02-22 15:34:05)

{'M1': {'AUC': 0.6737429996912574,
  'Brier': 0.22952517552647142,
  'LogLoss': 0.6515122138615618,
  'calib_intercept': 0.006310961847709158,
  'calib_slope': 1.3797969439937754},
 'M2': {'AUC': 0.6773699207560762,
  'Brier': 0.22924547355518826,
  'LogLoss': 0.6509396190802179,
  'calib_intercept': -0.04833896765594848,
  'calib_slope': 1.3988676557465647},
 'M3': {'AUC': 0.6706189913123274,
  'Brier': 0.2298414630102648,
  'LogLoss': 0.6522154389268394,
  'calib_intercept': 0.022035691433248537,
  'calib_slope': 1.34628721354462},
 'M4': {'AUC': 0.6737266513438877,
  'Brier': 0.22952964191496925,
  'LogLoss': 0.6515947375983164,
  'calib_intercept': -0.028564241370999367,
  'calib_slope': 1.3623194353796702}}

### Assistant (2026-02-22 15:34:07)

def auc_full_phase_only(theta, R, h, train_end):
    n=len(theta); N_y=n-h
    y=(R[h:]-R[:n-h]>0).astype(int)
    X=fourier_terms(theta[:N_y],3)
    tr=np.arange(N_y)<train_end
    te=~tr
    if len(np.unique(y[tr]))<2 or len(np.unique(y[te]))<2:
        return None
    clf=LogisticRegression(max_iter=500, solver="liblinear", n_jobs=1)
    clf.fit(X[tr], y[tr])
    p=clf.predict_proba(X[te])[:,1]
    return float(roc_auc_score(y[te], p))

auc_real_h25=auc_full_phase_only(real["theta"], R, h25, train_end)
gamma_list=[200,250,300,400]
gamma_sweep={}
for G in gamma_list:
    u_z=build_zero_u(t, float(G))
    zph=build_phase(u_z, train_end)
    auc_z=auc_full_phase_only(zph["theta"], R, h25, train_end)
    gamma_sweep[str(G)]={"AUC_zeroGamma": auc_z, "delta_vs_real": (auc_z-auc_real_h25) if auc_z is not None else None}
auc_real_h25, gamma_sweep

### Tool (2026-02-22 15:34:16)

(0.6737429996912574,
 {'200': {'AUC_zeroGamma': 0.6300935822284351,
   'delta_vs_real': -0.04364941746282225},
  '250': {'AUC_zeroGamma': 0.6687483115641242,
   'delta_vs_real': -0.004994688127133218},
  '300': {'AUC_zeroGamma': 0.6744723771890706,
   'delta_vs_real': 0.0007293774978132106},
  '400': {'AUC_zeroGamma': 0.674317737903295,
   'delta_vs_real': 0.0005747382120375732}})

### Assistant (2026-02-22 15:34:20)

out={
 "meta":{
   "dt": dt,
   "N_kept": int(N_kept),
   "train_end": int(train_end),
   "Gamma": Gamma,
   "shift_candidates": shift_candidates.tolist(),
   "rolling":{"train_window":roll_train,"test_window":roll_test,"step":step,"min_valid":min_valid},
   "bandpass_cycles_per_sample":[band_lo, band_hi],
   "W": W,
   "Delta": Delta,
   "x_range":[x_min, x_max],
   "horizons":[h25,h100]
 },
 "rolling_h100":{
   "by_shift_summary": {str(k):v for k,v in summary.items()},
   "series":{
     "real": auc_series_real,
     "zero_shift_-80": auc_series_zero_m80,
     "zero_shift_0": auc_series_zero0
   }
 },
 "feature_ablation":{
   "h=25": feat_ablation["h=25"],
   "h=100": feat_ablation["h=100"]
 },
 "gamma_sweep_h25": gamma_sweep,
 "interpretation":[
   "Rolling h=100: shift -80 beats REAL in 100% of valid rolling windows (13/13) and has higher mean AUC than q=0; this suggests the earlier-alignment effect is stable over time in this dataset/split regime, not a single-split fluke.",
   "Feature ablation: adding logA and/or omega does not rescue the h=25 deficit for ZERO200; REAL gains a small lift from logA at h=25, but ZERO200 remains well below REAL for Gamma=200 even with (logA,omega).",
   "Gamma sweep: increasing Gamma restores h=25 phase-only performance for the zero-based reconstruction (Gamma≈300 already matches/slightly exceeds REAL). This isolates the missing h=25 lift as higher-γ content rather than needing auxiliary phase-adjacent features."
 ]
}
# ensure JSON-serializable floats
import json, math
print(json.dumps(out, indent=2)[:1000])

### Tool (2026-02-22 15:34:37)

{
  "meta": {
    "dt": 0.00024594622340595577,
    "N_kept": 18296,
    "train_end": 12807,
    "Gamma": 200.0,
    "shift_candidates": [
      -120,
      -100,
      -80,
      -60,
      -44,
      -30,
      -20,
      -10,
      0,
      10,
      20
    ],
    "rolling": {
      "train_window": 4096,
      "test_window": 1024,
      "step": 1024,
      "min_valid": 800
    },
    "bandpass_cycles_per_sample": [
      0.002,
      0.01
    ],
    "W": 2000,
    "Delta": 0.01,
    "x_range": [
      100000.0,
      9000000.0
    ],
    "horizons": [
      25,
      100
    ]
  },
  "rolling_h100": {
    "by_shift_summary": {
      "-120": {
        "mean": 0.7346077721447742,
        "std": 0.07531147967400549,
        "frac_win_zero_gt_real": 0.8461538461538461,
        "n_windows": 13
      },
      "-100": {
        "mean": 0.7577447183667056,
        "std": 0.085497241575352,
        "frac_win_zero_gt_real": 0.7692307692307693,
        "n_windows": 13
      },
      "-80": {

### Assistant (2026-02-22 15:34:41)

json_str=json.dumps(out, separators=(',',':'))
len(json_str)

### Tool (2026-02-22 15:34:43)

7461

### Assistant (2026-02-22 15:34:46)

{"meta":{"dt":0.00024594622340595577,"N_kept":18296,"train_end":12807,"Gamma":200.0,"shift_candidates":[-120,-100,-80,-60,-44,-30,-20,-10,0,10,20],"rolling":{"train_window":4096,"test_window":1024,"step":1024,"min_valid":800},"bandpass_cycles_per_sample":[0.002,0.01],"W":2000,"Delta":0.01,"x_range":[100000.0,9000000.0],"horizons":[25,100]},"rolling_h100":{"by_shift_summary":{"-120":{"mean":0.7346077721447742,"std":0.07531147967400549,"frac_win_zero_gt_real":0.8461538461538461,"n_windows":13},"-100":{"mean":0.7577447183667056,"std":0.085497241575352,"frac_win_zero_gt_real":0.7692307692307693,"n_windows":13},"-80":{"mean":0.7772646207021554,"std":0.07305219276255105,"frac_win_zero_gt_real":1.0,"n_windows":13},"-60":{"mean":0.7630969819131853,"std":0.06739161587863445,"frac_win_zero_gt_real":0.9230769230769231,"n_windows":13},"-44":{"mean":0.7568516897271712,"std":0.060545504788119746,"frac_win_zero_gt_real":0.9230769230769231,"n_windows":13},"-30":{"mean":0.7476417102586019,"std":0.05965758693150367,"frac_win_zero_gt_real":0.8461538461538461,"n_windows":13},"-20":{"mean":0.7347069616665686,"std":0.0552773125711479,"frac_win_zero_gt_real":0.8461538461538461,"n_windows":13},"-10":{"mean":0.7216380257530106,"std":0.05610010412494872,"frac_win_zero_gt_real":0.7692307692307693,"n_windows":13},"0":{"mean":0.7068401974278391,"std":0.0558624278800934,"frac_win_zero_gt_real":0.6923076923076923,"n_windows":13},"10":{"mean":0.6942179080662382,"std":0.05553403634615975,"frac_win_zero_gt_real":0.5384615384615384,"n_windows":13},"20":{"mean":0.6779172702678832,"std":0.056356745811960555,"frac_win_zero_gt_real":0.38461538461538464,"n_windows":13}},"series":{"real":[0.7048112030318436,0.5331511693321547,0.7404693481106621,0.6824427480916031,0.7583951138638638,0.7547383375105392,0.6953550658096113,0.6202452934556621,0.6847645748521392,0.7066908396946566,0.7089190977583264,0.7444696213226824,0.6894104720191676],"zero_shift_-80":[0.7828988923294974,0.6670876338962813,0.8328696335817849,0.7318664122137404,0.8125977540040039,0.9153663428850826,0.7977808386899296,0.6719400309001088,0.718542130732007,0.8997595419847327,0.7450304345975396,0.7503533297488286,0.7783470935644848],"zero_shift_0":[0.6841367410762428,0.5720370604241053,0.7728138369532446,0.6850381679389314,0.7281031031031031,0.7782577393808496,0.6893633302724211,0.6400061037251799,0.6766418311698287,0.7632595419847328,0.7371054434009356,0.7475382133804439,0.7146214537518886]}},"feature_ablation":{"h=25":{"REAL":{"M1":{"AUC":0.6737429996912574,"Brier":0.22952517552647142,"LogLoss":0.6515122138615618,"calib_intercept":0.006310961847709158,"calib_slope":1.3797969439937754},"M2":{"AUC":0.6773699207560762,"Brier":0.22924547355518826,"LogLoss":0.6509396190802179,"calib_intercept":-0.04833896765594848,"calib_slope":1.3988676557465647},"M3":{"AUC":0.6706189913123274,"Brier":0.2298414630102648,"LogLoss":0.6522154389268394,"calib_intercept":0.022035691433248537,"calib_slope":1.34628721354462},"M4":{"AUC":0.6737266513438877,"Brier":0.22952964191496925,"LogLoss":0.6515947375983164,"calib_intercept":-0.028564241370999367,"calib_slope":1.3623194353796702}},"ZERO200_q0":{"M1":{"AUC":0.6300935822284351,"Brier":0.2365876504080204,"LogLoss":0.666078495782691,"calib_intercept":0.03502619971343482,"calib_slope":1.0391197795204137},"M2":{"AUC":0.6291514067862858,"Brier":0.23677224260575318,"LogLoss":0.6665077408528377,"calib_intercept":0.03457928911168264,"calib_slope":1.0201777160193578},"M3":{"AUC":0.6292263609855469,"Brier":0.2365774261284614,"LogLoss":0.6660601375510769,"calib_intercept":0.034977236657572865,"calib_slope":1.0396918741402845},"M4":{"AUC":0.6282753598839194,"Brier":0.23676229049016827,"LogLoss":0.6664901815029699,"calib_intercept":0.03452795091348292,"calib_slope":1.0207049602261516}},"ZERO200_q-44":{"M1":{"AUC":0.6318069849721323,"Brier":0.2373009216426529,"LogLoss":0.6673447066126502,"calib_intercept":0.0333592594608487,"calib_slope":1.0767095548340764},"M2":{"AUC":0.6302371569771317,"Brier":0.23749936807863947,"LogLoss":0.6678029804988949,"calib_intercept":0.03311598519693389,"calib_slope":1.0572642416076967},"M3":{"AUC":0.631890099941911,"Brier":0.23728146150485666,"LogLoss":0.6673052991163184,"calib_intercept":0.03380651781071,"calib_slope":1.077604174044004},"M4":{"AUC":0.6303364259666724,"Brier":0.2374817383515946,"LogLoss":0.6677670122136664,"calib_intercept":0.033554950452073615,"calib_slope":1.0580344130079415}}},"h=100":{"REAL":{"M1":{"AUC":0.7040840050664405,"Brier":0.21866747684233326,"LogLoss":0.6284377193975194,"calib_intercept":-0.03028905479754264,"calib_slope":1.091780238099885},"M2":{"AUC":0.7040501222810442,"Brier":0.21867663136043128,"LogLoss":0.6284562901215507,"calib_intercept":-0.031582549331347784,"calib_slope":1.0925950043254287},"M3":{"AUC":0.7039021949984602,"Brier":0.21867350619875273,"LogLoss":0.6284638307448681,"calib_intercept":-0.02897533591408438,"calib_slope":1.0911030795783352},"M4":{"AUC":0.7038769895117628,"Brier":0.2186807301633296,"LogLoss":0.6284784447084804,"calib_intercept":-0.030131407255116767,"calib_slope":1.0910781378656105}},"ZERO200_q0":{"M1":{"AUC":0.6931350366008831,"Brier":0.21987498708488074,"LogLoss":0.6311861024395904,"calib_intercept":-0.013514811443438075,"calib_slope":0.9973238755678796},"M2":{"AUC":0.6931216718302415,"Brier":0.21987819039390956,"LogLoss":0.6311931676474087,"calib_intercept":-0.013698952219156514,"calib_slope":0.9972477872316244},"M3":{"AUC":0.6930664444284889,"Brier":0.2198788332398073,"LogLoss":0.631194242989978,"calib_intercept":-0.013482235026510259,"calib_slope":0.9973112657351634},"M4":{"AUC":0.6930536133219469,"Brier":0.21988200662034975,"LogLoss":0.6312011523457231,"calib_intercept":-0.013666192114401468,"calib_slope":0.9972366153465631}},"ZERO200_q-80":{"M1":{"AUC":0.787472707829568,"Brier":0.18705087868576467,"LogLoss":0.5575334599801995,"calib_intercept":-0.03125460450770408,"calib_slope":0.9871901598247016},"M2":{"AUC":0.787248716977135,"Brier":0.18707491812251833,"LogLoss":0.5575937579484548,"calib_intercept":-0.031317770148732595,"calib_slope":0.9873841752595298},"M3":{"AUC":0.7876347934371583,"Brier":0.18704359829463402,"LogLoss":0.5575190131800782,"calib_intercept":-0.03146674593219465,"calib_slope":0.9871946163369295},"M4":{"AUC":0.7874090407614064,"Brier":0.1870676869871321,"LogLoss":0.557579168320424,"calib_intercept":-0.03152987037441771,"calib_slope":0.9873887399911677}}}}},"gamma_sweep_h25":{"200":{"AUC_zeroGamma":0.6300935822284351,"delta_vs_real":-0.04364941746282225},"250":{"AUC_zeroGamma":0.6687483115641242,"delta_vs_real":-0.004994688127133218},"300":{"AUC_zeroGamma":0.6744723771890706,"delta_vs_real":0.0007293774978132106},"400":{"AUC_zeroGamma":0.674317737903295,"delta_vs_real":0.0005747382120375732}},"interpretation":["Rolling h=100: shift -80 beats REAL in 100% of valid rolling windows (13/13) and has higher mean AUC than q=0; this suggests the earlier-alignment effect is stable over time in this dataset/split regime, not a single-split artifact.","Feature ablation: adding logA and/or omega does not rescue the h=25 deficit for ZERO200; REAL gains a small lift from logA at h=25, but ZERO200 remains well below REAL for Gamma=200 even with (logA,omega).","Gamma sweep: increasing Gamma restores h=25 phase-only performance for the zero-based reconstruction (Gamma≈300 already matches/slightly exceeds REAL). This isolates the missing h=25 lift as higher-γ content rather than needing auxiliary phase-adjacent features."]}

