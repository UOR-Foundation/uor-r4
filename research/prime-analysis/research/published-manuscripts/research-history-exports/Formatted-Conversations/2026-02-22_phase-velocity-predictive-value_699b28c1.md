# Phase Velocity Predictive Value

- **Conversation ID:** `699b28c1-d52c-8331-9c45-3da64a0b9c69`
- **Date:** 2026-02-22

---

### User (2026-02-22 11:03:18)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Determine whether phase velocities (omega_fast, omega_slow) provide incremental predictive value beyond phase angles (theta_fast, theta_slow) in the additive two-band model, using block-permutation conditional tests.

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

STEP 1 — Build u_std once (canonical)
Interpolate psi_minus_x to uniform log grid L, restrict x range, detrend W, standardize TRAIN-only.

STEP 2 — Extract phases and velocities
- FAST bandpass -> theta_f, logA_f, omega_f
- SLOW bandpass -> theta_s, omega_s
Align to N_feat=N_kept-1.

STEP 3 — Build R and targets y_h(i)=1{R_{i+h}-R_i>0} with train/test valid indices.

STEP 4 — Feature blocks
B1 = [R_i,R_{i-1},R_{i-2},logA_f]
F  = Fourier K=3 on theta_f
S  = Fourier K=3 on theta_s
WF = [omega_f, sin(theta_f)*omega_f, cos(theta_f)*omega_f]
WS = [omega_s, sin(theta_s)*omega_s, cos(theta_s)*omega_s]

Models:
M_phase = B1 + F + S
M_vel   = B1 + F + S + WF + WS

Train on TRAIN-valid, evaluate on TEST-valid:
AUC, Brier, Acc@0.5.

STEP 5 — Conditional permutation tests (TEST only), horizon-wise

(A) Does omega add value given thetas?
- Keep B1 + theta features fixed (use original theta_f, theta_s).
- Permute blocks of (omega_f, omega_s) together across TEST-valid rows (block_size=512).
  This preserves velocity autocorrelation but breaks its alignment with theta and target.
- Recompute WF, WS from permuted omegas (theta fixed).
- Evaluate AUC_perm for M_vel; compute p, z.

(B) Sanity: permuting theta blocks should hurt more
- Permute blocks of (theta_f, theta_s) together across TEST-valid rows, keep B1 fixed.
- Recompute F, S (and also omega-derived terms should be recomputed from permuted thetas if you choose; but simplest: only test M_phase here).
- Evaluate AUC_perm for M_phase; compute p, z.

Return JSON:
{
 "params": {...},
 "per_h": {
   "25": {
     "metrics": {"M_phase":{...}, "M_vel":{...}, "delta_auc":...},
     "perm": {
        "omega_given_theta": {"auc_true":..., "auc_mean":..., "auc_std":..., "p":..., "z":...},
        "theta_joint": {"auc_true":..., "auc_mean":..., "auc_std":..., "p":..., "z":...}
     }
   },
   "100": { ... }
 },
 "summary": {
   "omega_incremental_signal": {"h25":"strong/moderate/weak/none","h100":"..."},
   "theta_signal": {"h25":"...","h100":"..."}
 }
}

PRINT JSON ONLY.

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

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

### Tool (2026-02-22 11:03:20)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 11:03:20)

import numpy as np, math, json, os, textwrap, statistics
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score

# Load data
psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
prot_path="/mnt/data/Protocol.md"
psi=np.load(psi_path)
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]

pr=np.load(primes_path)
primes=pr["primes"]
Nmax=int(pr["N"]) if "N" in pr else int(primes[-1])

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

rng=np.random.default_rng(seed)

# STEP1: interpolate to uniform log grid
t_raw=np.log(x_grid.astype(float))
t_min=float(t_raw.min())
t_max=float(t_raw.max())
t=np.linspace(t_min,t_max,L)
dt=float(t[1]-t[0])
# interpolate psi_minus_x vs t_raw to t
u_interp=np.interp(t, t_raw, psi_minus_x.astype(float))
x=np.exp(t)

# Restrict x range
kept=(x>=x_min)&(x<=x_max)
t_k=t[kept]
x_k=x[kept]
u_raw=u_interp[kept]
N_kept=len(u_raw)

# causal moving average detrend W
def causal_moving_average(a, W):
    a=np.asarray(a, float)
    W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0,i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

ma=causal_moving_average(u_raw, W)
u_det=u_raw - ma

# train/test split indices on original kept series
train_end=int(math.floor(train_frac*N_kept))
# standardize using TRAIN only
mu=float(u_det[:train_end].mean())
sd=float(u_det[:train_end].std(ddof=0))
u_std=(u_det-mu)/(sd if sd>0 else 1.0)

# FFT bandpass (non-causal) helper
def bandpass_fft(sig, f_lo, f_hi):
    sig=np.asarray(sig,float)
    n=len(sig)
    X=rfft(sig)
    freqs=rfftfreq(n, d=1.0)  # cycles/sample
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    Xf=X*mask
    return irfft(Xf, n=n)

# Hilbert transform via FFT (analytic signal)
def analytic_signal(x):
    x=np.asarray(x,float)
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

# STEP2: phases and velocities
u_fast=bandpass_fft(u_std, FAST[0], FAST[1])
z_fast=analytic_signal(u_fast)
theta_f=np.angle(z_fast)  # in [-pi,pi]
# map to [0,2pi)
theta_f=(theta_f%(2*np.pi))
A_f=np.abs(z_fast)
logA_f=np.log(A_f + 1e-12)

u_slow=bandpass_fft(u_std, SLOW[0], SLOW[1])
z_slow=analytic_signal(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

# omega as wrapped phase difference
def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

omega_f=wrap_to_pi(theta_f[1:]-theta_f[:-1])
omega_s=wrap_to_pi(theta_s[1:]-theta_s[:-1])

N_feat=N_kept-1
# align theta/logA to length N_feat
theta_f_feat=theta_f[:-1]
theta_s_feat=theta_s[:-1]
logA_f_feat=logA_f[:-1]

# STEP3: Build R
expDelta=math.exp(Delta)
# count primes in [x, x*expDelta)
# use searchsorted on primes array
def count_primes_interval(primes, a, b):
    # primes sorted
    return int(np.searchsorted(primes, b, side="left") - np.searchsorted(primes, a, side="left"))

P=np.empty(N_kept, dtype=int)
E=np.empty(N_kept, dtype=float)
for i,xi in enumerate(x_k):
    a=float(xi)
    b=float(xi*expDelta)
    P[i]=count_primes_interval(primes, a, b)
    E[i]=a*(expDelta-1.0)/math.log(a)
R=(P-E)/np.sqrt(E)

# STEP4: feature construction helpers
def fourier_feats(theta, K):
    # theta in [0,2pi)
    feats=[]
    for k in range(1,K+1):
        feats.append(np.sin(k*theta))
        feats.append(np.cos(k*theta))
    return np.column_stack(feats)

def build_features(idx, use_vel, theta_f_arr, theta_s_arr, omega_f_arr=None, omega_s_arr=None):
    # idx: array of i indices in feature alignment 0..N_feat-1
    i=idx
    # B1
    Xb=np.column_stack([R[i], R[i-1], R[i-2], logA_f_feat[i]])
    F=fourier_feats(theta_f_arr[i], fourier_K)
    S=fourier_feats(theta_s_arr[i], fourier_K)
    X=[Xb, F, S]
    if use_vel:
        of=omega_f_arr[i]
        os=omega_s_arr[i]
        WF=np.column_stack([of, np.sin(theta_f_arr[i])*of, np.cos(theta_f_arr[i])*of])
        WS=np.column_stack([os, np.sin(theta_s_arr[i])*os, np.cos(theta_s_arr[i])*os])
        X += [WF, WS]
    return np.concatenate(X, axis=1)

# Split feature indices: omega uses i+1, so last train feature index is train_end-2
split_feat_end=train_end-1  # first test feature index
train_feat_mask = np.arange(N_feat) < split_feat_end
test_feat_mask = ~train_feat_mask

# metrics
def eval_model(model, X, y):
    p=model.predict_proba(X)[:,1]
    auc=roc_auc_score(y, p) if len(np.unique(y))==2 else float("nan")
    brier=brier_score_loss(y, p)
    acc=accuracy_score(y, (p>=0.5).astype(int))
    return auc,brier,acc

# block permutation function for an array of length n_rows
def block_permute_indices(n, block_size, rng):
    blocks=[np.arange(b, min(b+block_size,n)) for b in range(0,n,block_size)]
    order=np.arange(len(blocks))
    rng.shuffle(order)
    perm=np.concatenate([blocks[j] for j in order])
    return perm

results={"params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":fourier_K,
    "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end,"split_feat_end":split_feat_end
}, "per_h":{}, "summary":{}}

for h in H:
    # build valid indices
    i_all=np.arange(N_feat)
    # need i>=2 for lag features
    valid_base = i_all>=2
    # target needs i+h < N_kept
    valid_target = i_all + h < N_kept
    # train-valid: i in train features, i+h < train_end (both in train), and also (i+1) in train already by i<split_feat_end
    train_valid = valid_base & valid_target & (i_all < split_feat_end) & (i_all + h < train_end)
    # test-valid: i in test features, and i+h in test (>=train_end), and within bounds
    test_valid = valid_base & valid_target & (i_all >= split_feat_end) & (i_all + h >= train_end)
    tr_idx=i_all[train_valid]
    te_idx=i_all[test_valid]
    # target
    y = (R[i_all+h]-R[i_all] > 0).astype(int)
    y_tr=y[tr_idx]
    y_te=y[te_idx]
    # features
    Xtr_phase=build_features(tr_idx, False, theta_f_feat, theta_s_feat)
    Xte_phase=build_features(te_idx, False, theta_f_feat, theta_s_feat)
    Xtr_vel=build_features(tr_idx, True, theta_f_feat, theta_s_feat, omega_f, omega_s)
    Xte_vel=build_features(te_idx, True, theta_f_feat, theta_s_feat, omega_f, omega_s)

    # fit models
    clf_phase=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_vel=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_phase.fit(Xtr_phase, y_tr)
    clf_vel.fit(Xtr_vel, y_tr)
    auc_p,brier_p,acc_p=eval_model(clf_phase, Xte_phase, y_te)
    auc_v,brier_v,acc_v=eval_model(clf_vel, Xte_vel, y_te)
    delta_auc=auc_v-auc_p

    # permutation A: omega given theta (permute omegas together across test-valid rows)
    n_te=len(te_idx)
    auc_perm=[]
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        # permute omega features among test rows
        of_perm=omega_f[te_idx][perm]
        os_perm=omega_s[te_idx][perm]
        # build Xte with permuted omegas but fixed theta
        # need to rebuild velocity part only; easiest build full:
        Xte_perm=build_features(np.arange(n_te), True,
                                theta_f_arr=np.concatenate([theta_f_feat[te_idx]]), # dummy; we'll index directly
                                theta_s_arr=np.concatenate([theta_s_feat[te_idx]]),
                                omega_f_arr=of_perm, omega_s_arr=os_perm)
        # build_features expects idx into arrays; we gave arrays length n_te; idx 0..n_te-1
        p=clf_vel.predict_proba(Xte_perm)[:,1]
        auc=roc_auc_score(y_te, p) if len(np.unique(y_te))==2 else float("nan")
        auc_perm.append(auc)
    auc_perm=np.array(auc_perm,float)
    auc_mean=float(np.nanmean(auc_perm))
    auc_std=float(np.nanstd(auc_perm, ddof=1))
    pval=float((1+np.sum(auc_perm>=auc_v))/(Nperm+1))
    z=float((auc_v-auc_mean)/(auc_std if auc_std>0 else np.nan))

    permA={"auc_true":float(auc_v),"auc_mean":auc_mean,"auc_std":auc_std,"p":pval,"z":z}

    # permutation B: theta_joint on M_phase (permute theta blocks together across test-valid rows)
    auc_perm2=[]
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        thf_perm=theta_f_feat[te_idx][perm]
        ths_perm=theta_s_feat[te_idx][perm]
        # build phase features with permuted thetas; B1 fixed computed from te_idx in original order
        # We'll build manually: Xb in original order + Fourier from permuted
        Xb=np.column_stack([R[te_idx], R[te_idx-1], R[te_idx-2], logA_f_feat[te_idx]])
        Fp=fourier_feats(thf_perm, fourier_K)
        Sp=fourier_feats(ths_perm, fourier_K)
        Xte_pperm=np.concatenate([Xb,Fp,Sp],axis=1)
        p=clf_phase.predict_proba(Xte_pperm)[:,1]
        auc=roc_auc_score(y_te, p) if len(np.unique(y_te))==2 else float("nan")
        auc_perm2.append(auc)
    auc_perm2=np.array(auc_perm2,float)
    mean2=float(np.nanmean(auc_perm2))
    std2=float(np.nanstd(auc_perm2, ddof=1))
    pval2=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))
    z2=float((auc_p-mean2)/(std2 if std2>0 else np.nan))
    permB={"auc_true":float(auc_p),"auc_mean":mean2,"auc_std":std2,"p":pval2,"z":z2}

    results["per_h"][str(h)]={"metrics":{
        "M_phase":{"AUC":float(auc_p),"Brier":float(brier_p),"Acc@0.5":float(acc_p),
                   "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "M_vel":{"AUC":float(auc_v),"Brier":float(brier_v),"Acc@0.5":float(acc_v),
                 "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "delta_auc":float(delta_auc)
    },"perm":{
        "omega_given_theta":permA,
        "theta_joint":permB
    }}

def qual(p, delta):
    if (not np.isfinite(p)) or (not np.isfinite(delta)):
        return "none"
    if p<=0.01 and delta>0.01:
        return "strong"
    if p<=0.05 and delta>0.005:
        return "moderate"
    if p<=0.1 and delta>0:
        return "weak"
    return "none"

summary_omega={}
summary_theta={}
for h in H:
    ph=results["per_h"][str(h)]
    p_omega=ph["perm"]["omega_given_theta"]["p"]
    delta=ph["metrics"]["delta_auc"]
    summary_omega[f"h{h}"]=qual(p_omega, delta)
    p_theta=ph["perm"]["theta_joint"]["p"]
    # theta signal should be that permuting theta hurts, so auc_true > auc_perm -> low p
    # Use delta as auc_phase - mean_perm
    delta_theta=ph["perm"]["theta_joint"]["auc_true"] - ph["perm"]["theta_joint"]["auc_mean"]
    summary_theta[f"h{h}"]=qual(p_theta, delta_theta)

results["summary"]={"omega_incremental_signal":summary_omega,"theta_signal":summary_theta}

# Ensure JSON serializable
json_str=json.dumps(results, indent=2, sort_keys=True)
len(json_str), json_str[:500]

### Tool (2026-02-22 11:05:32)

---------------------------------------------------------------------------
IndexError                                Traceback (most recent call last)
Cell In[1], line 209
    207 te_idx=i_all[test_valid]
    208 # target
--> 209 y = (R[i_all+h]-R[i_all] > 0).astype(int)
    210 y_tr=y[tr_idx]
    211 y_te=y[te_idx]

IndexError: index 18296 is out of bounds for axis 0 with size 18296

### Assistant (2026-02-22 11:05:45)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score

# Load
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load("/mnt/data/primes.npz")
primes=pr["primes"]
Nmax=int(pr["N"]) if "N" in pr else int(primes[-1])

# Params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; fourier_K=3
rng=np.random.default_rng(seed)

# Helpers
def causal_moving_average(a, W):
    a=np.asarray(a, float); W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0, i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

def bandpass_fft(sig, f_lo, f_hi):
    sig=np.asarray(sig,float); n=len(sig)
    X=rfft(sig)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask, n=n)

def analytic_signal(x):
    x=np.asarray(x,float); n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1; h[1:n//2]=2
    else:
        h[0]=1; h[1:(n+1)//2]=2
    return np.fft.ifft(X*h)

def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

def fourier_feats(theta, K):
    feats=[]
    for k in range(1,K+1):
        feats.append(np.sin(k*theta))
        feats.append(np.cos(k*theta))
    return np.column_stack(feats)

def block_permute_indices(n, block_size, rng):
    blocks=[np.arange(b, min(b+block_size,n)) for b in range(0,n,block_size)]
    order=np.arange(len(blocks))
    rng.shuffle(order)
    return np.concatenate([blocks[j] for j in order])

def eval_model(model, X, y):
    p=model.predict_proba(X)[:,1]
    auc=roc_auc_score(y,p) if len(np.unique(y))==2 else float("nan")
    brier=brier_score_loss(y,p)
    acc=accuracy_score(y,(p>=0.5).astype(int))
    return float(auc), float(brier), float(acc)

# STEP 1: uniform log grid + restrict + detrend + standardize (TRAIN-only)
t_raw=np.log(x_grid.astype(float))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(float))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
t_k=t[kept]; x_k=x[kept]; u_raw=u_interp[kept]
N_kept=len(u_raw)
ma=causal_moving_average(u_raw, W)
u_det=u_raw-ma
train_end=int(math.floor(train_frac*N_kept))
mu=float(u_det[:train_end].mean()); sd=float(u_det[:train_end].std(ddof=0))
u_std=(u_det-mu)/(sd if sd>0 else 1.0)

# STEP 2: phases & velocities (two bands)
u_fast=bandpass_fft(u_std, *FAST)
z_fast=analytic_signal(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
A_f=np.abs(z_fast)
logA_f=np.log(A_f+1e-12)

u_slow=bandpass_fft(u_std, *SLOW)
z_slow=analytic_signal(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(theta_f[1:]-theta_f[:-1])
omega_s=wrap_to_pi(theta_s[1:]-theta_s[:-1])

N_feat=N_kept-1
theta_f_feat=theta_f[:-1]; theta_s_feat=theta_s[:-1]; logA_f_feat=logA_f[:-1]

# STEP 3: R
expDelta=math.exp(Delta)
def count_primes_interval(primes, a, b):
    return int(np.searchsorted(primes, b, side="left") - np.searchsorted(primes, a, side="left"))

P=np.empty(N_kept, dtype=int)
E=np.empty(N_kept, dtype=float)
for i,xi in enumerate(x_k):
    a=float(xi); b=float(xi*expDelta)
    P[i]=count_primes_interval(primes,a,b)
    E[i]=a*(expDelta-1.0)/math.log(a)
R=(P-E)/np.sqrt(E)

# Feature builder
def build_features(idx, use_vel, theta_f_arr, theta_s_arr, omega_f_arr=None, omega_s_arr=None):
    i=idx
    Xb=np.column_stack([R[i], R[i-1], R[i-2], logA_f_feat[i]])
    F=fourier_feats(theta_f_arr[i], fourier_K)
    S=fourier_feats(theta_s_arr[i], fourier_K)
    parts=[Xb,F,S]
    if use_vel:
        of=omega_f_arr[i]; os=omega_s_arr[i]
        WF=np.column_stack([of, np.sin(theta_f_arr[i])*of, np.cos(theta_f_arr[i])*of])
        WS=np.column_stack([os, np.sin(theta_s_arr[i])*os, np.cos(theta_s_arr[i])*os])
        parts += [WF,WS]
    return np.concatenate(parts, axis=1)

# Split feature indices: avoid omega boundary leakage (omega_i uses i+1)
split_feat_end=train_end-1  # first test feature index

res={"params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":fourier_K,
    "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end,"split_feat_end":split_feat_end
},"per_h":{}, "summary":{}}

i_all=np.arange(N_feat)
# precompute y for each h lazily
for h in H:
    # y defined for i where i+h < N_kept
    valid_target = (i_all + h) < N_kept
    y=np.zeros(N_feat, dtype=int)
    y[valid_target]=(R[i_all[valid_target]+h]-R[i_all[valid_target]]>0).astype(int)
    # valid for lags
    valid_base = i_all>=2
    # train-valid: i < split_feat_end and i+h < train_end
    train_valid = valid_base & valid_target & (i_all < split_feat_end) & ((i_all + h) < train_end)
    # test-valid: i >= split_feat_end and i+h >= train_end
    test_valid = valid_base & valid_target & (i_all >= split_feat_end) & ((i_all + h) >= train_end)
    tr_idx=i_all[train_valid]; te_idx=i_all[test_valid]
    y_tr=y[tr_idx]; y_te=y[te_idx]
    # Build features
    Xtr_phase=build_features(tr_idx, False, theta_f_feat, theta_s_feat)
    Xte_phase=build_features(te_idx, False, theta_f_feat, theta_s_feat)
    Xtr_vel=build_features(tr_idx, True, theta_f_feat, theta_s_feat, omega_f, omega_s)
    Xte_vel=build_features(te_idx, True, theta_f_feat, theta_s_feat, omega_f, omega_s)
    # Fit
    clf_phase=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_vel=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_phase.fit(Xtr_phase, y_tr)
    clf_vel.fit(Xtr_vel, y_tr)
    auc_p,brier_p,acc_p=eval_model(clf_phase, Xte_phase, y_te)
    auc_v,brier_v,acc_v=eval_model(clf_vel, Xte_vel, y_te)
    delta_auc=auc_v-auc_p

    # Perm A omega|theta (M_vel)
    n_te=len(te_idx)
    auc_perm=[]
    # arrays in test order
    thf_te=theta_f_feat[te_idx]
    ths_te=theta_s_feat[te_idx]
    # fixed B1 in test order
    Xb_te=np.column_stack([R[te_idx], R[te_idx-1], R[te_idx-2], logA_f_feat[te_idx]])
    F_te=fourier_feats(thf_te, fourier_K)
    S_te=fourier_feats(ths_te, fourier_K)
    base_te=np.concatenate([Xb_te, F_te, S_te], axis=1)
    for _ in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        of_perm=omega_f[te_idx][perm]
        os_perm=omega_s[te_idx][perm]
        WF=np.column_stack([of_perm, np.sin(thf_te)*of_perm, np.cos(thf_te)*of_perm])
        WS=np.column_stack([os_perm, np.sin(ths_te)*os_perm, np.cos(ths_te)*os_perm])
        Xte_perm=np.concatenate([base_te, WF, WS], axis=1)
        p=clf_vel.predict_proba(Xte_perm)[:,1]
        auc=float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan")
        auc_perm.append(auc)
    auc_perm=np.array(auc_perm,float)
    meanA=float(np.nanmean(auc_perm)); stdA=float(np.nanstd(auc_perm, ddof=1))
    pA=float((1+np.sum(auc_perm>=auc_v))/(Nperm+1))
    zA=float((auc_v-meanA)/(stdA if stdA>0 else np.nan))

    # Perm B theta_joint (M_phase)
    auc_perm2=[]
    for _ in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        thf_perm=thf_te[perm]
        ths_perm=ths_te[perm]
        Fp=fourier_feats(thf_perm, fourier_K)
        Sp=fourier_feats(ths_perm, fourier_K)
        Xte_pperm=np.concatenate([Xb_te, Fp, Sp], axis=1)
        p=clf_phase.predict_proba(Xte_pperm)[:,1]
        auc=float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan")
        auc_perm2.append(auc)
    auc_perm2=np.array(auc_perm2,float)
    meanB=float(np.nanmean(auc_perm2)); stdB=float(np.nanstd(auc_perm2, ddof=1))
    pB=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))
    zB=float((auc_p-meanB)/(stdB if stdB>0 else np.nan))

    res["per_h"][str(h)]={"metrics":{
        "M_phase":{"AUC":auc_p,"Brier":brier_p,"Acc@0.5":acc_p,"n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "M_vel":{"AUC":auc_v,"Brier":brier_v,"Acc@0.5":acc_v,"n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "delta_auc":float(delta_auc)
    },"perm":{
        "omega_given_theta":{"auc_true":auc_v,"auc_mean":meanA,"auc_std":stdA,"p":pA,"z":zA},
        "theta_joint":{"auc_true":auc_p,"auc_mean":meanB,"auc_std":stdB,"p":pB,"z":zB}
    }}

def qual_omega(p, delta):
    if not (np.isfinite(p) and np.isfinite(delta)): return "none"
    if p<=0.01 and delta>0.01: return "strong"
    if p<=0.05 and delta>0.005: return "moderate"
    if p<=0.1 and delta>0: return "weak"
    return "none"
def qual_theta(p, drop):
    if not (np.isfinite(p) and np.isfinite(drop)): return "none"
    if p<=0.01 and drop>0.02: return "strong"
    if p<=0.05 and drop>0.01: return "moderate"
    if p<=0.1 and drop>0: return "weak"
    return "none"

omega_sig={}
theta_sig={}
for h in H:
    ph=res["per_h"][str(h)]
    omega_sig[f"h{h}"]=qual_omega(ph["perm"]["omega_given_theta"]["p"], ph["metrics"]["delta_auc"])
    drop=ph["perm"]["theta_joint"]["auc_true"]-ph["perm"]["theta_joint"]["auc_mean"]
    theta_sig[f"h{h}"]=qual_theta(ph["perm"]["theta_joint"]["p"], drop)
res["summary"]={"omega_incremental_signal":omega_sig,"theta_signal":theta_sig}

out=json.dumps(res, indent=2, sort_keys=True)
len(out), out[:400]

### Tool (2026-02-22 11:06:50)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[2], line 202
    200 Sp=fourier_feats(ths_perm, fourier_K)
    201 Xte_pperm=np.concatenate([Xb_te, Fp, Sp], axis=1)
--> 202 p=clf_phase.predict_proba(Xte_pperm)[:,1]
    203 auc=float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan")
    204 auc_perm2.append(auc)

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/linear_model/_logistic.py:1377, in LogisticRegression.predict_proba(self, X)
   1369 ovr = self.multi_class in ["ovr", "warn"] or (
   1370     self.multi_class == "auto"
   1371     and (
   (...)   1374     )
   1375 )
   1376 if ovr:
-> 1377     return super()._predict_proba_lr(X)
   1378 else:
   1379     decision = self.decision_function(X)

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/linear_model/_base.py:366, in LinearClassifierMixin._predict_proba_lr(self, X)
    359 def _predict_proba_lr(self, X):
    360     """Probability estimation for OvR logistic regression.
    361 
    362     Positive class probabilities are computed as
    363     1. / (1. + np.exp(-self.decision_function(X)));
    364     multiclass is handled by normalizing that over all classes.
    365     """
--> 366     prob = self.decision_function(X)
    367     expit(prob, out=prob)
    368     if prob.ndim == 1:

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/linear_model/_base.py:333, in LinearClassifierMixin.decision_function(self, X)
    330 xp, _ = get_namespace(X)
    332 X = self._validate_data(X, accept_sparse="csr", reset=False)
--> 333 scores = safe_sparse_dot(X, self.coef_.T, dense_output=True) + self.intercept_
    334 return xp.reshape(scores, (-1,)) if scores.shape[1] == 1 else scores

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/extmath.py:211, in safe_sparse_dot(a, b, dense_output)
    207 else:
    208     ret = a @ b
    210 if (
--> 211     sparse.issparse(a)
    212     and sparse.issparse(b)
    213     and dense_output
    214     and hasattr(ret, "toarray")
    215 ):
    216     return ret.toarray()
    217 return ret

File /opt/pyvenv/lib/python3.11/site-packages/scipy/sparse/_base.py:1335, in issparse(x)
   1329     """A namespace class to separate sparray from spmatrix"""
   1332 sparray.__doc__ = _spbase.__doc__
-> 1335 def issparse(x):
   1336     """Is `x` of a sparse array or sparse matrix type?
   1337 
   1338     Parameters
   (...)   1359     False
   1360     """
   1361     return isinstance(x, _spbase)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 11:07:55)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from scipy.special import expit

# Load
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load("/mnt/data/primes.npz")
primes=pr["primes"]

# Params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3
rng=np.random.default_rng(seed)

# Helpers
def causal_moving_average(a, W):
    a=np.asarray(a, float); W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0, i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

def bandpass_fft(sig, f_lo, f_hi):
    sig=np.asarray(sig,float); n=len(sig)
    X=rfft(sig)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask, n=n)

def analytic_signal(x):
    x=np.asarray(x,float); n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1; h[1:n//2]=2
    else:
        h[0]=1; h[1:(n+1)//2]=2
    return np.fft.ifft(X*h)

def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

def count_primes_interval(primes, a, b):
    return int(np.searchsorted(primes, b, side="left") - np.searchsorted(primes, a, side="left"))

def block_permute_indices(n, block_size, rng):
    nb=(n + block_size - 1)//block_size
    order=np.arange(nb)
    rng.shuffle(order)
    # build perm indices
    perm=[]
    for j in order:
        start=j*block_size
        if start>=n: continue
        perm.append(np.arange(start, min(start+block_size,n)))
    return np.concatenate(perm)

def fourier_matrix(theta, K):
    # returns [sin(1θ),cos(1θ),...,sin(Kθ),cos(Kθ)]
    cols=[]
    for k in range(1,K+1):
        cols.append(np.sin(k*theta))
        cols.append(np.cos(k*theta))
    return np.column_stack(cols)

def eval_from_linear(coef, intercept, X, y):
    z=X@coef + intercept
    p=expit(z)
    auc=roc_auc_score(y,p) if len(np.unique(y))==2 else float("nan")
    return float(auc), float(brier_score_loss(y,p)), float(accuracy_score(y,(p>=0.5).astype(int)))

# STEP1 uniform log grid etc
t_raw=np.log(x_grid.astype(float))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(float))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
x_k=x[kept]; u_raw=u_interp[kept]
N_kept=len(u_raw)

ma=causal_moving_average(u_raw, W)
u_det=u_raw-ma
train_end=int(math.floor(train_frac*N_kept))
mu=float(u_det[:train_end].mean()); sd=float(u_det[:train_end].std(ddof=0))
u_std=(u_det-mu)/(sd if sd>0 else 1.0)

# STEP2 phases and velocities
u_fast=bandpass_fft(u_std, *FAST)
z_fast=analytic_signal(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=bandpass_fft(u_std, *SLOW)
z_slow=analytic_signal(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(theta_f[1:]-theta_f[:-1])
omega_s=wrap_to_pi(theta_s[1:]-theta_s[:-1])

N_feat=N_kept-1
theta_f_feat=theta_f[:-1]; theta_s_feat=theta_s[:-1]; logA_f_feat=logA_f[:-1]

# STEP3 R
expDelta=math.exp(Delta)
P=np.empty(N_kept, dtype=int)
E=np.empty(N_kept, dtype=float)
for i,xi in enumerate(x_k):
    a=float(xi); b=float(xi*expDelta)
    P[i]=count_primes_interval(primes,a,b)
    E[i]=a*(expDelta-1.0)/math.log(a)
R=(P-E)/np.sqrt(E)

# Split feature indices to avoid boundary leakage for omega
split_feat_end=train_end-1  # first test i
i_all=np.arange(N_feat)

res={"params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end,"split_feat_end":split_feat_end
},"per_h":{}, "summary":{}}

for h in H:
    valid_target=(i_all+h)<N_kept
    valid_base=i_all>=2
    train_valid=valid_base & valid_target & (i_all<split_feat_end) & ((i_all+h)<train_end)
    test_valid =valid_base & valid_target & (i_all>=split_feat_end) & ((i_all+h)>=train_end)
    tr_idx=i_all[train_valid]; te_idx=i_all[test_valid]

    y=np.zeros(N_feat,dtype=int)
    y[valid_target]=(R[i_all[valid_target]+h]-R[i_all[valid_target]]>0).astype(int)
    y_tr=y[tr_idx]; y_te=y[te_idx]

    # Build training matrices
    Xb_tr=np.column_stack([R[tr_idx], R[tr_idx-1], R[tr_idx-2], logA_f_feat[tr_idx]])
    F_tr=fourier_matrix(theta_f_feat[tr_idx], K)
    S_tr=fourier_matrix(theta_s_feat[tr_idx], K)
    X_phase_tr=np.concatenate([Xb_tr,F_tr,S_tr],axis=1)

    Xb_te=np.column_stack([R[te_idx], R[te_idx-1], R[te_idx-2], logA_f_feat[te_idx]])
    thf_te=theta_f_feat[te_idx]; ths_te=theta_s_feat[te_idx]
    F_te=fourier_matrix(thf_te,K); S_te=fourier_matrix(ths_te,K)
    X_phase_te=np.concatenate([Xb_te,F_te,S_te],axis=1)

    # velocity terms
    of_tr=omega_f[tr_idx]; os_tr=omega_s[tr_idx]
    WF_tr=np.column_stack([of_tr, np.sin(theta_f_feat[tr_idx])*of_tr, np.cos(theta_f_feat[tr_idx])*of_tr])
    WS_tr=np.column_stack([os_tr, np.sin(theta_s_feat[tr_idx])*os_tr, np.cos(theta_s_feat[tr_idx])*os_tr])
    X_vel_tr=np.concatenate([X_phase_tr, WF_tr, WS_tr], axis=1)

    of_te=omega_f[te_idx]; os_te=omega_s[te_idx]
    WF_te=np.column_stack([of_te, np.sin(thf_te)*of_te, np.cos(thf_te)*of_te])
    WS_te=np.column_stack([os_te, np.sin(ths_te)*os_te, np.cos(ths_te)*os_te])
    X_vel_te=np.concatenate([X_phase_te, WF_te, WS_te], axis=1)

    # Fit logistic
    clf_phase=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_vel=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_phase.fit(X_phase_tr, y_tr)
    clf_vel.fit(X_vel_tr, y_tr)

    coef_p=clf_phase.coef_.ravel(); int_p=float(clf_phase.intercept_[0])
    coef_v=clf_vel.coef_.ravel(); int_v=float(clf_vel.intercept_[0])

    auc_p,brier_p,acc_p=eval_from_linear(coef_p,int_p,X_phase_te,y_te)
    auc_v,brier_v,acc_v=eval_from_linear(coef_v,int_v,X_vel_te,y_te)
    delta_auc=auc_v-auc_p

    # Perm A omega|theta using fixed X_phase_te and permuting omega blocks
    n_te=len(te_idx)
    base_te=X_phase_te  # fixed
    sin_thf=np.sin(thf_te); cos_thf=np.cos(thf_te)
    sin_ths=np.sin(ths_te); cos_ths=np.cos(ths_te)

    auc_perm=[]
    for _ in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        ofp=of_te[perm]; osp=os_te[perm]
        WF=np.column_stack([ofp, sin_thf*ofp, cos_thf*ofp])
        WS=np.column_stack([osp, sin_ths*osp, cos_ths*osp])
        Xp=np.concatenate([base_te, WF, WS], axis=1)
        z=Xp@coef_v + int_v
        p=expit(z)
        auc_perm.append(float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan"))
    auc_perm=np.array(auc_perm)
    meanA=float(np.nanmean(auc_perm)); stdA=float(np.nanstd(auc_perm, ddof=1))
    pA=float((1+np.sum(auc_perm>=auc_v))/(Nperm+1))
    zA=float((auc_v-meanA)/(stdA if stdA>0 else np.nan))

    # Perm B theta_joint on M_phase
    auc_perm2=[]
    for _ in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        thf_p=thf_te[perm]; ths_p=ths_te[perm]
        Fp=fourier_matrix(thf_p,K); Sp=fourier_matrix(ths_p,K)
        Xp=np.concatenate([Xb_te, Fp, Sp], axis=1)
        z=Xp@coef_p + int_p
        p=expit(z)
        auc_perm2.append(float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan"))
    auc_perm2=np.array(auc_perm2)
    meanB=float(np.nanmean(auc_perm2)); stdB=float(np.nanstd(auc_perm2, ddof=1))
    pB=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))
    zB=float((auc_p-meanB)/(stdB if stdB>0 else np.nan))

    res["per_h"][str(h)]={"metrics":{
        "M_phase":{"AUC":auc_p,"Brier":brier_p,"Acc@0.5":acc_p,
                   "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "M_vel":{"AUC":auc_v,"Brier":brier_v,"Acc@0.5":acc_v,
                 "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "delta_auc":float(delta_auc)
    },"perm":{
        "omega_given_theta":{"auc_true":auc_v,"auc_mean":meanA,"auc_std":stdA,"p":pA,"z":zA},
        "theta_joint":{"auc_true":auc_p,"auc_mean":meanB,"auc_std":stdB,"p":pB,"z":zB}
    }}

def qual_omega(p, delta):
    if not (np.isfinite(p) and np.isfinite(delta)): return "none"
    if p<=0.01 and delta>0.01: return "strong"
    if p<=0.05 and delta>0.005: return "moderate"
    if p<=0.1 and delta>0: return "weak"
    return "none"
def qual_theta(p, drop):
    if not (np.isfinite(p) and np.isfinite(drop)): return "none"
    if p<=0.01 and drop>0.02: return "strong"
    if p<=0.05 and drop>0.01: return "moderate"
    if p<=0.1 and drop>0: return "weak"
    return "none"

omega_sig={}
theta_sig={}
for h in H:
    ph=res["per_h"][str(h)]
    omega_sig[f"h{h}"]=qual_omega(ph["perm"]["omega_given_theta"]["p"], ph["metrics"]["delta_auc"])
    drop=ph["perm"]["theta_joint"]["auc_true"]-ph["perm"]["theta_joint"]["auc_mean"]
    theta_sig[f"h{h}"]=qual_theta(ph["perm"]["theta_joint"]["p"], drop)
res["summary"]={"omega_incremental_signal":omega_sig,"theta_signal":theta_sig}

out=json.dumps(res, indent=2, sort_keys=True)
out[:600], len(out)

### Tool (2026-02-22 11:08:49)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[3], line 206
    204     Xp=np.concatenate([Xb_te, Fp, Sp], axis=1)
    205     z=Xp@coef_p + int_p
--> 206     p=expit(z)
    207     auc_perm2.append(float(roc_auc_score(y_te,p)) if len(np.unique(y_te))==2 else float("nan"))
    208 auc_perm2=np.array(auc_perm2)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 11:09:54)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import roc_auc_score, brier_score_loss, accuracy_score
from scipy.special import expit

# Load data
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load("/mnt/data/primes.npz")
primes=pr["primes"]

# Params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3
rng=np.random.default_rng(seed)

# Helpers
def causal_moving_average(a, W):
    a=np.asarray(a,float); W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0,i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

def bandpass_fft(sig, f_lo, f_hi):
    sig=np.asarray(sig,float); n=len(sig)
    X=rfft(sig)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask, n=n)

def analytic_signal(x):
    x=np.asarray(x,float); n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1; h[1:n//2]=2
    else:
        h[0]=1; h[1:(n+1)//2]=2
    return np.fft.ifft(X*h)

def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

def count_primes_interval(primes, a, b):
    return int(np.searchsorted(primes, b, side="left") - np.searchsorted(primes, a, side="left"))

def block_permute_indices(n, block_size, rng):
    nb=(n + block_size - 1)//block_size
    order=np.arange(nb); rng.shuffle(order)
    perm=np.empty(n, dtype=int)
    pos=0
    for j in order:
        start=j*block_size
        if start>=n: continue
        bidx=np.arange(start, min(start+block_size,n))
        perm[pos:pos+len(bidx)]=bidx
        pos+=len(bidx)
    return perm

def eval_from_linear(coef, intercept, X, y):
    z=X@coef + intercept
    p=expit(z)
    auc=roc_auc_score(y,p) if len(np.unique(y))==2 else float("nan")
    return float(auc), float(brier_score_loss(y,p)), float(accuracy_score(y,(p>=0.5).astype(int)))

# STEP1 uniform log grid
t_raw=np.log(x_grid.astype(float))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(float))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
x_k=x[kept]; u_raw=u_interp[kept]
N_kept=len(u_raw)

ma=causal_moving_average(u_raw, W)
u_det=u_raw-ma
train_end=int(math.floor(train_frac*N_kept))
mu=float(u_det[:train_end].mean()); sd=float(u_det[:train_end].std(ddof=0))
u_std=(u_det-mu)/(sd if sd>0 else 1.0)

# STEP2 phases
u_fast=bandpass_fft(u_std, *FAST)
z_fast=analytic_signal(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=bandpass_fft(u_std, *SLOW)
z_slow=analytic_signal(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(theta_f[1:]-theta_f[:-1])
omega_s=wrap_to_pi(theta_s[1:]-theta_s[:-1])

N_feat=N_kept-1
theta_f_feat=theta_f[:-1]; theta_s_feat=theta_s[:-1]; logA_f_feat=logA_f[:-1]

# STEP3 R
expDelta=math.exp(Delta)
P=np.empty(N_kept,dtype=int)
E=np.empty(N_kept,dtype=float)
for i,xi in enumerate(x_k):
    a=float(xi); b=float(xi*expDelta)
    P[i]=count_primes_interval(primes,a,b)
    E[i]=a*(expDelta-1.0)/math.log(a)
R=(P-E)/np.sqrt(E)

split_feat_end=train_end-1
i_all=np.arange(N_feat)

res={"params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end,"split_feat_end":split_feat_end
},"per_h":{}, "summary":{}}

# precompute trig for test thetas will be horizon-specific (since te_idx differs), but we can do inside loop.
for h in H:
    valid_target=(i_all+h)<N_kept
    valid_base=i_all>=2
    train_valid=valid_base & valid_target & (i_all<split_feat_end) & ((i_all+h)<train_end)
    test_valid =valid_base & valid_target & (i_all>=split_feat_end) & ((i_all+h)>=train_end)
    tr_idx=i_all[train_valid]; te_idx=i_all[test_valid]

    y=np.zeros(N_feat,dtype=int)
    y[valid_target]=(R[i_all[valid_target]+h]-R[i_all[valid_target]]>0).astype(int)
    y_tr=y[tr_idx]; y_te=y[te_idx]

    # TRAIN design
    Xb_tr=np.column_stack([R[tr_idx], R[tr_idx-1], R[tr_idx-2], logA_f_feat[tr_idx]])
    # Fourier
    thf_tr=theta_f_feat[tr_idx]; ths_tr=theta_s_feat[tr_idx]
    F_tr=np.column_stack([np.sin(k*thf_tr) for k in range(1,K+1)] + [np.cos(k*thf_tr) for k in range(1,K+1)])
    # rearrange to sin1,cos1,... to match earlier? We'll match same order sin,cos per k
    F_tr=np.column_stack([np.sin(k*thf_tr) if j%2==0 else np.cos(k*thf_tr) 
                          for k in range(1,K+1) for j in range(2)])
    S_tr=np.column_stack([np.sin(k*ths_tr) if j%2==0 else np.cos(k*ths_tr) 
                          for k in range(1,K+1) for j in range(2)])
    X_phase_tr=np.concatenate([Xb_tr,F_tr,S_tr],axis=1)

    # TEST base
    Xb_te=np.column_stack([R[te_idx], R[te_idx-1], R[te_idx-2], logA_f_feat[te_idx]])
    thf_te=theta_f_feat[te_idx]; ths_te=theta_s_feat[te_idx]
    # precompute sin/cos blocks for K for reuse
    sincos_f=[(np.sin(k*thf_te), np.cos(k*thf_te)) for k in range(1,K+1)]
    sincos_s=[(np.sin(k*ths_te), np.cos(k*ths_te)) for k in range(1,K+1)]
    F_te=np.column_stack([arr for k in range(K) for arr in sincos_f[k]])
    S_te=np.column_stack([arr for k in range(K) for arr in sincos_s[k]])
    X_phase_te=np.concatenate([Xb_te,F_te,S_te],axis=1)

    # velocities
    of_tr=omega_f[tr_idx]; os_tr=omega_s[tr_idx]
    WF_tr=np.column_stack([of_tr, np.sin(thf_tr)*of_tr, np.cos(thf_tr)*of_tr])
    WS_tr=np.column_stack([os_tr, np.sin(ths_tr)*os_tr, np.cos(ths_tr)*os_tr])
    X_vel_tr=np.concatenate([X_phase_tr, WF_tr, WS_tr], axis=1)

    of_te=omega_f[te_idx]; os_te=omega_s[te_idx]
    sin_thf=np.sin(thf_te); cos_thf=np.cos(thf_te)
    sin_ths=np.sin(ths_te); cos_ths=np.cos(ths_te)
    WF_te=np.column_stack([of_te, sin_thf*of_te, cos_thf*of_te])
    WS_te=np.column_stack([os_te, sin_ths*os_te, cos_ths*os_te])
    X_vel_te=np.concatenate([X_phase_te, WF_te, WS_te], axis=1)

    clf_phase=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_vel=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_phase.fit(X_phase_tr, y_tr)
    clf_vel.fit(X_vel_tr, y_tr)
    coef_p=clf_phase.coef_.ravel(); int_p=float(clf_phase.intercept_[0])
    coef_v=clf_vel.coef_.ravel(); int_v=float(clf_vel.intercept_[0])

    auc_p,brier_p,acc_p=eval_from_linear(coef_p,int_p,X_phase_te,y_te)
    auc_v,brier_v,acc_v=eval_from_linear(coef_v,int_v,X_vel_te,y_te)
    delta_auc=auc_v-auc_p

    # Perm A omega|theta
    n_te=len(te_idx)
    base_te=X_phase_te
    auc_perm=np.empty(Nperm)
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        ofp=of_te[perm]; osp=os_te[perm]
        WF=np.column_stack([ofp, sin_thf*ofp, cos_thf*ofp])
        WS=np.column_stack([osp, sin_ths*osp, cos_ths*osp])
        Xp=np.concatenate([base_te, WF, WS], axis=1)
        p=expit(Xp@coef_v + int_v)
        auc_perm[r]=roc_auc_score(y_te,p) if len(np.unique(y_te))==2 else np.nan
    meanA=float(np.nanmean(auc_perm)); stdA=float(np.nanstd(auc_perm, ddof=1))
    pA=float((1+np.sum(auc_perm>=auc_v))/(Nperm+1))
    zA=float((auc_v-meanA)/(stdA if stdA>0 else np.nan))

    # Perm B theta_joint using reindex of precomputed Fourier columns
    # Build index mapping: permute theta means permute Fourier columns in same way for each theta series.
    # Since F_te columns are already functions of thf_te, permuting theta corresponds to permuting rows of those function vectors by perm.
    # So we can just take F_te_perm = F_te[perm], S_te_perm = S_te[perm]
    auc_perm2=np.empty(Nperm)
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        Xp=np.concatenate([Xb_te, F_te[perm], S_te[perm]], axis=1)
        p=expit(Xp@coef_p + int_p)
        auc_perm2[r]=roc_auc_score(y_te,p) if len(np.unique(y_te))==2 else np.nan
    meanB=float(np.nanmean(auc_perm2)); stdB=float(np.nanstd(auc_perm2, ddof=1))
    pB=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))
    zB=float((auc_p-meanB)/(stdB if stdB>0 else np.nan))

    res["per_h"][str(h)]={"metrics":{
        "M_phase":{"AUC":auc_p,"Brier":brier_p,"Acc@0.5":acc_p,
                   "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "M_vel":{"AUC":auc_v,"Brier":brier_v,"Acc@0.5":acc_v,
                 "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "delta_auc":float(delta_auc)
    },"perm":{
        "omega_given_theta":{"auc_true":auc_v,"auc_mean":meanA,"auc_std":stdA,"p":pA,"z":zA},
        "theta_joint":{"auc_true":auc_p,"auc_mean":meanB,"auc_std":stdB,"p":pB,"z":zB}
    }}

def qual_omega(p, delta):
    if not (np.isfinite(p) and np.isfinite(delta)): return "none"
    if p<=0.01 and delta>0.01: return "strong"
    if p<=0.05 and delta>0.005: return "moderate"
    if p<=0.1 and delta>0: return "weak"
    return "none"
def qual_theta(p, drop):
    if not (np.isfinite(p) and np.isfinite(drop)): return "none"
    if p<=0.01 and drop>0.02: return "strong"
    if p<=0.05 and drop>0.01: return "moderate"
    if p<=0.1 and drop>0: return "weak"
    return "none"

omega_sig={}
theta_sig={}
for h in H:
    ph=res["per_h"][str(h)]
    omega_sig[f"h{h}"]=qual_omega(ph["perm"]["omega_given_theta"]["p"], ph["metrics"]["delta_auc"])
    drop=ph["perm"]["theta_joint"]["auc_true"]-ph["perm"]["theta_joint"]["auc_mean"]
    theta_sig[f"h{h}"]=qual_theta(ph["perm"]["theta_joint"]["p"], drop)
res["summary"]={"omega_incremental_signal":omega_sig,"theta_signal":theta_sig}

out=json.dumps(res, indent=2, sort_keys=True)
out[:500], len(out)

### Tool (2026-02-22 11:10:41)

---------------------------------------------------------------------------
KeyboardInterrupt                         Traceback (most recent call last)
Cell In[4], line 206
    204     Xp=np.concatenate([Xb_te, F_te[perm], S_te[perm]], axis=1)
    205     p=expit(Xp@coef_p + int_p)
--> 206     auc_perm2[r]=roc_auc_score(y_te,p) if len(np.unique(y_te))==2 else np.nan
    207 meanB=float(np.nanmean(auc_perm2)); stdB=float(np.nanstd(auc_perm2, ddof=1))
    208 pB=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))

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

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/metrics/_ranking.py:618, in roc_auc_score(y_true, y_score, average, sample_weight, max_fpr, multi_class, labels)
    430 """Compute Area Under the Receiver Operating Characteristic Curve (ROC AUC) \
    431 from prediction scores.
    432 
   (...)    614 array([0.81..., 0.84... , 0.93..., 0.87..., 0.94...])
    615 """
    617 y_type = type_of_target(y_true, input_name="y_true")
--> 618 y_true = check_array(y_true, ensure_2d=False, dtype=None)
    619 y_score = check_array(y_score, ensure_2d=False)
    621 if y_type == "multiclass" or (
    622     y_type == "binary" and y_score.ndim == 2 and y_score.shape[1] > 2
    623 ):
    624     # do not support partial ROC computation for multiclass

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/validation.py:1049, in check_array(array, accept_sparse, accept_large_sparse, dtype, order, copy, force_all_finite, ensure_2d, allow_nd, ensure_min_samples, ensure_min_features, estimator, input_name)
   1043     raise ValueError(
   1044         "Found array with dim %d. %s expected <= 2."
   1045         % (array.ndim, estimator_name)
   1046     )
   1048 if force_all_finite:
-> 1049     _assert_all_finite(
   1050         array,
   1051         input_name=input_name,
   1052         estimator_name=estimator_name,
   1053         allow_nan=force_all_finite == "allow-nan",
   1054     )
   1056 if copy:
   1057     if _is_numpy_namespace(xp):
   1058         # only make a copy if `array` and `array_orig` may share memory`

File /opt/pyvenv/lib/python3.11/site-packages/sklearn/utils/validation.py:96, in _assert_all_finite(X, allow_nan, msg_dtype, estimator_name, input_name)
     91         return _inner_deprecate_positional_args(func)
     93     return _inner_deprecate_positional_args
---> 96 def _assert_all_finite(
     97     X, allow_nan=False, msg_dtype=None, estimator_name=None, input_name=""
     98 ):
     99     """Like assert_all_finite, but only for ndarray."""
    101     xp, _ = get_namespace(X)

KeyboardInterrupt: 

WARNING: Automatic interrupt triggered after 60.0 seconds.

### Assistant (2026-02-22 11:11:45)

import numpy as np, math, json
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import brier_score_loss, accuracy_score
from scipy.special import expit
from scipy.stats import rankdata

# Load data
psi=np.load("/mnt/data/chebyshev_psi.npz")
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load("/mnt/data/primes.npz")
primes=pr["primes"]

# Params
L=65536; x_min=1e5; x_max=9e6; W=2000; Delta=0.01; train_frac=0.7; seed=12345
FAST=(0.002,0.01); SLOW=(0.0002,0.002); H=[25,100]; block_size=512; Nperm=80; K=3
rng=np.random.default_rng(seed)

# Helpers
def causal_moving_average(a, W):
    a=np.asarray(a,float); W=int(W)
    c=np.cumsum(a)
    out=np.empty_like(a)
    for i in range(len(a)):
        j0=max(0,i-W+1)
        s=c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i]=s/(i-j0+1)
    return out

def bandpass_fft(sig, f_lo, f_hi):
    sig=np.asarray(sig,float); n=len(sig)
    X=rfft(sig)
    freqs=rfftfreq(n, d=1.0)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    return irfft(X*mask, n=n)

def analytic_signal(x):
    x=np.asarray(x,float); n=len(x)
    X=np.fft.fft(x)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1; h[1:n//2]=2
    else:
        h[0]=1; h[1:(n+1)//2]=2
    return np.fft.ifft(X*h)

def wrap_to_pi(a):
    return (a + np.pi)%(2*np.pi) - np.pi

def count_primes_interval(primes, a, b):
    return int(np.searchsorted(primes, b, side="left") - np.searchsorted(primes, a, side="left"))

def block_permute_indices(n, block_size, rng):
    nb=(n + block_size - 1)//block_size
    order=np.arange(nb); rng.shuffle(order)
    perm=np.empty(n, dtype=int)
    pos=0
    for j in order:
        start=j*block_size
        if start>=n: continue
        bidx=np.arange(start, min(start+block_size,n))
        perm[pos:pos+len(bidx)]=bidx
        pos+=len(bidx)
    return perm

def auc_fast(y, s):
    # y in {0,1}
    y=np.asarray(y)
    s=np.asarray(s)
    n1=int(y.sum())
    n0=int(len(y)-n1)
    if n1==0 or n0==0:
        return float("nan")
    r=rankdata(s, method="average")  # 1..n
    sum_r1=float(r[y==1].sum())
    return (sum_r1 - n1*(n1+1)/2.0)/(n1*n0)

def eval_linear_auc_brier_acc(coef, intercept, X, y):
    p=expit(X@coef + intercept)
    return float(auc_fast(y,p)), float(brier_score_loss(y,p)), float(accuracy_score(y,(p>=0.5).astype(int)))

# STEP1
t_raw=np.log(x_grid.astype(float))
t=np.linspace(float(t_raw.min()), float(t_raw.max()), L)
dt=float(t[1]-t[0])
u_interp=np.interp(t, t_raw, psi_minus_x.astype(float))
x=np.exp(t)
kept=(x>=x_min)&(x<=x_max)
x_k=x[kept]; u_raw=u_interp[kept]
N_kept=len(u_raw)

ma=causal_moving_average(u_raw, W)
u_det=u_raw-ma
train_end=int(math.floor(train_frac*N_kept))
mu=float(u_det[:train_end].mean()); sd=float(u_det[:train_end].std(ddof=0))
u_std=(u_det-mu)/(sd if sd>0 else 1.0)

# STEP2
u_fast=bandpass_fft(u_std, *FAST)
z_fast=analytic_signal(u_fast)
theta_f=(np.angle(z_fast)%(2*np.pi))
logA_f=np.log(np.abs(z_fast)+1e-12)

u_slow=bandpass_fft(u_std, *SLOW)
z_slow=analytic_signal(u_slow)
theta_s=(np.angle(z_slow)%(2*np.pi))

omega_f=wrap_to_pi(theta_f[1:]-theta_f[:-1])
omega_s=wrap_to_pi(theta_s[1:]-theta_s[:-1])

N_feat=N_kept-1
theta_f_feat=theta_f[:-1]; theta_s_feat=theta_s[:-1]; logA_f_feat=logA_f[:-1]

# STEP3 R
expDelta=math.exp(Delta)
P=np.empty(N_kept,dtype=int)
E=np.empty(N_kept,dtype=float)
for i,xi in enumerate(x_k):
    a=float(xi); b=float(xi*expDelta)
    P[i]=count_primes_interval(primes,a,b)
    E[i]=a*(expDelta-1.0)/math.log(a)
R=(P-E)/np.sqrt(E)

split_feat_end=train_end-1
i_all=np.arange(N_feat)

res={"params":{
    "L":L,"x_min":x_min,"x_max":x_max,"W":W,"Delta":Delta,"train_frac":train_frac,"seed":seed,
    "FAST":list(FAST),"SLOW":list(SLOW),"H":H,"block_size":block_size,"Nperm":Nperm,"fourier_K":K,
    "dt":dt,"n_kept":N_kept,"n_feat":N_feat,"train_end":train_end,"split_feat_end":split_feat_end
},"per_h":{}, "summary":{}}

for h in H:
    valid_target=(i_all+h)<N_kept
    valid_base=i_all>=2
    train_valid=valid_base & valid_target & (i_all<split_feat_end) & ((i_all+h)<train_end)
    test_valid =valid_base & valid_target & (i_all>=split_feat_end) & ((i_all+h)>=train_end)
    tr_idx=i_all[train_valid]; te_idx=i_all[test_valid]

    y=np.zeros(N_feat,dtype=int)
    y[valid_target]=(R[i_all[valid_target]+h]-R[i_all[valid_target]]>0).astype(int)
    y_tr=y[tr_idx]; y_te=y[te_idx]

    # Design matrices
    Xb_tr=np.column_stack([R[tr_idx], R[tr_idx-1], R[tr_idx-2], logA_f_feat[tr_idx]])
    thf_tr=theta_f_feat[tr_idx]; ths_tr=theta_s_feat[tr_idx]
    F_tr=np.column_stack([np.sin(k*thf_tr) if j%2==0 else np.cos(k*thf_tr) for k in range(1,K+1) for j in range(2)])
    S_tr=np.column_stack([np.sin(k*ths_tr) if j%2==0 else np.cos(k*ths_tr) for k in range(1,K+1) for j in range(2)])
    X_phase_tr=np.concatenate([Xb_tr,F_tr,S_tr],axis=1)

    Xb_te=np.column_stack([R[te_idx], R[te_idx-1], R[te_idx-2], logA_f_feat[te_idx]])
    thf_te=theta_f_feat[te_idx]; ths_te=theta_s_feat[te_idx]
    F_te=np.column_stack([np.sin(k*thf_te) if j%2==0 else np.cos(k*thf_te) for k in range(1,K+1) for j in range(2)])
    S_te=np.column_stack([np.sin(k*ths_te) if j%2==0 else np.cos(k*ths_te) for k in range(1,K+1) for j in range(2)])
    X_phase_te=np.concatenate([Xb_te,F_te,S_te],axis=1)

    of_tr=omega_f[tr_idx]; os_tr=omega_s[tr_idx]
    WF_tr=np.column_stack([of_tr, np.sin(thf_tr)*of_tr, np.cos(thf_tr)*of_tr])
    WS_tr=np.column_stack([os_tr, np.sin(ths_tr)*os_tr, np.cos(ths_tr)*os_tr])
    X_vel_tr=np.concatenate([X_phase_tr, WF_tr, WS_tr], axis=1)

    of_te=omega_f[te_idx]; os_te=omega_s[te_idx]
    sin_thf=np.sin(thf_te); cos_thf=np.cos(thf_te)
    sin_ths=np.sin(ths_te); cos_ths=np.cos(ths_te)
    WF_te=np.column_stack([of_te, sin_thf*of_te, cos_thf*of_te])
    WS_te=np.column_stack([os_te, sin_ths*os_te, cos_ths*os_te])
    X_vel_te=np.concatenate([X_phase_te, WF_te, WS_te], axis=1)

    clf_phase=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_vel=LogisticRegression(penalty="l2", C=1.0, solver="lbfgs", max_iter=2000, random_state=seed)
    clf_phase.fit(X_phase_tr, y_tr)
    clf_vel.fit(X_vel_tr, y_tr)
    coef_p=clf_phase.coef_.ravel(); int_p=float(clf_phase.intercept_[0])
    coef_v=clf_vel.coef_.ravel(); int_v=float(clf_vel.intercept_[0])

    auc_p,brier_p,acc_p=eval_linear_auc_brier_acc(coef_p,int_p,X_phase_te,y_te)
    auc_v,brier_v,acc_v=eval_linear_auc_brier_acc(coef_v,int_v,X_vel_te,y_te)
    delta_auc=auc_v-auc_p

    # Perm A omega|theta (fast auc)
    n_te=len(te_idx)
    base_te=X_phase_te
    auc_perm=np.empty(Nperm)
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        ofp=of_te[perm]; osp=os_te[perm]
        WF=np.column_stack([ofp, sin_thf*ofp, cos_thf*ofp])
        WS=np.column_stack([osp, sin_ths*osp, cos_ths*osp])
        Xp=np.concatenate([base_te, WF, WS], axis=1)
        p=expit(Xp@coef_v + int_v)
        auc_perm[r]=auc_fast(y_te,p)
    meanA=float(np.nanmean(auc_perm)); stdA=float(np.nanstd(auc_perm, ddof=1))
    pA=float((1+np.sum(auc_perm>=auc_v))/(Nperm+1))
    zA=float((auc_v-meanA)/(stdA if stdA>0 else np.nan))

    # Perm B theta_joint (reuse precomputed Fourier rows)
    auc_perm2=np.empty(Nperm)
    for r in range(Nperm):
        perm=block_permute_indices(n_te, block_size, rng)
        Xp=np.concatenate([Xb_te, F_te[perm], S_te[perm]], axis=1)
        p=expit(Xp@coef_p + int_p)
        auc_perm2[r]=auc_fast(y_te,p)
    meanB=float(np.nanmean(auc_perm2)); stdB=float(np.nanstd(auc_perm2, ddof=1))
    pB=float((1+np.sum(auc_perm2>=auc_p))/(Nperm+1))
    zB=float((auc_p-meanB)/(stdB if stdB>0 else np.nan))

    res["per_h"][str(h)]={"metrics":{
        "M_phase":{"AUC":auc_p,"Brier":brier_p,"Acc@0.5":acc_p,
                   "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "M_vel":{"AUC":auc_v,"Brier":brier_v,"Acc@0.5":acc_v,
                 "n_train":int(len(tr_idx)),"n_test":int(len(te_idx)),"pos_rate_test":float(y_te.mean())},
        "delta_auc":float(delta_auc)
    },"perm":{
        "omega_given_theta":{"auc_true":auc_v,"auc_mean":meanA,"auc_std":stdA,"p":pA,"z":zA},
        "theta_joint":{"auc_true":auc_p,"auc_mean":meanB,"auc_std":stdB,"p":pB,"z":zB}
    }}

def qual_omega(p, delta):
    if not (np.isfinite(p) and np.isfinite(delta)): return "none"
    if p<=0.01 and delta>0.01: return "strong"
    if p<=0.05 and delta>0.005: return "moderate"
    if p<=0.1 and delta>0: return "weak"
    return "none"
def qual_theta(p, drop):
    if not (np.isfinite(p) and np.isfinite(drop)): return "none"
    if p<=0.01 and drop>0.02: return "strong"
    if p<=0.05 and drop>0.01: return "moderate"
    if p<=0.1 and drop>0: return "weak"
    return "none"

omega_sig={}
theta_sig={}
for h in H:
    ph=res["per_h"][str(h)]
    omega_sig[f"h{h}"]=qual_omega(ph["perm"]["omega_given_theta"]["p"], ph["metrics"]["delta_auc"])
    drop=ph["perm"]["theta_joint"]["auc_true"]-ph["perm"]["theta_joint"]["auc_mean"]
    theta_sig[f"h{h}"]=qual_theta(ph["perm"]["theta_joint"]["p"], drop)
res["summary"]={"omega_incremental_signal":omega_sig,"theta_signal":theta_sig}

out=json.dumps(res, indent=2, sort_keys=True)
out[:500], len(out)

### Tool (2026-02-22 11:12:36)

('{\n  "params": {\n    "Delta": 0.01,\n    "FAST": [\n      0.002,\n      0.01\n    ],\n    "H": [\n      25,\n      100\n    ],\n    "L": 65536,\n    "Nperm": 80,\n    "SLOW": [\n      0.0002,\n      0.002\n    ],\n    "W": 2000,\n    "block_size": 512,\n    "dt": 0.00024594622340595577,\n    "fourier_K": 3,\n    "n_feat": 18295,\n    "n_kept": 18296,\n    "seed": 12345,\n    "split_feat_end": 12806,\n    "train_end": 12807,\n    "train_frac": 0.7,\n    "x_max": 9000000.0,\n    "x_min": 100000.0\n  },\n  "per_h": {\n    "100"',
 2843)

### Assistant (2026-02-22 11:13:17)

out

### Tool (2026-02-22 11:13:19)

'{\n  "params": {\n    "Delta": 0.01,\n    "FAST": [\n      0.002,\n      0.01\n    ],\n    "H": [\n      25,\n      100\n    ],\n    "L": 65536,\n    "Nperm": 80,\n    "SLOW": [\n      0.0002,\n      0.002\n    ],\n    "W": 2000,\n    "block_size": 512,\n    "dt": 0.00024594622340595577,\n    "fourier_K": 3,\n    "n_feat": 18295,\n    "n_kept": 18296,\n    "seed": 12345,\n    "split_feat_end": 12806,\n    "train_end": 12807,\n    "train_frac": 0.7,\n    "x_max": 9000000.0,\n    "x_min": 100000.0\n  },\n  "per_h": {\n    "100": {\n      "metrics": {\n        "M_phase": {\n          "AUC": 0.8413868288325401,\n          "Acc@0.5": 0.7601113172541744,\n          "Brier": 0.16327649546802048,\n          "n_test": 5390,\n          "n_train": 12705,\n          "pos_rate_test": 0.500556586270872\n        },\n        "M_vel": {\n          "AUC": 0.8445747606779332,\n          "Acc@0.5": 0.7602968460111317,\n          "Brier": 0.16159715273890835,\n          "n_test": 5390,\n          "n_train": 12705,\n          "pos_rate_test": 0.500556586270872\n        },\n        "delta_auc": 0.0031879318453931216\n      },\n      "perm": {\n        "omega_given_theta": {\n          "auc_mean": 0.8411224310947407,\n          "auc_std": 0.0010011057497710972,\n          "auc_true": 0.8445747606779332,\n          "p": 0.012345679012345678,\n          "z": 3.44851638698699\n        },\n        "theta_joint": {\n          "auc_mean": 0.8281155738194712,\n          "auc_std": 0.005133706018483743,\n          "auc_true": 0.8413868288325401,\n          "p": 0.012345679012345678,\n          "z": 2.5851217356985754\n        }\n      }\n    },\n    "25": {\n      "metrics": {\n        "M_phase": {\n          "AUC": 0.8366671917681238,\n          "Acc@0.5": 0.7513266239707228,\n          "Brier": 0.1654093368307039,\n          "n_test": 5465,\n          "n_train": 12780,\n          "pos_rate_test": 0.5066788655077767\n        },\n        "M_vel": {\n          "AUC": 0.8449427639411757,\n          "Acc@0.5": 0.7560841720036596,\n          "Brier": 0.1611142968803234,\n          "n_test": 5465,\n          "n_train": 12780,\n          "pos_rate_test": 0.5066788655077767\n        },\n        "delta_auc": 0.00827557217305197\n      },\n      "perm": {\n        "omega_given_theta": {\n          "auc_mean": 0.8370544189296931,\n          "auc_std": 0.0018861533731660593,\n          "auc_true": 0.8449427639411757,\n          "p": 0.012345679012345678,\n          "z": 4.1822394316965825\n        },\n        "theta_joint": {\n          "auc_mean": 0.764231245773737,\n          "auc_std": 0.011557495800003736,\n          "auc_true": 0.8366671917681238,\n          "p": 0.012345679012345678,\n          "z": 6.267442986599513\n        }\n      }\n    }\n  },\n  "summary": {\n    "omega_incremental_signal": {\n      "h100": "weak",\n      "h25": "moderate"\n    },\n    "theta_signal": {\n      "h100": "moderate",\n      "h25": "moderate"\n    }\n  }\n}'

### Assistant (2026-02-22 11:13:20)

{
  "params": {
    "Delta": 0.01,
    "FAST": [
      0.002,
      0.01
    ],
    "H": [
      25,
      100
    ],
    "L": 65536,
    "Nperm": 80,
    "SLOW": [
      0.0002,
      0.002
    ],
    "W": 2000,
    "block_size": 512,
    "dt": 0.00024594622340595577,
    "fourier_K": 3,
    "n_feat": 18295,
    "n_kept": 18296,
    "seed": 12345,
    "split_feat_end": 12806,
    "train_end": 12807,
    "train_frac": 0.7,
    "x_max": 9000000.0,
    "x_min": 100000.0
  },
  "per_h": {
    "100": {
      "metrics": {
        "M_phase": {
          "AUC": 0.8413868288325401,
          "Acc@0.5": 0.7601113172541744,
          "Brier": 0.16327649546802048,
          "n_test": 5390,
          "n_train": 12705,
          "pos_rate_test": 0.500556586270872
        },
        "M_vel": {
          "AUC": 0.8445747606779332,
          "Acc@0.5": 0.7602968460111317,
          "Brier": 0.16159715273890835,
          "n_test": 5390,
          "n_train": 12705,
          "pos_rate_test": 0.500556586270872
        },
        "delta_auc": 0.0031879318453931216
      },
      "perm": {
        "omega_given_theta": {
          "auc_mean": 0.8411224310947407,
          "auc_std": 0.0010011057497710972,
          "auc_true": 0.8445747606779332,
          "p": 0.012345679012345678,
          "z": 3.44851638698699
        },
        "theta_joint": {
          "auc_mean": 0.8281155738194712,
          "auc_std": 0.005133706018483743,
          "auc_true": 0.8413868288325401,
          "p": 0.012345679012345678,
          "z": 2.5851217356985754
        }
      }
    },
    "25": {
      "metrics": {
        "M_phase": {
          "AUC": 0.8366671917681238,
          "Acc@0.5": 0.7513266239707228,
          "Brier": 0.1654093368307039,
          "n_test": 5465,
          "n_train": 12780,
          "pos_rate_test": 0.5066788655077767
        },
        "M_vel": {
          "AUC": 0.8449427639411757,
          "Acc@0.5": 0.7560841720036596,
          "Brier": 0.1611142968803234,
          "n_test": 5465,
          "n_train": 12780,
          "pos_rate_test": 0.5066788655077767
        },
        "delta_auc": 0.00827557217305197
      },
      "perm": {
        "omega_given_theta": {
          "auc_mean": 0.8370544189296931,
          "auc_std": 0.0018861533731660593,
          "auc_true": 0.8449427639411757,
          "p": 0.012345679012345678,
          "z": 4.1822394316965825
        },
        "theta_joint": {
          "auc_mean": 0.764231245773737,
          "auc_std": 0.011557495800003736,
          "auc_true": 0.8366671917681238,
          "p": 0.012345679012345678,
          "z": 6.267442986599513
        }
      }
    }
  },
  "summary": {
    "omega_incremental_signal": {
      "h100": "weak",
      "h25": "moderate"
    },
    "theta_signal": {
      "h100": "moderate",
      "h25": "moderate"
    }
  }
}

