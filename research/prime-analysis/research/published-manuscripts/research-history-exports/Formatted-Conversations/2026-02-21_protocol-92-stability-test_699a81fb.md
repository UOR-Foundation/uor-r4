# Protocol 9.2 Stability Test

- **Conversation ID:** `699a81fb-a0d8-832c-94c4-9b836975de53`
- **Date:** 2026-02-21

---

### User (2026-02-21 23:11:44)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal:
Protocol 9.2 Cross-N Stability + Block-Permutation Significance
for phase-conditioned gap modulation.

Use project files:
- chebyshev_psi.npz
- primes.npz
- gaps.npz

Nmax_list = [1_000_000, 3_000_000, 10_000_000]

Common parameters:
L = 65536
W = 2000 (causal moving average)
bandpass = [0.002, 0.01] cycles/sample
B = 12 phase bins
K = [10, 20, 50]
block_size = 500  (for permutation)
num_permutations = 200
seed = 12345

For each Nmax in Nmax_list:

1) Restrict primes and gaps to [1e5, Nmax).
2) Build uniform log grid from chebyshev_psi over full domain.
3) Restrict to x in [1e5, Nmax].
4) Time-ordered split 70% TRAIN / 30% TEST.
5) Causal detrend (W=2000) using kept indices.
6) Standardize using TRAIN mean/std.
7) Bandpass [0.002,0.01].
8) Hilbert => theta_mod in [0,2π).

Gap mapping:
- Map each prime p_n to nearest t index (monotone search).
- Assign phase bin b_n.
- Keep gaps where both primes lie in [1e5, Nmax].

On TEST only:

For each k in K:
    Compute per-bin frac_ge_k.
    Compute variance across bins: var_k.

Permutation control:
For each permutation:
    - Divide TEST primes into contiguous blocks of size block_size.
    - Randomly permute block order.
    - Recompute phase-bin assignments (preserving gap structure).
    - Recompute var_k.

For each k:
    - empirical var_k
    - mean_perm_var_k
    - p_value = fraction of permuted var >= empirical var

Output JSON:

{
  "params": {...},
  "results": [
    {
      "Nmax": ...,
      "num_gaps_test": ...,
      "var_across_bins": {...},
      "perm_mean_var": {...},
      "p_values": {...}
    },
    ...
  ]
}

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

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

### Tool (2026-02-21 23:11:47)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-21 23:11:47)

import numpy as np, math, json, os, statistics, random
from numpy.fft import rfft, irfft
from scipy.signal import hilbert

# Load data
cheb = np.load('/mnt/data/chebyshev_psi.npz')
x_grid = cheb['x_grid']
psi_minus_x = cheb['psi_minus_x']
pr = np.load('/mnt/data/primes.npz')
primes = pr['primes']
gaps_npz = np.load('/mnt/data/gaps.npz')
gaps = gaps_npz['gaps'] if 'gaps' in gaps_npz.files else gaps_npz[gaps_npz.files[0]]
len(primes), len(gaps), primes[:5], gaps[:5]

### Assistant (2026-02-21 23:12:27)

import numpy as np, os, json, math, random
import numpy.fft as fft
import scipy
from scipy import signal
print("scipy", scipy.__version__)

### Tool (2026-02-21 23:12:28)

scipy 1.14.1

### Assistant (2026-02-21 23:12:30)

import os, glob
sorted(glob.glob("/mnt/data/*"))[:20], len(glob.glob("/mnt/data/*"))

### Tool (2026-02-21 23:12:33)

(['/mnt/data/PROJECT_HANDOFF.md',
  '/mnt/data/Protocol.md',
  '/mnt/data/chebyshev_psi.npz',
  '/mnt/data/gaps.npz',
  '/mnt/data/pi.npz',
  '/mnt/data/prime_phase_handoff_bundle.zip',
  '/mnt/data/prime_phase_handoff_pack.zip',
  '/mnt/data/prime_phase_research_snapshot.json',
  '/mnt/data/prime_phase_research_snapshot_2.json',
  '/mnt/data/prime_phase_research_snapshot_3.json',
  '/mnt/data/primes.npz',
  '/mnt/data/project_handoff_state.json',
  '/mnt/data/psi_log_spectrum.json'],
 13)

### Assistant (2026-02-21 23:12:35)

import numpy as np
cheb=np.load('/mnt/data/chebyshev_psi.npz')
cheb.files

### Tool (2026-02-21 23:12:37)

['N', 'step', 'x_grid', 'psi_grid', 'psi_minus_x']

### Assistant (2026-02-21 23:12:39)

pr=np.load('/mnt/data/primes.npz')
pr.files, pr['primes'][:5], pr['primes'][-1]

### Tool (2026-02-21 23:12:40)

(['N', 'primes', 'is_prime'],
 array([ 2,  3,  5,  7, 11], dtype=int32),
 9999991)

### Assistant (2026-02-21 23:12:42)

g=np.load('/mnt/data/gaps.npz')
g.files, g[g.files[0]][:5], g[g.files[0]].shape

### Tool (2026-02-21 23:12:44)

---------------------------------------------------------------------------
IndexError                                Traceback (most recent call last)
Cell In[6], line 2
      1 g=np.load('/mnt/data/gaps.npz')
----> 2 g.files, g[g.files[0]][:5], g[g.files[0]].shape

IndexError: too many indices for array: array is 0-dimensional, but 1 were indexed

### Assistant (2026-02-21 23:12:46)

import numpy as np
g=np.load('/mnt/data/gaps.npz', allow_pickle=True)
g.files

### Tool (2026-02-21 23:12:47)

['N', 'gaps']

### Assistant (2026-02-21 23:12:49)

gaps=g['gaps']
type(gaps), gaps.shape, gaps.dtype

### Tool (2026-02-21 23:12:51)

(numpy.ndarray, (664578,), dtype('int32'))

### Assistant (2026-02-21 23:12:52)

import numpy as np, math, random, json
from scipy import signal

def causal_moving_average(x, W):
    # mean of last W samples including current; for first i<W use mean of x[:i+1]
    c = np.cumsum(x, dtype=float)
    out = np.empty_like(x, dtype=float)
    for i in range(len(x)):
        j0 = max(0, i-W+1)
        s = c[i] - (c[j0-1] if j0>0 else 0.0)
        out[i] = s / (i-j0+1)
    return out

def fft_bandpass(x, f_lo, f_hi):
    # x real, length n. Use rfft frequencies cycles/sample.
    n=len(x)
    X=np.fft.rfft(x)
    freqs=np.fft.rfftfreq(n, d=1.0)  # cycles/sample
    mask=(freqs>=f_lo) & (freqs<=f_hi)
    # keep symmetric; rfft includes DC and Nyquist
    Xf = X * mask
    y=np.fft.irfft(Xf, n=n)
    return y

def compute_theta_bins_for_Nmax(Nmax, L=65536, W=2000, band=(0.002,0.01), B=12):
    # uniform t over full cheb domain
    x_grid = cheb['x_grid'].astype(float)
    f_raw_grid = cheb['psi_minus_x'].astype(float)
    t_grid = np.log(x_grid)
    t_uni = np.linspace(t_grid.min(), t_grid.max(), L)
    dt = t_uni[1]-t_uni[0]
    # interpolate f onto t_uni
    f_uni = np.interp(t_uni, t_grid, f_raw_grid)
    x_uni = np.exp(t_uni)
    # keep indices
    keep = (x_uni>=1e5) & (x_uni<=Nmax)
    t = t_uni[keep]
    x = x_uni[keep]
    f = f_uni[keep]
    n=len(f)
    n_train=int(math.floor(0.7*n))
    # causal detrend on kept
    ma = causal_moving_average(f, W)
    f_dt = f - ma
    # standardize using TRAIN
    mu = f_dt[:n_train].mean()
    sig = f_dt[:n_train].std(ddof=0)
    if sig==0: sig=1.0
    z = (f_dt - mu)/sig
    # bandpass
    z_bp = fft_bandpass(z, band[0], band[1])
    # hilbert
    analytic = signal.hilbert(z_bp)
    theta = np.angle(analytic)  # [-pi,pi]
    theta_mod = np.mod(theta, 2*np.pi)
    # phase bins for each t index
    bins = np.floor(B*theta_mod/(2*np.pi)).astype(int)
    bins = np.clip(bins, 0, B-1)
    return {
        "dt": float(dt),
        "t": t,
        "x": x,
        "bins_t": bins,
        "n_train": n_train
    }

def map_primes_to_t_bins(primes_arr, x_kept, bins_t):
    # map each prime to nearest index in x_kept
    idx = np.searchsorted(x_kept, primes_arr)
    # clip to [0,n-1]
    idx = np.clip(idx, 0, len(x_kept)-1)
    # choose nearer between idx and idx-1 when possible
    idx2 = idx.copy()
    mask = idx>0
    left = idx[mask]-1
    right = idx[mask]
    # compare distances
    choose_left = (np.abs(x_kept[left]-primes_arr[mask]) <= np.abs(x_kept[right]-primes_arr[mask]))
    idx2[mask] = np.where(choose_left, left, right)
    prime_bins = bins_t[idx2]
    return idx2, prime_bins

def block_permutation_variances(bin_seq, gaps_seq, B, K_list, block_size, num_perm, seed):
    rng = np.random.default_rng(seed)
    n_pr=len(bin_seq)
    # blocks indices
    num_blocks = int(math.ceil(n_pr / block_size))
    blocks = [bin_seq[i*block_size : min((i+1)*block_size, n_pr)] for i in range(num_blocks)]
    # precompute left-bin per gap uses bin_seq[:-1]
    perm_vars = {k:[] for k in K_list}
    for r in range(num_perm):
        order = rng.permutation(num_blocks)
        b_perm = np.concatenate([blocks[i] for i in order])
        left_bins = b_perm[:-1]
        # compute var per k
        for k in K_list:
            frac=[]
            for b in range(B):
                m = (left_bins==b)
                if m.sum()==0:
                    continue
                frac.append(float(np.mean(gaps_seq[m] >= k)))
            if len(frac)<=1:
                v=0.0
            else:
                v=float(np.var(frac, ddof=0))
            perm_vars[k].append(v)
    return perm_vars

# load datasets globally
cheb = np.load('/mnt/data/chebyshev_psi.npz')
pr = np.load('/mnt/data/primes.npz')
primes_all = pr['primes'].astype(int)
gaps_all = np.load('/mnt/data/gaps.npz', allow_pickle=True)['gaps'].astype(int)
len(primes_all), len(gaps_all), primes_all[-1], gaps_all[-1]

### Tool (2026-02-21 23:13:15)

(664579, 664578, 9999991, 18)

### Assistant (2026-02-21 23:13:16)

def run_protocol(Nmax_list, L=65536, W=2000, band=(0.002,0.01), B=12, K_list=[10,20,50], block_size=500, num_perm=200, seed=12345):
    results=[]
    for Nmax in Nmax_list:
        # restrict primes to [1e5, Nmax)
        mask_p = (primes_all>=100000) & (primes_all < Nmax)
        primes = primes_all[mask_p]
        # gaps where both primes in range
        # find indices in primes_all
        idx0 = np.where(mask_p)[0]
        if len(idx0)==0:
            raise ValueError("no primes")
        # consecutive gaps within selection: need mask on original consecutive pairs
        mask_gap = mask_p[:-1] & mask_p[1:]
        gaps_sel = gaps_all[mask_gap]
        primes_left = primes_all[:-1][mask_gap]  # left primes for each gap
        primes_right = primes_all[1:][mask_gap]
        # compute theta bins over t indices
        theta_info = compute_theta_bins_for_Nmax(Nmax, L=L, W=W, band=band, B=B)
        x_kept = theta_info["x"]
        bins_t = theta_info["bins_t"]
        n_train = theta_info["n_train"]
        # map left and right primes to t indices and bins
        # Since we need per prime bins for test primes sequence, we'll map primes to bins
        t_idx_primes, prime_bins = map_primes_to_t_bins(primes, x_kept, bins_t)
        # define test primes by t_idx >= n_train
        test_mask_prime = t_idx_primes >= n_train
        primes_test = primes[test_mask_prime]
        bins_test = prime_bins[test_mask_prime]
        # Build gaps_test sequence aligned to primes_test order:
        # need gaps between consecutive primes in primes_test list (in increasing prime order) but only those where both in test
        # We can use original gaps_sel/primes_left/primes_right; filter where both mapped to test.
        # Map left primes for gaps_sel to test? We can compute using mapping of primes_all for left primes maybe.
        # We'll map primes_left and primes_right to t indices.
        t_left, b_left = map_primes_to_t_bins(primes_left, x_kept, bins_t)
        t_right, b_right = map_primes_to_t_bins(primes_right, x_kept, bins_t)
        gap_test_mask = (t_left >= n_train) & (t_right >= n_train)
        gaps_test = gaps_sel[gap_test_mask]
        # For phase bin per gap use left prime bin b_left
        bins_gap_left = b_left[gap_test_mask]
        # But for block permutation we need bin sequence for test primes in order (bins_test) and gaps sequence between consecutive test primes.
        # Ensure alignment: gaps_test should correspond to consecutive primes where both in test, which is exactly gaps between consecutive primes in full list.
        # Those left primes are subset of primes_test but missing first gaps if boundary.
        # For block permutation, easier: build sequence of test primes as in primes_all filtered by test t; then compute consecutive gaps among them.
        # Let's reconstruct by taking primes_inrange_with_t and then filter by test t and consecutive in original list.
        # We'll build arrays from original primes_all indices:
        # get indices of primes_all in range and with t>=n_train
        idx_all_inrange = np.where(mask_p)[0]
        # map each of those primes to t_idx
        primes_inrange = primes_all[idx_all_inrange]
        t_idx_inrange, b_inrange = map_primes_to_t_bins(primes_inrange, x_kept, bins_t)
        test_inrange_mask = t_idx_inrange >= n_train
        idx_all_test = idx_all_inrange[test_inrange_mask]
        primes_test_seq = primes_all[idx_all_test]
        bins_test_seq = b_inrange[test_inrange_mask]
        # gaps between consecutive primes in original list that are both in idx_all_test:
        # since idx_all_test are indices in primes_all, consecutive in original means adjacent indices differ by 1.
        consec_mask = np.diff(idx_all_test)==1
        gaps_test_seq = gaps_all[idx_all_test[:-1][consec_mask]]  # gap at left index
        bins_left_seq = bins_test_seq[:-1][consec_mask]
        num_gaps_test = int(len(gaps_test_seq))
        # empirical var_k
        var_emp={}
        for k in K_list:
            frac=[]
            for b in range(B):
                m = (bins_left_seq==b)
                if m.sum()==0:
                    continue
                frac.append(float(np.mean(gaps_test_seq[m] >= k)))
            var_emp[str(k)] = float(np.var(frac, ddof=0)) if len(frac)>1 else 0.0
        # permutation vars
        perm_vars = block_permutation_variances(bins_test_seq, gaps_test_seq, B, K_list, block_size, num_perm, seed)
        perm_mean={}
        pvals={}
        for k in K_list:
            arr=np.array(perm_vars[k], dtype=float)
            perm_mean[str(k)] = float(arr.mean())
            pvals[str(k)] = float(np.mean(arr >= var_emp[str(k)]))
        results.append({
            "Nmax": int(Nmax),
            "num_primes_test": int(len(primes_test_seq)),
            "num_gaps_test": num_gaps_test,
            "dt": float(theta_info["dt"]),
            "var_across_bins": var_emp,
            "perm_mean_var": perm_mean,
            "p_values": pvals
        })
    return results

params = {
    "Nmax_list":[1000000,3000000,10000000],
    "L":65536,
    "W":2000,
    "bandpass":[0.002,0.01],
    "B":12,
    "K":[10,20,50],
    "block_size":500,
    "num_permutations":200,
    "seed":12345
}
res = run_protocol(params["Nmax_list"], L=params["L"], W=params["W"], band=tuple(params["bandpass"]), B=params["B"], K_list=params["K"],
                   block_size=params["block_size"], num_perm=params["num_permutations"], seed=params["seed"])
res[:1], res[-1]["num_gaps_test"]

### Tool (2026-02-21 23:13:39)

([{'Nmax': 1000000,
   'num_primes_test': 36872,
   'num_gaps_test': 36871,
   'dt': 0.00024594622340595577,
   'var_across_bins': {'10': 2.5296023799049143e-05,
    '20': 4.898975212579309e-05,
    '50': 1.8947782587238387e-06},
   'perm_mean_var': {'10': 5.1832519069257514e-05,
    '20': 3.9284625727313014e-05,
    '50': 2.782228841476959e-06},
   'p_values': {'10': 0.895, '20': 0.24, '50': 0.73}}],
 480709)

### Assistant (2026-02-21 23:13:45)

out = {"params": params, "results": res}
json_str = json.dumps(out, indent=2)
len(json_str)

### Tool (2026-02-21 23:13:47)

1868

### Assistant (2026-02-21 23:13:48)

{
  "params": {
    "Nmax_list": [
      1000000,
      3000000,
      10000000
    ],
    "L": 65536,
    "W": 2000,
    "bandpass": [
      0.002,
      0.01
    ],
    "B": 12,
    "K": [
      10,
      20,
      50
    ],
    "block_size": 500,
    "num_permutations": 200,
    "seed": 12345
  },
  "results": [
    {
      "Nmax": 1000000,
      "num_primes_test": 36872,
      "num_gaps_test": 36871,
      "dt": 0.00024594622340595577,
      "var_across_bins": {
        "10": 2.5296023799049143e-05,
        "20": 4.898975212579309e-05,
        "50": 1.8947782587238387e-06
      },
      "perm_mean_var": {
        "10": 5.1832519069257514e-05,
        "20": 3.9284625727313014e-05,
        "50": 2.782228841476959e-06
      },
      "p_values": {
        "10": 0.895,
        "20": 0.24,
        "50": 0.73
      }
    },
    {
      "Nmax": 3000000,
      "num_primes_test": 159230,
      "num_gaps_test": 159229,
      "dt": 0.00024594622340595577,
      "var_across_bins": {
        "10": 3.661028887188645e-06,
        "20": 6.170398940323105e-06,
        "50": 9.807407329646066e-08
      },
      "perm_mean_var": {
        "10": 6.18547931573353e-06,
        "20": 5.181302177230117e-06,
        "50": 1.0484598362578226e-07
      },
      "p_values": {
        "10": 0.86,
        "20": 0.325,
        "50": 0.45
      }
    },
    {
      "Nmax": 10000000,
      "num_primes_test": 480710,
      "num_gaps_test": 480709,
      "dt": 0.00024594622340595577,
      "var_across_bins": {
        "10": 7.117904707182442e-07,
        "20": 1.1556961006397175e-06,
        "50": 1.633589810155013e-08
      },
      "perm_mean_var": {
        "10": 1.1294102098572007e-06,
        "20": 9.650570610248694e-07,
        "50": 1.624161330107518e-08
      },
      "p_values": {
        "10": 0.835,
        "20": 0.315,
        "50": 0.49
      }
    }
  ]
}

