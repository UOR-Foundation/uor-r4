# Target Drift Analysis

- **Conversation ID:** `699b3be5-c800-832d-ae10-d402a5bc25fd`
- **Date:** 2026-02-22

---

### User (2026-02-22 12:24:57)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Determine whether apparent nonstationarity of the FAST gating surface is caused by distribution drift in R (target artifact) by using a locally-normalized target.

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
train_frac=0.7
seed=12345
FAST=[0.002,0.01]
H=[25,100]
B_theta_f=12
B_omega_f=10 (omega edges from TRAIN quantiles)
min_count_bin=20
eps=1.0

Target variants:
T1 (current): y=1{R_{i+h}-R_i>0}
T2 (local-quantile): Define S_i = R_{i+h}-R_i.
    For each i, compute q_i = percentile rank of S_i within a causal rolling window of length Wq=2000
    (use only past S values up to i, i.e., S_{i-Wq+1}..S_i; on TRAIN and TEST separately but causal).
    Define y=1{ q_i > 0.5 }  (median in rolling window)
This forces local base rate ~0.5 and should reduce drift due to changing S distribution.
Make sure not to leak future: rolling window is causal.

Steps:
1) Build theta_f, omega_f (canonical) aligned to omega indices.
2) Build R and S_i=R_{i+h}-R_i on valid indices.
3) Build y_T1 and y_T2 on TRAIN-valid and TEST-valid indices for each h.
4) For each target T in {T1,T2} and each h:
   - compute FAST 2D surface p_train[b], p_test[b] using only TRAIN-valid and TEST-valid indices.
   - compute stationarity metrics: corr, wrmse weighted by n_test, frac|z|>2.
   - compute occupancy KL(test||train) for bins (add-eps smoothing).
   - compute transfer perf on TEST using p_train[b]: AUC, Brier, logloss.

Return JSON:
{
 "params": {..., "Wq":2000},
 "per_h": {
   "25": {
     "T1": {...baseline metrics...},
     "T2": {...same metrics...},
     "delta_stationarity": {"corr_T2_minus_T1":..., "wrmse_T2_minus_T1":..., "fracz_T2_minus_T1":...},
     "delta_transfer": {"auc":..., "brier":..., "logloss":...}
   },
   "100": { ... }
 },
 "summary": {
   "is_drift_target_artifact": {"25":"yes/no/partial","100":"yes/no/partial"},
   "notes":"If T2 greatly improves stationarity while keeping predictive structure, drift is largely target-distribution shift."
 }
}

PRINT JSON ONLY.

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

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

### Tool (2026-02-22 12:25:00)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 12:25:00)

import numpy as np, math, json, os, statistics
from numpy.fft import rfft, irfft, rfftfreq

### Assistant (2026-02-22 12:25:32)

import numpy as np, math, json, os
import numpy.linalg as la
from scipy.signal import hilbert

### Assistant (2026-02-22 12:25:36)

from pathlib import Path
base="/mnt/data"
cheb=np.load(Path(base)/"chebyshev_psi.npz")
pr=np.load(Path(base)/"primes.npz")
cheb.files, pr.files[:5], pr['N'] if 'N' in pr.files else None

### Tool (2026-02-22 12:25:38)

(['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x'],
 ['N', 'primes', 'is_prime'],
 array(10000000, dtype=int32))

### Assistant (2026-02-22 12:25:39)

x_grid=cheb['x_grid'].astype(np.float64)
psi_minus_x=cheb['psi_minus_x'].astype(np.float64)
primes=pr['primes'].astype(np.int64)
len(x_grid), x_grid[:3], x_grid[-3:], psi_minus_x[:3], primes[:5], primes[-1]

### Tool (2026-02-22 12:25:41)

(100000,
 array([  1., 101., 201.]),
 array([9999701., 9999801., 9999901.]),
 array([-1.        , -2.33956825,  5.14585683]),
 array([ 2,  3,  5,  7, 11]),
 9999991)

### Assistant (2026-02-22 12:25:44)

# params
L=65536
x_min=1e5
x_max=9e6
W=2000
Delta=0.01
train_frac=0.7
seed=12345
FAST=(0.002,0.01)
H=[25,100]
B_theta=12
B_omega=10
min_count_bin=20
eps=1.0
Wq=2000

### Assistant (2026-02-22 12:25:47)

# Build uniform log grid over available data
t_raw=np.log(x_grid)
t_uni=np.linspace(t_raw.min(), t_raw.max(), L)
dt=float(t_uni[1]-t_uni[0])
# interpolate psi_minus_x onto t_uni
u_interp=np.interp(t_uni, t_raw, psi_minus_x)
x_uni=np.exp(t_uni)
# restrict to x range
kept=(x_uni>=x_min)&(x_uni<=x_max)
idx=np.where(kept)[0]
t=t_uni[idx]
x=x_uni[idx]
u_raw=u_interp[idx]
N_kept=len(u_raw)
N_kept, (x[0], x[-1]), dt

### Tool (2026-02-22 12:25:51)

(18296, (100006.32008830582, 8998557.287555724), 0.00024594622340595577)

### Assistant (2026-02-22 12:25:52)

# split
train_end=int(math.floor(train_frac*N_kept))
train_idx=np.arange(train_end)
test_idx=np.arange(train_end, N_kept)
train_end, N_kept-train_end

### Tool (2026-02-22 12:25:54)

(12807, 5489)

### Assistant (2026-02-22 12:25:55)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0,i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

u_detr = u_raw - causal_moving_average(u_raw, W)
mu=float(u_detr[train_idx].mean())
sd=float(u_detr[train_idx].std(ddof=0))
u_std=(u_detr-mu)/(sd+1e-12)
# bandpass via FFT mask on full kept signal (as in protocol)
n=N_kept
U=rfft(u_std)
freq=rfftfreq(n, d=1.0)  # cycles/sample
mask=(freq>=FAST[0])&(freq<=FAST[1])
U_bp=U*mask
u_bp=irfft(U_bp, n)
# Hilbert analytic signal
z=hilbert(u_bp)
theta=np.mod(np.angle(z), 2*np.pi)
theta_unwrap=np.unwrap(np.angle(z))  # for omega
omega=np.diff(theta_unwrap)
# wrap omega to [-pi,pi]
omega_wrapped=( (omega + np.pi)%(2*np.pi) )-np.pi
len(theta), len(omega_wrapped)

### Tool (2026-02-22 12:26:05)

(18296, 18295)

### Assistant (2026-02-22 12:26:07)

# Align to omega indices: i=0..N_kept-2
N_feat=len(omega_wrapped)
theta_f=theta[:N_feat]
omega_f=omega_wrapped
x_f=x[:N_feat]
# omega bin edges from TRAIN quantiles (on aligned train indices)
train_feat_end=min(train_end, N_feat)  # since N_feat=N_kept-1, train_end <= N_feat
omega_train=omega_f[:train_feat_end]
# quantile edges for 10 bins => 11 edges
qs=np.linspace(0,1,B_omega+1)
omega_edges=np.quantile(omega_train, qs)
# ensure strictly increasing (handle duplicates by jitter)
omega_edges_adj=omega_edges.copy()
for k in range(1,len(omega_edges_adj)):
    if omega_edges_adj[k] <= omega_edges_adj[k-1]:
        omega_edges_adj[k]=omega_edges_adj[k-1]+1e-9
omega_edges=omega_edges_adj
# theta bins uniform
theta_edges=np.linspace(0,2*np.pi,B_theta+1)
# assign bins
theta_bin=np.clip(np.digitize(theta_f, theta_edges)-1, 0, B_theta-1)
omega_bin=np.clip(np.digitize(omega_f, omega_edges)-1, 0, B_omega-1)
# combined bin index
bin_id = theta_bin*B_omega + omega_bin
n_bins=B_theta*B_omega

### Assistant (2026-02-22 12:26:17)

# Build R_i on x grid using primes (exact counts)
# For each x_i (in kept), interval [x_i, x_i*exp(Delta))
mult=math.exp(Delta)
x_all=x  # length N_kept
hi_all=x_all*mult
# prime counts via searchsorted
primes_sorted=primes
lo_idx=np.searchsorted(primes_sorted, x_all, side='left')
hi_idx=np.searchsorted(primes_sorted, hi_all, side='left')
P=hi_idx-lo_idx
E = x_all*(mult-1.0)/np.log(x_all)
R = (P - E)/np.sqrt(E)
len(R), R[:5], P[:5], E[:5]

### Tool (2026-02-22 12:26:21)

(18296,
 array([-0.67423635, -0.56924397, -0.67828172, -0.68030442, -0.57534812]),
 array([81, 82, 81, 81, 82]),
 array([87.29968001, 87.31928832, 87.33890108, 87.35851827, 87.37813992]))

### Assistant (2026-02-22 12:26:23)

def rolling_percentile_rank_causal(values, Wq):
    # values: 1D array in time order (segment-specific). 
    # returns q_i in [0,1] computed from causal window within segment
    v=np.asarray(values, dtype=np.float64)
    n=len(v)
    q=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0=max(0,i-Wq+1)
        window=v[j0:i+1]
        # percentile rank with tie handling: (#< +0.5#=)/n
        lt=np.sum(window < v[i])
        eq=np.sum(window == v[i])
        q[i]=(lt + 0.5*eq)/len(window)
    return q

def auc_score(y_true, y_score):
    y_true=np.asarray(y_true).astype(int)
    y_score=np.asarray(y_score).astype(float)
    n1=y_true.sum()
    n0=len(y_true)-n1
    if n1==0 or n0==0:
        return None
    # rank-based AUC (handle ties average)
    order=np.argsort(y_score)
    ranks=np.empty_like(order, dtype=float)
    ranks[order]=np.arange(1,len(y_score)+1)
    # tie adjustment
    # compute average ranks for ties
    sorted_scores=y_score[order]
    i=0
    while i<len(y_score):
        j=i+1
        while j<len(y_score) and sorted_scores[j]==sorted_scores[i]:
            j+=1
        if j-i>1:
            avg=(i+1 + j)/2.0
            ranks[order[i:j]]=avg
        i=j
    sum_ranks_pos=ranks[y_true==1].sum()
    auc=(sum_ranks_pos - n1*(n1+1)/2.0)/(n1*n0)
    return float(auc)

def brier_score(y_true, p):
    y=np.asarray(y_true).astype(float)
    p=np.asarray(p).astype(float)
    return float(np.mean((p-y)**2))

def logloss(y_true, p, clip=1e-12):
    y=np.asarray(y_true).astype(int)
    p=np.clip(np.asarray(p).astype(float), clip, 1-clip)
    return float(-np.mean(y*np.log(p)+(1-y)*np.log(1-p)))

def build_surface_and_metrics(bin_id, y, train_mask, test_mask, n_bins, min_count_bin, eps):
    # returns dict with surfaces and metrics comparing train/test surfaces
    bin_id=np.asarray(bin_id)
    y=np.asarray(y).astype(int)
    # counts and sums
    n_train=np.bincount(bin_id[train_mask], minlength=n_bins)
    s_train=np.bincount(bin_id[train_mask], weights=y[train_mask], minlength=n_bins)
    n_test=np.bincount(bin_id[test_mask], minlength=n_bins)
    s_test=np.bincount(bin_id[test_mask], weights=y[test_mask], minlength=n_bins)
    # probabilities
    p_train=np.full(n_bins, np.nan)
    p_test=np.full(n_bins, np.nan)
    ok_train=n_train>=min_count_bin
    ok_test=n_test>=min_count_bin
    p_train[ok_train]=s_train[ok_train]/n_train[ok_train]
    p_test[ok_test]=s_test[ok_test]/n_test[ok_test]
    ok_both=ok_train & ok_test
    # stationarity metrics
    if ok_both.sum()>=2:
        corr=float(np.corrcoef(p_train[ok_both], p_test[ok_both])[0,1])
    else:
        corr=None
    # weighted RMSE on bins where test ok and train prob available (use smoothed train prob for se)
    ok_eval = ok_test & np.isfinite(p_train)  # train must be defined
    if ok_eval.sum()>0:
        w=n_test[ok_eval].astype(float)
        rmse=np.sqrt(np.sum(w*(p_test[ok_eval]-p_train[ok_eval])**2)/np.sum(w))
        wrmse=float(rmse)
        # z-scores
        p0=np.clip(p_train[ok_eval], 1e-6, 1-1e-6)
        se=np.sqrt(p0*(1-p0)/n_test[ok_eval])
        z=(p_test[ok_eval]-p_train[ok_eval])/(se+1e-12)
        fracz=float(np.mean(np.abs(z)>2.0))
    else:
        wrmse=None
        fracz=None
    # occupancy KL(test||train)
    p_occ_test=(n_test+eps)/float(n_test.sum()+eps*n_bins)
    p_occ_train=(n_train+eps)/float(n_train.sum()+eps*n_bins)
    kl=float(np.sum(p_occ_test*np.log(p_occ_test/p_occ_train)))
    # transfer perf: predict test using p_train for bin, fallback global train mean
    global_train=float(s_train.sum()/max(1,n_train.sum()))
    p_pred = np.where(np.isfinite(p_train[bin_id]), p_train[bin_id], global_train)
    # only evaluate on test_mask indices
    y_te=y[test_mask]
    p_te=p_pred[test_mask]
    auc=auc_score(y_te, p_te)
    brier=brier_score(y_te, p_te) if len(y_te)>0 else None
    ll=logloss(y_te, p_te) if len(y_te)>0 else None
    return {
        "n_train_total": int(n_train.sum()),
        "n_test_total": int(n_test.sum()),
        "pos_rate_train": float(y[train_mask].mean()) if train_mask.sum()>0 else None,
        "pos_rate_test": float(y[test_mask].mean()) if test_mask.sum()>0 else None,
        "surface": {
            "p_train": p_train.reshape(-1).tolist(),
            "p_test": p_test.reshape(-1).tolist(),
            "n_train": n_train.tolist(),
            "n_test": n_test.tolist(),
            "ok_train_bins": int(ok_train.sum()),
            "ok_test_bins": int(ok_test.sum()),
            "ok_both_bins": int(ok_both.sum())
        },
        "stationarity": {
            "corr": corr,
            "wrmse": wrmse,
            "frac_abs_z_gt2": fracz,
            "occupancy_KL_test_train": kl
        },
        "transfer": {
            "AUC": auc,
            "Brier": brier,
            "LogLoss": ll,
            "fallback_global_train_mean": global_train
        }
    }

### Assistant (2026-02-22 12:26:57)

results={}
for h in H:
    # valid indices for S on aligned feature index set i in [0,N_feat-1] and i+h < N_kept
    max_i = min(N_feat-1, N_kept-h-1)  # since i+h <= N_kept-1; i <= N_kept-h-1
    valid = np.arange(0, max_i+1)
    # masks for train/test within aligned indices
    train_mask = valid < train_end  # i in train portion
    test_mask = valid >= train_end
    # build S
    S = R[valid + h] - R[valid]
    # T1
    y1 = (S > 0).astype(int)
    # T2: causal rolling percentile rank separately within train and test segments on valid indices
    # Need S_train and S_test sequences in time order
    S_train = S[train_mask]
    S_test = S[test_mask]
    q_train = rolling_percentile_rank_causal(S_train, Wq)
    q_test = rolling_percentile_rank_causal(S_test, Wq)
    y2_train = (q_train > 0.5).astype(int)
    y2_test = (q_test > 0.5).astype(int)
    y2 = np.empty_like(y1)
    y2[train_mask]=y2_train
    y2[test_mask]=y2_test
    # build surfaces
    bin_valid = bin_id[valid]
    out1 = build_surface_and_metrics(bin_valid, y1, train_mask, test_mask, n_bins, min_count_bin, eps)
    out2 = build_surface_and_metrics(bin_valid, y2, train_mask, test_mask, n_bins, min_count_bin, eps)
    # deltas
    def diff(a,b):
        if a is None or b is None: return None
        return float(a-b)
    delta_station = {
        "corr_T2_minus_T1": None if out1["stationarity"]["corr"] is None or out2["stationarity"]["corr"] is None else float(out2["stationarity"]["corr"]-out1["stationarity"]["corr"]),
        "wrmse_T2_minus_T1": None if out1["stationarity"]["wrmse"] is None or out2["stationarity"]["wrmse"] is None else float(out2["stationarity"]["wrmse"]-out1["stationarity"]["wrmse"]),
        "fracz_T2_minus_T1": None if out1["stationarity"]["frac_abs_z_gt2"] is None or out2["stationarity"]["frac_abs_z_gt2"] is None else float(out2["stationarity"]["frac_abs_z_gt2"]-out1["stationarity"]["frac_abs_z_gt2"]),
        "occKL_T2_minus_T1": float(out2["stationarity"]["occupancy_KL_test_train"]-out1["stationarity"]["occupancy_KL_test_train"])
    }
    delta_transfer = {
        "auc": None if out1["transfer"]["AUC"] is None or out2["transfer"]["AUC"] is None else float(out2["transfer"]["AUC"]-out1["transfer"]["AUC"]),
        "brier": None if out1["transfer"]["Brier"] is None or out2["transfer"]["Brier"] is None else float(out2["transfer"]["Brier"]-out1["transfer"]["Brier"]),
        "logloss": None if out1["transfer"]["LogLoss"] is None or out2["transfer"]["LogLoss"] is None else float(out2["transfer"]["LogLoss"]-out1["transfer"]["LogLoss"])
    }
    results[str(h)]={"T1":out1, "T2":out2, "delta_stationarity":delta_station, "delta_transfer":delta_transfer,
                     "valid_counts":{"n_valid": int(len(valid)), "n_train_valid": int(train_mask.sum()), "n_test_valid": int(test_mask.sum())}}
results.keys()

### Tool (2026-02-22 12:27:14)

dict_keys(['25', '100'])

### Assistant (2026-02-22 12:27:16)

# Determine summary yes/no/partial based on improvements thresholds
summary={}
notes=[]
for h in H:
    r=results[str(h)]
    dS=r["delta_stationarity"]
    dT=r["delta_transfer"]
    # heuristics
    corr_improve = (dS["corr_T2_minus_T1"] is not None and dS["corr_T2_minus_T1"]>0.05)
    wrmse_improve = (dS["wrmse_T2_minus_T1"] is not None and dS["wrmse_T2_minus_T1"]< -0.01)
    fracz_improve = (dS["fracz_T2_minus_T1"] is not None and dS["fracz_T2_minus_T1"]< -0.05)
    # keep predictive structure: AUC drop not too big
    auc_ok = (dT["auc"] is None or dT["auc"]>-0.02)
    if (corr_improve or wrmse_improve or fracz_improve) and auc_ok:
        verdict="yes" if (corr_improve and wrmse_improve) else "partial"
    else:
        verdict="no"
    summary[str(h)]=verdict
    notes.append(f"h={h}: Δcorr={dS['corr_T2_minus_T1']}, Δwrmse={dS['wrmse_T2_minus_T1']}, Δfracz={dS['fracz_T2_minus_T1']}, ΔAUC={dT['auc']}")
summary, notes[:2]

### Tool (2026-02-22 12:27:26)

({'25': 'no', '100': 'no'},
 ['h=25: Δcorr=0.006696818506017332, Δwrmse=-0.003426656208605383, Δfracz=0.0, ΔAUC=-0.0028238535690449096',
  'h=100: Δcorr=0.007102150391317674, Δwrmse=-8.051883249954317e-05, Δfracz=0.03370786516853941, ΔAUC=0.0017400482023955943'])

### Assistant (2026-02-22 12:27:28)

out={
 "params":{
   "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
   "FAST":list(FAST),"H":H,"B_theta_f":B_theta,"B_omega_f":B_omega,"min_count_bin":min_count_bin,"eps":eps,"Wq":Wq,
   "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end
 },
 "binning":{
   "theta_edges":theta_edges.tolist(),
   "omega_edges_train_quantiles":omega_edges.tolist()
 },
 "per_h":results,
 "summary":{
   "is_drift_target_artifact":summary,
   "notes":"; ".join(notes)
 }
}
json_str=json.dumps(out, separators=(",",":"), ensure_ascii=False)
len(json_str)

### Tool (2026-02-22 12:27:36)

22323

### Assistant (2026-02-22 12:27:37)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"Delta":0.01,"train_frac":0.7,"seed":12345,"FAST":[0.002,0.01],"H":[25,100],"B_theta_f":12,"B_omega_f":10,"min_count_bin":20,"eps":1.0,"Wq":2000,"dt":0.00024594622340595577,"n_kept":18296,"n_feat":18295,"train_end":12807},"binning":{"theta_edges":[0.0,0.5235987755982988,1.0471975511965976,1.5707963267948966,2.0943951023931953,2.617993877991494,3.141592653589793,3.665191429188092,4.1887902047863905,4.71238898038469,5.235987755982988,5.759586531581287,6.283185307179586],"omega_edges_train_quantiles":[-0.10118410874035318,-0.05775781526907423,-0.03778927391119801,-0.022663136284488397,-0.010404185080263343,0.0010455627381818522,0.012234992916431388,0.024385254140652113,0.039632288008339375,0.05960784473996328,0.10398545918468583]},"per_h":{"25":{"T1":{"n_train_total":12807,"n_test_total":5464,"pos_rate_train":0.4994924642765681,"pos_rate_test":0.4967057104685212,"surface":{"p_train":[0.4659090909090909,0.494949494949495,0.4642857142857143,0.4838709677419355,0.49038461538461536,0.47368421052631576,0.5396825396825397,0.5396825396825397,0.52,0.46938775510204084,0.5555555555555556,0.55,0.4838709677419355,0.5238095238095238,0.5,0.4864864864864865,0.5333333333333333,0.43478260869565216,0.47619047619047616,0.375,0.5384615384615384,0.55,0.5384615384615384,0.47368421052631576,0.47619047619047616,0.5510204081632653,0.5294117647058824,0.5227272727272727,0.45,0.4482758620689655,0.5,0.43478260869565216,0.4782608695652174,0.48,0.4444444444444444,0.5666666666666667,0.47368421052631576,0.6086956521739131,0.42424242424242425,0.4782608695652174,0.5714285714285714,0.5833333333333334,0.4444444444444444,0.5263157894736842,0.3939393939393939,0.5769230769230769,0.44,0.5,0.6666666666666666,0.36,0.52,0.5,0.42105263157894735,0.5238095238095238,0.4642857142857143,0.4,0.5,0.6842105263157895,0.5,0.375,0.5789473684210527,0.45,0.45454545454545453,0.5384615384615384,0.4,0.3333333333333333,0.3333333333333333,0.6666666666666666,0.2,0.5384615384615384,0.5,0.6,0.46153846153846156,0.4444444444444444,0.5714285714285714,0.5,0.2,0.25,0.5,0.0,0.5,0.5,0.0,0.5,0.6666666666666666,0.0,0.6666666666666666,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"p_test":[0.45161290322580644,0.47058823529411764,0.5614035087719298,0.5087719298245614,0.5254237288135594,0.59375,0.40540540540540543,0.391304347826087,0.48,0.5454545454545454,0.4375,0.4482758620689655,0.4642857142857143,0.35294117647058826,0.48484848484848486,0.47368421052631576,0.39473684210526316,0.4,0.5294117647058824,0.5,0.52,0.38461538461538464,0.6666666666666666,0.52,0.6046511627906976,0.4583333333333333,0.5272727272727272,0.5384615384615384,0.5833333333333334,0.6086956521739131,0.0,0.6363636363636364,0.5384615384615384,0.46153846153846156,0.5,0.3333333333333333,0.42105263157894735,0.6086956521739131,0.30434782608695654,0.5454545454545454,0.6153846153846154,0.56,0.5294117647058824,0.36363636363636365,0.46153846153846156,0.5,0.5142857142857142,0.55,0.47619047619047616,0.6190476190476191,0.46511627906976744,0.4375,0.5454545454545454,0.5384615384615384,0.6923076923076923,0.5555555555555556,0.36363636363636365,0.5625,0.3076923076923077,0.25,0.5454545454545454,0.45454545454545453,0.5,0.5,0.25,0.5,0.0,0.5555555555555556,0.3333333333333333,0.6666666666666666,0.25,0.6666666666666666,0.3333333333333333,0.3333333333333333,0.5,0.0,0.0,0.5,0.5,0.5,0.0,0.5,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"n_train":[88,99,84,93,104,95,63,63,75,98,63,60,62,63,78,74,75,69,63,56,65,60,65,76,63,98,68,88,80,58,62,69,69,75,63,60,57,69,66,69,70,72,63,76,66,78,75,70,63,75,62,75,76,63,70,56,58,57,70,64,56,57,60,65,65,60,63,60,65,75,60,55,65,63,63,70,60,55,60,60,55,60,63,55,55,60,63,58,57,62,59,59,58,53,57,55,58,56,54,53,53,51,50,52,47,52,46,50,44,47,45,47,43,41,45,42,42,39,40,36],"n_test":[31,34,57,57,59,64,37,46,75,55,64,58,56,51,66,57,76,80,68,70,50,52,42,50,43,72,55,52,36,46,15,33,39,39,32,30,38,46,46,33,39,25,34,33,26,28,35,40,21,42,43,48,33,39,26,27,22,32,26,20,22,22,26,22,20,18,16,18,16,12,16,12,9,12,12,6,4,4,2,6,4,4,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"ok_train_bins":120,"ok_test_bins":71,"ok_both_bins":71},"stationarity":{"corr":0.43846920131207026,"wrmse":0.0952250114811555,"frac_abs_z_gt2":0.0,"occupancy_KL_test_train":0.10274264323034226},"transfer":{"AUC":0.4855711828975468,"Brier":0.25581465836281124,"LogLoss":0.7153879326298197,"fallback_global_train_mean":0.4994924642765681}},"T2":{"n_train_total":12807,"n_test_total":5464,"pos_rate_train":0.5086271579617404,"pos_rate_test":0.5060395311120059,"surface":{"p_train":[0.45454545454545453,0.5353535353535354,0.47619047619047616,0.4838709677419355,0.5288461538461539,0.5473684210526316,0.5396825396825397,0.5555555555555556,0.5733333333333334,0.5102040816326531,0.5396825396825397,0.55,0.5161290322580645,0.47619047619047616,0.5,0.4864864864864865,0.5066666666666667,0.4782608695652174,0.47619047619047616,0.5178571428571429,0.5384615384615384,0.5,0.5538461538461539,0.5131578947368421,0.47619047619047616,0.5714285714285714,0.5147058823529411,0.5113636363636364,0.575,0.5517241379310345,0.532258064516129,0.4782608695652174,0.4927536231884058,0.52,0.5396825396825397,0.5166666666666667,0.49122807017543857,0.5217391304347826,0.4696969696969697,0.6086956521739131,0.45714285714285713,0.5416666666666666,0.49206349206349204,0.4342105263157895,0.4696969696969697,0.5256410256410257,0.52,0.5714285714285714,0.5714285714285714,0.6,0.46774193548387094,0.5466666666666666,0.5,0.5079365079365079,0.5285714285714286,0.4642857142857143,0.5,0.5614035087719298,0.4857142857142857,0.5,0.5789473684210527,0.6,0.5454545454545454,0.5384615384615384,0.5333333333333333,0.4666666666666667,0.45,0.5,0.4,0.5384615384615384,0.5,0.5846153846153846,0.5384615384615384,0.5238095238095238,0.5714285714285714,0.5166666666666667,0.4,0.5,0.5,0.4,0.5,0.5,0.0,0.5,0.6666666666666666,0.0,0.3333333333333333,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"p_test":[0.5483870967741935,0.4411764705882353,0.47368421052631576,0.45614035087719296,0.5254237288135594,0.453125,0.4594594594594595,0.41304347826086957,0.48,0.4909090909090909,0.46875,0.5862068965517241,0.44642857142857145,0.5490196078431373,0.5757575757575758,0.49122807017543857,0.5526315789473685,0.5375,0.45588235294117646,0.5142857142857142,0.44,0.5192307692307693,0.5714285714285714,0.62,0.5581395348837209,0.5,0.5454545454545454,0.5384615384615384,0.3333333333333333,0.6304347826086957,0.4666666666666667,0.45454545454545453,0.5641025641025641,0.5128205128205128,0.46875,0.43333333333333335,0.4473684210526316,0.45652173913043476,0.41304347826086957,0.5454545454545454,0.48717948717948717,0.6,0.5294117647058824,0.45454545454545453,0.4230769230769231,0.5357142857142857,0.45714285714285713,0.45,0.5714285714285714,0.47619047619047616,0.5116279069767442,0.5416666666666666,0.5151515151515151,0.5128205128205128,0.5384615384615384,0.48148148148148145,0.36363636363636365,0.65625,0.34615384615384615,0.35,0.5,0.4090909090909091,0.5,0.6363636363636364,0.35,0.6111111111111112,0.3125,0.625,0.625,0.25,0.75,0.3333333333333333,0.5,0.25,0.6666666666666666,0.5,0.3333333333333333,0.5,0.0,0.25,0.5,0.5,0.5,0.0,0.0,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"n_train":[88,99,84,93,104,95,63,63,75,98,63,60,62,63,78,74,75,69,63,56,65,60,65,76,63,98,68,88,80,58,62,69,69,75,63,60,57,69,66,69,70,72,63,76,66,78,75,70,63,75,62,75,76,63,70,56,58,57,70,64,56,57,60,65,65,60,63,60,65,75,60,55,65,63,63,70,60,55,60,60,55,60,63,55,55,60,63,58,57,62,59,59,58,53,57,55,58,56,54,53,53,51,50,52,47,52,46,50,44,47,45,47,43,41,45,42,42,39,40,36],"n_test":[31,34,57,57,59,64,37,46,75,55,64,58,56,51,66,57,76,80,68,70,50,52,42,50,43,72,55,52,36,46,15,33,39,39,32,30,38,46,46,33,39,25,34,33,26,28,35,40,21,42,43,48,33,39,26,27,22,32,26,20,22,22,26,22,20,18,16,18,16,12,16,12,9,12,12,6,4,4,2,6,4,4,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"ok_train_bins":120,"ok_test_bins":71,"ok_both_bins":71},"stationarity":{"corr":0.4451660198180876,"wrmse":0.09179835527255011,"frac_abs_z_gt2":0.0,"occupancy_KL_test_train":0.10274264323034226},"transfer":{"AUC":0.4827473293285019,"Brier":0.25463935901589744,"LogLoss":0.712970254170509,"fallback_global_train_mean":0.5086271579617404}},"delta_stationarity":{"corr_T2_minus_T1":0.006696818506017332,"wrmse_T2_minus_T1":-0.003426656208605383,"fracz_T2_minus_T1":0.0,"occKL_T2_minus_T1":0.0},"delta_transfer":{"auc":-0.0028238535690449096,"brier":-0.0011752993469138022,"logloss":-0.002417678459310708},"valid_counts":{"n_valid":18271,"n_train_valid":12807,"n_test_valid":5464}},"100":{"T1":{"n_train_total":12796,"n_test_total":5390,"pos_rate_train":0.4991403563613629,"pos_rate_test":0.4962894248608534,"surface":{"p_train":[0.48863636363636365,0.47474747474747475,0.47619047619047616,0.5,0.5096153846153846,0.5684210526315789,0.5079365079365079,0.5079365079365079,0.49333333333333335,0.46938775510204084,0.5079365079365079,0.48333333333333334,0.41935483870967744,0.5396825396825397,0.44871794871794873,0.5135135135135135,0.5466666666666666,0.4492753623188406,0.49206349206349204,0.46875,0.49230769230769234,0.4666666666666667,0.5076923076923077,0.4868421052631579,0.4365079365079365,0.4897959183673469,0.5294117647058824,0.5340909090909091,0.475,0.603448275862069,0.4838709677419355,0.4492753623188406,0.5217391304347826,0.5066666666666667,0.5396825396825397,0.48333333333333334,0.543859649122807,0.4782608695652174,0.5454545454545454,0.6666666666666666,0.5,0.6111111111111112,0.4603174603174603,0.4605263157894737,0.45454545454545453,0.47435897435897434,0.5066666666666667,0.5142857142857142,0.5079365079365079,0.52,0.5161290322580645,0.4533333333333333,0.47368421052631576,0.5079365079365079,0.5142857142857142,0.5178571428571429,0.5,0.5964912280701754,0.5,0.5625,0.525,0.4824561403508772,0.42857142857142855,0.5,0.5076923076923077,0.5,0.48333333333333334,0.4,0.5,0.3333333333333333,0.5384615384615384,0.5,0.5692307692307692,0.5846153846153846,0.4444444444444444,0.5384615384615384,0.5,0.4,0.5,0.5,0.6,0.5,0.5,0.0,0.5,0.3333333333333333,0.0,0.3333333333333333,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"p_test":[0.5238095238095238,0.45454545454545453,0.4057971014492754,0.4782608695652174,0.46551724137931033,0.4166666666666667,0.5277777777777778,0.5306122448979592,0.5128205128205128,0.5,0.43478260869565216,0.5172413793103449,0.46875,0.425531914893617,0.5373134328358209,0.4444444444444444,0.6153846153846154,0.4666666666666667,0.4745762711864407,0.4262295081967213,0.5740740740740741,0.4444444444444444,0.4888888888888889,0.5294117647058824,0.5588235294117647,0.4838709677419355,0.43478260869565216,0.4857142857142857,0.4583333333333333,0.42857142857142855,0.5,0.5428571428571428,0.55,0.5,0.5,0.46153846153846156,0.6,0.4186046511627907,0.4069767441860465,0.5,0.47619047619047616,0.6666666666666666,0.5,0.46153846153846156,0.3333333333333333,0.5384615384615384,0.5285714285714286,0.5,0.5625,0.5,0.5897435897435898,0.43902439024390244,0.46153846153846156,0.46153846153846156,0.5357142857142857,0.2962962962962963,0.5555555555555556,0.5,0.45454545454545453,0.7142857142857143,0.4166666666666667,0.6,0.6363636363636364,0.5,0.0,0.5555555555555556,0.4,0.3333333333333333,0.6,0.0,0.5,0.375,0.5,0.75,0.5,0.3333333333333333,0.6666666666666666,0.0,0.0,0.5,0.5,0.5,0.0,0.5,0.0,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"n_train":[88,99,84,92,104,95,63,63,75,98,63,60,62,63,78,74,75,69,63,64,65,60,65,76,63,98,68,88,80,58,62,69,69,75,63,60,57,69,66,66,70,72,63,76,66,78,75,70,63,75,62,75,76,63,70,56,58,57,70,64,56,57,60,65,65,60,63,60,65,75,60,55,65,63,63,70,60,55,60,60,55,60,63,55,55,60,63,58,57,62,59,59,58,53,57,55,58,56,54,53,53,51,50,52,47,52,46,50,44,47,45,47,43,41,45,42,42,39,40,36],"n_test":[21,22,69,46,58,60,36,49,78,48,46,58,64,47,67,72,65,75,59,61,54,54,45,51,34,62,46,70,48,56,32,35,40,34,28,26,20,43,43,32,21,21,26,26,18,26,35,26,16,22,39,41,26,26,28,27,18,28,22,21,24,30,33,30,26,25,15,15,10,5,8,5,5,8,4,4,2,2,2,4,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"ok_train_bins":120,"ok_test_bins":68,"ok_both_bins":68},"stationarity":{"corr":0.5076803743965021,"wrmse":0.1039052310164613,"frac_abs_z_gt2":0.0,"occupancy_KL_test_train":0.11126513911250982},"transfer":{"AUC":0.4960793672388633,"Brier":0.2566230949949323,"LogLoss":0.7184585528022328,"fallback_global_train_mean":0.4991403563613629}},"T2":{"n_train_total":12796,"n_test_total":5390,"pos_rate_train":0.508674585808065,"pos_rate_test":0.5076066790352505,"surface":{"p_train":[0.5,0.5252525252525253,0.4880952380952381,0.5,0.5192307692307693,0.4842105263157895,0.5238095238095238,0.5396825396825397,0.49333333333333335,0.5,0.47619047619047616,0.45,0.4838709677419355,0.4444444444444444,0.5256410256410257,0.527027027027027,0.5066666666666667,0.4782608695652174,0.5079365079365079,0.5,0.5076923076923077,0.45,0.5230769230769231,0.4605263157894737,0.5714285714285714,0.5306122448979592,0.5147058823529411,0.5795454545454546,0.575,0.5689655172413793,0.5483870967741935,0.5217391304347826,0.463768115942029,0.5466666666666666,0.4126984126984127,0.48333333333333334,0.5263157894736842,0.5507246376811594,0.4393939393939394,0.5151515151515151,0.5857142857142857,0.5277777777777778,0.49206349206349204,0.4868421052631579,0.5,0.5641025641025641,0.44,0.5,0.5238095238095238,0.4533333333333333,0.43548387096774194,0.5866666666666667,0.47368421052631576,0.49206349206349204,0.45714285714285713,0.5357142857142857,0.4827586206896552,0.49122807017543857,0.5285714285714286,0.484375,0.44642857142857145,0.6140350877192983,0.4444444444444444,0.49230769230769234,0.49230769230769234,0.5,0.5,0.6,0.46153846153846156,0.3333333333333333,0.46153846153846156,0.5,0.5692307692307692,0.5846153846153846,0.5,0.5,0.5,0.4166666666666667,0.2,0.5,0.5,0.5,0.5,0.0,0.5,0.0,0.0,0.3333333333333333,0.0,0.3333333333333333,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"p_test":[0.47619047619047616,0.5909090909090909,0.4927536231884058,0.4782608695652174,0.4827586206896552,0.5166666666666667,0.5555555555555556,0.46938775510204084,0.5128205128205128,0.5208333333333334,0.4782608695652174,0.4827586206896552,0.546875,0.6170212765957447,0.5223880597014925,0.5277777777777778,0.49230769230769234,0.49333333333333335,0.4745762711864407,0.45901639344262296,0.46296296296296297,0.5185185185185185,0.6,0.5686274509803921,0.35294117647058826,0.532258064516129,0.5652173913043478,0.37142857142857144,0.4375,0.5714285714285714,0.4375,0.4,0.525,0.38235294117647056,0.39285714285714285,0.5769230769230769,0.55,0.627906976744186,0.5581395348837209,0.4375,0.6190476190476191,0.5714285714285714,0.5,0.5,0.3888888888888889,0.4230769230769231,0.4,0.4230769230769231,0.5625,0.5454545454545454,0.5641025641025641,0.5121951219512195,0.4230769230769231,0.46153846153846156,0.4642857142857143,0.4444444444444444,0.6111111111111112,0.5357142857142857,0.5909090909090909,0.42857142857142855,0.5416666666666666,0.3,0.2727272727272727,0.45454545454545453,0.5666666666666667,0.4444444444444444,0.8,0.26666666666666666,0.4,0.5,0.0,0.5,0.25,0.25,0.25,0.0,0.5,0.6666666666666666,0.5,0.0,0.5,0.0,0.0,0.5,0.0,null,0.0,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"n_train":[88,99,84,92,104,95,63,63,75,98,63,60,62,63,78,74,75,69,63,64,65,60,65,76,63,98,68,88,80,58,62,69,69,75,63,60,57,69,66,66,70,72,63,76,66,78,75,70,63,75,62,75,76,63,70,56,58,57,70,64,56,57,60,65,65,60,63,60,65,75,60,55,65,63,63,70,60,55,60,60,55,60,63,55,55,60,63,58,57,62,59,59,58,53,57,55,58,56,54,53,53,51,50,52,47,52,46,50,44,47,45,47,43,41,45,42,42,39,40,36],"n_test":[21,22,69,46,58,60,36,49,78,48,46,58,64,47,67,72,65,75,59,61,54,54,45,51,34,62,46,70,48,56,32,35,40,34,28,26,20,43,43,32,21,21,26,26,18,26,35,26,16,22,39,41,26,26,28,27,18,28,22,21,24,30,33,30,26,25,15,15,10,5,8,5,5,8,4,4,2,2,2,4,2,2,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"ok_train_bins":120,"ok_test_bins":68,"ok_both_bins":68},"stationarity":{"corr":0.5147825247878198,"wrmse":0.10382471218496175,"frac_abs_z_gt2":0.033707865168539325,"occupancy_KL_test_train":0.11126513911250982},"transfer":{"AUC":0.49781941544125886,"Brier":0.2571152806208874,"LogLoss":0.7194576966624294,"fallback_global_train_mean":0.508674585808065}},"delta_stationarity":{"corr_T2_minus_T1":0.007102150391317674,"wrmse_T2_minus_T1":-8.051883249954317e-05,"fracz_T2_minus_T1":0.033707865168539325,"occKL_T2_minus_T1":0.0},"delta_transfer":{"auc":0.0017400482023955943,"brier":0.0004921856259550711,"logloss":0.0009991438601966138},"valid_counts":{"n_valid":18196,"n_train_valid":12796,"n_test_valid":5390}}},"summary":{"is_drift_target_artifact":{"25":"no","100":"no"},"notes":"h=25: Δcorr=0.006696818506017332, Δwrmse=-0.003426656208605383, Δfracz=0.0, ΔAUC=-0.0028238535690449096; h=100: Δcorr=0.007102150391317674, Δwrmse=-8.051883249954317e-05, Δfracz=0.03370786516853941, ΔAUC=0.0017400482023955943"}}

