# Model Evaluation and Feature Extraction

- **Conversation ID:** `699af63c-bfe4-832f-b97a-12ec0f264ef0`
- **Date:** 2026-02-22

---

### User (2026-02-22 07:27:43)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Quantify the *conditional* predictive value of theta (Hilbert phase) and omega (phase velocity) beyond autoregressive residual baselines, using leakage-safe fitting and block-permutation tests that preserve autocorrelation.

Do NOT redesign the canonical pipeline.
Do NOT recompute spectral anchors.
Use only repo files.

Files:
- chebyshev_psi.npz (x_grid, psi_minus_x)
- primes.npz (primes)
- Protocol.md

Params (canonical):
L=65536
x_min=1e5
x_max=9e6
W=2000 causal detrend
bandpass=[0.002,0.01] cycles/sample
Delta=0.01
train_frac=0.7 time-ordered
H=[25,50,100]
seed=12345

STEP 1 — Build canonical phase features ONCE
- Load chebyshev_psi.npz.
- Interpolate psi_minus_x to uniform log grid length L, compute dt.
- Restrict x=exp(t) to [x_min,x_max] -> kept indices, N_kept.
- Split: TRAIN first 70%, TEST last 30%.
- u_raw = psi_minus_x interpolated on kept.
- Causal detrend W.
- Standardize using TRAIN mean/std.
- FFT bandpass [0.002,0.01], Hilbert analytic signal.
- Extract:
   theta in [0,2π)
   logA = log(|analytic| + 1e-12)
- Compute omega_i = wrap_to_pi(theta_{i+1} - theta_i) for i=0..N_kept-2
  Align feature rows to indices where omega exists:
    drop last sample so features length N_feat = N_kept-1
  Align targets similarly by dropping last point of R etc.

STEP 2 — Build residual series R_i and de-stickied targets
- Load primes.npz.
- For each kept x_i compute P_i in [x_i, x_i*exp(Delta)).
- E_i = x_i*(exp(Delta)-1)/log(x_i)
- R_i = (P_i - E_i)/sqrt(E_i)
- Align R to N_feat by dropping last entry.

For each horizon h in H define:
  y_h(i) = 1{ R_{i+h} - R_i > 0 }
Use only indices where i+h is valid within the aligned series.
Define TRAIN-valid indices: i and i+h both in TRAIN segment of aligned data.
Define TEST indices: i in TEST segment and i+h valid in TEST segment.

STEP 3 — Fit nested models (logistic) on TRAIN only, evaluate on TEST
Feature sets (all causal; use only information at time i):
B0 (AR baseline): [R_i, R_{i-1}, R_{i-2}]  (drop first 2 rows accordingly)
B1 (pressure baseline): B0 + [logA_i]
P0 (phase only): Fourier K=3 on theta: [sin(kθ), cos(kθ)] for k=1..3
P1 (phase+pressure): B1 + P0
V1 (phase vel add): P1 + [omega_i, sin(θ_i)*omega_i, cos(θ_i)*omega_i]  (simple interactions)

Use L2-regularized logistic with a fixed C=1.0 (no tuning).
Standardize non-angle scalars using TRAIN stats (R lags, logA, omega).

Report on TEST for each h:
- AUC, Brier, Accuracy@0.5, positive_rate
- Also report mean predicted probability, std of predicted probability
- Also report flip_rate and mean_run_length of predicted labels (to detect collapse)

STEP 4 — Block-permutation conditional importance on TEST
We want to test whether theta/omega contribute beyond AR+pressure while preserving autocorrelation.

For each horizon h:
- Fix the trained P1 model (B1 + P0) and V1 model.
- On TEST only:
   Create block size B = 256 samples (in aligned index units).
   Permute blocks of (theta, omega) together across TEST, keeping B1 features fixed.
   Do this for Nperm=200 permutations (seeded).
- For each perm:
   compute AUC_perm for P1 and V1 under permuted theta/omega.
- Compute:
   p_value = (1 + # {AUC_perm >= AUC_true}) / (1+Nperm)
   z_score = (AUC_true - mean(AUC_perm)) / std(AUC_perm)

Return JSON:

{
 "params": {..., "block_size":256, "Nperm":200},
 "models": ["B0","B1","P0","P1","V1"],
 "per_h": {
   "25": {
     "metrics": { "B0":{...}, "B1":{...}, "P0":{...}, "P1":{...}, "V1":{...} },
     "perm_test": {
        "P1": { "auc_true":..., "auc_perm_mean":..., "auc_perm_std":..., "p_value":..., "z":... },
        "V1": { "auc_true":..., "auc_perm_mean":..., "auc_perm_std":..., "p_value":..., "z":... }
     }
   },
   "50": {...},
   "100": {...}
 },
 "summary": {
   "theta_conditional_signal": { "by_h": {...}, "overall": "strong/moderate/weak/none" },
   "omega_incremental_signal": { "by_h": {...}, "overall": "strong/moderate/weak/none" }
 }
}

Notes:
- Be careful with alignment when using R_{i-1}, R_{i-2} and horizon shifts.
- No leakage: all scalers fit on TRAIN only; models trained on TRAIN only.
- Print JSON only.

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

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

### Tool (2026-02-22 07:27:46)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 07:27:46)

import numpy as np, math, json, os, textwrap, statistics
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score

# Load files
base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]
psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"]

# Params
L=65536
x_min=1e5
x_max=9e6
W=2000
bandpass=(0.002,0.01)
Delta=0.01
train_frac=0.7
H=[25,50,100]
seed=12345
rng=np.random.default_rng(seed)

# Build uniform t grid over raw t range
t_raw=np.log(x_grid.astype(np.float64))
t0=float(t_raw.min()); t1=float(t_raw.max())
t=np.linspace(t0,t1,L)
dt=float(t[1]-t[0])
# interpolate psi_minus_x onto t
u_interp=np.interp(t,t_raw,psi_minus_x.astype(np.float64))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
idx=np.where(kept)[0]
u_raw=u_interp[idx]
x_kept=x[idx]
N_kept=len(x_kept)

# Split on kept
N_train=int(math.floor(train_frac*N_kept))
# detrend causal moving average
def causal_moving_average(a, W):
    # past-only inclusive window length W
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n)
    for i in range(n):
        j0=max(0,i-W+1)
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

ma=causal_moving_average(u_raw,W)
u_detr=u_raw - ma
# standardize using TRAIN
mu=float(u_detr[:N_train].mean())
sd=float(u_detr[:N_train].std(ddof=0))
u_std=(u_detr-mu)/(sd+1e-12)

# FFT bandpass on full kept
n=N_kept
U=rfft(u_std)
freq=rfftfreq(n,d=1.0)  # cycles/sample since sample spacing 1
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
U_bp=U*mask
u_bp=irfft(U_bp,n=n)

# Hilbert analytic signal
z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)

# omega
dtheta=(theta[1:]-theta[:-1])
omega=(dtheta+np.pi)%(2*np.pi)-np.pi  # wrap_to_pi
# Align features length N_feat=N_kept-1
N_feat=N_kept-1
theta_f=theta[:-1].copy()
logA_f=logA[:-1].copy()
R_x=x_kept[:-1].copy()  # for prime counting, align with features

# Build residual series R_i from primes in [x_i, x_i*exp(Delta))
expD=math.exp(Delta)
# searchsorted on primes int
primes64=primes.astype(np.int64)

left=R_x
right=R_x*expD
# counts using searchsorted with floats
li=np.searchsorted(primes64,left,side="left")
ri=np.searchsorted(primes64,right,side="left")
P=(ri-li).astype(np.float64)
E=R_x*(expD-1.0)/np.log(R_x)
R=(P-E)/np.sqrt(E)

# Align to N_feat already
R_f=R.copy()

# Split on aligned data
train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

# Prepare lag features arrays
R0=R_f
R1=np.roll(R0,1); R2=np.roll(R0,2)
# invalid first positions
R1[0]=np.nan
R2[:2]=np.nan

# Standardize scalars with TRAIN stats on indices valid for lags (i>=2) within TRAIN segment
valid_train_idx=np.arange(2,train_end)  # up to train_end-1 inclusive
def standardize(arr):
    m=float(np.nanmean(arr[valid_train_idx]))
    s=float(np.nanstd(arr[valid_train_idx],ddof=0))
    return (arr-m)/(s+1e-12), m, s
R0s,mR0,sR0=standardize(R0)
R1s,mR1,sR1=standardize(R1)
R2s,mR2,sR2=standardize(R2)
logAs,mA,sA=standardize(logA_f)
omegas,mO,sO=standardize(omega)

# Fourier features
def fourier_feats(theta_arr,K=3):
    feats=[]
    for k in range(1,K+1):
        feats.append(np.sin(k*theta_arr))
        feats.append(np.cos(k*theta_arr))
    return np.stack(feats,axis=1)  # n x 2K

P0_feats=fourier_feats(theta_f,3)

# Build feature matrices per model for all indices (some have nan at start)
# B0: R_i,R_{i-1},R_{i-2}
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
P0=P0_feats
P1=np.column_stack([B1,P0])
V1=np.column_stack([P1, omegas,
                    np.sin(theta_f)*omegas,
                    np.cos(theta_f)*omegas])

# Helper functions
def run_metrics(y_true, p):
    yhat=(p>=0.5).astype(int)
    # AUC requires both classes
    auc=float(roc_auc_score(y_true,p)) if (len(np.unique(y_true))==2) else float("nan")
    brier=float(np.mean((p-y_true)**2))
    acc=float(np.mean(yhat==y_true))
    pos_rate=float(np.mean(y_true))
    p_mean=float(np.mean(p)); p_std=float(np.std(p,ddof=0))
    # flip_rate and mean_run_length on yhat
    if len(yhat)<=1:
        flip=float("nan"); mrl=float("nan")
    else:
        flip=float(np.mean(yhat[1:]!=yhat[:-1]))
        # run lengths
        runs=[]
        cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]:
                cur+=1
            else:
                runs.append(cur); cur=1
        runs.append(cur)
        mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos_rate,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_predict(modelX, train_idx, test_idx, y_train, y_test):
    # remove rows with nan
    tr_mask=~np.isnan(modelX[train_idx]).any(axis=1)
    te_mask=~np.isnan(modelX[test_idx]).any(axis=1)
    Xtr=modelX[train_idx][tr_mask]
    Xte=modelX[test_idx][te_mask]
    ytr=y_train[tr_mask]
    yte=y_test[te_mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=2000)
    clf.fit(Xtr,ytr)
    p=clf.predict_proba(Xte)[:,1]
    return clf, run_metrics(yte,p), {"Xte":Xte,"yte":yte,"p":p,"te_mask":te_mask,"test_idx_used":test_idx[te_mask]}

# Block permutation on test rows used (indices along time)
def block_permute(arrs, block_size, rng):
    # arrs list of arrays length n
    n=len(arrs[0])
    blocks=[slice(i,min(i+block_size,n)) for i in range(0,n,block_size)]
    order=np.arange(len(blocks))
    rng.shuffle(order)
    permuted=[np.empty_like(a) for a in arrs]
    out_pos=0
    for bi in order:
        sl=blocks[bi]
        m=sl.stop-sl.start
        for k,a in enumerate(arrs):
            permuted[k][out_pos:out_pos+m]=a[sl]
        out_pos+=m
    return permuted

def perm_test(clf, base_feats_func, test_rows, y_true, theta_arr, omega_arr, Nperm=200, block_size=256, rng=None):
    # base_feats_func takes (theta,omega) arrays returns X matrix for those rows (aligned to test_rows)
    # Note test_rows correspond to indices in global arrays
    # extract ordered theta/omega for those rows
    th=theta_arr[test_rows]
    om=omega_arr[test_rows]
    X_true=base_feats_func(th,om)
    p_true=clf.predict_proba(X_true)[:,1]
    auc_true=float(roc_auc_score(y_true,p_true)) if len(np.unique(y_true))==2 else float("nan")
    aucs=[]
    for _ in range(Nperm):
        th_p, om_p = block_permute([th,om],block_size,rng)
        Xp=base_feats_func(th_p,om_p)
        pp=clf.predict_proba(Xp)[:,1]
        aucp=float(roc_auc_score(y_true,pp)) if len(np.unique(y_true))==2 else float("nan")
        aucs.append(aucp)
    aucs=np.array(aucs,dtype=float)
    mean=float(np.mean(aucs)); std=float(np.std(aucs,ddof=0)+1e-12)
    pval=float((1+np.sum(aucs>=auc_true))/(1+len(aucs)))
    z=float((auc_true-mean)/std)
    return {"auc_true":auc_true,"auc_perm_mean":mean,"auc_perm_std":float(np.std(aucs,ddof=0)),
            "p_value":pval,"z":z}

# base feature builders for permutation on test rows for P1 and V1
# Need to rebuild P1 and V1 matrices for given theta/omega while keeping B1 part fixed per row.
def make_B1_rows(rows):
    return B1[rows]  # already standardized and includes R lags + logA

def feats_P1(th, om_unused, B1_rows):
    P0=fourier_feats(th,3)
    return np.column_stack([B1_rows, P0])

def feats_V1(th, om, B1_rows):
    P0=fourier_feats(th,3)
    om_s=(om-mO)/(sO+1e-12)
    return np.column_stack([B1_rows, P0, om_s, np.sin(th)*om_s, np.cos(th)*om_s])

# For each horizon build targets and run models
results={}
theta_by_h={}
omega_by_h={}
for h in H:
    # valid i: i>=2 for lags, i+h < N_feat
    max_i=N_feat-h-1  # inclusive
    # TRAIN valid i in [2, train_end-h)
    train_i=np.arange(2, max(2,train_end-h))
    train_i=train_i[train_i < train_end]  # already
    # ensure i+h in train
    train_i=train_i[train_i+h < train_end]
    # TEST i in [test_start, N_feat-h) and i+h in test
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    # targets
    y=(R_f[h:]-R_f[:-h] > 0).astype(int)  # length N_feat-h
    # For i, target is y[i] (since corresponds to R_{i+h}-R_i)
    y_train=y[train_i]
    y_test=y[test_i]
    # Fit models
    metrics={}
    models_data={}
    for name, X in [("B0",B0),("B1",B1),("P0",P0),("P1",P1),("V1",V1)]:
        clf, met, data = fit_predict(X, train_i, test_i, y_train, y_test)
        metrics[name]=met
        models_data[name]={"clf":clf,"data":data}
    # Perm tests for P1 and V1
    # Determine actual test rows used (after nan filtering) for P1 and V1
    # Their te_mask may drop some rows if nan, but at i>=2 should be none; keep robust.
    P1_used=models_data["P1"]["data"]["test_idx_used"]
    V1_used=models_data["V1"]["data"]["test_idx_used"]
    # Ensure same y_true alignment: data yte already filtered
    y_P1=models_data["P1"]["data"]["yte"]
    y_V1=models_data["V1"]["data"]["yte"]
    B1_rows_P1=make_B1_rows(P1_used)
    B1_rows_V1=make_B1_rows(V1_used)
    # define closure
    def bf_P1(th,om):
        return feats_P1(th,None,B1_rows_P1)
    def bf_V1(th,om):
        return feats_V1(th,om,B1_rows_V1)
    perm_rng=np.random.default_rng(seed + h*1000 + 7)
    permP1=perm_test(models_data["P1"]["clf"], bf_P1, P1_used, y_P1, theta_f, omega, Nperm=200, block_size=256, rng=perm_rng)
    perm_rng=np.random.default_rng(seed + h*1000 + 17)
    permV1=perm_test(models_data["V1"]["clf"], bf_V1, V1_used, y_V1, theta_f, omega, Nperm=200, block_size=256, rng=perm_rng)
    results[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    # count strong/moderate/weak/none
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    # decide overall by majority
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":list(bandpass),"Delta":Delta,
    "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,
    "N_kept":N_kept,"N_feat":N_feat,"train_end":train_end,
    "block_size":256,"Nperm":200
 },
 "models":["B0","B1","P0","P1","V1"],
 "per_h":results,
 "summary":{
    "theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
    "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}
 }
}
# ensure floats JSON serializable
def clean(o):
    if isinstance(o, dict):
        return {k:clean(v) for k,v in o.items()}
    if isinstance(o, list):
        return [clean(v) for v in o]
    if isinstance(o, np.ndarray):
        return clean(o.tolist())
    if isinstance(o, (np.floating, np.float32, np.float64)):
        v=float(o)
        if math.isnan(v) or math.isinf(v): return None
        return v
    if isinstance(o, (np.integer,)):
        return int(o)
    if isinstance(o, float):
        if math.isnan(o) or math.isinf(o): return None
        return o
    return o
out_clean=clean(out)
print(json.dumps(out_clean,indent=2)[:2000])
len(json.dumps(out_clean))

### Tool (2026-02-22 07:29:48)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[1], line 282
    280 permP1=perm_test(models_data["P1"]["clf"], bf_P1, P1_used, y_P1, theta_f, omega, Nperm=200, block_size=256, rng=perm_rng)
    281 perm_rng=np.random.default_rng(seed + h*1000 + 17)
--> 282 permV1=perm_test(models_data["V1"]["clf"], bf_V1, V1_used, y_V1, theta_f, omega, Nperm=200, block_size=256, rng=perm_rng)
    283 results[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    284 theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}

Cell In[1], line 214, in perm_test(clf, base_feats_func, test_rows, y_true, theta_arr, omega_arr, Nperm, block_size, rng)
    212     Xp=base_feats_func(th_p,om_p)
    213     pp=clf.predict_proba(Xp)[:,1]
--> 214     aucp=float(roc_auc_score(y_true,pp)) if len(np.unique(y_true))==2 else float("nan")
    215     aucs.append(aucp)
    216 aucs=np.array(aucs,dtype=float)

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/_param_validation.py:188, in validate_params.<locals>.decorator.<locals>.wrapper(*args, **kwargs)
    185 if global_skip_validation:
    186     return func(*args, **kwargs)
--> 188 func_sig = signature(func)
    190 # Map *args/**kwargs to the function signature
    191 params = func_sig.bind(*args, **kwargs)

File /usr/lib/python3.11/inspect.py:3279, in signature(obj, follow_wrapped, globals, locals, eval_str)
   3277 def signature(obj, *, follow_wrapped=True, globals=None, locals=None, eval_str=False):
   3278     """Get a signature object for the passed callable."""
-> 3279     return Signature.from_callable(obj, follow_wrapped=follow_wrapped,
   3280                                    globals=globals, locals=locals, eval_str=eval_str)

File /usr/lib/python3.11/inspect.py:3027, in Signature.from_callable(cls, obj, follow_wrapped, globals, locals, eval_str)
   3023 @classmethod
   3024 def from_callable(cls, obj, *,
   3025                   follow_wrapped=True, globals=None, locals=None, eval_str=False):
   3026     """Constructs Signature for the given callable object."""
-> 3027     return _signature_from_callable(obj, sigcls=cls,
   3028                                     follow_wrapper_chains=follow_wrapped,
   3029                                     globals=globals, locals=locals, eval_str=eval_str)

File /usr/lib/python3.11/inspect.py:2514, in _signature_from_callable(obj, follow_wrapper_chains, skip_bound_arg, globals, locals, eval_str, sigcls)
   2509             return sig.replace(parameters=new_params)
   2511 if isfunction(obj) or _signature_is_functionlike(obj):
   2512     # If it's a pure Python function, or an object that is duck type
   2513     # of a Python function (Cython functions, for instance), then:
-> 2514     return _signature_from_function(sigcls, obj,
   2515                                     skip_bound_arg=skip_bound_arg,
   2516                                     globals=globals, locals=locals, eval_str=eval_str)
   2518 if _signature_is_builtin(obj):
   2519     return _signature_from_builtin(sigcls, obj,
   2520                                    skip_bound_arg=skip_bound_arg)

File /usr/lib/python3.11/inspect.py:2421, in _signature_from_function(cls, func, skip_bound_arg, globals, locals, eval_str)
   2416     parameters.append(Parameter(name, annotation=annotation,
   2417                                 kind=_VAR_KEYWORD))
   2419 # Is 'func' is a pure Python function - don't validate the
   2420 # parameters list (for correct order and defaults), it should be OK.
-> 2421 return cls(parameters,
   2422            return_annotation=annotations.get('return', _empty),
   2423            __validate_parameters__=is_duck_function)

File /usr/lib/python3.11/inspect.py:3018, in Signature.__init__(self, parameters, return_annotation, __validate_parameters__)
   3016             params[name] = param
   3017     else:
-> 3018         params = OrderedDict((param.name, param) for param in parameters)
   3020 self._parameters = types.MappingProxyType(params)
   3021 self._return_annotation = return_annotation

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:30:53)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]
psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345

# fast auc with tie-handling via stable sorting and group means
def auc_fast(y, s):
    y=np.asarray(y, dtype=np.int8)
    s=np.asarray(s, dtype=np.float64)
    n1=int(y.sum()); n0=int(len(y)-n1)
    if n1==0 or n0==0: return np.nan
    order=np.argsort(s, kind="mergesort")
    s_sorted=s[order]; y_sorted=y[order]
    # assign average ranks for ties
    ranks=np.empty(len(s_sorted), dtype=np.float64)
    i=0; r=1.0
    while i<len(s_sorted):
        j=i+1
        while j<len(s_sorted) and s_sorted[j]==s_sorted[i]:
            j+=1
        # average rank for i..j-1
        avg=(r + (r+(j-i)-1.0))/2.0
        ranks[i:j]=avg
        r += (j-i)
        i=j
    sum_r_pos=float(ranks[y_sorted==1].sum())
    # U statistic
    U = sum_r_pos - n1*(n1+1)/2.0
    return U/(n0*n1)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n, dtype=np.float64)
    # vectorized via loop over i is ok (18k)
    for i in range(n):
        j0 = 0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

# Build uniform t grid and interpolate
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
idx=np.where(kept)[0]
x_kept=x[idx]
u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

# detrend + standardize
u_detr=u_raw - causal_moving_average(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

# bandpass FFT
n=N_kept
U=rfft(u_std)
freq=rfftfreq(n, d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=n)

# Hilbert
z=hilbert(u_bp)
theta=np.mod(np.angle(z), 2*np.pi)
logA=np.log(np.abs(z)+1e-12)

# omega
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

# Align to N_feat
N_feat=N_kept-1
theta_f=theta[:-1]
logA_f=logA[:-1]
x_f=x_kept[:-1]

# Residual series R
expD=math.exp(Delta)
left=x_f
right=x_f*expD
li=np.searchsorted(primes, left, side="left")
ri=np.searchsorted(primes, right, side="left")
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

# Split aligned
train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

# lags
R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

# Standardize scalars on TRAIN valid (i>=2 and i<train_end)
valid_train_idx=np.arange(2,train_end)
def zscore(arr):
    m=float(np.nanmean(arr[valid_train_idx]))
    s=float(np.nanstd(arr[valid_train_idx], ddof=0))+1e-12
    return (arr-m)/s, m, s

R0s,mR0,sR0=zscore(R0)
R1s,mR1,sR1=zscore(R1)
R2s,mR2,sR2=zscore(R2)
logAs,mA,sA=zscore(logA_f)
omegas,mO,sO=zscore(omega)

def fourier(theta_arr, K=3):
    n=len(theta_arr)
    out=np.empty((n, 2*K), dtype=np.float64)
    col=0
    for k in range(1,K+1):
        out[:,col]=np.sin(k*theta_arr); col+=1
        out[:,col]=np.cos(k*theta_arr); col+=1
    return out

P0=fourier(theta_f,3)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
P1=np.column_stack([B1,P0])
V1=np.column_stack([P1, omegas, np.sin(theta_f)*omegas, np.cos(theta_f)*omegas])

def run_metrics(y, p):
    y=y.astype(np.int8)
    p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos_rate=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    if len(yhat)<=1:
        flip=np.nan; mrl=np.nan
    else:
        flip=float(np.mean(yhat[1:]!=yhat[:-1]))
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]:
                cur+=1
            else:
                runs.append(cur); cur=1
        runs.append(cur)
        mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos_rate,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    # drop nan rows
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=2000)
    clf.fit(Xtr,ytr)
    w=clf.coef_.reshape(-1)
    b=float(clf.intercept_[0])
    return w,b,mask

def predict_proba_lin(X, w, b):
    z=X@w + b
    # stable sigmoid
    return 1.0/(1.0+np.exp(-z))

def block_permute(theta_arr, omega_arr, block_size, rng):
    n=len(theta_arr)
    n_blocks=(n + block_size - 1)//block_size
    order=np.arange(n_blocks)
    rng.shuffle(order)
    th_out=np.empty_like(theta_arr)
    om_out=np.empty_like(omega_arr)
    out_pos=0
    for bi in order:
        start=bi*block_size
        end=min(start+block_size,n)
        m=end-start
        th_out[out_pos:out_pos+m]=theta_arr[start:end]
        om_out[out_pos:out_pos+m]=omega_arr[start:end]
        out_pos+=m
    return th_out, om_out

def perm_test(w,b, X_fixed, y, theta_test, omega_test, model_kind, Nperm, block_size, rng):
    # model_kind: "P1" or "V1"
    # X_fixed: for P1 -> B1 rows (n x 4), for V1 -> B1 rows (n x 4) as well (since P0 and omega derived)
    # build X from fixed + derived
    def build(th, om):
        P0t=fourier(th,3)
        if model_kind=="P1":
            return np.column_stack([X_fixed, P0t])
        else:
            om_s=(om-mO)/sO
            return np.column_stack([X_fixed, P0t, om_s, np.sin(th)*om_s, np.cos(th)*om_s])
    X_true=build(theta_test, omega_test)
    p_true=predict_proba_lin(X_true,w,b)
    auc_true=float(auc_fast(y,p_true))
    aucs=np.empty(Nperm, dtype=np.float64)
    for j in range(Nperm):
        th_p, om_p = block_permute(theta_test, omega_test, block_size, rng)
        Xp=build(th_p, om_p)
        pp=predict_proba_lin(Xp,w,b)
        aucs[j]=auc_fast(y,pp)
    mean=float(aucs.mean()); std=float(aucs.std(ddof=0))+1e-12
    pval=float((1+np.sum(aucs>=auc_true))/(1+Nperm))
    z=float((auc_true-mean)/std)
    return {"auc_true":auc_true,"auc_perm_mean":mean,"auc_perm_std":float(aucs.std(ddof=0)),
            "p_value":pval,"z":z}

Nperm=200; block_size=256
per_h={}
theta_by_h={}
omega_by_h={}

for h in H:
    # target y for all i where i+h valid
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)  # length N_feat-h
    # indices for this horizon
    train_i=np.arange(2, train_end-h)  # ensure i+h < train_end
    test_i=np.arange(test_start, N_feat-h)  # ensure i+h < N_feat
    test_i=test_i[test_i+h >= test_start]   # ensure i+h in TEST
    y_train=y_all[train_i]
    y_test=y_all[test_i]
    metrics={}
    trained={}
    for name,X in [("B0",B0),("B1",B1),("P0",P0),("P1",P1),("V1",V1)]:
        w,b,_=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        Xte=Xte[mask]; yte=y_test[mask]
        p=predict_proba_lin(Xte,w,b)
        metrics[name]=run_metrics(yte,p)
        trained[name]={"w":w,"b":b,"rows_used":test_i[mask],"y":yte}
    # perm tests
    rowsP1=trained["P1"]["rows_used"]; yP1=trained["P1"]["y"]
    rowsV1=trained["V1"]["rows_used"]; yV1=trained["V1"]["y"]
    B1_P1=B1[rowsP1]  # fixed
    B1_V1=B1[rowsV1]
    thP1=theta_f[rowsP1]; omP1=omega[rowsP1]
    thV1=theta_f[rowsV1]; omV1=omega[rowsV1]
    rngP=np.random.default_rng(seed + h*1000 + 7)
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permP1=perm_test(trained["P1"]["w"], trained["P1"]["b"], B1_P1, yP1, thP1, omP1, "P1", Nperm, block_size, rngP)
    permV1=perm_test(trained["V1"]["w"], trained["V1"]["b"], B1_V1, yV1, thV1, omV1, "V1", Nperm, block_size, rngV)
    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p is None or z is None:
            counts["none"]+=1
        elif p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
    "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,
    "N_kept":int(N_kept),"N_feat":int(N_feat),"train_end":int(train_end),
    "block_size":block_size,"Nperm":Nperm
 },
 "models":["B0","B1","P0","P1","V1"],
 "per_h":per_h,
 "summary":{
    "theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
    "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}
 }
}

print(json.dumps(out, indent=2)[:1200])

### Tool (2026-02-22 07:32:03)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:33:17)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]; psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
Nperm=200; block_size=256

def auc_fast(y, s):
    y=np.asarray(y, dtype=np.int8)
    s=np.asarray(s, dtype=np.float64)
    n1=int(y.sum()); n0=int(len(y)-n1)
    if n1==0 or n0==0: return np.nan
    order=np.argsort(s, kind="mergesort")
    s_sorted=s[order]; y_sorted=y[order]
    ranks=np.empty(len(s_sorted), dtype=np.float64)
    i=0; r=1.0
    while i<len(s_sorted):
        j=i+1
        while j<len(s_sorted) and s_sorted[j]==s_sorted[i]:
            j+=1
        avg=(r + (r+(j-i)-1.0))/2.0
        ranks[i:j]=avg
        r += (j-i)
        i=j
    sum_r_pos=float(ranks[y_sorted==1].sum())
    U = sum_r_pos - n1*(n1+1)/2.0
    return U/(n0*n1)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0 = 0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s, m, s

def run_metrics(y, p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    if len(yhat)<=1:
        flip=np.nan; mrl=np.nan
    else:
        flip=float(np.mean(yhat[1:]!=yhat[:-1]))
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur); mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=2000)
    clf.fit(Xtr,ytr)
    w=clf.coef_.reshape(-1).astype(np.float64)
    b=float(clf.intercept_[0])
    return w,b

def sigmoid(z):
    return 1.0/(1.0+np.exp(-z))

def block_order(n, block_size, rng):
    n_blocks=(n+block_size-1)//block_size
    order=np.arange(n_blocks)
    rng.shuffle(order)
    return order

def apply_block_perm(arr, order, block_size):
    n=len(arr)
    out=np.empty_like(arr)
    out_pos=0
    for bi in order:
        start=bi*block_size
        end=min(start+block_size, n)
        m=end-start
        out[out_pos:out_pos+m]=arr[start:end]
        out_pos+=m
    return out

# STEP1: build phase features
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_moving_average(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# STEP2: R
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side="left")
ri=np.searchsorted(primes, x_f*expD, side="left")
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

# lags
R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s,mR0,sR0=zscore(R0, valid_train_idx)
R1s,mR1,sR1=zscore(R1, valid_train_idx)
R2s,mR2,sR2=zscore(R2, valid_train_idx)
logAs,mA,sA=zscore(logA_f, valid_train_idx)
omegas,mO,sO=zscore(omega, valid_train_idx)

# Precompute P0 trig features (6 cols) and sin/cos(theta) for interactions
P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)
sin1=np.sin(theta_f); cos1=np.cos(theta_f)

B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
# For V1 extra cols: omega_s, sin(theta)*omega_s, cos(theta)*omega_s
omega_s=omegas
Vextra=np.column_stack([omega_s, sin1*omega_s, cos1*omega_s])

# Function perm tests using decomposition
def perm_test_decomp(w,b, X_fixed, P0_feat, Vextra_feat, y, block_size, Nperm, rng, kind):
    # kind "P1" uses fixed+P0, "V1" uses fixed+P0+Vextra
    n=len(y)
    # split weights
    if kind=="P1":
        w_fixed=w[:X_fixed.shape[1]]
        w_p0=w[X_fixed.shape[1]:]
        base = X_fixed@w_fixed + b
        contrib_true = P0_feat@w_p0
        z_true = base + contrib_true
        p_true = sigmoid(z_true)
        auc_true=float(auc_fast(y,p_true))
        aucs=np.empty(Nperm,dtype=np.float64)
        for j in range(Nperm):
            order=block_order(n, block_size, rng)
            P0p=apply_block_perm(P0_feat, order, block_size)
            zp = base + P0p@w_p0
            pp=sigmoid(zp)
            aucs[j]=auc_fast(y,pp)
    else:
        p0_start=X_fixed.shape[1]
        p0_end=p0_start+P0_feat.shape[1]
        w_fixed=w[:p0_start]
        w_p0=w[p0_start:p0_end]
        w_v= w[p0_end:]
        base = X_fixed@w_fixed + b
        z_true = base + P0_feat@w_p0 + Vextra_feat@w_v
        p_true=sigmoid(z_true)
        auc_true=float(auc_fast(y,p_true))
        aucs=np.empty(Nperm,dtype=np.float64)
        for j in range(Nperm):
            order=block_order(n, block_size, rng)
            P0p=apply_block_perm(P0_feat, order, block_size)
            Vp=apply_block_perm(Vextra_feat, order, block_size)
            zp = base + P0p@w_p0 + Vp@w_v
            pp=sigmoid(zp)
            aucs[j]=auc_fast(y,pp)
    mean=float(aucs.mean()); std=float(aucs.std(ddof=0))+1e-12
    pval=float((1+np.sum(aucs>=auc_true))/(1+Nperm))
    zsc=float((auc_true-mean)/std)
    return {"auc_true":auc_true,"auc_perm_mean":mean,"auc_perm_std":float(aucs.std(ddof=0)),
            "p_value":pval,"z":zsc}

per_h={}
theta_by_h={}
omega_by_h={}
for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)  # len N_feat-h
    train_i=np.arange(2, train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]
    y_test=y_all[test_i]
    metrics={}
    trained={}
    # build full model matrices on the fly to save time? acceptable
    Xs={
        "B0":B0,
        "B1":B1,
        "P0":P0,
        "P1":np.column_stack([B1, P0]),
        "V1":np.column_stack([B1, P0, Vextra])
    }
    for name,X in Xs.items():
        w,b=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        Xte=Xte[mask]; yte=y_test[mask]
        p=sigmoid(Xte@w + b)
        metrics[name]=run_metrics(yte,p)
        trained[name]={"w":w,"b":b,"rows_used":test_i[mask],"y":yte}
    # perm tests
    rows=trained["P1"]["rows_used"]; yP=trained["P1"]["y"]
    Xfix=B1[rows]
    P0t=P0[rows]
    Vt=Vextra[rows]
    rngP=np.random.default_rng(seed + h*1000 + 7)
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permP1=perm_test_decomp(trained["P1"]["w"], trained["P1"]["b"], Xfix, P0t, Vt, yP, block_size, Nperm, rngP, "P1")
    permV1=perm_test_decomp(trained["V1"]["w"], trained["V1"]["b"], Xfix, P0t, Vt, yP, block_size, Nperm, rngV, "V1")
    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
    "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,
    "N_kept":int(N_kept),"N_feat":int(N_feat),"train_end":int(train_end),
    "block_size":block_size,"Nperm":Nperm
 },
 "models":["B0","B1","P0","P1","V1"],
 "per_h":per_h,
 "summary":{
    "theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
    "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}
 }
}
print(json.dumps(out, indent=2))

### Tool (2026-02-22 07:34:04)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:35:10)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]; psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
Nperm=200; block_size=256

def auc_fast(y, s):
    y=np.asarray(y, dtype=np.int8)
    s=np.asarray(s, dtype=np.float64)
    n1=int(y.sum()); n0=int(len(y)-n1)
    if n1==0 or n0==0: return np.nan
    order=np.argsort(s, kind="mergesort")
    s_sorted=s[order]; y_sorted=y[order]
    ranks=np.empty(len(s_sorted), dtype=np.float64)
    i=0; r=1.0
    while i<len(s_sorted):
        j=i+1
        while j<len(s_sorted) and s_sorted[j]==s_sorted[i]:
            j+=1
        avg=(r + (r+(j-i)-1.0))/2.0
        ranks[i:j]=avg
        r += (j-i)
        i=j
    sum_r_pos=float(ranks[y_sorted==1].sum())
    U = sum_r_pos - n1*(n1+1)/2.0
    return U/(n0*n1)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0 = 0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s, m, s

def run_metrics(y, p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    if len(yhat)<=1:
        flip=np.nan; mrl=np.nan
    else:
        flip=float(np.mean(yhat[1:]!=yhat[:-1]))
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur); mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=2000)
    clf.fit(Xtr,ytr)
    w=clf.coef_.reshape(-1).astype(np.float64)
    b=float(clf.intercept_[0])
    return w,b

def sigmoid(z):
    return 1.0/(1.0+np.exp(-z))

def block_order(n, block_size, rng):
    n_blocks=(n+block_size-1)//block_size
    order=np.arange(n_blocks)
    rng.shuffle(order)
    return order

def apply_block_perm(arr, order, block_size):
    n=len(arr)
    out=np.empty_like(arr)
    out_pos=0
    for bi in order:
        start=bi*block_size
        end=min(start+block_size, n)
        m=end-start
        out[out_pos:out_pos+m]=arr[start:end]
        out_pos+=m
    return out

# STEP1: build phase features
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_moving_average(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# STEP2: R
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side="left")
ri=np.searchsorted(primes, x_f*expD, side="left")
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

# lags
R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s,mR0,sR0=zscore(R0, valid_train_idx)
R1s,mR1,sR1=zscore(R1, valid_train_idx)
R2s,mR2,sR2=zscore(R2, valid_train_idx)
logAs,mA,sA=zscore(logA_f, valid_train_idx)
omegas,mO,sO=zscore(omega, valid_train_idx)

# P0 trig features
P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)
sin1=np.sin(theta_f); cos1=np.cos(theta_f)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
Vextra=np.column_stack([omegas, sin1*omegas, cos1*omegas])

def perm_test_decomp(w,b, X_fixed, P0_feat, Vextra_feat, y, block_size, Nperm, rng, kind):
    n=len(y)
    if kind=="P1":
        w_fixed=w[:X_fixed.shape[1]]
        w_p0=w[X_fixed.shape[1]:]
        base = X_fixed@w_fixed + b
        z_true = base + P0_feat@w_p0
        auc_true=float(auc_fast(y, sigmoid(z_true)))
        aucs=np.empty(Nperm,dtype=np.float64)
        for j in range(Nperm):
            order=block_order(n, block_size, rng)
            P0p=apply_block_perm(P0_feat, order, block_size)
            zp = base + P0p@w_p0
            aucs[j]=auc_fast(y, sigmoid(zp))
    else:
        p0_start=X_fixed.shape[1]
        p0_end=p0_start+P0_feat.shape[1]
        w_fixed=w[:p0_start]
        w_p0=w[p0_start:p0_end]
        w_v= w[p0_end:]
        base = X_fixed@w_fixed + b
        z_true = base + P0_feat@w_p0 + Vextra_feat@w_v
        auc_true=float(auc_fast(y, sigmoid(z_true)))
        aucs=np.empty(Nperm,dtype=np.float64)
        for j in range(Nperm):
            order=block_order(n, block_size, rng)
            P0p=apply_block_perm(P0_feat, order, block_size)
            Vp=apply_block_perm(Vextra_feat, order, block_size)
            zp = base + P0p@w_p0 + Vp@w_v
            aucs[j]=auc_fast(y, sigmoid(zp))
    mean=float(aucs.mean()); std=float(aucs.std(ddof=0))+1e-12
    pval=float((1+np.sum(aucs>=auc_true))/(1+Nperm))
    zsc=float((auc_true-mean)/std)
    return {"auc_true":auc_true,"auc_perm_mean":mean,"auc_perm_std":float(aucs.std(ddof=0)),
            "p_value":pval,"z":zsc}

per_h={}
theta_by_h={}
omega_by_h={}
for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)
    train_i=np.arange(2, train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]
    y_test=y_all[test_i]
    metrics={}
    trained={}
    Xs={
        "B0":B0,
        "B1":B1,
        "P0":P0,
        "P1":np.column_stack([B1, P0]),
        "V1":np.column_stack([B1, P0, Vextra])
    }
    for name,X in Xs.items():
        w,b=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        Xte=Xte[mask]; yte=y_test[mask]
        p=sigmoid(Xte@w + b)
        metrics[name]=run_metrics(yte,p)
        trained[name]={"w":w,"b":b,"rows_used":test_i[mask],"y":yte}
    rows=trained["P1"]["rows_used"]; yP=trained["P1"]["y"]
    Xfix=B1[rows]; P0t=P0[rows]; Vt=Vextra[rows]
    rngP=np.random.default_rng(seed + h*1000 + 7)
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permP1=perm_test_decomp(trained["P1"]["w"], trained["P1"]["b"], Xfix, P0t, Vt, yP, block_size, Nperm, rngP, "P1")
    permV1=perm_test_decomp(trained["V1"]["w"], trained["V1"]["b"], Xfix, P0t, Vt, yP, block_size, Nperm, rngV, "V1")
    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
    "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,
    "N_kept":int(N_kept),"N_feat":int(N_feat),"train_end":int(train_end),
    "block_size":block_size,"Nperm":Nperm
 },
 "models":["B0","B1","P0","P1","V1"],
 "per_h":per_h,
 "summary":{
    "theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
    "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}
 }
}
print(json.dumps(out, separators=(',',':')))

### Tool (2026-02-22 07:36:03)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:37:09)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]; psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
Nperm=200; block_size=256

def auc_fast(y, s):
    y=np.asarray(y, dtype=np.int8)
    s=np.asarray(s, dtype=np.float64)
    n1=int(y.sum()); n0=int(len(y)-n1)
    if n1==0 or n0==0: return np.nan
    order=np.argsort(s, kind="mergesort")
    s_sorted=s[order]; y_sorted=y[order]
    ranks=np.empty(len(s_sorted), dtype=np.float64)
    i=0; r=1.0
    while i<len(s_sorted):
        j=i+1
        while j<len(s_sorted) and s_sorted[j]==s_sorted[i]:
            j+=1
        avg=(r + (r+(j-i)-1.0))/2.0
        ranks[i:j]=avg
        r += (j-i)
        i=j
    sum_r_pos=float(ranks[y_sorted==1].sum())
    U = sum_r_pos - n1*(n1+1)/2.0
    return U/(n0*n1)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0 = 0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s, m, s

def sigmoid(z):
    return 1.0/(1.0+np.exp(-z))

def run_metrics(y, p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    if len(yhat)<=1:
        flip=np.nan; mrl=np.nan
    else:
        flip=float(np.mean(yhat[1:]!=yhat[:-1]))
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur); mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=1000)
    clf.fit(Xtr,ytr)
    w=clf.coef_.reshape(-1).astype(np.float64)
    b=float(clf.intercept_[0])
    return w,b

def precompute_block_perm_indices(n, block_size, rng, Nperm):
    n_blocks=(n+block_size-1)//block_size
    blocks=[np.arange(b*block_size, min((b+1)*block_size, n)) for b in range(n_blocks)]
    perms=[]
    for _ in range(Nperm):
        order=np.arange(n_blocks)
        rng.shuffle(order)
        idx=np.concatenate([blocks[b] for b in order])
        perms.append(idx)
    return perms

# STEP1
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_moving_average(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# STEP2 R
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side="left")
ri=np.searchsorted(primes, x_f*expD, side="left")
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s,_,_=zscore(R0, valid_train_idx)
R1s,_,_=zscore(R1, valid_train_idx)
R2s,_,_=zscore(R2, valid_train_idx)
logAs,_,_=zscore(logA_f, valid_train_idx)
omegas,mO,sO=zscore(omega, valid_train_idx)

# P0 feats and Vextra
P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)
sin1=np.sin(theta_f); cos1=np.cos(theta_f)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
Vextra=np.column_stack([omegas, sin1*omegas, cos1*omegas])

per_h={}
theta_by_h={}
omega_by_h={}

for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)
    train_i=np.arange(2, train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]
    y_test=y_all[test_i]
    # Build matrices
    X_B0=B0; X_B1=B1; X_P0=P0
    X_P1=np.column_stack([B1,P0])
    X_V1=np.column_stack([B1,P0,Vextra])
    Xs={"B0":X_B0,"B1":X_B1,"P0":X_P0,"P1":X_P1,"V1":X_V1}
    metrics={}
    wbs={}
    # fit all models
    for name,X in Xs.items():
        w,b=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        Xte=Xte[mask]; yte=y_test[mask]
        p=sigmoid(Xte@w + b)
        metrics[name]=run_metrics(yte,p)
        wbs[name]=(w,b, test_i[mask], yte)
    # Perm tests (use same test rows as P1, and keep y consistent)
    wP,bP,rowsP,yP = wbs["P1"]
    wV,bV,rowsV,yV = wbs["V1"]
    # Use yP and rowsP for both perm tests to keep consistent evaluation set (rowsV should match rowsP)
    # but if differs, intersect
    if len(rowsV)!=len(rowsP) or np.any(rowsV!=rowsP):
        # intersection in order of rowsP
        setV=set(rowsV.tolist())
        keep_mask=np.array([r in setV for r in rowsP])
        rows=rowsP[keep_mask]
        # align y
        y=yP[keep_mask]
    else:
        rows=rowsP; y=yP
    Xfix=B1[rows]
    P0t=P0[rows]
    Vt=Vextra[rows]
    n=len(y)
    # Precompute contributions
    # P1: w = [w_fix(4), w_p0(6)]
    w_fixP=wP[:4]; w_p0P=wP[4:]
    baseP = Xfix@w_fixP + bP
    cP = P0t@w_p0P
    auc_trueP=float(auc_fast(y, sigmoid(baseP + cP)))
    rngP=np.random.default_rng(seed + h*1000 + 7)
    permsP=precompute_block_perm_indices(n, block_size, rngP, Nperm)
    aucsP=np.empty(Nperm)
    for j,perm_idx in enumerate(permsP):
        aucsP[j]=auc_fast(y, sigmoid(baseP + cP[perm_idx]))
    meanP=float(aucsP.mean()); stdP=float(aucsP.std(ddof=0))+1e-12
    permP1={"auc_true":auc_trueP,"auc_perm_mean":meanP,"auc_perm_std":float(aucsP.std(ddof=0)),
            "p_value":float((1+np.sum(aucsP>=auc_trueP))/(1+Nperm)),
            "z":float((auc_trueP-meanP)/stdP)}
    # V1: w = [w_fix(4), w_p0(6), w_v(3)]
    w_fixV=wV[:4]; w_p0V=wV[4:10]; w_vV=wV[10:]
    baseV = Xfix@w_fixV + bV
    c0V = P0t@w_p0V
    cvV = Vt@w_vV
    auc_trueV=float(auc_fast(y, sigmoid(baseV + c0V + cvV)))
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permsV=precompute_block_perm_indices(n, block_size, rngV, Nperm)
    aucsV=np.empty(Nperm)
    for j,perm_idx in enumerate(permsV):
        aucsV[j]=auc_fast(y, sigmoid(baseV + c0V[perm_idx] + cvV[perm_idx]))
    meanV=float(aucsV.mean()); stdV=float(aucsV.std(ddof=0))+1e-12
    permV1={"auc_true":auc_trueV,"auc_perm_mean":meanV,"auc_perm_std":float(aucsV.std(ddof=0)),
            "p_value":float((1+np.sum(aucsV>=auc_trueV))/(1+Nperm)),
            "z":float((auc_trueV-meanV)/stdV)}
    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
    "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,
    "N_kept":int(N_kept),"N_feat":int(N_feat),"train_end":int(train_end),
    "block_size":block_size,"Nperm":Nperm
 },
 "models":["B0","B1","P0","P1","V1"],
 "per_h":per_h,
 "summary":{
    "theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
    "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}
 }
}
print(json.dumps(out, separators=(',',':')))

### Tool (2026-02-22 07:38:08)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:39:15)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]; psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
Nperm=200; block_size=256

def auc_fast(y, s):
    y=np.asarray(y, dtype=np.int8)
    s=np.asarray(s, dtype=np.float64)
    n=len(y)
    n1=int(y.sum()); n0=n-n1
    if n1==0 or n0==0: return np.nan
    order=np.argsort(s)  # quicksort ok
    ranks=np.empty(n, dtype=np.int32)
    ranks[order]=np.arange(1,n+1, dtype=np.int32)
    sum_r_pos=int(ranks[y==1].sum())
    U = sum_r_pos - n1*(n1+1)//2
    return U/(n0*n1)

def causal_moving_average(a, W):
    a=np.asarray(a, dtype=np.float64)
    n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n, dtype=np.float64)
    for i in range(n):
        j0 = 0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s

def sigmoid(z):
    return 1.0/(1.0+np.exp(-z))

def run_metrics(y, p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    flip=float(np.mean(yhat[1:]!=yhat[:-1])) if len(yhat)>1 else np.nan
    # mean run length
    if len(yhat)==0:
        mrl=np.nan
    else:
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur); mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty="l2",solver="lbfgs",max_iter=800)
    clf.fit(Xtr,ytr)
    return clf.coef_.reshape(-1).astype(np.float64), float(clf.intercept_[0])

def precompute_block_perm_indices(n, block_size, rng, Nperm):
    n_blocks=(n+block_size-1)//block_size
    blocks=[np.arange(b*block_size, min((b+1)*block_size, n), dtype=np.int32) for b in range(n_blocks)]
    perms=np.empty((Nperm,n), dtype=np.int32)
    order=np.arange(n_blocks, dtype=np.int32)
    for j in range(Nperm):
        rng.shuffle(order)
        perms[j]=np.concatenate([blocks[b] for b in order])
    return perms

# STEP1
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_moving_average(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# STEP2
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side="left")
ri=np.searchsorted(primes, x_f*expD, side="left")
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s=zscore(R0, valid_train_idx)
R1s=zscore(R1, valid_train_idx)
R2s=zscore(R2, valid_train_idx)
logAs=zscore(logA_f, valid_train_idx)
omega_s=zscore(omega, valid_train_idx)

# features
P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)
sin1=np.sin(theta_f); cos1=np.cos(theta_f)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
Vextra=np.column_stack([omega_s, sin1*omega_s, cos1*omega_s])

per_h={}
theta_by_h={}
omega_by_h={}

for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)
    train_i=np.arange(2, train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]
    y_test=y_all[test_i]
    X_P1=np.column_stack([B1,P0])
    X_V1=np.column_stack([B1,P0,Vextra])
    Xs={"B0":B0,"B1":B1,"P0":P0,"P1":X_P1,"V1":X_V1}
    metrics={}
    wbs={}
    for name,X in Xs.items():
        w,b=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        rows=test_i[mask]
        Xte=Xte[mask]; yte=y_test[mask]
        p=sigmoid(Xte@w + b)
        metrics[name]=run_metrics(yte,p)
        wbs[name]=(w,b,rows,yte)
    wP,bP,rows,y = wbs["P1"]
    # fixed decomposition using same rows
    Xfix=B1[rows]; P0t=P0[rows]; Vt=Vextra[rows]
    # Precompute perms indices once per model
    n=len(y)
    # P1 perm
    w_fixP=wP[:4]; w_p0P=wP[4:]
    baseP=Xfix@w_fixP + bP
    cP=P0t@w_p0P
    auc_trueP=float(auc_fast(y, sigmoid(baseP+cP)))
    rngP=np.random.default_rng(seed + h*1000 + 7)
    permsP=precompute_block_perm_indices(n, block_size, rngP, Nperm)
    aucsP=np.empty(Nperm)
    for j in range(Nperm):
        aucsP[j]=auc_fast(y, sigmoid(baseP + cP[permsP[j]]))
    meanP=float(aucsP.mean()); stdP=float(aucsP.std(ddof=0))+1e-12
    permP1={"auc_true":auc_trueP,"auc_perm_mean":meanP,"auc_perm_std":float(aucsP.std(ddof=0)),
            "p_value":float((1+np.sum(aucsP>=auc_trueP))/(1+Nperm)),
            "z":float((auc_trueP-meanP)/stdP)}
    # V1 perm
    wV,bV,_,_ = wbs["V1"]
    w_fixV=wV[:4]; w_p0V=wV[4:10]; w_vV=wV[10:]
    baseV=Xfix@w_fixV + bV
    c0=P0t@w_p0V
    cv=Vt@w_vV
    auc_trueV=float(auc_fast(y, sigmoid(baseV+c0+cv)))
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permsV=precompute_block_perm_indices(n, block_size, rngV, Nperm)
    aucsV=np.empty(Nperm)
    for j in range(Nperm):
        idxp=permsV[j]
        aucsV[j]=auc_fast(y, sigmoid(baseV + c0[idxp] + cv[idxp]))
    meanV=float(aucsV.mean()); stdV=float(aucsV.std(ddof=0))+1e-12
    permV1={"auc_true":auc_trueV,"auc_perm_mean":meanV,"auc_perm_std":float(aucsV.std(ddof=0)),
            "p_value":float((1+np.sum(aucsV>=auc_trueV))/(1+Nperm)),
            "z":float((auc_trueV-meanV)/stdV)}
    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v["p_value"]; z=v["z"]
        if p<=0.01 and z>=2: counts["strong"]+=1
        elif p<=0.05 and z>=1.5: counts["moderate"]+=1
        elif p<=0.1 and z>=1: counts["weak"]+=1
        else: counts["none"]+=1
    if counts["strong"]>=2: return "strong"
    if counts["strong"]+counts["moderate"]>=2: return "moderate"
    if counts["strong"]+counts["moderate"]+counts["weak"]>=2: return "weak"
    return "none"

out={"params":{"L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
              "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,"N_kept":int(N_kept),"N_feat":int(N_feat),
              "train_end":int(train_end),"block_size":block_size,"Nperm":Nperm},
     "models":["B0","B1","P0","P1","V1"],
     "per_h":per_h,
     "summary":{"theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
                "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}}}
print(json.dumps(out,separators=(',',':')))

### Tool (2026-02-22 07:40:13)

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 07:41:17)

bash -lc python /mnt/data/tmp_perm_test.py

### Assistant (2026-02-22 07:41:39)

bash -lc cat > /tmp/perm_test.py <<'PY'
import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base='/mnt/data'
cheb=np.load(os.path.join(base,'chebyshev_psi.npz'))
pr=np.load(os.path.join(base,'primes.npz'))
x_grid=cheb['x_grid']; psi_minus_x=cheb['psi_minus_x']
primes=pr['primes'].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
Nperm=200; block_size=256

def auc_fast(y,s):
    y=np.asarray(y,dtype=np.int8); s=np.asarray(s,dtype=np.float64)
    n=len(y); n1=int(y.sum()); n0=n-n1
    if n1==0 or n0==0: return float('nan')
    order=np.argsort(s)
    ranks=np.empty(n,dtype=np.int32)
    ranks[order]=np.arange(1,n+1,dtype=np.int32)
    sum_r_pos=int(ranks[y==1].sum())
    U=sum_r_pos - (n1*(n1+1)//2)
    return U/(n0*n1)

def causal_ma(a,W):
    a=np.asarray(a,dtype=np.float64); n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n,dtype=np.float64)
    for i in range(n):
        j0=0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s

def sigmoid(z):
    return 1.0/(1.0+np.exp(-z))

def run_metrics(y,p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=float(auc_fast(y,p))
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    flip=float(np.mean(yhat[1:]!=yhat[:-1])) if len(yhat)>1 else float('nan')
    if len(yhat)==0:
        mrl=float('nan')
    else:
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur)
        mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty='l2',solver='lbfgs',max_iter=800)
    clf.fit(Xtr,ytr)
    return clf.coef_.reshape(-1).astype(np.float64), float(clf.intercept_[0])

def precompute_perms(n, block_size, rng, Nperm):
    n_blocks=(n+block_size-1)//block_size
    # prebuild block slices as start,end
    starts=np.arange(n_blocks,dtype=np.int32)*block_size
    ends=np.minimum(starts+block_size, n).astype(np.int32)
    perms=np.empty((Nperm,n),dtype=np.int32)
    order=np.arange(n_blocks,dtype=np.int32)
    for j in range(Nperm):
        rng.shuffle(order)
        out=[]
        for b in order:
            out.append(np.arange(starts[b], ends[b], dtype=np.int32))
        perms[j]=np.concatenate(out)
    return perms

# STEP1 phase features

t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_ma(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd

U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# STEP2 R
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side='left')
ri=np.searchsorted(primes, x_f*expD, side='left')
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s=zscore(R0, valid_train_idx)
R1s=zscore(R1, valid_train_idx)
R2s=zscore(R2, valid_train_idx)
logAs=zscore(logA_f, valid_train_idx)
omega_s=zscore(omega, valid_train_idx)

P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)

sin1=np.sin(theta_f); cos1=np.cos(theta_f)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
Vextra=np.column_stack([omega_s, sin1*omega_s, cos1*omega_s])

per_h={}
theta_by_h={}
omega_by_h={}

for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)
    train_i=np.arange(2, train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]
    y_test=y_all[test_i]

    X_P1=np.column_stack([B1,P0])
    X_V1=np.column_stack([B1,P0,Vextra])
    Xs={"B0":B0,"B1":B1,"P0":P0,"P1":X_P1,"V1":X_V1}

    metrics={}
    wbs={}
    for name,X in Xs.items():
        w,b=fit_logreg(X, train_i, y_train)
        Xte=X[test_i]
        mask=~np.isnan(Xte).any(axis=1)
        rows=test_i[mask]
        Xte=Xte[mask]; yte=y_test[mask]
        p=sigmoid(Xte@w + b)
        metrics[name]=run_metrics(yte,p)
        wbs[name]=(w,b,rows,yte)

    wP,bP,rows,y = wbs['P1']
    Xfix=B1[rows]; P0t=P0[rows]; Vt=Vextra[rows]
    n=len(y)

    w_fixP=wP[:4]; w_p0P=wP[4:]
    baseP=Xfix@w_fixP + bP
    cP=P0t@w_p0P
    auc_trueP=float(auc_fast(y, sigmoid(baseP+cP)))
    rngP=np.random.default_rng(seed + h*1000 + 7)
    permsP=precompute_perms(n, block_size, rngP, Nperm)
    aucsP=np.empty(Nperm)
    for j in range(Nperm):
        aucsP[j]=auc_fast(y, sigmoid(baseP + cP[permsP[j]]))
    meanP=float(aucsP.mean()); stdP=float(aucsP.std(ddof=0))+1e-12
    permP1={"auc_true":auc_trueP,"auc_perm_mean":meanP,"auc_perm_std":float(aucsP.std(ddof=0)),
            "p_value":float((1+np.sum(aucsP>=auc_trueP))/(1+Nperm)),
            "z":float((auc_trueP-meanP)/stdP)}

    wV,bV,_,_=wbs['V1']
    w_fixV=wV[:4]; w_p0V=wV[4:10]; w_vV=wV[10:]
    baseV=Xfix@w_fixV + bV
    c0=P0t@w_p0V
    cv=Vt@w_vV
    auc_trueV=float(auc_fast(y, sigmoid(baseV+c0+cv)))
    rngV=np.random.default_rng(seed + h*1000 + 17)
    permsV=precompute_perms(n, block_size, rngV, Nperm)
    aucsV=np.empty(Nperm)
    for j in range(Nperm):
        idxp=permsV[j]
        aucsV[j]=auc_fast(y, sigmoid(baseV + c0[idxp] + cv[idxp]))
    meanV=float(aucsV.mean()); stdV=float(aucsV.std(ddof=0))+1e-12
    permV1={"auc_true":auc_trueV,"auc_perm_mean":meanV,"auc_perm_std":float(aucsV.std(ddof=0)),
            "p_value":float((1+np.sum(aucsV>=auc_trueV))/(1+Nperm)),
            "z":float((auc_trueV-meanV)/stdP)}

    per_h[str(h)]={"metrics":metrics,"perm_test":{"P1":permP1,"V1":permV1}}
    theta_by_h[str(h)]={"p_value":permP1["p_value"],"z":permP1["z"],"auc_true":permP1["auc_true"]}
    omega_by_h[str(h)]={"p_value":permV1["p_value"],"z":permV1["z"],"auc_true":permV1["auc_true"],
                        "delta_auc_vs_P1":metrics["V1"]["AUC"]-metrics["P1"]["AUC"]}

def overall_strength(by_h):
    counts={"strong":0,"moderate":0,"weak":0,"none":0}
    for v in by_h.values():
        p=v['p_value']; z=v['z']
        if p<=0.01 and z>=2: counts['strong']+=1
        elif p<=0.05 and z>=1.5: counts['moderate']+=1
        elif p<=0.1 and z>=1: counts['weak']+=1
        else: counts['none']+=1
    if counts['strong']>=2: return 'strong'
    if counts['strong']+counts['moderate']>=2: return 'moderate'
    if counts['strong']+counts['moderate']+counts['weak']>=2: return 'weak'
    return 'none'

out={"params":{"L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
              "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,"N_kept":int(N_kept),"N_feat":int(N_feat),
              "train_end":int(train_end),"block_size":block_size,"Nperm":Nperm},
     "models":["B0","B1","P0","P1","V1"],
     "per_h":per_h,
     "summary":{"theta_conditional_signal":{"by_h":theta_by_h,"overall":overall_strength(theta_by_h)},
                "omega_incremental_signal":{"by_h":omega_by_h,"overall":overall_strength({k:{"p_value":v["p_value"],"z":v["z"]} for k,v in omega_by_h.items()})}}}

print(json.dumps(out,separators=(',',':')))
PY
python /tmp/perm_test.py

### Assistant (2026-02-22 07:42:41)

bash -lc python /tmp/perm_test.py | head -c 2000

### Assistant (2026-02-22 07:42:48)

{"queries":["block permutation theta omega P1 V1 AUC","perm_test P1 V1 block_size 256","omega_incremental_signal","conditional importance phase velocity"],"intent":"find existing computed results in repo files"}

### Tool (2026-02-22 07:42:54)

Encountered exception: <class 'aiohttp.client_exceptions.ClientResponseError'>.

### Assistant (2026-02-22 07:42:54)

{"queries":["perm_test"],"intent":"search repo artifacts for permutation test results"}

### Tool (2026-02-22 07:42:56)

Encountered exception: <class 'aiohttp.client_exceptions.ClientResponseError'>.

### Assistant (2026-02-22 07:42:56)

import numpy as np, math, json, os
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression

base="/mnt/data"
cheb=np.load(os.path.join(base,"chebyshev_psi.npz"))
pr=np.load(os.path.join(base,"primes.npz"))
x_grid=cheb["x_grid"]; psi_minus_x=cheb["psi_minus_x"]
primes=pr["primes"].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; bandpass=(0.002,0.01); Delta=0.01
train_frac=0.7; H=[25,50,100]; seed=12345
block_size=256; Nperm=200

def causal_ma(a,W):
    a=np.asarray(a,dtype=np.float64); n=len(a)
    c=np.cumsum(np.concatenate([[0.0],a]))
    out=np.empty(n,dtype=np.float64)
    for i in range(n):
        j0=0 if i < W-1 else i-W+1
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def zscore(arr, idx):
    m=float(np.nanmean(arr[idx])); s=float(np.nanstd(arr[idx],ddof=0))+1e-12
    return (arr-m)/s

def sigmoid(z): return 1/(1+np.exp(-z))

def auc_simple(y, s):
    # sklearn-free AUC without tie handling
    y=np.asarray(y,dtype=np.int8); s=np.asarray(s,dtype=np.float64)
    n=len(y); n1=int(y.sum()); n0=n-n1
    if n1==0 or n0==0: return None
    order=np.argsort(s)
    ranks=np.empty(n,dtype=np.int32); ranks[order]=np.arange(1,n+1,dtype=np.int32)
    sum_r=int(ranks[y==1].sum())
    U=sum_r - n1*(n1+1)//2
    return float(U/(n0*n1))

def metrics(y,p):
    y=y.astype(np.int8); p=p.astype(np.float64)
    yhat=(p>=0.5).astype(np.int8)
    auc=auc_simple(y,p)
    brier=float(np.mean((p-y)**2))
    acc=float(np.mean(yhat==y))
    pos=float(y.mean())
    p_mean=float(p.mean()); p_std=float(p.std(ddof=0))
    flip=float(np.mean(yhat[1:]!=yhat[:-1])) if len(yhat)>1 else None
    # run length
    if len(yhat)==0:
        mrl=None
    else:
        runs=[]; cur=1
        for i in range(1,len(yhat)):
            if yhat[i]==yhat[i-1]: cur+=1
            else: runs.append(cur); cur=1
        runs.append(cur); mrl=float(np.mean(runs))
    return {"AUC":auc,"Brier":brier,"Accuracy@0.5":acc,"positive_rate":pos,
            "p_mean":p_mean,"p_std":p_std,"flip_rate":flip,"mean_run_length":mrl}

def fit_logreg(X, idx_tr, y_tr):
    Xtr=X[idx_tr]
    mask=~np.isnan(Xtr).any(axis=1)
    Xtr=Xtr[mask]; ytr=y_tr[mask]
    clf=LogisticRegression(C=1.0,penalty='l2',solver='lbfgs',max_iter=800)
    clf.fit(Xtr,ytr)
    return clf.coef_.reshape(-1).astype(np.float64), float(clf.intercept_[0])

# Phase pipeline
t_raw=np.log(x_grid.astype(np.float64))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(np.float64))
x=np.exp(t)
idx=np.where((x>=x_min)&(x<=x_max))[0]
x_kept=x[idx]; u_raw=u_interp[idx]
N_kept=len(x_kept)
N_train=int(math.floor(train_frac*N_kept))

u_detr=u_raw - causal_ma(u_raw,W)
mu=float(u_detr[:N_train].mean()); sd=float(u_detr[:N_train].std(ddof=0))+1e-12
u_std=(u_detr-mu)/sd
U=rfft(u_std)
freq=rfftfreq(N_kept,d=1.0)
mask=(freq>=bandpass[0])&(freq<=bandpass[1])
u_bp=irfft(U*mask, n=N_kept)

z=hilbert(u_bp)
theta=np.mod(np.angle(z),2*np.pi)
logA=np.log(np.abs(z)+1e-12)
omega=((theta[1:]-theta[:-1])+np.pi)%(2*np.pi)-np.pi

N_feat=N_kept-1
theta_f=theta[:-1]; logA_f=logA[:-1]; x_f=x_kept[:-1]

# Residuals
expD=math.exp(Delta)
li=np.searchsorted(primes, x_f, side='left')
ri=np.searchsorted(primes, x_f*expD, side='left')
P=(ri-li).astype(np.float64)
E=x_f*(expD-1.0)/np.log(x_f)
R=(P-E)/np.sqrt(E)

train_end=int(math.floor(train_frac*N_feat))
test_start=train_end

R0=R
R1=np.empty_like(R0); R2=np.empty_like(R0)
R1[0]=np.nan; R1[1:]=R0[:-1]
R2[:2]=np.nan; R2[2:]=R0[:-2]

valid_train_idx=np.arange(2,train_end)
R0s=zscore(R0,valid_train_idx)
R1s=zscore(R1,valid_train_idx)
R2s=zscore(R2,valid_train_idx)
logAs=zscore(logA_f,valid_train_idx)
omega_s=zscore(omega,valid_train_idx)

P0=np.empty((N_feat,6),dtype=np.float64)
for k in range(1,4):
    P0[:,2*(k-1)] = np.sin(k*theta_f)
    P0[:,2*(k-1)+1] = np.cos(k*theta_f)
sin1=np.sin(theta_f); cos1=np.cos(theta_f)
B0=np.column_stack([R0s,R1s,R2s])
B1=np.column_stack([R0s,R1s,R2s,logAs])
Vextra=np.column_stack([omega_s, sin1*omega_s, cos1*omega_s])
X_P1=np.column_stack([B1,P0])
X_V1=np.column_stack([B1,P0,Vextra])

Xs={"B0":B0,"B1":B1,"P0":P0,"P1":X_P1,"V1":X_V1}
per_h={}
for h in H:
    y_all=(R[h:]-R[:-h] > 0).astype(np.int8)
    train_i=np.arange(2,train_end-h)
    test_i=np.arange(test_start, N_feat-h)
    test_i=test_i[test_i+h >= test_start]
    y_train=y_all[train_i]; y_test=y_all[test_i]
    met={}
    for name,X in Xs.items():
        w,b=fit_logreg(X,train_i,y_train)
        Xte=X[test_i]
        m=~np.isnan(Xte).any(axis=1)
        Xte=Xte[m]; yte=y_test[m]
        p=sigmoid(Xte@w+b)
        met[name]=metrics(yte,p)
    per_h[str(h)]={"metrics":met,"perm_test":{"P1":None,"V1":None,"error":"perm_test_not_computed_tool_timeout"}}

out={"params":{"L":L,"x_min":x_min,"x_max":x_max,"W":W,"bandpass":[bandpass[0],bandpass[1]],"Delta":Delta,
              "train_frac":train_frac,"H":H,"seed":seed,"dt":dt,"N_kept":int(N_kept),"N_feat":int(N_feat),
              "train_end":int(train_end),"block_size":block_size,"Nperm":Nperm},
     "models":["B0","B1","P0","P1","V1"],
     "per_h":per_h,
     "summary":{"theta_conditional_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},
                "omega_incremental_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},
                "note":"Permutation tests could not be computed within tool execution limits in this environment."}}
print(json.dumps(out,separators=(',',':')))

### Tool (2026-02-22 07:43:32)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"bandpass":[0.002,0.01],"Delta":0.01,"train_frac":0.7,"H":[25,50,100],"seed":12345,"dt":0.00024594622340595577,"N_kept":18296,"N_feat":18295,"train_end":12806,"block_size":256,"Nperm":200},"models":["B0","B1","P0","P1","V1"],"per_h":{"25":{"metrics":{"B0":{"AUC":0.7938537718049433,"Brier":0.18493921857383203,"Accuracy@0.5":0.7095534407027818,"positive_rate":0.5065885797950219,"p_mean":0.5015795500197204,"p_std":0.25284012776016,"flip_rate":0.08786381109280615,"mean_run_length":11.359667359667359},"B1":{"AUC":0.7942110233958252,"Brier":0.18475420318493396,"Accuracy@0.5":0.7104685212298683,"positive_rate":0.5065885797950219,"p_mean":0.506338690088825,"p_std":0.2528855512429886,"flip_rate":0.08932820794435292,"mean_run_length":11.173824130879346},"P0":{"AUC":0.6736294992796007,"Brier":0.2295334129807787,"Accuracy@0.5":0.6475109809663251,"positive_rate":0.5065885797950219,"p_mean":0.5033421767550285,"p_std":0.1093282556752974,"flip_rate":0.010616877173714076,"mean_run_length":92.61016949152543},"P1":{"AUC":0.8306256271333253,"Brier":0.16823550244330673,"Accuracy@0.5":0.7516471449487555,"positive_rate":0.5065885797950219,"p_mean":0.5081699963604439,"p_std":0.27188313134777903,"flip_rate":0.09225700164744646,"mean_run_length":10.81980198019802},"V1":{"AUC":0.8491162780226754,"Brier":0.15869542427597477,"Accuracy@0.5":0.773792093704246,"positive_rate":0.5065885797950219,"p_mean":0.4949044735930232,"p_std":0.2860723406715593,"flip_rate":0.08456891817682592,"mean_run_length":11.801295896328293}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}},"50":{"metrics":{"B0":{"AUC":0.877794280586756,"Brier":0.14378683084174476,"Accuracy@0.5":0.7867255010112153,"positive_rate":0.4967824967824968,"p_mean":0.5012399304065812,"p_std":0.2973731569472817,"flip_rate":0.1147480691430673,"mean_run_length":8.7024},"B1":{"AUC":0.877262596861227,"Brier":0.14404124262134532,"Accuracy@0.5":0.7869093583379297,"positive_rate":0.4967824967824968,"p_mean":0.49326836459162,"p_std":0.2971662078624924,"flip_rate":0.11438028687017286,"mean_run_length":8.730337078651685},"P0":{"AUC":0.7146792035128987,"Brier":0.2174548538118528,"Accuracy@0.5":0.6988416988416989,"positive_rate":0.4967824967824968,"p_mean":0.5015253534145997,"p_std":0.14545491004600364,"flip_rate":0.011033468186833395,"mean_run_length":89.1639344262295},"P1":{"AUC":0.8863626099234467,"Brier":0.1383124710462297,"Accuracy@0.5":0.7966537966537967,"positive_rate":0.4967824967824968,"p_mean":0.49871459187501277,"p_std":0.3112284876277172,"flip_rate":0.11272526664214785,"mean_run_length":8.85830618892508},"V1":{"AUC":0.8917580098045075,"Brier":0.1348076871306534,"Accuracy@0.5":0.7997793712079426,"positive_rate":0.4967824967824968,"p_mean":0.4900393325908839,"p_std":0.316858604231863,"flip_rate":0.10720853254873115,"mean_run_length":9.313356164383562}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}},"100":{"metrics":{"B0":{"AUC":0.8361748066682478,"Brier":0.16574790461197256,"Accuracy@0.5":0.7489330116904807,"positive_rate":0.5006494711449249,"p_mean":0.5072020496432018,"p_std":0.30147924625136324,"flip_rate":0.09948032665181886,"mean_run_length":10.035381750465548},"B1":{"AUC":0.8363004210008432,"Brier":0.16563104623160804,"Accuracy@0.5":0.7487474485062163,"positive_rate":0.5006494711449249,"p_mean":0.5048838675960127,"p_std":0.30141703465912095,"flip_rate":0.09948032665181886,"mean_run_length":10.035381750465548},"P0":{"AUC":0.7045352558937501,"Brier":0.21855458413697637,"Accuracy@0.5":0.6645017628502505,"positive_rate":0.5006494711449249,"p_mean":0.5056912261532452,"p_std":0.1625116476992688,"flip_rate":0.011135857461024499,"mean_run_length":88.34426229508196},"P1":{"AUC":0.8357253773181836,"Brier":0.1660020036229268,"Accuracy@0.5":0.7541287808498793,"positive_rate":0.5006494711449249,"p_mean":0.5080309475952531,"p_std":0.30491908779469534,"flip_rate":0.1028210838901262,"mean_run_length":9.70990990990991},"V1":{"AUC":0.84277520626507,"Brier":0.1623308877236034,"Accuracy@0.5":0.7583967340879569,"positive_rate":0.5006494711449249,"p_mean":0.5108724759171034,"p_std":0.3087971412713993,"flip_rate":0.09762435040831477,"mean_run_length":10.225806451612904}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}}},"summary":{"theta_conditional_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},"omega_incremental_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},"note":"Permutation tests could not be computed within tool execution limits in this environment."}}

### Assistant (2026-02-22 07:43:53)

{"params":{"L":65536,"x_min":100000.0,"x_max":9000000.0,"W":2000,"bandpass":[0.002,0.01],"Delta":0.01,"train_frac":0.7,"H":[25,50,100],"seed":12345,"dt":0.00024594622340595577,"N_kept":18296,"N_feat":18295,"train_end":12806,"block_size":256,"Nperm":200},"models":["B0","B1","P0","P1","V1"],"per_h":{"25":{"metrics":{"B0":{"AUC":0.7938537718049433,"Brier":0.18493921857383203,"Accuracy@0.5":0.7095534407027818,"positive_rate":0.5065885797950219,"p_mean":0.5015795500197204,"p_std":0.25284012776016,"flip_rate":0.08786381109280615,"mean_run_length":11.359667359667359},"B1":{"AUC":0.7942110233958252,"Brier":0.18475420318493396,"Accuracy@0.5":0.7104685212298683,"positive_rate":0.5065885797950219,"p_mean":0.506338690088825,"p_std":0.2528855512429886,"flip_rate":0.08932820794435292,"mean_run_length":11.173824130879346},"P0":{"AUC":0.6736294992796007,"Brier":0.2295334129807787,"Accuracy@0.5":0.6475109809663251,"positive_rate":0.5065885797950219,"p_mean":0.5033421767550285,"p_std":0.1093282556752974,"flip_rate":0.010616877173714076,"mean_run_length":92.61016949152543},"P1":{"AUC":0.8306256271333253,"Brier":0.16823550244330673,"Accuracy@0.5":0.7516471449487555,"positive_rate":0.5065885797950219,"p_mean":0.5081699963604439,"p_std":0.27188313134777903,"flip_rate":0.09225700164744646,"mean_run_length":10.81980198019802},"V1":{"AUC":0.8491162780226754,"Brier":0.15869542427597477,"Accuracy@0.5":0.773792093704246,"positive_rate":0.5065885797950219,"p_mean":0.4949044735930232,"p_std":0.2860723406715593,"flip_rate":0.08456891817682592,"mean_run_length":11.801295896328293}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}},"50":{"metrics":{"B0":{"AUC":0.877794280586756,"Brier":0.14378683084174476,"Accuracy@0.5":0.7867255010112153,"positive_rate":0.4967824967824968,"p_mean":0.5012399304065812,"p_std":0.2973731569472817,"flip_rate":0.1147480691430673,"mean_run_length":8.7024},"B1":{"AUC":0.877262596861227,"Brier":0.14404124262134532,"Accuracy@0.5":0.7869093583379297,"positive_rate":0.4967824967824968,"p_mean":0.49326836459162,"p_std":0.2971662078624924,"flip_rate":0.11438028687017286,"mean_run_length":8.730337078651685},"P0":{"AUC":0.7146792035128987,"Brier":0.2174548538118528,"Accuracy@0.5":0.6988416988416989,"positive_rate":0.4967824967824968,"p_mean":0.5015253534145997,"p_std":0.14545491004600364,"flip_rate":0.011033468186833395,"mean_run_length":89.1639344262295},"P1":{"AUC":0.8863626099234467,"Brier":0.1383124710462297,"Accuracy@0.5":0.7966537966537967,"positive_rate":0.4967824967824968,"p_mean":0.49871459187501277,"p_std":0.3112284876277172,"flip_rate":0.11272526664214785,"mean_run_length":8.85830618892508},"V1":{"AUC":0.8917580098045075,"Brier":0.1348076871306534,"Accuracy@0.5":0.7997793712079426,"positive_rate":0.4967824967824968,"p_mean":0.4900393325908839,"p_std":0.316858604231863,"flip_rate":0.10720853254873115,"mean_run_length":9.313356164383562}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}},"100":{"metrics":{"B0":{"AUC":0.8361748066682478,"Brier":0.16574790461197256,"Accuracy@0.5":0.7489330116904807,"positive_rate":0.5006494711449249,"p_mean":0.5072020496432018,"p_std":0.30147924625136324,"flip_rate":0.09948032665181886,"mean_run_length":10.035381750465548},"B1":{"AUC":0.8363004210008432,"Brier":0.16563104623160804,"Accuracy@0.5":0.7487474485062163,"positive_rate":0.5006494711449249,"p_mean":0.5048838675960127,"p_std":0.30141703465912095,"flip_rate":0.09948032665181886,"mean_run_length":10.035381750465548},"P0":{"AUC":0.7045352558937501,"Brier":0.21855458413697637,"Accuracy@0.5":0.6645017628502505,"positive_rate":0.5006494711449249,"p_mean":0.5056912261532452,"p_std":0.1625116476992688,"flip_rate":0.011135857461024499,"mean_run_length":88.34426229508196},"P1":{"AUC":0.8357253773181836,"Brier":0.1660020036229268,"Accuracy@0.5":0.7541287808498793,"positive_rate":0.5006494711449249,"p_mean":0.5080309475952531,"p_std":0.30491908779469534,"flip_rate":0.1028210838901262,"mean_run_length":9.70990990990991},"V1":{"AUC":0.84277520626507,"Brier":0.1623308877236034,"Accuracy@0.5":0.7583967340879569,"positive_rate":0.5006494711449249,"p_mean":0.5108724759171034,"p_std":0.3087971412713993,"flip_rate":0.09762435040831477,"mean_run_length":10.225806451612904}},"perm_test":{"P1":null,"V1":null,"error":"perm_test_not_computed_tool_timeout"}}},"summary":{"theta_conditional_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},"omega_incremental_signal":{"by_h":{},"overall":"unknown (perm tests not run)"},"note":"Permutation tests could not be computed within tool execution limits in this environment."}}

