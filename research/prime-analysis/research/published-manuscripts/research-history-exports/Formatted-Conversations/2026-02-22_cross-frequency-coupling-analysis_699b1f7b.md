# Cross-frequency Coupling Analysis

- **Conversation ID:** `699b1f7b-3cd8-832d-8989-265ea62052a5`
- **Date:** 2026-02-22

---

### User (2026-02-22 10:23:44)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Determine whether cross-frequency coupling (FAST vs SLOW phase alignment) provides predictive power beyond additive FAST+SLOW phase-state, using block permutation that destroys coupling while preserving each band’s autocorrelation.

Files:
- chebyshev_psi.npz
- primes.npz
- Protocol.md

Params:
L=65536
x_min=1e5
x_max=9e6
W=2000 causal detrend
Delta=0.01
train_frac=0.7 time-ordered
seed=12345
FAST=[0.002,0.01]
SLOW=[0.0002,0.002]
H=[25,100]
block_size=512
Nperm=80
fourier_K=3
logistic: L2, C=1.0, solver=lbfgs, max_iter=2000

STEP 1 — Build base u_std once (canonical)
- Interpolate psi_minus_x to uniform log grid length L, dt.
- Restrict x to [x_min,x_max] => u_raw (kept).
- Split TRAIN/TEST by time.
- Causal detrend W, standardize TRAIN-only -> u_std.

STEP 2 — Extract phases
- Bandpass FAST on u_std -> u_fast, Hilbert -> theta_f in [0,2π), logA_f
- Bandpass SLOW on u_std -> u_slow, Hilbert -> theta_s in [0,2π)
- Compute omega_f, omega_s via wrapped diff
- Align to N_feat = N_kept-1 (drop last samples; align R similarly later).

STEP 3 — Build R and targets y_h(i)=1{R_{i+h}-R_i>0} (same as prior)
Use TRAIN-valid and TEST-valid indices per horizon.

STEP 4 — Define features
B1 = [R_i,R_{i-1},R_{i-2},logA_f]
F  = Fourier K=3 on theta_f
WF = [omega_f, sin(theta_f)*omega_f, cos(theta_f)*omega_f]
S  = Fourier K=3 on theta_s
WS = [omega_s, sin(theta_s)*omega_s, cos(theta_s)*omega_s]

Coupling features Cpl:
- dtheta = wrap_to_pi(theta_f - theta_s)
- Cpl1 = [sin(dtheta), cos(dtheta)]
- CplK = for k=2..3: [sin(k*dtheta), cos(k*dtheta)]
- Optional small lock terms: [cos(theta_f - theta_s), cos(2*theta_f - theta_s), cos(3*theta_f - theta_s)]

Models:
M_add  = B1 + F + WF + S + WS
M_cpl  = M_add + Cpl

Fit logistic (C=1.0 L2) on TRAIN-valid, eval on TEST-valid:
AUC, Brier, Accuracy@0.5.

STEP 5 — Permutation tests (TEST only), horizon-wise

(A) Test incremental value of coupling:
- Keep B1 + FAST + SLOW additive features fixed.
- Permute SLOW blocks (theta_s, omega_s) across TEST-valid rows (block_size=512).
  This destroys coupling alignment but preserves SLOW autocorrelation.
- Recompute Cpl from (theta_f fixed, theta_s permuted).
- Evaluate AUC_perm for M_cpl using trained coefficients.
- Compute p and z for AUC_true vs perm.

(B) Sanity: coupling-free model should be less sensitive
- For the same permutations, evaluate M_add AUC_perm (should drop less).

Return JSON:

{
 "params": {...},
 "derived": {"dt":..., "N_feat":..., "train_end_feat":...},
 "per_h": {
   "25": {
     "metrics": {"M_add":{...}, "M_cpl":{...}, "delta_auc":...},
     "perm": {
       "M_cpl": {"auc_true":..., "auc_mean":..., "auc_std":..., "p":..., "z":...},
       "M_add": {"auc_true":..., "auc_mean":..., "auc_std":..., "p":..., "z":...}
     }
   },
   "100": { ... }
 },
 "summary": {
   "coupling_incremental_signal": {"h25":"strong/moderate/weak/none","h100":"..."},
   "interpretation": "..."
 }
}

PRINT JSON ONLY.

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

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

### Tool (2026-02-22 10:23:46)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 10:23:46)

import numpy as np, math, os, json
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from numpy.random import default_rng

# Load data
psi_npz = np.load('/mnt/data/chebyshev_psi.npz')
pr_npz = np.load('/mnt/data/primes.npz')
x_grid = psi_npz['x_grid'].astype(float)
psi_minus_x = psi_npz['psi_minus_x'].astype(float)
primes = pr_npz['primes'].astype(np.int64)

# Params
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
block_size=512
Nperm=80
fourier_K=3

# Step1: uniform log grid interpolate
t_raw = np.log(x_grid)
t_uniform = np.linspace(t_raw.min(), t_raw.max(), L)
dt = float(t_uniform[1]-t_uniform[0])
u_interp = np.interp(t_uniform, t_raw, psi_minus_x)
x_uniform = np.exp(t_uniform)

kept = np.where((x_uniform>=x_min) & (x_uniform<=x_max))[0]
u_raw = u_interp[kept]
x_kept = x_uniform[kept]
N_kept = len(kept)

# split
N_feat = N_kept-1
train_end_feat = int(np.floor(train_frac*N_feat))
train_slice = slice(0, train_end_feat)
test_slice = slice(train_end_feat, N_feat)

# causal moving average (past-only) on u_raw
def causal_moving_average(x, W):
    # y[i] = mean(x[max(0,i-W+1):i+1])
    c = np.cumsum(x, dtype=float)
    y = np.empty_like(x, dtype=float)
    for i in range(len(x)):
        j0 = i-W+1
        if j0 <= 0:
            y[i] = c[i]/(i+1)
        else:
            y[i] = (c[i]-c[j0-1])/W
    return y

ma = causal_moving_average(u_raw, W)
u_detr = u_raw - ma

# standardize using TRAIN-only (on feature-aligned or raw? protocol says on kept u after detrend)
mu = float(u_detr[:train_end_feat].mean())
sd = float(u_detr[:train_end_feat].std(ddof=0))
u_std = (u_detr - mu)/ (sd if sd>0 else 1.0)

# Step2: bandpass + hilbert for FAST and SLOW
def fft_bandpass(x, f_lo, f_hi):
    n=len(x)
    X = rfft(x)
    freqs = rfftfreq(n, d=1.0)  # cycles/sample
    mask = (freqs>=f_lo) & (freqs<=f_hi)
    X_f = X*mask
    y = irfft(X_f, n=n)
    return y

u_fast = fft_bandpass(u_std, FAST[0], FAST[1])
z_fast = hilbert(u_fast)
theta_f = np.angle(z_fast) % (2*np.pi)
A_f = np.abs(z_fast)
logA_f = np.log(A_f + 1e-12)

u_slow = fft_bandpass(u_std, SLOW[0], SLOW[1])
z_slow = hilbert(u_slow)
theta_s = np.angle(z_slow) % (2*np.pi)

# omega via wrapped diff; align drop last
def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

omega_f = wrap_to_pi(np.diff(theta_f))
omega_s = wrap_to_pi(np.diff(theta_s))

theta_f_feat = theta_f[:-1]
theta_s_feat = theta_s[:-1]
logA_f_feat = logA_f[:-1]

# Step3: Build R on kept (length N_kept)
expDelta = float(np.exp(Delta))
# prime counts in [x, x*expDelta)
# use searchsorted on primes
starts = x_kept
ends = x_kept*expDelta
idx_lo = np.searchsorted(primes, starts, side='left')
idx_hi = np.searchsorted(primes, ends, side='left')
P = (idx_hi - idx_lo).astype(float)
E = (x_kept*(expDelta-1.0))/np.log(x_kept)
R = (P - E)/np.sqrt(E)

# Targets for each horizon on feature indices i
# y_h(i)=1{R_{i+h} - R_i > 0}
def make_targets(h):
    # length N_feat, but only valid where i+h < len(R)
    y = np.zeros(N_feat, dtype=int)
    valid = np.arange(N_feat)
    ok = valid + h < len(R)
    valid = valid[ok]
    y[valid] = (R[valid+h] - R[valid] > 0).astype(int)
    return y

# Feature construction helpers
def fourier_feats(theta, K):
    cols=[]
    for k in range(1, K+1):
        cols.append(np.sin(k*theta))
        cols.append(np.cos(k*theta))
    return np.column_stack(cols)

F = fourier_feats(theta_f_feat, fourier_K)
S = fourier_feats(theta_s_feat, fourier_K)
WF = np.column_stack([omega_f, np.sin(theta_f_feat)*omega_f, np.cos(theta_f_feat)*omega_f])
WS = np.column_stack([omega_s, np.sin(theta_s_feat)*omega_s, np.cos(theta_s_feat)*omega_s])

# B1 with lags: use R aligned to feature indices (R_i corresponds to i)
R_i = R[:-1]
# lagged, pad with nan then later filter valid i>=2
R_im1 = np.roll(R_i, 1); R_im1[0]=np.nan
R_im2 = np.roll(R_i, 2); R_im2[:2]=np.nan
B1 = np.column_stack([R_i, R_im1, R_im2, logA_f_feat])

# Coupling
dtheta = wrap_to_pi(theta_f_feat - theta_s_feat)
Cpl_cols=[np.sin(dtheta), np.cos(dtheta)]
for k in [2,3]:
    Cpl_cols += [np.sin(k*dtheta), np.cos(k*dtheta)]
# lock terms
Cpl_cols += [np.cos(theta_f_feat - theta_s_feat),
             np.cos(2*theta_f_feat - theta_s_feat),
             np.cos(3*theta_f_feat - theta_s_feat)]
Cpl = np.column_stack(Cpl_cols)

# Assemble model matrices
M_add = np.column_stack([B1, F, WF, S, WS])
M_cpl = np.column_stack([M_add, Cpl])

# validity mask for lags (i>=2) and finite
base_valid = np.isfinite(M_add).all(axis=1)

# Train models per horizon using train_valid indices
rng = default_rng(seed)

def fit_and_eval(X_add, X_cpl, y, h):
    # indices
    idx = np.arange(N_feat)
    train_valid = idx[(idx < train_end_feat) & (idx+h < train_end_feat) & base_valid & (idx>=2)]
    test_valid  = idx[(idx >= train_end_feat) & (idx+h < len(R)) & base_valid & (idx>=2)]
    y_train = y[train_valid]
    y_test = y[test_valid]
    # ensure both classes exist in test
    def metrics(model, X_test, y_test):
        p = model.predict_proba(X_test)[:,1]
        auc = roc_auc_score(y_test, p) if len(np.unique(y_test))==2 else float('nan')
        brier = brier_score_loss(y_test, p)
        acc = accuracy_score(y_test, (p>=0.5).astype(int))
        return {"AUC": float(auc), "Brier": float(brier), "Acc": float(acc)}
    # fit
    clf_add = LogisticRegression(penalty='l2', C=1.0, solver='lbfgs', max_iter=2000)
    clf_cpl = LogisticRegression(penalty='l2', C=1.0, solver='lbfgs', max_iter=2000)
    clf_add.fit(X_add[train_valid], y_train)
    clf_cpl.fit(X_cpl[train_valid], y_train)
    met_add = metrics(clf_add, X_add[test_valid], y_test)
    met_cpl = metrics(clf_cpl, X_cpl[test_valid], y_test)
    return train_valid, test_valid, clf_add, clf_cpl, met_add, met_cpl

results={}
for h in H:
    y = make_targets(h)
    results[h] = fit_and_eval(M_add, M_cpl, y, h)

# Permutations on TEST only
# Prepare test range arrays for theta_s and omega_s
test_idx_all = np.arange(train_end_feat, N_feat)  # contiguous
theta_s_test_all = theta_s_feat[test_idx_all]
omega_s_test_all = omega_s[test_idx_all]

n_test_all = len(test_idx_all)
n_blocks = n_test_all // block_size
usable = n_blocks*block_size
# indices within test part
blocks = np.arange(n_blocks)
# store perm aucs per horizon per model
perm_aucs = {h: {"M_cpl": [], "M_add": []} for h in H}

for perm in range(Nperm):
    perm_order = rng.permutation(n_blocks)
    # build permuted arrays for the first usable segment; leave tail unchanged at end to keep length same
    theta_perm = theta_s_test_all.copy()
    omega_perm = omega_s_test_all.copy()
    src_theta = theta_s_test_all[:usable].reshape(n_blocks, block_size)
    src_omega = omega_s_test_all[:usable].reshape(n_blocks, block_size)
    theta_perm[:usable] = src_theta[perm_order].reshape(usable)
    omega_perm[:usable] = src_omega[perm_order].reshape(usable)

    # build coupling features for all feature indices in test using permuted theta_s
    theta_s_perm_full = theta_s_feat.copy()
    omega_s_perm_full = omega_s.copy()
    theta_s_perm_full[test_idx_all] = theta_perm
    omega_s_perm_full[test_idx_all] = omega_perm
    
    dtheta_perm = wrap_to_pi(theta_f_feat - theta_s_perm_full)
    Cpl_cols_p=[np.sin(dtheta_perm), np.cos(dtheta_perm),
                np.sin(2*dtheta_perm), np.cos(2*dtheta_perm),
                np.sin(3*dtheta_perm), np.cos(3*dtheta_perm),
                np.cos(theta_f_feat - theta_s_perm_full),
                np.cos(2*theta_f_feat - theta_s_perm_full),
                np.cos(3*theta_f_feat - theta_s_perm_full)]
    Cpl_perm = np.column_stack(Cpl_cols_p)
    # For M_cpl evaluation: additive fixed, coupling replaced
    X_cpl_eval = np.column_stack([M_add, Cpl_perm])
    # For M_add evaluation: permute S and WS (slow additive) as well
    S_perm = fourier_feats(theta_s_perm_full, fourier_K)
    WS_perm = np.column_stack([omega_s_perm_full, np.sin(theta_s_perm_full)*omega_s_perm_full, np.cos(theta_s_perm_full)*omega_s_perm_full])
    X_add_eval = np.column_stack([B1, F, WF, S_perm, WS_perm])
    # note: base_valid for permuted add differs due to finite (should be same)
    base_valid_perm = np.isfinite(X_add_eval).all(axis=1)
    for h in H:
        train_valid, test_valid, clf_add, clf_cpl, met_add, met_cpl = results[h]
        # recompute test_valid with perm base_valid? but indices should same since no NaNs; still use intersection
        idx = np.arange(N_feat)
        test_valid_h = idx[(idx >= train_end_feat) & (idx+h < len(R)) & (idx>=2) & base_valid_perm]
        y = make_targets(h)
        y_test = y[test_valid_h]
        # AUC evaluation with fixed trained coefs
        p_cpl = clf_cpl.predict_proba(X_cpl_eval[test_valid_h])[:,1]
        p_add = clf_add.predict_proba(X_add_eval[test_valid_h])[:,1]
        # handle potential single-class
        if len(np.unique(y_test))==2:
            perm_aucs[h]["M_cpl"].append(float(roc_auc_score(y_test, p_cpl)))
            perm_aucs[h]["M_add"].append(float(roc_auc_score(y_test, p_add)))
        else:
            perm_aucs[h]["M_cpl"].append(float('nan'))
            perm_aucs[h]["M_add"].append(float('nan'))

# Summarize per h
out = {
 "params": {
    "L": L, "x_min": x_min, "x_max": x_max, "W": W, "Delta": Delta,
    "train_frac": train_frac, "seed": seed,
    "FAST": list(FAST), "SLOW": list(SLOW),
    "H": H, "block_size": block_size, "Nperm": Nperm,
    "fourier_K": fourier_K,
    "logistic": {"penalty":"l2","C":1.0,"solver":"lbfgs","max_iter":2000}
 },
 "derived": {"dt": dt, "N_feat": int(N_feat), "train_end_feat": int(train_end_feat)},
 "per_h": {},
 "summary": {}
}

def perm_stats(arr, true_val):
    a = np.array(arr, dtype=float)
    a = a[np.isfinite(a)]
    mean = float(a.mean()) if len(a)>0 else float('nan')
    std = float(a.std(ddof=0)) if len(a)>0 else float('nan')
    # one-sided p: perm >= true (higher auc better)
    p = float((np.sum(a >= true_val) + 1.0) / (len(a) + 1.0)) if len(a)>0 else float('nan')
    z = float((true_val - mean)/std) if (len(a)>0 and std>0) else float('nan')
    return {"auc_true": float(true_val), "auc_mean": mean, "auc_std": std, "p": p, "z": z, "n_eff": int(len(a))}

signals={}
for h in H:
    train_valid, test_valid, clf_add, clf_cpl, met_add, met_cpl = results[h]
    delta_auc = met_cpl["AUC"] - met_add["AUC"] if (np.isfinite(met_cpl["AUC"]) and np.isfinite(met_add["AUC"])) else float('nan')
    ps_cpl = perm_stats(perm_aucs[h]["M_cpl"], met_cpl["AUC"])
    ps_add = perm_stats(perm_aucs[h]["M_add"], met_add["AUC"])
    out["per_h"][str(h)] = {
        "metrics": {"M_add": met_add, "M_cpl": met_cpl, "delta_auc": float(delta_auc)},
        "perm": {"M_cpl": ps_cpl, "M_add": ps_add},
        "counts": {
            "train_valid_n": int(len(train_valid)),
            "test_valid_n": int(len(test_valid)),
            "test_pos_rate": float(make_targets(h)[test_valid].mean()) if len(test_valid)>0 else float('nan')
        }
    }
    # qualitative
    if np.isfinite(ps_cpl["z"]):
        if ps_cpl["p"] < 0.01 and ps_cpl["z"] > 2.33:
            signals[h]="strong"
        elif ps_cpl["p"] < 0.05 and ps_cpl["z"] > 1.64:
            signals[h]="moderate"
        elif ps_cpl["p"] < 0.2 and ps_cpl["z"] > 0.84:
            signals[h]="weak"
        else:
            signals[h]="none"
    else:
        signals[h]="unknown"

out["summary"]["coupling_incremental_signal"] = {f"h{h}": signals[h] for h in H}
out["summary"]["interpretation"] = (
    "Permutation destroys FAST–SLOW alignment (coupling) while preserving SLOW band autocorrelation via block shuffles in TEST. "
    "If M_cpl AUC is significantly above its permutation distribution, coupling features add information beyond additive FAST+SLOW state. "
    "M_add permutation sensitivity reflects how much SLOW additive terms contribute to target association."
)

out

### Tool (2026-02-22 10:26:10)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[1], line 229
    222 dtheta_perm = wrap_to_pi(theta_f_feat - theta_s_perm_full)
    223 Cpl_cols_p=[np.sin(dtheta_perm), np.cos(dtheta_perm),
    224             np.sin(2*dtheta_perm), np.cos(2*dtheta_perm),
    225             np.sin(3*dtheta_perm), np.cos(3*dtheta_perm),
    226             np.cos(theta_f_feat - theta_s_perm_full),
    227             np.cos(2*theta_f_feat - theta_s_perm_full),
    228             np.cos(3*theta_f_feat - theta_s_perm_full)]
--> 229 Cpl_perm = np.column_stack(Cpl_cols_p)
    230 # For M_cpl evaluation: additive fixed, coupling replaced
    231 X_cpl_eval = np.column_stack([M_add, Cpl_perm])

File <__array_function__ internals>:200, in column_stack(*args, **kwargs)

File /opt/pyvenv/lib/python3.11/site-packages/numpy/lib/shape_base.py:656, in column_stack(tup)
    654         arr = array(arr, copy=False, subok=True, ndmin=2).T
    655     arrays.append(arr)
--> 656 return _nx.concatenate(arrays, 1)

File <__array_function__ internals>:200, in concatenate(*args, **kwargs)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 10:27:14)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from numpy.random import default_rng

# Load
psi_npz = np.load('/mnt/data/chebyshev_psi.npz')
pr_npz = np.load('/mnt/data/primes.npz')
x_grid = psi_npz['x_grid'].astype(float)
psi_minus_x = psi_npz['psi_minus_x'].astype(float)
primes = pr_npz['primes'].astype(np.int64)

# Params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3

# Helpers
def causal_moving_average(x, W):
    c = np.cumsum(x, dtype=float)
    y = np.empty_like(x, dtype=float)
    n=len(x)
    for i in range(n):
        j0=i-W+1
        if j0<=0:
            y[i]=c[i]/(i+1)
        else:
            y[i]=(c[i]-c[j0-1])/W
    return y

def fft_bandpass(x, f_lo, f_hi):
    n=len(x)
    X=rfft(x)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask, n=n)

def wrap_to_pi(a):
    return (a+np.pi)%(2*np.pi)-np.pi

def fourier(theta, K):
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*theta)); cols.append(np.cos(k*theta))
    return np.column_stack(cols)

def metrics_from_probs(y_true, p):
    auc = float(roc_auc_score(y_true, p)) if len(np.unique(y_true))==2 else float('nan')
    brier = float(brier_score_loss(y_true, p))
    acc = float(accuracy_score(y_true, (p>=0.5).astype(int)))
    return {"AUC": auc, "Brier": brier, "Acc": acc}

def perm_stats(arr, true_val):
    a=np.array(arr, dtype=float)
    a=a[np.isfinite(a)]
    mean=float(a.mean()) if a.size else float('nan')
    std=float(a.std(ddof=0)) if a.size else float('nan')
    p=float((np.sum(a>=true_val)+1.0)/(a.size+1.0)) if a.size else float('nan')
    z=float((true_val-mean)/std) if (a.size and std>0) else float('nan')
    return {"auc_true": float(true_val), "auc_mean": mean, "auc_std": std, "p": p, "z": z, "n_eff": int(a.size)}

# Step1: interpolate to uniform log grid
t_raw=np.log(x_grid)
t_uniform=np.linspace(t_raw.min(), t_raw.max(), L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform, t_raw, psi_minus_x)
x_uniform=np.exp(t_uniform)

kept=np.where((x_uniform>=x_min)&(x_uniform<=x_max))[0]
u_raw=u_interp[kept]
x_kept=x_uniform[kept]
N_kept=len(u_raw); N_feat=N_kept-1
train_end_feat=int(np.floor(train_frac*N_feat))

ma=causal_moving_average(u_raw, W)
u_detr=u_raw-ma
mu=float(u_detr[:train_end_feat].mean())
sd=float(u_detr[:train_end_feat].std(ddof=0))
u_std=(u_detr-mu)/(sd if sd>0 else 1.0)

# Step2: phases
u_fast=fft_bandpass(u_std, FAST[0], FAST[1])
z_fast=hilbert(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=fft_bandpass(u_std, SLOW[0], SLOW[1])
z_slow=hilbert(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(np.diff(theta_f))
omega_s=wrap_to_pi(np.diff(theta_s))

theta_f=theta_f[:-1]; theta_s=theta_s[:-1]; logA_f=logA_f[:-1]

# Step3: R and targets
expDelta=float(np.exp(Delta))
idx_lo=np.searchsorted(primes, x_kept, side='left')
idx_hi=np.searchsorted(primes, x_kept*expDelta, side='left')
P=(idx_hi-idx_lo).astype(float)
E=(x_kept*(expDelta-1.0))/np.log(x_kept)
R=(P-E)/np.sqrt(E)
R_i=R[:-1]

# Base pieces
F = fourier(theta_f, K)
WF = np.column_stack([omega_f, np.sin(theta_f)*omega_f, np.cos(theta_f)*omega_f])
S_true = fourier(theta_s, K)
WS_true = np.column_stack([omega_s, np.sin(theta_s)*omega_s, np.cos(theta_s)*omega_s])

R_im1=np.roll(R_i,1); R_im1[0]=np.nan
R_im2=np.roll(R_i,2); R_im2[:2]=np.nan
B1=np.column_stack([R_i, R_im1, R_im2, logA_f])

# Coupling true
dtheta_true=wrap_to_pi(theta_f-theta_s)
Cpl_true=np.column_stack([
    np.sin(dtheta_true), np.cos(dtheta_true),
    np.sin(2*dtheta_true), np.cos(2*dtheta_true),
    np.sin(3*dtheta_true), np.cos(3*dtheta_true),
    np.cos(theta_f-theta_s),
    np.cos(2*theta_f-theta_s),
    np.cos(3*theta_f-theta_s)
])

# Assemble true feature blocks for training/eval
X_fixed = np.column_stack([B1, F, WF])  # always fixed
X_slow_true = np.column_stack([S_true, WS_true])
X_add_true = np.column_stack([X_fixed, X_slow_true])
X_cpl_true = np.column_stack([X_add_true, Cpl_true])

finite_add = np.isfinite(X_add_true).all(axis=1)
finite_cpl = np.isfinite(X_cpl_true).all(axis=1)

rng=default_rng(seed)

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "logistic":{"penalty":"l2","C":1.0,"solver":"lbfgs","max_iter":2000}
 },
 "derived":{"dt":dt,"N_feat":int(N_feat),"train_end_feat":int(train_end_feat)},
 "per_h":{},
 "summary":{}
}

signals={}
for h in H:
    # targets
    y = (R[np.arange(N_feat)+h] - R[np.arange(N_feat)] > 0).astype(int)  # valid only where i+h < len(R); safe because numpy wraps? avoid

### Tool (2026-02-22 10:27:53)

---------------------------------------------------------------------------
IndexError                                Traceback (most recent call last)
Cell In[2], line 152
    149 signals={}
    150 for h in H:
    151     # targets
--> 152     y = (R[np.arange(N_feat)+h] - R[np.arange(N_feat)] > 0).astype(int)  # valid only where i+h < len(R); safe because numpy wraps? avoid

IndexError: index 18296 is out of bounds for axis 0 with size 18296

### Assistant (2026-02-22 10:27:56)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from numpy.random import default_rng

psi_npz=np.load('/mnt/data/chebyshev_psi.npz')
pr_npz=np.load('/mnt/data/primes.npz')
x_grid=psi_npz['x_grid'].astype(float)
psi_minus_x=psi_npz['psi_minus_x'].astype(float)
primes=pr_npz['primes'].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3

def causal_moving_average(x,W):
    c=np.cumsum(x,dtype=float)
    y=np.empty_like(x,dtype=float)
    n=len(x)
    for i in range(n):
        j0=i-W+1
        if j0<=0: y[i]=c[i]/(i+1)
        else: y[i]=(c[i]-c[j0-1])/W
    return y

def fft_bandpass(x,f_lo,f_hi):
    n=len(x)
    X=rfft(x)
    freqs=rfftfreq(n,d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask,n=n)

def wrap_to_pi(a):
    return (a+np.pi)%(2*np.pi)-np.pi

def fourier(theta,K):
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*theta)); cols.append(np.cos(k*theta))
    return np.column_stack(cols)

def metrics_from_probs(y_true,p):
    auc=float(roc_auc_score(y_true,p)) if len(np.unique(y_true))==2 else float('nan')
    brier=float(brier_score_loss(y_true,p))
    acc=float(accuracy_score(y_true,(p>=0.5).astype(int)))
    return {"AUC":auc,"Brier":brier,"Acc":acc}

def perm_stats(arr,true_val):
    a=np.array(arr,dtype=float); a=a[np.isfinite(a)]
    mean=float(a.mean()) if a.size else float('nan')
    std=float(a.std(ddof=0)) if a.size else float('nan')
    p=float((np.sum(a>=true_val)+1.0)/(a.size+1.0)) if a.size else float('nan')
    z=float((true_val-mean)/std) if (a.size and std>0) else float('nan')
    return {"auc_true":float(true_val),"auc_mean":mean,"auc_std":std,"p":p,"z":z,"n_eff":int(a.size)}

# Step1 interpolate
t_raw=np.log(x_grid)
t_uniform=np.linspace(t_raw.min(),t_raw.max(),L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform,t_raw,psi_minus_x)
x_uniform=np.exp(t_uniform)

kept=np.where((x_uniform>=x_min)&(x_uniform<=x_max))[0]
u_raw=u_interp[kept]; x_kept=x_uniform[kept]
N_kept=len(u_raw); N_feat=N_kept-1
train_end_feat=int(np.floor(train_frac*N_feat))

ma=causal_moving_average(u_raw,W)
u_detr=u_raw-ma
mu=float(u_detr[:train_end_feat].mean())
sd=float(u_detr[:train_end_feat].std(ddof=0))
u_std=(u_detr-mu)/(sd if sd>0 else 1.0)

# Step2 phases
u_fast=fft_bandpass(u_std,FAST[0],FAST[1])
z_fast=hilbert(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=fft_bandpass(u_std,SLOW[0],SLOW[1])
z_slow=hilbert(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(np.diff(theta_f))
omega_s=wrap_to_pi(np.diff(theta_s))
theta_f=theta_f[:-1]; theta_s=theta_s[:-1]; logA_f=logA_f[:-1]

# Step3 R
expDelta=float(np.exp(Delta))
idx_lo=np.searchsorted(primes,x_kept,side='left')
idx_hi=np.searchsorted(primes,x_kept*expDelta,side='left')
P=(idx_hi-idx_lo).astype(float)
E=(x_kept*(expDelta-1.0))/np.log(x_kept)
R=(P-E)/np.sqrt(E)
R_i=R[:-1]
R_im1=np.roll(R_i,1); R_im1[0]=np.nan
R_im2=np.roll(R_i,2); R_im2[:2]=np.nan

# Feature blocks
B1=np.column_stack([R_i,R_im1,R_im2,logA_f])
F=fourier(theta_f,K)
WF=np.column_stack([omega_f,np.sin(theta_f)*omega_f,np.cos(theta_f)*omega_f])
S_true=fourier(theta_s,K)
WS_true=np.column_stack([omega_s,np.sin(theta_s)*omega_s,np.cos(theta_s)*omega_s])
X_fixed=np.column_stack([B1,F,WF])
X_slow_true=np.column_stack([S_true,WS_true])

dtheta_true=wrap_to_pi(theta_f-theta_s)
Cpl_true=np.column_stack([
    np.sin(dtheta_true),np.cos(dtheta_true),
    np.sin(2*dtheta_true),np.cos(2*dtheta_true),
    np.sin(3*dtheta_true),np.cos(3*dtheta_true),
    np.cos(theta_f-theta_s),
    np.cos(2*theta_f-theta_s),
    np.cos(3*theta_f-theta_s)
])

X_add_true=np.column_stack([X_fixed,X_slow_true])
X_cpl_true=np.column_stack([X_add_true,Cpl_true])

finite_add=np.isfinite(X_add_true).all(axis=1)
finite_cpl=np.isfinite(X_cpl_true).all(axis=1)

rng=default_rng(seed)

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "logistic":{"penalty":"l2","C":1.0,"solver":"lbfgs","max_iter":2000}
 },
 "derived":{"dt":dt,"N_feat":int(N_feat),"train_end_feat":int(train_end_feat)},
 "per_h":{},
 "summary":{}
}

signals={}

# Precompute test block permutation indices for SLOW within test segment
test_idx_all=np.arange(train_end_feat,N_feat)
theta_s_test=theta_s[test_idx_all]
omega_s_test=omega_s[test_idx_all]
n_test=len(test_idx_all)
n_blocks=n_test//block_size
usable=n_blocks*block_size
theta_blocks=theta_s_test[:usable].reshape(n_blocks,block_size)
omega_blocks=omega_s_test[:usable].reshape(n_blocks,block_size)

perm_orders=[rng.permutation(n_blocks) for _ in range(Nperm)]

for h in H:
    idx=np.arange(N_feat)
    y=np.zeros(N_feat,dtype=int)
    valid_idx=idx[idx+h < len(R)]
    y[valid_idx]=(R[valid_idx+h]-R[valid_idx] > 0).astype(int)

    train_valid=idx[(idx<train_end_feat)&(idx+h<train_end_feat)&finite_cpl&(idx>=2)]
    test_valid=idx[(idx>=train_end_feat)&(idx+h < len(R))&finite_cpl&(idx>=2)]
    y_train=y[train_valid]; y_test=y[test_valid]

    clf_add=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_cpl=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_add.fit(X_add_true[train_valid], y_train)
    clf_cpl.fit(X_cpl_true[train_valid], y_train)

    p_add_true=clf_add.predict_proba(X_add_true[test_valid])[:,1]
    p_cpl_true=clf_cpl.predict_proba(X_cpl_true[test_valid])[:,1]
    met_add=metrics_from_probs(y_test,p_add_true)
    met_cpl=metrics_from_probs(y_test,p_cpl_true)
    delta_auc = met_cpl["AUC"]-met_add["AUC"] if (np.isfinite(met_add["AUC"]) and np.isfinite(met_cpl["AUC"])) else float('nan')

    # Prepare coefficients split
    coef_add=clf_add.coef_.ravel(); int_add=float(clf_add.intercept_[0])
    coef_cpl=clf_cpl.coef_.ravel(); int_cpl=float(clf_cpl.intercept_[0])

    p_add_perm=[]; p_cpl_perm=[]
    # fixed parts logits on test_valid
    X_fixed_test=X_fixed[test_valid]
    X_slow_test_true=X_slow_true[test_valid]
    Cpl_test_true=Cpl_true[test_valid]

    # indexes within test_idx_all for mapping (for permuting slow)
    # test_valid are absolute feature indices; map to offset within test segment
    test_offsets = test_valid - train_end_feat
    # Create theta_s_perm_full, omega_s_perm_full only on needed indices: test_valid
    for po in perm_orders:
        # build permuted slow arrays for test segment
        theta_perm = theta_s_test.copy()
        omega_perm = omega_s_test.copy()
        theta_perm[:usable] = theta_blocks[po].reshape(usable)
        omega_perm[:usable] = omega_blocks[po].reshape(usable)
        # grab permuted values at test_valid offsets
        th_sv = theta_perm[test_offsets]
        om_sv = omega_perm[test_offsets]

        # M_add perm: replace slow features (S, WS)
        S_sv = np.column_stack([np.sin(th_sv), np.cos(th_sv),
                                np.sin(2*th_sv), np.cos(2*th_sv),
                                np.sin(3*th_sv), np.cos(3*th_sv)])
        WS_sv = np.column_stack([om_sv, np.sin(th_sv)*om_sv, np.cos(th_sv)*om_sv])
        X_add_sv = np.column_stack([X_fixed_test, S_sv, WS_sv])
        # logits
        logit_add = int_add + X_add_sv.dot(coef_add)
        prob_add = 1.0/(1.0+np.exp(-logit_add))
        if len(np.unique(y_test))==2:
            p_add_perm.append(float(roc_auc_score(y_test, prob_add)))
        else:
            p_add_perm.append(float('nan'))

        # M_cpl perm: keep additive TRUE (X_fixed_test + true slow), but coupling recomputed with permuted theta_s
        dth = wrap_to_pi(theta_f[test_valid] - th_sv)
        Cpl_sv = np.column_stack([
            np.sin(dth), np.cos(dth),
            np.sin(2*dth), np.cos(2*dth),
            np.sin(3*dth), np.cos(3*dth),
            np.cos(theta_f[test_valid]-th_sv),
            np.cos(2*theta_f[test_valid]-th_sv),
            np.cos(3*theta_f[test_valid]-th_sv)
        ])
        X_cpl_sv = np.column_stack([X_fixed_test, X_slow_test_true, Cpl_sv])
        logit_cpl = int_cpl + X_cpl_sv.dot(coef_cpl)
        prob_cpl = 1.0/(1.0+np.exp(-logit_cpl))
        if len(np.unique(y_test))==2:
            p_cpl_perm.append(float(roc_auc_score(y_test, prob_cpl)))
        else:
            p_cpl_perm.append(float('nan'))

    out["per_h"][str(h)] = {
        "metrics":{"M_add":met_add,"M_cpl":met_cpl,"delta_auc":float(delta_auc)},
        "perm":{
            "M_cpl":perm_stats(p_cpl_perm, met_cpl["AUC"]),
            "M_add":perm_stats(p_add_perm, met_add["AUC"])
        },
        "counts":{
            "train_valid_n":int(train_valid.size),
            "test_valid_n":int(test_valid.size),
            "test_pos_rate":float(y_test.mean()) if y_test.size else float('nan')
        }
    }

    ps = out["per_h"][str(h)]["perm"]["M_cpl"]
    if np.isfinite(ps["z"]):
        if ps["p"] < 0.01 and ps["z"] > 2.33: signals[h]="strong"
        elif ps["p"] < 0.05 and ps["z"] > 1.64: signals[h]="moderate"
        elif ps["p"] < 0.2 and ps["z"] > 0.84: signals[h]="weak"
        else: signals[h]="none"
    else:
        signals[h]="unknown"

out["summary"]["coupling_incremental_signal"]={f"h{h}":signals[h] for h in H}
out["summary"]["interpretation"]=(
    "M_cpl permutation test isolates cross-frequency coupling: additive FAST+SLOW terms are held fixed while "
    "the SLOW phase is block-permuted in TEST only for coupling construction. Significant drop implies coupling adds info "
    "beyond marginal band states. M_add permutation test permutes SLOW additive terms themselves, measuring how much SLOW "
    "features matter marginally for the target."
)

out

### Tool (2026-02-22 10:28:50)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[3], line 225
    223 prob_cpl = 1.0/(1.0+np.exp(-logit_cpl))
    224 if len(np.unique(y_test))==2:
--> 225     p_cpl_perm.append(float(roc_auc_score(y_test, prob_cpl)))
    226 else:
    227     p_cpl_perm.append(float('nan'))

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/_param_validation.py:213, in validate_params.<locals>.decorator.<locals>.wrapper(*args, **kwargs)
    207 try:
    208     with config_context(
    209         skip_parameter_validation=(
    210             prefer_skip_nested_validation or global_skip_validation
    211         )
    212     ):
--> 213         return func(*args, **kwargs)
    214 except InvalidParameterError as e:
    215     # When the function is just a wrapper around an estimator, we allow
    216     # the function to delegate validation to the estimator, but we replace
    217     # the name of the estimator by the name of the function in the error
    218     # message to avoid confusion.
    219     msg = re.sub(
    220         r"parameter of \w+ must be",
    221         f"parameter of {func.__qualname__} must be",
    222         str(e),
    223     )

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/metrics/_ranking.py:639, in roc_auc_score(y_true, y_score, average, sample_weight, max_fpr, multi_class, labels)
    637 elif y_type == "binary":
    638     labels = np.unique(y_true)
--> 639     y_true = label_binarize(y_true, classes=labels)[:, 0]
    640     return _average_binary_score(
    641         partial(_binary_roc_auc_score, max_fpr=max_fpr),
    642         y_true,
   (...)    645         sample_weight=sample_weight,
    646     )
    647 else:  # multilabel-indicator

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/_param_validation.py:186, in validate_params.<locals>.decorator.<locals>.wrapper(*args, **kwargs)
    184 global_skip_validation = get_config()["skip_parameter_validation"]
    185 if global_skip_validation:
--> 186     return func(*args, **kwargs)
    188 func_sig = signature(func)
    190 # Map *args/**kwargs to the function signature

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/preprocessing/_label.py:576, in label_binarize(y, classes, neg_label, pos_label, sparse_output)
    571     raise ValueError(
    572         "%s target data is not supported with label binarization" % y_type
    573     )
    575 if not sparse_output:
--> 576     Y = Y.toarray()
    577     Y = Y.astype(int, copy=False)
    579     if neg_label != 0:

File /opt/pyvenv/lib/python3.11/site-packages/scipy/sparse/_compressed.py:1181, in _cs_matrix.toarray(self, order, out)
   1179 if out is None and order is None:
   1180     order = self._swap('cf')[0]
-> 1181 out = self._process_toarray_args(order, out)
   1182 if not (out.flags.c_contiguous or out.flags.f_contiguous):
   1183     raise ValueError('Output array must be C or F contiguous')

File /opt/pyvenv/lib/python3.11/site-packages/scipy/sparse/_base.py:1301, in _spbase._process_toarray_args(self, order, out)
   1299     return out
   1300 else:
-> 1301     return np.zeros(self.shape, dtype=self.dtype, order=order)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 10:29:56)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from numpy.random import default_rng

psi=np.load('/mnt/data/chebyshev_psi.npz')
pr=np.load('/mnt/data/primes.npz')
x_grid=psi['x_grid'].astype(float)
psi_minus_x=psi['psi_minus_x'].astype(float)
primes=pr['primes'].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3

def causal_moving_average(x,W):
    c=np.cumsum(x,dtype=float)
    y=np.empty_like(x,dtype=float)
    n=len(x)
    for i in range(n):
        j0=i-W+1
        if j0<=0: y[i]=c[i]/(i+1)
        else: y[i]=(c[i]-c[j0-1])/W
    return y

def fft_bandpass(x,f_lo,f_hi):
    n=len(x)
    X=rfft(x)
    freqs=rfftfreq(n,d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask,n=n)

def wrap_to_pi(a):
    return (a+np.pi)%(2*np.pi)-np.pi

def fourier(theta,K):
    # returns [sinθ,cosθ,sin2θ,cos2θ,...]
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*theta)); cols.append(np.cos(k*theta))
    return np.column_stack(cols)

def auc_fast(y, score):
    # y: 0/1 int
    y = y.astype(np.int8, copy=False)
    n_pos = int(y.sum())
    n = y.size
    n_neg = n - n_pos
    if n_pos==0 or n_neg==0: 
        return float('nan')
    # rank data with average ranks for ties using two-pass stable sort
    order = np.argsort(score, kind='mergesort')
    s_sorted = score[order]
    y_sorted = y[order]
    # ranks 1..n with average for ties
    ranks = np.empty(n, dtype=float)
    i=0
    r=1
    while i<n:
        j=i+1
        while j<n and s_sorted[j]==s_sorted[i]:
            j+=1
        # average rank for i..j-1
        avg = 0.5*(r + (r + (j-i) -1))
        ranks[i:j]=avg
        r += (j-i)
        i=j
    # sum of ranks for positives in sorted order
    sum_ranks_pos = float((ranks * y_sorted).sum())
    auc = (sum_ranks_pos - n_pos*(n_pos+1)/2.0)/(n_pos*n_neg)
    return float(auc)

def probs_from_logit(logit):
    return 1.0/(1.0+np.exp(-logit))

def metrics_from_probs(y_true,p):
    return {
        "AUC": float(roc_auc_score(y_true,p)) if len(np.unique(y_true))==2 else float('nan'),
        "Brier": float(brier_score_loss(y_true,p)),
        "Acc": float(accuracy_score(y_true,(p>=0.5).astype(int)))
    }

def perm_stats(arr,true_val):
    a=np.array(arr,dtype=float); a=a[np.isfinite(a)]
    mean=float(a.mean()) if a.size else float('nan')
    std=float(a.std(ddof=0)) if a.size else float('nan')
    p=float((np.sum(a>=true_val)+1.0)/(a.size+1.0)) if a.size else float('nan')
    z=float((true_val-mean)/std) if (a.size and std>0) else float('nan')
    return {"auc_true":float(true_val),"auc_mean":mean,"auc_std":std,"p":p,"z":z,"n_eff":int(a.size)}

# Step1 interpolate
t_raw=np.log(x_grid)
t_uniform=np.linspace(t_raw.min(),t_raw.max(),L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform,t_raw,psi_minus_x)
x_uniform=np.exp(t_uniform)

kept=np.where((x_uniform>=x_min)&(x_uniform<=x_max))[0]
u_raw=u_interp[kept]; x_kept=x_uniform[kept]
N_kept=u_raw.size; N_feat=N_kept-1
train_end_feat=int(np.floor(train_frac*N_feat))

ma=causal_moving_average(u_raw,W)
u_detr=u_raw-ma
mu=float(u_detr[:train_end_feat].mean())
sd=float(u_detr[:train_end_feat].std(ddof=0))
u_std=(u_detr-mu)/(sd if sd>0 else 1.0)

# Step2 phases
u_fast=fft_bandpass(u_std,FAST[0],FAST[1])
z_fast=hilbert(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=fft_bandpass(u_std,SLOW[0],SLOW[1])
z_slow=hilbert(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(np.diff(theta_f))
omega_s=wrap_to_pi(np.diff(theta_s))
theta_f=theta_f[:-1]; theta_s=theta_s[:-1]; logA_f=logA_f[:-1]

# Step3 R
expDelta=float(np.exp(Delta))
idx_lo=np.searchsorted(primes,x_kept,side='left')
idx_hi=np.searchsorted(primes,x_kept*expDelta,side='left')
P=(idx_hi-idx_lo).astype(float)
E=(x_kept*(expDelta-1.0))/np.log(x_kept)
R=(P-E)/np.sqrt(E)
R_i=R[:-1]
R_im1=np.roll(R_i,1); R_im1[0]=np.nan
R_im2=np.roll(R_i,2); R_im2[:2]=np.nan

# base feature blocks
B1=np.column_stack([R_i,R_im1,R_im2,logA_f])
F=fourier(theta_f,K)
WF=np.column_stack([omega_f,np.sin(theta_f)*omega_f,np.cos(theta_f)*omega_f])
S_true=fourier(theta_s,K)
WS_true=np.column_stack([omega_s,np.sin(theta_s)*omega_s,np.cos(theta_s)*omega_s])
X_fixed=np.column_stack([B1,F,WF])
X_slow_true=np.column_stack([S_true,WS_true])

dtheta_true=wrap_to_pi(theta_f-theta_s)
Cpl_true=np.column_stack([
    np.sin(dtheta_true),np.cos(dtheta_true),
    np.sin(2*dtheta_true),np.cos(2*dtheta_true),
    np.sin(3*dtheta_true),np.cos(3*dtheta_true),
    np.cos(theta_f-theta_s),
    np.cos(2*theta_f-theta_s),
    np.cos(3*theta_f-theta_s)
])

X_add_true=np.column_stack([X_fixed,X_slow_true])
X_cpl_true=np.column_stack([X_add_true,Cpl_true])
finite=np.isfinite(X_cpl_true).all(axis=1)

# permutation prep
rng=default_rng(seed)
test_idx_all=np.arange(train_end_feat,N_feat)
theta_s_test=theta_s[test_idx_all]; omega_s_test=omega_s[test_idx_all]
n_test=test_idx_all.size
n_blocks=n_test//block_size
usable=n_blocks*block_size
theta_blocks=theta_s_test[:usable].reshape(n_blocks,block_size)
omega_blocks=omega_s_test[:usable].reshape(n_blocks,block_size)
perm_orders=[rng.permutation(n_blocks) for _ in range(Nperm)]

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "logistic":{"penalty":"l2","C":1.0,"solver":"lbfgs","max_iter":2000}
 },
 "derived":{"dt":dt,"N_feat":int(N_feat),"train_end_feat":int(train_end_feat)},
 "per_h":{},
 "summary":{}
}

signals={}
for h in H:
    idx=np.arange(N_feat)
    y=np.zeros(N_feat,dtype=np.int8)
    valid=idx[idx+h < R.size]
    y[valid]=(R[valid+h]-R[valid] > 0).astype(np.int8)

    train_valid=idx[(idx<train_end_feat)&(idx+h<train_end_feat)&finite&(idx>=2)]
    test_valid=idx[(idx>=train_end_feat)&(idx+h < R.size)&finite&(idx>=2)]
    y_train=y[train_valid]; y_test=y[test_valid]

    clf_add=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_cpl=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_add.fit(X_add_true[train_valid], y_train)
    clf_cpl.fit(X_cpl_true[train_valid], y_train)

    p_add_true=clf_add.predict_proba(X_add_true[test_valid])[:,1]
    p_cpl_true=clf_cpl.predict_proba(X_cpl_true[test_valid])[:,1]
    met_add=metrics_from_probs(y_test,p_add_true)
    met_cpl=metrics_from_probs(y_test,p_cpl_true)
    delta_auc = met_cpl["AUC"]-met_add["AUC"] if (np.isfinite(met_add["AUC"]) and np.isfinite(met_cpl["AUC"])) else float('nan')

    coef_add=clf_add.coef_.ravel(); int_add=float(clf_add.intercept_[0])
    coef_cpl=clf_cpl.coef_.ravel(); int_cpl=float(clf_cpl.intercept_[0])

    # Precompute fixed pieces for logits on test_valid
    X_fixed_tv=X_fixed[test_valid]
    X_slow_tv_true=X_slow_true[test_valid]
    thf_tv=theta_f[test_valid]

    offsets=(test_valid-train_end_feat).astype(int)

    auc_add_perm=[]
    auc_cpl_perm=[]
    # indices of coef splits
    # X_add = [X_fixed (n_fixed) , S(2K) , WS(3)]
    n_fixed=X_fixed.shape[1]
    n_s=2*K
    n_ws=3
    # X_cpl = X_add + Cpl(9)
    # For speed, compute logit contributions for fixed additive pieces in each model
    base_logit_add = int_add + X_fixed_tv.dot(coef_add[:n_fixed])  # slow varies
    base_logit_cpl = int_cpl + X_fixed_tv.dot(coef_cpl[:n_fixed]) + X_slow_tv_true.dot(coef_cpl[n_fixed:n_fixed+n_s+n_ws])

    coef_add_slow = coef_add[n_fixed:n_fixed+n_s+n_ws]
    coef_cpl_cpl = coef_cpl[n_fixed+n_s+n_ws:]  # length 9

    for po in perm_orders:
        theta_perm=theta_s_test.copy()
        omega_perm=omega_s_test.copy()
        theta_perm[:usable]=theta_blocks[po].reshape(usable)
        omega_perm[:usable]=omega_blocks[po].reshape(usable)
        th_sv=theta_perm[offsets]
        om_sv=omega_perm[offsets]

        # M_add: slow terms permuted
        S_sv=np.column_stack([np.sin(th_sv),np.cos(th_sv),
                              np.sin(2*th_sv),np.cos(2*th_sv),
                              np.sin(3*th_sv),np.cos(3*th_sv)])
        WS_sv=np.column_stack([om_sv,np.sin(th_sv)*om_sv,np.cos(th_sv)*om_sv])
        slow_sv=np.column_stack([S_sv,WS_sv])
        logit_add = base_logit_add + slow_sv.dot(coef_add_slow)
        prob_add=probs_from_logit(logit_add)
        auc_add_perm.append(auc_fast(y_test, prob_add))

        # M_cpl: coupling recomputed, additive slow fixed
        dth=wrap_to_pi(thf_tv - th_sv)
        Cpl_sv=np.column_stack([
            np.sin(dth),np.cos(dth),
            np.sin(2*dth),np.cos(2*dth),
            np.sin(3*dth),np.cos(3*dth),
            np.cos(thf_tv-th_sv),
            np.cos(2*thf_tv-th_sv),
            np.cos(3*thf_tv-th_sv)
        ])
        logit_cpl = base_logit_cpl + Cpl_sv.dot(coef_cpl_cpl)
        prob_cpl=probs_from_logit(logit_cpl)
        auc_cpl_perm.append(auc_fast(y_test, prob_cpl))

    out["per_h"][str(h)] = {
        "metrics":{"M_add":met_add,"M_cpl":met_cpl,"delta_auc":float(delta_auc)},
        "perm":{
            "M_cpl":perm_stats(auc_cpl_perm, met_cpl["AUC"]),
            "M_add":perm_stats(auc_add_perm, met_add["AUC"])
        },
        "counts":{
            "train_valid_n":int(train_valid.size),
            "test_valid_n":int(test_valid.size),
            "test_pos_rate":float(y_test.mean()) if y_test.size else float('nan')
        }
    }

    ps=out["per_h"][str(h)]["perm"]["M_cpl"]
    if np.isfinite(ps["z"]):
        if ps["p"] < 0.01 and ps["z"] > 2.33: signals[h]="strong"
        elif ps["p"] < 0.05 and ps["z"] > 1.64: signals[h]="moderate"
        elif ps["p"] < 0.2 and ps["z"] > 0.84: signals[h]="weak"
        else: signals[h]="none"
    else:
        signals[h]="unknown"

out["summary"]["coupling_incremental_signal"]={f"h{h}":signals[h] for h in H}
out["summary"]["interpretation"]=(
    "Block-permuting SLOW in TEST preserves each band’s autocorrelation but breaks FAST–SLOW phase alignment. "
    "M_cpl permutation isolates coupling value by holding additive FAST+SLOW state fixed and only scrambling coupling terms. "
    "M_add permutation scrambles SLOW additive terms themselves, so it measures marginal SLOW contribution (not coupling)."
)

out

### Tool (2026-02-22 10:30:58)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[4], line 241
    239 WS_sv=np.column_stack([om_sv,np.sin(th_sv)*om_sv,np.cos(th_sv)*om_sv])
    240 slow_sv=np.column_stack([S_sv,WS_sv])
--> 241 logit_add = base_logit_add + slow_sv.dot(coef_add_slow)
    242 prob_add=probs_from_logit(logit_add)
    243 auc_add_perm.append(auc_fast(y_test, prob_add))

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 10:32:02)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from scipy.signal import hilbert
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from numpy.random import default_rng

psi=np.load('/mnt/data/chebyshev_psi.npz')
pr=np.load('/mnt/data/primes.npz')
x_grid=psi['x_grid'].astype(float)
psi_minus_x=psi['psi_minus_x'].astype(float)
primes=pr['primes'].astype(np.int64)

L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3

def causal_moving_average(x,W):
    c=np.cumsum(x,dtype=float)
    y=np.empty_like(x,dtype=float)
    for i in range(len(x)):
        j0=i-W+1
        if j0<=0: y[i]=c[i]/(i+1)
        else: y[i]=(c[i]-c[j0-1])/W
    return y

def fft_bandpass(x,f_lo,f_hi):
    n=len(x)
    X=rfft(x); freqs=rfftfreq(n,d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask,n=n)

def wrap_to_pi(a):
    return (a+np.pi)%(2*np.pi)-np.pi

def auc_fast(y, score):
    y=y.astype(np.int8, copy=False)
    n_pos=int(y.sum()); n=y.size; n_neg=n-n_pos
    if n_pos==0 or n_neg==0: return float('nan')
    order=np.argsort(score, kind='mergesort')
    s=score[order]; yy=y[order]
    ranks=np.empty(n, dtype=float)
    i=0; r=1
    while i<n:
        j=i+1
        while j<n and s[j]==s[i]:
            j+=1
        avg=0.5*(r + (r+(j-i)-1))
        ranks[i:j]=avg
        r += (j-i)
        i=j
    sum_r=float((ranks*yy).sum())
    return float((sum_r - n_pos*(n_pos+1)/2.0)/(n_pos*n_neg))

def probs_from_logit(z):
    return 1.0/(1.0+np.exp(-z))

def metrics_from_probs(y_true,p):
    return {
        "AUC": float(roc_auc_score(y_true,p)) if len(np.unique(y_true))==2 else float('nan'),
        "Brier": float(brier_score_loss(y_true,p)),
        "Acc": float(accuracy_score(y_true,(p>=0.5).astype(int)))
    }

def perm_stats(arr,true_val):
    a=np.array(arr,dtype=float); a=a[np.isfinite(a)]
    mean=float(a.mean()) if a.size else float('nan')
    std=float(a.std(ddof=0)) if a.size else float('nan')
    p=float((np.sum(a>=true_val)+1.0)/(a.size+1.0)) if a.size else float('nan')
    z=float((true_val-mean)/std) if (a.size and std>0) else float('nan')
    return {"auc_true":float(true_val),"auc_mean":mean,"auc_std":std,"p":p,"z":z,"n_eff":int(a.size)}

# Step1: interpolate uniform log grid
t_raw=np.log(x_grid)
t_uniform=np.linspace(t_raw.min(),t_raw.max(),L)
dt=float(t_uniform[1]-t_uniform[0])
u_interp=np.interp(t_uniform,t_raw,psi_minus_x)
x_uniform=np.exp(t_uniform)
kept=np.where((x_uniform>=x_min)&(x_uniform<=x_max))[0]
u_raw=u_interp[kept]; x_kept=x_uniform[kept]
N_kept=u_raw.size; N_feat=N_kept-1
train_end_feat=int(np.floor(train_frac*N_feat))

ma=causal_moving_average(u_raw,W)
u_detr=u_raw-ma
mu=float(u_detr[:train_end_feat].mean())
sd=float(u_detr[:train_end_feat].std(ddof=0))
u_std=(u_detr-mu)/(sd if sd>0 else 1.0)

# Step2: phases
u_fast=fft_bandpass(u_std,FAST[0],FAST[1])
z_fast=hilbert(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=fft_bandpass(u_std,SLOW[0],SLOW[1])
z_slow=hilbert(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(np.diff(theta_f))
omega_s=wrap_to_pi(np.diff(theta_s))
theta_f=theta_f[:-1]; theta_s=theta_s[:-1]; logA_f=logA_f[:-1]

# Step3: R
expDelta=float(np.exp(Delta))
idx_lo=np.searchsorted(primes,x_kept,side='left')
idx_hi=np.searchsorted(primes,x_kept*expDelta,side='left')
P=(idx_hi-idx_lo).astype(float)
E=(x_kept*(expDelta-1.0))/np.log(x_kept)
R=(P-E)/np.sqrt(E)
R_i=R[:-1]
R_im1=np.roll(R_i,1); R_im1[0]=np.nan
R_im2=np.roll(R_i,2); R_im2[:2]=np.nan

# Fixed feature matrix for all i
# B1
B1=np.column_stack([R_i,R_im1,R_im2,logA_f])
# FAST Fourier+WF
sinf1=np.sin(theta_f); cosf1=np.cos(theta_f)
sin2f=np.sin(2*theta_f); cos2f=np.cos(2*theta_f)
sin3f=np.sin(3*theta_f); cos3f=np.cos(3*theta_f)
F=np.column_stack([sinf1,cosf1,sin2f,cos2f,sin3f,cos3f])
WF=np.column_stack([omega_f,sinf1*omega_f,cosf1*omega_f])
X_fixed=np.column_stack([B1,F,WF])  # shape (N_feat, n_fixed)
n_fixed=X_fixed.shape[1]

# True SLOW components for training and baseline logits (M_cpl)
sins1=np.sin(theta_s); coss1=np.cos(theta_s)
sin2s=np.sin(2*theta_s); cos2s=np.cos(2*theta_s)
sin3s=np.sin(3*theta_s); cos3s=np.cos(3*theta_s)
S_true=np.column_stack([sins1,coss1,sin2s,cos2s,sin3s,cos3s])
WS_true=np.column_stack([omega_s,sins1*omega_s,coss1*omega_s])
X_slow_true=np.column_stack([S_true,WS_true])
n_slow=X_slow_true.shape[1]  # 9

# True coupling
dth_true=wrap_to_pi(theta_f-theta_s)
sin_d=np.sin(dth_true); cos_d=np.cos(dth_true)
sin2d=np.sin(2*dth_true); cos2d=np.cos(2*dth_true)
sin3d=np.sin(3*dth_true); cos3d=np.cos(3*dth_true)
Cpl_true=np.column_stack([
    sin_d,cos_d,sin2d,cos2d,sin3d,cos3d,
    np.cos(theta_f-theta_s),
    np.cos(2*theta_f-theta_s),
    np.cos(3*theta_f-theta_s)
])
n_cpl=Cpl_true.shape[1]  # 9

X_add_true=np.column_stack([X_fixed,X_slow_true])
X_cpl_true=np.column_stack([X_add_true,Cpl_true])
finite=np.isfinite(X_cpl_true).all(axis=1)

# Permutation prep
rng=default_rng(seed)
test_idx_all=np.arange(train_end_feat,N_feat)
theta_s_test=theta_s[test_idx_all]; omega_s_test=omega_s[test_idx_all]
n_test=test_idx_all.size
n_blocks=n_test//block_size
usable=n_blocks*block_size
theta_blocks=theta_s_test[:usable].reshape(n_blocks,block_size)
omega_blocks=omega_s_test[:usable].reshape(n_blocks,block_size)
perm_orders=[rng.permutation(n_blocks) for _ in range(Nperm)]

out={
 "params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "logistic":{"penalty":"l2","C":1.0,"solver":"lbfgs","max_iter":2000}
 },
 "derived":{"dt":dt,"N_feat":int(N_feat),"train_end_feat":int(train_end_feat)},
 "per_h":{},
 "summary":{}
}

signals={}

for h in H:
    idx=np.arange(N_feat)
    y=np.zeros(N_feat,dtype=np.int8)
    valid=idx[idx+h < R.size]
    y[valid]=(R[valid+h]-R[valid] > 0).astype(np.int8)

    train_valid=idx[(idx<train_end_feat)&(idx+h<train_end_feat)&finite&(idx>=2)]
    test_valid=idx[(idx>=train_end_feat)&(idx+h < R.size)&finite&(idx>=2)]
    y_train=y[train_valid]; y_test=y[test_valid]

    clf_add=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_cpl=LogisticRegression(penalty='l2',C=1.0,solver='lbfgs',max_iter=2000)
    clf_add.fit(X_add_true[train_valid], y_train)
    clf_cpl.fit(X_cpl_true[train_valid], y_train)

    p_add_true=clf_add.predict_proba(X_add_true[test_valid])[:,1]
    p_cpl_true=clf_cpl.predict_proba(X_cpl_true[test_valid])[:,1]
    met_add=metrics_from_probs(y_test,p_add_true)
    met_cpl=metrics_from_probs(y_test,p_cpl_true)
    delta_auc = met_cpl["AUC"]-met_add["AUC"] if (np.isfinite(met_add["AUC"]) and np.isfinite(met_cpl["AUC"])) else float('nan')

    coef_add=clf_add.coef_.ravel(); int_add=float(clf_add.intercept_[0])
    coef_cpl=clf_cpl.coef_.ravel(); int_cpl=float(clf_cpl.intercept_[0])

    # Split coefficients
    coef_add_fixed = coef_add[:n_fixed]
    coef_add_slow  = coef_add[n_fixed:n_fixed+n_slow]
    coef_cpl_fixed = coef_cpl[:n_fixed]
    coef_cpl_slow  = coef_cpl[n_fixed:n_fixed+n_slow]
    coef_cpl_cpl   = coef_cpl[n_fixed+n_slow:]

    # Fixed logits
    Xf_tv=X_fixed[test_valid]
    Xslow_tv_true=X_slow_true[test_valid]
    thf_tv=theta_f[test_valid]
    base_logit_add = int_add + Xf_tv.dot(coef_add_fixed)
    base_logit_cpl = int_cpl + Xf_tv.dot(coef_cpl_fixed) + Xslow_tv_true.dot(coef_cpl_slow)

    offsets=(test_valid-train_end_feat).astype(int)

    auc_add_perm=[]
    auc_cpl_perm=[]
    for po in perm_orders:
        # permute blocks
        theta_perm=theta_s_test.copy()
        omega_perm=omega_s_test.copy()
        theta_perm[:usable]=theta_blocks[po].reshape(usable)
        omega_perm[:usable]=omega_blocks[po].reshape(usable)
        th=theta_perm[offsets]
        om=omega_perm[offsets]

        # M_add slow contribution (direct)
        s1=np.sin(th); c1=np.cos(th)
        s2=np.sin(2*th); c2=np.cos(2*th)
        s3=np.sin(3*th); c3=np.cos(3*th)
        # slow vector order: [s1,c1,s2,c2,s3,c3, om, s1*om, c1*om]
        slow_dot = (coef_add_slow[0]*s1 + coef_add_slow[1]*c1 +
                    coef_add_slow[2]*s2 + coef_add_slow[3]*c2 +
                    coef_add_slow[4]*s3 + coef_add_slow[5]*c3 +
                    coef_add_slow[6]*om + coef_add_slow[7]*(s1*om) + coef_add_slow[8]*(c1*om))
        prob_add = probs_from_logit(base_logit_add + slow_dot)
        auc_add_perm.append(auc_fast(y_test, prob_add))

        # M_cpl coupling dot (additive held fixed)
        dth=wrap_to_pi(thf_tv - th)
        sd=np.sin(dth); cd=np.cos(dth)
        s2d=np.sin(2*dth); c2d=np.cos(2*dth)
        s3d=np.sin(3*dth); c3d=np.cos(3*dth)
        c_lock1=np.cos(thf_tv - th)
        c_lock2=np.cos(2*thf_tv - th)
        c_lock3=np.cos(3*thf_tv - th)
        cpl_dot = (coef_cpl_cpl[0]*sd + coef_cpl_cpl[1]*cd +
                   coef_cpl_cpl[2]*s2d + coef_cpl_cpl[3]*c2d +
                   coef_cpl_cpl[4]*s3d + coef_cpl_cpl[5]*c3d +
                   coef_cpl_cpl[6]*c_lock1 + coef_cpl_cpl[7]*c_lock2 + coef_cpl_cpl[8]*c_lock3)
        prob_cpl = probs_from_logit(base_logit_cpl + cpl_dot)
        auc_cpl_perm.append(auc_fast(y_test, prob_cpl))

    out["per_h"][str(h)] = {
        "metrics":{"M_add":met_add,"M_cpl":met_cpl,"delta_auc":float(delta_auc)},
        "perm":{
            "M_cpl":perm_stats(auc_cpl_perm, met_cpl["AUC"]),
            "M_add":perm_stats(auc_add_perm, met_add["AUC"])
        },
        "counts":{
            "train_valid_n":int(train_valid.size),
            "test_valid_n":int(test_valid.size),
            "test_pos_rate":float(y_test.mean()) if y_test.size else float('nan')
        }
    }
    ps=out["per_h"][str(h)]["perm"]["M_cpl"]
    if np.isfinite(ps["z"]):
        if ps["p"] < 0.01 and ps["z"] > 2.33: signals[h]="strong"
        elif ps["p"] < 0.05 and ps["z"] > 1.64: signals[h]="moderate"
        elif ps["p"] < 0.2 and ps["z"] > 0.84: signals[h]="weak"
        else: signals[h]="none"
    else:
        signals[h]="unknown"

out["summary"]["coupling_incremental_signal"]={f"h{h}":signals[h] for h in H}
out["summary"]["interpretation"]=(
    "Block-permuting SLOW in TEST preserves SLOW autocorrelation but breaks FAST–SLOW alignment. "
    "M_cpl permutation holds additive FAST+SLOW fixed and scrambles only coupling terms. "
    "M_add permutation scrambles SLOW additive terms themselves."
)

out

### Tool (2026-02-22 10:33:04)

{'params': {'L': 65536,
  'x_min': 100000.0,
  'x_max': 9000000.0,
  'W': 2000,
  'Delta': 0.01,
  'train_frac': 0.7,
  'seed': 12345,
  'FAST': [0.002, 0.01],
  'SLOW': [0.0002, 0.002],
  'H': [25, 100],
  'block_size': 512,
  'Nperm': 80,
  'fourier_K': 3,
  'logistic': {'penalty': 'l2',
   'C': 1.0,
   'solver': 'lbfgs',
   'max_iter': 2000}},
 'derived': {'dt': 0.0002459462234059558,
  'N_feat': 18295,
  'train_end_feat': 12806},
 'per_h': {'25': {'metrics': {'M_add': {'AUC': 0.844967411560591,
     'Brier': 0.1611060963467691,
     'Acc': 0.7569990850869167},
    'M_cpl': {'AUC': 0.8436338949775654,
     'Brier': 0.16167320514612826,
     'Acc': 0.7602927721866423},
    'delta_auc': -0.0013335165830256068},
   'perm': {'M_cpl': {'auc_true': 0.8436338949775654,
     'auc_mean': 0.8427589463490982,
     'auc_std': 0.0020426150440819744,
     'p': 0.345679012345679,
     'z': 0.4283472948082528,
     'n_eff': 80},
    'M_add': {'auc_true': 0.844967411560591,
     'auc_mean': 0.8243578943511943,
     'auc_std': 0.006723411295084162,
     'p': 0.012345679012345678,
     'z': 3.0653363753702423,
     'n_eff': 80}},
   'counts': {'train_valid_n': 12779,
    'test_valid_n': 5465,
    'test_pos_rate': 0.5066788655077767}},
  '100': {'metrics': {'M_add': {'AUC': 0.8445699417432095,
     'Brier': 0.16159644698916636,
     'Acc': 0.7602968460111317},
    'M_cpl': {'AUC': 0.8463272007111097,
     'Brier': 0.16020995378760394,
     'Acc': 0.7684601113172542},
    'delta_auc': 0.0017572589679001327},
   'perm': {'M_cpl': {'auc_true': 0.8463272007111097,
     'auc_mean': 0.8439071988138261,
     'auc_std': 0.0031895079947776134,
     'p': 0.2345679012345679,
     'z': 0.7587383073646328,
     'n_eff': 80},
    'M_add': {'auc_true': 0.8445699417432095,
     'auc_mean': 0.8352545657341249,
     'auc_std': 0.004855548129546441,
     'p': 0.04938271604938271,
     'z': 1.9185014256989277,
     'n_eff': 80}},
   'counts': {'train_valid_n': 12704,
    'test_valid_n': 5390,
    'test_pos_rate': 0.500556586270872}}},
 'summary': {'coupling_incremental_signal': {'h25': 'none', 'h100': 'none'},
  'interpretation': 'Block-permuting SLOW in TEST preserves SLOW autocorrelation but breaks FAST–SLOW alignment. M_cpl permutation holds additive FAST+SLOW fixed and scrambles only coupling terms. M_add permutation scrambles SLOW additive terms themselves.'}}

### Assistant (2026-02-22 10:33:26)

{
  "params": {
    "L": 65536,
    "x_min": 100000.0,
    "x_max": 9000000.0,
    "W": 2000,
    "Delta": 0.01,
    "train_frac": 0.7,
    "seed": 12345,
    "FAST": [
      0.002,
      0.01
    ],
    "SLOW": [
      0.0002,
      0.002
    ],
    "H": [
      25,
      100
    ],
    "block_size": 512,
    "Nperm": 80,
    "fourier_K": 3,
    "logistic": {
      "penalty": "l2",
      "C": 1.0,
      "solver": "lbfgs",
      "max_iter": 2000
    }
  },
  "derived": {
    "dt": 0.0002459462234059558,
    "N_feat": 18295,
    "train_end_feat": 12806
  },
  "per_h": {
    "25": {
      "metrics": {
        "M_add": {
          "AUC": 0.844967411560591,
          "Brier": 0.1611060963467691,
          "Acc": 0.7569990850869167
        },
        "M_cpl": {
          "AUC": 0.8436338949775654,
          "Brier": 0.16167320514612826,
          "Acc": 0.7602927721866423
        },
        "delta_auc": -0.0013335165830256068
      },
      "perm": {
        "M_cpl": {
          "auc_true": 0.8436338949775654,
          "auc_mean": 0.8427589463490982,
          "auc_std": 0.0020426150440819744,
          "p": 0.345679012345679,
          "z": 0.4283472948082528,
          "n_eff": 80
        },
        "M_add": {
          "auc_true": 0.844967411560591,
          "auc_mean": 0.8243578943511943,
          "auc_std": 0.006723411295084162,
          "p": 0.012345679012345678,
          "z": 3.0653363753702423,
          "n_eff": 80
        }
      },
      "counts": {
        "train_valid_n": 12779,
        "test_valid_n": 5465,
        "test_pos_rate": 0.5066788655077767
      }
    },
    "100": {
      "metrics": {
        "M_add": {
          "AUC": 0.8445699417432095,
          "Brier": 0.16159644698916636,
          "Acc": 0.7602968460111317
        },
        "M_cpl": {
          "AUC": 0.8463272007111097,
          "Brier": 0.16020995378760394,
          "Acc": 0.7684601113172542
        },
        "delta_auc": 0.0017572589679001327
      },
      "perm": {
        "M_cpl": {
          "auc_true": 0.8463272007111097,
          "auc_mean": 0.8439071988138261,
          "auc_std": 0.0031895079947776134,
          "p": 0.2345679012345679,
          "z": 0.7587383073646328,
          "n_eff": 80
        },
        "M_add": {
          "auc_true": 0.8445699417432095,
          "auc_mean": 0.8352545657341249,
          "auc_std": 0.004855548129546441,
          "p": 0.04938271604938271,
          "z": 1.9185014256989277,
          "n_eff": 80
        }
      },
      "counts": {
        "train_valid_n": 12704,
        "test_valid_n": 5390,
        "test_pos_rate": 0.500556586270872
      }
    }
  },
  "summary": {
    "coupling_incremental_signal": {
      "h25": "none",
      "h100": "none"
    },
    "interpretation": "Block-permuting SLOW in TEST preserves SLOW autocorrelation but breaks FAST\u2013SLOW alignment. M_cpl permutation holds additive FAST+SLOW fixed and scrambles only coupling terms, so if coupling mattered you\u2019d see auc_true sitting way out in the perm tail (you don\u2019t). Meanwhile M_add *does* move under SLOW permutation, meaning SLOW carries marginal signal for the target, but cross-frequency alignment (as encoded by dtheta harmonics/locks) does not add detectable incremental value here."
  }
}

