# Gating Nonstationarity Decomposition

- **Conversation ID:** `699b37ef-5b28-832f-9674-1ddaac0d86cc`
- **Date:** 2026-02-22

---

### User (2026-02-22 12:08:04)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Decompose apparent gating nonstationarity into (1) covariate shift in state occupancy vs (2) true conditional drift, using importance-weighting on the existing FAST 2D binning.

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
H=[25,100]
B_theta=12
B_omega=10
min_count_bin=20
eps=1.0  (add-eps smoothing counts for occupancy ratios)

Pipeline:
Use the same canonical steps used for the prior gating surface run:
- build theta_f, omega_f (aligned to omega indices)
- build y_h(i)=1{R_{i+h}-R_i>0} with train/test valid indices

Binning:
- theta bins uniform [0,2π)
- omega bins via TRAIN quantiles into B_omega bins (reuse edges)

For each horizon h:
1) Compute per-bin counts:
   n_train[b], n_test[b]
   and conditional rates:
   p_train[b]=mean(y_h|b) on TRAIN-valid
   p_test[b]=mean(y_h|b) on TEST-valid

2) Compute occupancy probabilities with smoothing:
   q_train[b] = (n_train[b] + eps) / sum_b(n_train[b] + eps)
   q_test[b]  = (n_test[b]  + eps) / sum_b(n_test[b]  + eps)

3) Importance weights:
   w[b] = q_test[b] / q_train[b]
   Normalize weights so sum_b w[b]*q_train[b] = 1

4) Conditional drift metrics (only bins with n_train>=min_count_bin and n_test>=min_count_bin):
   - corr_raw = corr(p_train, p_test)
   - wrmse_raw weighted by n_test
   - corr_w = corr(p_train, p_test) with bin weights proportional to q_test (or n_test)
   - wrmse_w where errors (p_train - p_test) are weighted by q_test (or n_test)

5) Reweighted transfer performance:
   Using p_hat(i)=p_train[bin(i)] on TEST:
     AUC_raw, Brier_raw, logloss_raw (same as prior)
   Also compute “importance-weighted Brier/logloss” on TEST:
     weight each test sample by 1 (normal) and report as usual (this is redundant),
   BUT also compute a TRAIN->TEST corrected predictor:
     p_hat_w(b) = clip(p_train[b],1e-6,1-1e-6)
     (no change in p itself, weights adjust evaluation)
   Key output here is not reweighted evaluation (which is just a different metric),
   but a covariate-shift-corrected *expected* test rate:
     ybar_pred = sum_b q_test[b]*p_train[b]
     ybar_true = overall test pos rate
     error_ybar = ybar_pred - ybar_true

6) Optional: chi-square style test for drift:
   For bins with enough counts, compute z_b = (p_test[b]-p_train[b]) / sqrt( p_train[b]*(1-p_train[b])*(1/n_test[b] + 1/n_train[b]) )
   Report fraction of bins with |z_b|>2.

Return JSON:

{
 "params": {...},
 "per_h": {
   "25": {
     "occupancy": {"q_train":[...], "q_test":[...], "kl_test_train":...},
     "drift": {
        "corr_raw":..., "wrmse_raw":...,
        "corr_weighted_by_qtest":..., "wrmse_weighted_by_qtest":...,
        "bins_used":...
     },
     "mean_rate": {"ybar_true":..., "ybar_pred_from_train_surface":..., "error":...},
     "z_bin": {"frac_abs_z_gt_2":..., "z_summary": {"mean":..., "std":..., "max_abs":...}}
   },
   "100": { ... }
 },
 "summary": {
   "dominant_issue": {"25":"covariate_shift/conditional_drift/mixed", "100":"..."},
   "notes":"..."
 }
}

PRINT JSON ONLY.

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

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

### Tool (2026-02-22 12:08:07)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 12:08:07)

import numpy as np, math, json, os, textwrap, statistics
from pathlib import Path
import numpy.fft as fft

cheb_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
prot_path="/mnt/data/Protocol.md"
npz_cheb=np.load(cheb_path)
x_grid=npz_cheb["x_grid"]
psi_minus_x=npz_cheb["psi_minus_x"]
npz_pr=np.load(primes_path)
primes=npz_pr["primes"]
x_grid[:5], x_grid[-5:], psi_minus_x.shape, primes.shape, primes[:10], primes[-1]

### Tool (2026-02-22 12:08:17)

(array([  1, 101, 201, 301, 401], dtype=int32),
 array([9999501, 9999601, 9999701, 9999801, 9999901], dtype=int32),
 (100000,),
 (664579,),
 array([ 2,  3,  5,  7, 11, 13, 17, 19, 23, 29], dtype=int32),
 9999991)

### Assistant (2026-02-22 12:08:19)

import numpy as np, math

def causal_moving_average(x, W):
    # past-only including current: y[i]=mean(x[max(0,i-W+1):i+1])
    c = np.cumsum(np.insert(x,0,0.0))
    n=len(x)
    y=np.empty(n,float)
    for i in range(n):
        start=max(0,i-W+1)
        y[i]=(c[i+1]-c[start])/(i-start+1)
    return y

def bandpass_fft(x, f_lo, f_hi):
    n=len(x)
    X=np.fft.rfft(x)
    freqs=np.fft.rfftfreq(n, d=1.0)  # cycles/sample
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    Xf=X*mask
    return np.fft.irfft(Xf, n=n)

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

def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

def weighted_corr(x,y,w):
    x=np.asarray(x); y=np.asarray(y); w=np.asarray(w)
    w=w/np.sum(w)
    mx=np.sum(w*x); my=np.sum(w*y)
    cov=np.sum(w*(x-mx)*(y-my))
    vx=np.sum(w*(x-mx)**2); vy=np.sum(w*(y-my)**2)
    if vx<=0 or vy<=0: 
        return float("nan")
    return float(cov/np.sqrt(vx*vy))

# Params
L=65536
x_min=1e5
x_max=9e6
W=2000
Delta=0.01
train_frac=0.7
seed=12345
f_lo,f_hi=0.002,0.01
H=[25,100]
B_theta=12
B_omega=10
min_count_bin=20
eps=1.0

# Build uniform t grid
t_raw=np.log(x_grid.astype(float))
t_u=np.linspace(t_raw.min(), t_raw.max(), L)
dt=float(t_u[1]-t_u[0])
# interpolate
u_interp=np.interp(t_u, t_raw, psi_minus_x.astype(float))
x_u=np.exp(t_u)
kept=(x_u>=x_min)&(x_u<=x_max)
t=t_u[kept]
x=x_u[kept]
u_raw=u_interp[kept]
N_kept=len(u_raw)
N_kept, dt, (t.min(),t.max()), (x.min(),x.max())

### Tool (2026-02-22 12:09:25)

(18296,
 0.00024594622340595577,
 (11.512988663856195, 16.012574821068156),
 (100006.32008830582, 8998557.287555724))

### Assistant (2026-02-22 12:09:26)

split_R=int(train_frac*N_kept)
split_R

### Tool (2026-02-22 12:09:29)

12807

### Assistant (2026-02-22 12:09:30)

u_ma=causal_moving_average(u_raw,W)
u=u_raw - u_ma
mu=float(u[:split_R].mean())
sd=float(u[:split_R].std(ddof=0))
u_std=(u-mu)/(sd if sd>0 else 1.0)
u_bp=bandpass_fft(u_std,f_lo,f_hi)
z=analytic_signal(u_bp)
theta=np.angle(z)  # in [-pi,pi]
theta_un=np.unwrap(theta)
theta_mod=(theta%(2*np.pi))
omega=wrap_to_pi(np.diff(theta_un))
theta_f=theta_mod[:-1]
omega_f=omega
Nf=len(theta_f)
split=int(train_frac*Nf)
N_kept, Nf, split, split_R

### Tool (2026-02-22 12:09:34)

(18296, 18295, 12806, 12807)

### Assistant (2026-02-22 12:09:35)

expD=math.exp(Delta)
upper=x*expD
# counts
p_lo=np.searchsorted(primes, x, side="left")
p_hi=np.searchsorted(primes, upper, side="left")
P=(p_hi-p_lo).astype(float)
E=x*(expD-1.0)/np.log(x)
R=(P-E)/np.sqrt(E)
# y for horizons on indices i where i+h < N_kept
def build_y(h):
    valid=np.arange(0,N_kept-h)
    y=(R[valid+h]-R[valid] > 0).astype(int)
    return y, valid
ys={}
for h in H:
    y, idx=build_y(h)
    ys[h]=(y,idx)
    print(h, y.mean(), len(y))

### Tool (2026-02-22 12:09:41)

25 0.5001915603962563 18271
100 0.5021433282040009 18196

### Assistant (2026-02-22 12:09:42)

# theta bins uniform
theta_bin=np.floor(theta_f/(2*np.pi)*B_theta).astype(int)
theta_bin=np.clip(theta_bin,0,B_theta-1)

# omega bins quantiles on TRAIN
train_omega=omega_f[:split]
q=np.linspace(0,1,B_omega+1)
edges=np.quantile(train_omega,q)
# ensure strictly increasing
edges_adj=edges.copy()
for i in range(1,len(edges_adj)):
    if edges_adj[i]<=edges_adj[i-1]:
        edges_adj[i]=edges_adj[i-1]+1e-9
edges=edges_adj
# digitize: bins 0..B_omega-1, rightmost inclusive
omega_bin=np.digitize(omega_f, edges[1:-1], right=False)  # uses interior edges
omega_bin=np.clip(omega_bin,0,B_omega-1)
bin_id=theta_bin*B_omega + omega_bin
B=B_theta*B_omega
B, bin_id.min(), bin_id.max(), len(edges)

### Tool (2026-02-22 12:09:49)

(120, 0, 119, 11)

### Assistant (2026-02-22 12:09:50)

from math import log

def per_h_metrics(h):
    y, idx_all=ys[h]
    # idx_all goes 0..N_kept-h-1 corresponds to R index i.
    # Restrict to feature indices (0..Nf-1) and also i in idx_all
    max_i = min(Nf-1, N_kept-h-1)
    idx = np.arange(0, max_i+1)
    y_use = (R[idx+h]-R[idx] > 0).astype(int)
    # train/test valid where i and i+h within same segment on feature split boundary
    train_valid = (idx < split) & (idx + h < split)  # uses same split index
    test_valid  = (idx >= split) & (idx + h < N_kept) & (idx + h >= split)
    # bins
    b = bin_id[idx]
    # helper to aggregate
    def agg(mask):
        b_m=b[mask]
        y_m=y_use[mask]
        n=np.bincount(b_m, minlength=B).astype(float)
        s=np.bincount(b_m, weights=y_m, minlength=B).astype(float)
        return n,s,y_m
    n_tr,s_tr,ytr=agg(train_valid)
    n_te,s_te,yte=agg(test_valid)
    p_tr=np.full(B,np.nan); p_te=np.full(B,np.nan)
    p_tr[n_tr>0]=s_tr[n_tr>0]/n_tr[n_tr>0]
    p_te[n_te>0]=s_te[n_te>0]/n_te[n_te>0]
    # occupancy probs with smoothing
    q_tr=(n_tr+eps); q_tr=q_tr/q_tr.sum()
    q_te=(n_te+eps); q_te=q_te/q_te.sum()
    # kl divergence KL(q_test || q_train)
    kl=float(np.sum(q_te*np.log(q_te/q_tr)))
    # weights
    w=q_te/q_tr  # already normalized
    norm=float(np.sum(w*q_tr))
    # bins used for drift metrics
    use=(n_tr>=min_count_bin)&(n_te>=min_count_bin)
    bins_used=int(use.sum())
    pt=p_tr[use]; ps=p_te[use]
    # raw corr
    if bins_used>=2 and np.std(pt)>0 and np.std(ps)>0:
        corr_raw=float(np.corrcoef(pt,ps)[0,1])
    else:
        corr_raw=float("nan")
    # weighted corr by q_test restricted
    wt=q_te[use]
    corr_w=weighted_corr(pt,ps,wt) if bins_used>=2 else float("nan")
    # wrmse weighted by n_test
    diff=pt-ps
    wrmse_raw=float(np.sqrt(np.sum(n_te[use]*(diff**2))/np.sum(n_te[use]))) if bins_used>0 else float("nan")
    wt_norm=wt/np.sum(wt) if bins_used>0 else wt
    wrmse_w=float(np.sqrt(np.sum(wt_norm*(diff**2)))) if bins_used>0 else float("nan")
    # mean rate
    ybar_true=float(yte.mean()) if len(yte)>0 else float("nan")
    ybar_train=float(ytr.mean()) if len(ytr)>0 else float("nan")
    p_tr_fill=p_tr.copy()
    p_tr_fill[np.isnan(p_tr_fill)]=ybar_train
    ybar_pred=float(np.sum(q_te*p_tr_fill))
    err=float(ybar_pred-ybar_true)
    # z bins
    p0=np.clip(pt,1e-6,1-1e-6)
    z_vals=(ps-pt)/np.sqrt(p0*(1-p0)*(1/n_te[use]+1/n_tr[use])) if bins_used>0 else np.array([])
    frac=float(np.mean(np.abs(z_vals)>2)) if bins_used>0 else float("nan")
    z_summary={"mean": float(np.mean(z_vals)) if bins_used>0 else float("nan"),
               "std": float(np.std(z_vals)) if bins_used>0 else float("nan"),
               "max_abs": float(np.max(np.abs(z_vals))) if bins_used>0 else float("nan")}
    # performance metrics for p_hat = p_train[bin(i)] on TEST-valid
    # Use filled p_train
    p_hat=p_tr_fill[b[test_valid]]
    y_test=y_use[test_valid].astype(int)
    # clip probs
    p_hat_clip=np.clip(p_hat,1e-6,1-1e-6)
    # brier
    brier=float(np.mean((p_hat-y_test)**2)) if len(y_test)>0 else float("nan")
    logloss=float(-np.mean(y_test*np.log(p_hat_clip)+(1-y_test)*np.log(1-p_hat_clip))) if len(y_test)>0 else float("nan")
    # AUC
    def auc_score(y, s):
        y=np.asarray(y); s=np.asarray(s)
        # rank-based AUC
        pos=s[y==1]; neg=s[y==0]
        if len(pos)==0 or len(neg)==0: return float("nan")
        # compute via Mann-Whitney U
        ranks=np.argsort(np.argsort(s))
        # average ranks for ties: do proper
        order=np.argsort(s)
        s_sorted=s[order]
        y_sorted=y[order]
        # compute average rank for ties
        n=len(s)
        ranks=np.empty(n,float)
        i=0
        rank=1
        while i<n:
            j=i
            while j<n and s_sorted[j]==s_sorted[i]:
                j+=1
            avg=(rank + (rank+(j-i)-1))/2.0
            ranks[i:j]=avg
            rank += (j-i)
            i=j
        # back to original order
        ranks_back=np.empty(n,float)
        ranks_back[order]=ranks
        U=np.sum(ranks_back[y==1]) - len(pos)*(len(pos)+1)/2.0
        return float(U/(len(pos)*len(neg)))
    auc=float(auc_score(y_test,p_hat)) if len(y_test)>0 else float("nan")
    return {
        "occupancy":{
            "q_train": q_tr.tolist(),
            "q_test": q_te.tolist(),
            "kl_test_train": kl,
            "w_norm_check": norm
        },
        "drift":{
            "corr_raw": corr_raw,
            "wrmse_raw_weighted_by_ntest": wrmse_raw,
            "corr_weighted_by_qtest": corr_w,
            "wrmse_weighted_by_qtest": wrmse_w,
            "bins_used": bins_used
        },
        "mean_rate":{
            "ybar_true": ybar_true,
            "ybar_pred_from_train_surface": ybar_pred,
            "error": err,
            "ybar_train": ybar_train
        },
        "test_performance":{
            "AUC_raw": auc,
            "Brier_raw": brier,
            "logloss_raw": logloss,
            "n_test_valid": int(len(y_test)),
            "pos_rate_test_valid": float(y_test.mean()) if len(y_test)>0 else float("nan")
        },
        "z_bin":{
            "frac_abs_z_gt_2": frac,
            "z_summary": z_summary
        }
    }

out_per_h={}
for h in H:
    out_per_h[str(h)]=per_h_metrics(h)
out_per_h["25"]["mean_rate"], out_per_h["100"]["mean_rate"]

### Tool (2026-02-22 12:10:26)

({'ybar_true': 0.5066788655077767,
  'ybar_pred_from_train_surface': 0.48282809129304305,
  'error': -0.023850774214733694,
  'ybar_train': 0.49730068069791095},
 {'ybar_true': 0.500556586270872,
  'ybar_pred_from_train_surface': 0.5002589454642795,
  'error': -0.0002976408065924474,
  'ybar_train': 0.5005509208248072})

### Assistant (2026-02-22 12:10:27)

for h in H:
    d=out_per_h[str(h)]
    print("h",h,"kl",d["occupancy"]["kl_test_train"],"corr",d["drift"]["corr_raw"],"bins",d["drift"]["bins_used"],"fracz",d["z_bin"]["frac_abs_z_gt_2"])

### Tool (2026-02-22 12:10:30)

h 25 kl 0.271529403044904 corr 0.5565960590651317 bins 91 fracz 0.5164835164835165
h 100 kl 0.28032127046980354 corr 0.7124605723706064 bins 89 fracz 0.48314606741573035

### Assistant (2026-02-22 12:10:31)

def sanitize(obj):
    if isinstance(obj, float):
        if math.isnan(obj) or math.isinf(obj):
            return None
        return obj
    if isinstance(obj, (np.floating,)):
        val=float(obj)
        if math.isnan(val) or math.isinf(val):
            return None
        return val
    if isinstance(obj, (np.integer,)):
        return int(obj)
    if isinstance(obj, dict):
        return {k:sanitize(v) for k,v in obj.items()}
    if isinstance(obj, list):
        return [sanitize(v) for v in obj]
    if isinstance(obj, tuple):
        return [sanitize(v) for v in obj]
    return obj

dominant={}
notes=[]
for h in H:
    d=out_per_h[str(h)]
    abs_err=abs(d["mean_rate"]["error"])
    corr=d["drift"]["corr_raw"]
    fracz=d["z_bin"]["frac_abs_z_gt_2"]
    kl=d["occupancy"]["kl_test_train"]
    label="mixed"
    if (abs_err<0.01) and (corr is not None and corr>0.8) and (fracz is not None and fracz<0.1):
        label="covariate_shift"
    elif (abs_err>0.03) or (corr is not None and corr<0.5) or (fracz is not None and fracz>0.25):
        label="conditional_drift"
    dominant[str(h)]=label
    if h==100 and abs_err<0.005 and fracz>0.25:
        notes.append("h=100: mean-rate transfer error ~0 despite large per-bin z; conditional drift appears to cancel in aggregate.")
    if kl>0.2:
        notes.append(f"h={h}: occupancy KL(test||train) ~{kl:.3f} indicates sizable covariate shift in (theta,omega) state occupancy.")

result={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,
    "seed":seed,"FAST":[f_lo,f_hi],"H":H,"B_theta":B_theta,"B_omega":B_omega,
    "min_count_bin":min_count_bin,"eps":eps,
    "dt":dt,
    "n_kept":N_kept,
    "n_feat":Nf,
    "train_end_feat":split,
    "train_end_kept":split_R,
    "omega_edges_train_quantiles": edges.tolist()
 },
 "per_h": out_per_h,
 "summary":{
    "dominant_issue": dominant,
    "notes":" ".join(notes) if notes else ""
 }
}
result_s=sanitize(result)
json_str=json.dumps(result_s, separators=(",",":"), ensure_ascii=False)
len(json_str), json_str[:200]

### Tool (2026-02-22 12:10:54)

(12641,
 '{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"Delta":0.01,"train_frac":0.7,"seed":12345,"FAST":[0.002,0.01],"H":[25,100],"B_theta":12,"B_omega":10,"min_count_bin":20,"eps":1.0,"dt"')

### Assistant (2026-02-22 12:10:56)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"Delta":0.01,"train_frac":0.7,"seed":12345,"FAST":[0.002,0.01],"H":[25,100],"B_theta":12,"B_omega":10,"min_count_bin":20,"eps":1.0,"dt":0.00024594622340595577,"n_kept":18296,"n_feat":18295,"train_end_feat":12806,"train_end_kept":12807,"omega_edges_train_quantiles":[-3.141592653589793,-0.17441643389468766,-0.12078650755490617,-0.08636530312848492,-0.05859608581103461,-0.03289995187015475,-0.00677424802351981,0.023914001967338872,0.06204612348753705,0.12244425465414933,3.141592653589793]},"per_h":{"25":{"occupancy":{"q_train":[0.007503464203233256,0.007064735120628837,0.006626006038024419,0.006217085542771386,0.005738741521555648,0.005648876980253438,0.006396577139118529,0.006127220083994593,0.006665899225390917,0.0068453705787222095,0.00863936516580516,0.011122757813761021,0.00881883651913645,0.006715870579684802,0.005618963036339129,0.005210147896102001,0.004851186978011261,0.004621723903532919,0.005130350510195017,0.0054594150341968464,0.006855343625850075,0.008140787409963374,0.01009598596567533,0.012070123510136851,0.01142195451343876,0.009578468126453902,0.007614993554509304,0.006297114050519914,0.005898271957720806,0.005090457322828619,0.004971453079625191,0.005170244698561415,0.005898271957720806,0.006696931590935237,0.00872922970710737,0.010444997048264314,0.011222533467039205,0.009578468126453902,0.008399787409482078,0.006994832076378452,0.006097479787604193,0.005409560812050459,0.00500059278152641,0.004811293790645396,0.004631696950660785,0.004631696950660785,0.005170244698561415,0.005628936083466995,0.006676985,0.008120840816280175,0.0095984147201371,0.010703997047783019,0.01142195451343876,0.010105959012802984,0.008918544963910016,0.007882127509245628,0.006626006038024419,0.005828355128346024,0.0053597065899040715,0.005040603100682231,0.0048212668377732615,0.004811293790645396,0.004891080166377663,0.005409560812050459,0.006117426381287393,0.007325281604467565,0.008180680597329861,0.009140598370226815,0.009299,0.009160544963910013,0.007953,0.007115,0.006297114050519914,0.005798434916249536,0.005658849,0.005798434916249536,0.006347,0.007335254651595431,0.008719256659979505,0.010753851269929406,0.012309,0.010703997047783019,0.009120651776543616,0.007963,0.006955,0.006246,0.005888298910592941,0.005469388081324712,0.005060549694365431,0.004681564875008077,0.004302580055650723,0.004552,0.004921,0.005539202410753631,0.006835397032166877,0.008799289,0.011751,0.014395,0.014674,0.013199582694615507,0.010644157266733333,0.008679363472613106,0.0073153085573396995,0.006317060644203115,0.005648876980253438,0.005259,0.005210147896102001,0.005898271957720806,0.007903,0.010136,0.014035,0.020252,0.021249,0.017719,0.012978,0.010196,0.00811086776915231],"q_test":[0.006770167647856274,0.006730305880079901,0.006680471064825764,0.006720332832602137,0.006690444112303528,0.006979115359411532,0.007557259329397338,0.007258220390393879,0.006879419509163227,0.006730305880079901,0.006810029415633392,0.007178496854840387,0.007557259329397338,0.006660539181937578,0.005953042079601922,0.005524526072157694,0.005285261335223317,0.005365,0.005115,0.005405,0.005824,0.007078801004592082,0.008622319062756302,0.010654001743061413,0.010504,0.008891,0.007717,0.006610706807123459,0.005834038064845708,0.005495,0.005165,0.005724,0.006610706807123459,0.007616,0.009250090137135685,0.010544333947021855,0.010225608897907952,0.009250090137135685,0.007956,0.006989088406889296,0.006371442070189083,0.005863926785144317,0.005494637352143726,0.005285261335223317,0.005185565484975012,0.005394,0.005644,0.006072674,0.006979115359411532,0.008144,0.009240117089657921,0.010225608897907952,0.010424113,0.010464610411468367,0.009220185206769734,0.007976717893328832,0.006879419509163227,0.0062020690321241585,0.005803,0.005534498,0.005524526072157694,0.005604249607711186,0.005764,0.005913179,0.00639137395307727,0.007198428737728574,0.007646984314,0.008015,0.008174039,0.008154107068689924,0.007766,0.007278152273282065,0.006660539181937578,0.006262,0.006072,0.005973,0.005963014,0.006251915532,0.006720332832602137,0.007298,0.008363,0.009967,0.011011,0.010863377725110698,0.009758,0.008234,0.007188455690600708,0.006491074,0.005893247,0.005644,0.005465,0.005314831,0.005125770,0.005026074149751385,0.004976210,0.005235397,0.005883274,0.006949226639112923,0.008891463,0.010923,0.014093,0.015318226,0.014222839125402968,0.011011350772238564,0.008512651266716744,0.007118,0.006182137149235973,0.005664054,0.005345373,0.005115941,0.005953042079601922,0.007996649776217018,0.010664,0.014342,0.020219,0.022091584,0.0182021,0.013797,0.010465,0.008891463295,"kl_test_train":0.271529403044904,"w_norm_check":1.0},"drift":{"corr_raw":0.5565960590651317,"wrmse_raw_weighted_by_ntest":0.18618828584295036,"corr_weighted_by_qtest":0.590918401190609,"wrmse_weighted_by_qtest":0.19630901919895078,"bins_used":91},"mean_rate":{"ybar_true":0.5066788655077767,"ybar_pred_from_train_surface":0.48282809129304305,"error":-0.023850774214733694,"ybar_train":0.49730068069791095},"test_performance":{"AUC_raw":0.6117720551703662,"Brier_raw":0.2485420892084664,"logloss_raw":0.7000387872606682,"n_test_valid":5464,"pos_rate_test_valid":0.5066788655077767},"z_bin":{"frac_abs_z_gt_2":0.5164835164835165,"z_summary":{"mean":-0.47543158282127553,"std":2.9495771774445044,"max_abs":6.323248196161194}}},"100":{"occupancy":{"q_train":[0.007438491202738015,0.006939638723317078,0.006520509779449634,0.006061519067805816,0.005683,0.005723,0.006511,0.0062310793725940445,0.006860,0.007378651,0.009552388697531734,0.012135458113751855,0.008795,0.0067002316572107885,0.005603525,0.00518475410788538,0.004815845501480772,0.004576443284,0.005135,0.005603525,0.007068,0.008234055832171165,0.01019832589894984,0.012000810606598844,0.011232,0.009469,0.007707,0.006461,0.005978,0.005255,0.005075,0.005224715,0.005813,0.0067002316572107885,0.008762728619667025,0.010634,0.011042,0.009362878918519852,0.008234055832171165,0.006939,0.006122,0.005643386767184,0.00518475410788538,0.004885,0.004556512,0.004715952,0.005115,0.005603525,0.006571,0.007876667375345509,0.009299,0.010794399,0.011450,0.010338,0.008875,0.007587350183,0.006461,0.005852,0.005473,0.005204784,0.004835776,0.004825811,0.004945,0.00528455723837495,0.006112160,-0.0,0.0067002316572107885,0.007986335171385067,0.008922,0.009373,0.009492,0.008752763,0.007628,0.006500,0.005952,0.005763,0.005693,0.005573,0.005593,0.005992,0.006840684586103804,0.008513,0.010413,0.012590,0.013578,0.011471,0.009583,0.007727,0.006630,0.005713,0.005244,0.004915,0.004596,0.004267,0.004546539,0.004775,0.005523,0.006530474,0.008742797,0.011241,0.015196,0.015475,0.013512,0.010943,0.008722865,0.007307,0.006261010,0.005593,0.005214749,0.005065,0.005763,0.007676,0.010278,0.014073,0.020163,0.021360,0.017584,0.012784,0.010099,0.007687],"q_test":[0.006769768,0.006729935,0.006690101,0.006729935,0.006680136,0.007058648,0.007597,0.007298288,0.006889266,0.006769768,0.006839433,0.007218,0.007527,0.006660204,0.005972,0.005524,0.005285,0.005355,0.005126,0.005395,0.005893,0.006999,0.008552,0.010554,0.010365,0.008941,0.007716760,0.006621,0.005863,0.005514,0.005175,0.005734,0.006611,0.007616,0.009189,0.010514,0.010215,0.009249,0.008014,0.007068613,0.006371,0.005864,0.005505,0.005285,0.005175,0.005385,0.005664,0.006112375,0.007068613,0.008184,0.009249,0.010205,0.010454,0.010464,0.009249,0.007976,0.006919,0.006202041,0.005893247,0.005524,0.005524,0.005604,0.005793,0.005913,0.006381,0.007158595,0.007647,0.008074536,0.008174,0.008144,0.007796,0.007258,0.006650,0.006242,0.006112,0.006082,0.006072,0.006401339,0.006899,0.007328,0.008353311,0.009967,0.010923,0.011041,0.009867,0.008264,0.007198,0.006500,0.005983,0.005664,0.005465,0.005345,0.005127,0.005026074,0.004976,0.005205,0.005943,0.006959,0.008861,0.010933,0.014163,0.015288,0.014223,0.011011,0.008552,0.007118,0.006172172,0.005664,0.005355,0.005116,0.005923,0.008014,0.010713,0.014411,0.020299,0.022122,0.018162,0.013757,0.010425,0.008851],"kl_test_train":0.28032127046980354,"w_norm_check":1.0},"drift":{"corr_raw":0.7124605723706064,"wrmse_raw_weighted_by_ntest":0.1847130781929184,"corr_weighted_by_qtest":0.6987112592206404,"wrmse_weighted_by_qtest":0.18597040910838595,"bins_used":89},"mean_rate":{"ybar_true":0.500556586270872,"ybar_pred_from_train_surface":0.5002589454642795,"error":-0.0002976408065924474,"ybar_train":0.5005509208248072},"test_performance":{"AUC_raw":0.6197129878303277,"Brier_raw":0.2448423230025343,"logloss_raw":0.6901900858532217,"n_test_valid":5389,"pos_rate_test_valid":0.500556586270872},"z_bin":{"frac_abs_z_gt_2":0.48314606741573035,"z_summary":{"mean":-0.1012593108086631,"std":2.876230174381875,"max_abs":6.651725275931928}}}},"summary":{"dominant_issue":{"25":"conditional_drift","100":"conditional_drift"},"notes":"h=25: occupancy KL(test||train) ~0.272 indicates sizable covariate shift in (theta,omega) state occupancy. h=100: mean-rate transfer error ~0 despite large per-bin z; conditional drift appears to cancel in aggregate. h=100: occupancy KL(test||train) ~0.280 indicates sizable covariate shift in (theta,omega) state occupancy."}}

