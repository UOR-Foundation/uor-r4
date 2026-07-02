# Phase-Lift Analysis

- **Conversation ID:** `699b4bc5-68f8-8326-80c0-6f3444f40e8a`
- **Date:** 2026-02-22

---

### User (2026-02-22 13:32:41)

You are a stateless compute worker.
PRINT JSON ONLY. No files.

Goal
Attribute phase-lift to subsets of zeros within the FAST band, and test whether lift is driven by
(a) a few strong modes, or (b) interference of many modes.

Files:
- chebyshev_psi.npz (x_grid, psi_minus_x)
- primes.npz (primes)
- zeros1.gz (first 100k zeros)

Canonical params:
L=65536
x_min=1e5, x_max=9e6
W=2000 causal detrend
train_frac=0.7
Delta=0.01
FAST=[0.002,0.01] cycles/sample
H=[25,100]
seed=12345

Steps (build once):
1) Build uniform log grid t length L; compute dt.
2) Restrict kept indices by x=exp(t) in [x_min,x_max]. Define TRAIN/TEST split (70/30).
3) Build real u_bp and theta_real (canonical pipeline).
4) Build R_i and desticky targets y_h(i)=1{R_{i+h}-R_i>0} aligned to feature indices.
5) Define phase-only predictor model: Fourier logistic K=3 on theta.

Zeta zeros mapping:
- gamma(f)=2π f / dt
- gamma_min=gamma(0.002), gamma_max=gamma(0.01)
- Load gamma_all from zeros1.gz and select band gammas: gamma_min <= gamma <= gamma_max.
Call this set G_band with size K_band. Report K_band.

Part A: Quartile / decile band-splits
1) Sort G_band by gamma.
2) Partition into M=5 contiguous gamma blocks with equal counts (quintiles).
For each block m:
   - reconstruct u_m(t) using ONLY gammas in that block (explicit formula term, same as before)
   - apply same detrend/standardize/bandpass => u_m_bp
   - Hilbert => theta_m
   - Evaluate phase-only predictiveness on TEST for each h in H (AUC,Brier,LogLoss).
Also compute similarity to real theta on TEST: R_theta.

Part B: Additive accumulation curve
Construct cumulative reconstructions:
  u_cum_j uses first j quintiles (j=1..5).
For each j:
  build theta_cum_j, compute predictiveness + R_theta on TEST.
This gives a “lift saturation curve” vs included gamma range.

Part C: Leave-one-block-out (LOBO)
For each quintile m:
  reconstruct using all band gammas EXCEPT block m.
Compute:
  similarity drop (delta corr of u_bp, delta R_theta)
  predictiveness drop (delta AUC vs full-band zero reconstruction)

Output JSON:
{
 "meta":{"dt":..., "gamma_min":..., "gamma_max":..., "K_band":..., "split":"70/30"},
 "real_phase_perf":{"25":{...},"100":{...}},
 "full_band_zero_perf":{"25":{...},"100":{...}},
 "quintiles":[
   {"m":1,"gamma_range":[...,...],"K":...,
    "theta_similarity":{"R_theta":...},
    "perf":{"25":{...},"100":{...}}
   }, ...],
 "cumulative":[
   {"j":1,"gamma_max_included":...,"K":...,"R_theta":...,"perf":{...}}, ...],
 "lobo":[
   {"m":1,"excluded_gamma_range":[...,...],
    "delta_similarity":{"delta_corr_u":...,"delta_R_theta":...},
    "delta_perf":{"25":{"delta_AUC":...},"100":{"delta_AUC":...}}
   }, ...],
 "conclusion":{
   "is_lift_localized_or_distributed":"localized/distributed/mixed",
   "brief_summary":"one paragraph factual"
 }
}

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

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

### Tool (2026-02-22 13:32:43)

All the files uploaded by the user have been fully loaded. Searching won't provide additional information.

### Assistant (2026-02-22 13:32:43)

import numpy as np, gzip, json, math, os, pandas as pd
from numpy.fft import rfft, irfft, rfftfreq
from sklearn.metrics import roc_auc_score, brier_score_loss, log_loss
from sklearn.linear_model import LogisticRegression

# Load data
psi_path="/mnt/data/chebyshev_psi.npz"
primes_path="/mnt/data/primes.npz"
zeros_path="/mnt/data/zeros1.gz"
psi=np.load(psi_path)
x_grid=psi["x_grid"]
psi_minus_x=psi["psi_minus_x"]
pr=np.load(primes_path)
primes=pr["primes"]

L=65536
x_min=1e5
x_max=9e6
W=2000
train_frac=0.7
Delta=0.01
FAST=(0.002,0.01)
H=[25,100]
seed=12345
np.random.seed(seed)

# Step 1: uniform log grid
t_raw=np.log(x_grid.astype(float))
t_min=float(t_raw.min())
t_max=float(t_raw.max())
t=np.linspace(t_min,t_max,L)
dt=float(t[1]-t[0])
# interpolate psi_minus_x onto t
u_interp=np.interp(t,t_raw,psi_minus_x.astype(float))
x=np.exp(t)
# Step 2: restrict kept
mask=(x>=x_min)&(x<=x_max)
t_k=t[mask]
x_k=x[mask]
u_raw=u_interp[mask]
N=len(t_k)
train_end=int(np.floor(train_frac*N))

def causal_moving_average(arr, W):
    # past-only including current; for i, mean of arr[max(0,i-W+1):i+1]
    c=np.cumsum(np.insert(arr,0,0.0))
    out=np.empty_like(arr)
    for i in range(len(arr)):
        j0=max(0,i-W+1)
        out[i]=(c[i+1]-c[j0])/(i-j0+1)
    return out

def bandpass_fft(sig, f_lo, f_hi):
    n=len(sig)
    freqs=rfftfreq(n, d=1.0)  # cycles/sample
    S=rfft(sig)
    mask=(freqs>=f_lo)&(freqs<=f_hi)
    S_f=np.where(mask, S, 0)
    return irfft(S_f, n)

def hilbert_phase(sig):
    # analytic signal via FFT method
    n=len(sig)
    X=np.fft.fft(sig)
    h=np.zeros(n)
    if n%2==0:
        h[0]=1; h[n//2]=1
        h[1:n//2]=2
    else:
        h[0]=1
        h[1:(n+1)//2]=2
    z=np.fft.ifft(X*h)
    theta=np.angle(z)  # [-pi,pi]
    # map to [0,2pi)
    theta=(theta%(2*np.pi))
    return theta, np.abs(z)

# Step 3: build real u_bp and theta_real
u_detr = u_raw - causal_moving_average(u_raw, W)
mu=float(u_detr[:train_end].mean())
sd=float(u_detr[:train_end].std(ddof=0))
u_std=(u_detr-mu)/(sd if sd>0 else 1.0)
u_bp=bandpass_fft(u_std, FAST[0], FAST[1])
theta_real, A_real = hilbert_phase(u_bp)

# Step 4: Build R_i and targets
# count primes in [x_i, x_i*exp(Delta))
expD=math.exp(Delta)
# primes is int32; convert to int64 for searchsorted
primes64=primes.astype(np.int64)
lo = x_k
hi = x_k*expD
# searchsorted needs sorted array
idx_lo=np.searchsorted(primes64, lo, side="left")
idx_hi=np.searchsorted(primes64, hi, side="left")
P=(idx_hi-idx_lo).astype(float)
E= x_k*(expD-1.0)/np.log(x_k)
R=(P-E)/np.sqrt(E)
# targets for each h: y=1{R_{i+h}-R_i>0} for i=0..N-h-1
def make_dataset(theta, h):
    n=N-h
    y=(R[h:]-R[:-h] > 0).astype(int)
    th=theta[:-h]
    # train/test split on i
    tr_end=int(np.floor(train_frac*n))
    tr_idx=np.arange(tr_end)
    te_idx=np.arange(tr_end,n)
    return th, y, tr_idx, te_idx

def fourier_features(theta, K=3):
    X=[np.ones_like(theta)]
    for k in range(1,K+1):
        X.append(np.cos(k*theta))
        X.append(np.sin(k*theta))
    return np.vstack(X).T

def fit_eval(theta, h):
    th,y,tr_idx,te_idx=make_dataset(theta,h)
    X=fourier_features(th,3)
    Xtr,Xte=X[tr_idx],X[te_idx]
    ytr,yte=y[tr_idx],y[te_idx]
    # logistic regression
    # Use strong regularization? set C=10, solver lbfgs
    clf=LogisticRegression(max_iter=2000, solver="lbfgs")
    clf.fit(Xtr,ytr)
    p=clf.predict_proba(Xte)[:,1]
    # metrics
    out={}
    out["pos_rate"]=float(yte.mean()) if len(yte)>0 else None
    # AUC requires both classes
    if len(np.unique(yte))==2:
        out["AUC"]=float(roc_auc_score(yte,p))
    else:
        out["AUC"]=None
    out["Brier"]=float(brier_score_loss(yte,p))
    # log_loss needs eps
    out["LogLoss"]=float(log_loss(yte,p,labels=[0,1]))
    return out

def circ_similarity(theta1, theta2, idx):
    # mean cos difference (circular correlation proxy)
    d = theta1[idx]-theta2[idx]
    return float(np.mean(np.cos(d)))

def corr(a,b):
    if a.std()==0 or b.std()==0: return 0.0
    return float(np.corrcoef(a,b)[0,1])

# Performance for real phase
real_perf={str(h):fit_eval(theta_real,h) for h in H}

# Zeta zeros band
with gzip.open(zeros_path,'rt') as f:
    txt=f.read()
# extract floats robustly
vals=[]
for tok in txt.replace(',',' ').split():
    try:
        vals.append(float(tok))
    except:
        pass
gamma_all=np.array(vals,dtype=float)
# gamma range for band
gamma_min=2*np.pi*FAST[0]/dt
gamma_max=2*np.pi*FAST[1]/dt
G_band=gamma_all[(gamma_all>=gamma_min)&(gamma_all<=gamma_max)]
G_band=np.sort(G_band)
K_band=int(G_band.size)

# reconstruct u from gammas
def reconstruct_from_gammas(gammas, t_vec):
    if len(gammas)==0:
        return np.zeros_like(t_vec)
    g=gammas.astype(float)
    w = 2.0/np.sqrt(0.25 + g*g)
    # compute cos(g t) matrix in chunks to avoid memory blow
    out=np.zeros_like(t_vec,dtype=float)
    # chunk over gammas
    chunk=256
    for i in range(0,len(g),chunk):
        gg=g[i:i+chunk][:,None]
        ww=w[i:i+chunk][:,None]
        out += np.sum(ww*np.cos(gg*t_vec[None,:]), axis=0)
    return out

def pipeline_theta_from_u(u_recon_raw):
    # apply same detrend/standardize/bandpass as real
    u_d = u_recon_raw - causal_moving_average(u_recon_raw, W)
    mu=float(u_d[:train_end].mean())
    sd=float(u_d[:train_end].std(ddof=0))
    u_s=(u_d-mu)/(sd if sd>0 else 1.0)
    u_bp_m=bandpass_fft(u_s, FAST[0], FAST[1])
    theta,_=hilbert_phase(u_bp_m)
    return u_bp_m, theta

# Full-band zero reconstruction
u_full_raw=reconstruct_from_gammas(G_band, t_k)
u_full_bp, theta_full = pipeline_theta_from_u(u_full_raw)
full_perf={str(h):fit_eval(theta_full,h) for h in H}

# similarity indices on TEST for phase arrays align to N; for similarity of theta use test indices for base N?
test_idx=np.arange(train_end,N)  # for full length
R_theta_full=circ_similarity(theta_full, theta_real, test_idx)
corr_u_full=corr(u_full_bp[test_idx], u_bp[test_idx])

# Quintiles
M=5
# split indices into 5 equal count blocks
q_bounds=[int(round(i*K_band/M)) for i in range(M+1)]
quintiles=[]
u_bp_list=[]
theta_list=[]
for m in range(M):
    a=q_bounds[m]; b=q_bounds[m+1]
    gam=G_band[a:b]
    u_m_raw=reconstruct_from_gammas(gam, t_k)
    u_m_bp, theta_m = pipeline_theta_from_u(u_m_raw)
    u_bp_list.append(u_m_bp); theta_list.append(theta_m)
    entry={
        "m": m+1,
        "gamma_range":[float(gam[0]) if len(gam)>0 else None, float(gam[-1]) if len(gam)>0 else None],
        "K": int(len(gam)),
        "theta_similarity":{"R_theta": circ_similarity(theta_m, theta_real, test_idx)},
        "perf": {str(h):fit_eval(theta_m,h) for h in H}
    }
    quintiles.append(entry)

# cumulative
cumulative=[]
for j in range(1,M+1):
    gam=G_band[:q_bounds[j]]
    u_raw_c=reconstruct_from_gammas(gam, t_k)
    u_bp_c, theta_c = pipeline_theta_from_u(u_raw_c)
    cumulative.append({
        "j": j,
        "gamma_max_included": float(gam[-1]) if len(gam)>0 else None,
        "K": int(len(gam)),
        "R_theta": circ_similarity(theta_c, theta_real, test_idx),
        "perf": {str(h):fit_eval(theta_c,h) for h in H}
    })

# LOBO
lobo=[]
for m in range(M):
    a=q_bounds[m]; b=q_bounds[m+1]
    gam_ex=G_band[a:b]
    gam_keep=np.concatenate([G_band[:a], G_band[b:]]) if (a>0 or b<K_band) else np.array([],dtype=float)
    u_raw_k=reconstruct_from_gammas(gam_keep, t_k)
    u_bp_k, theta_km = pipeline_theta_from_u(u_raw_k)
    delta_corr_u = corr(u_bp_k[test_idx], u_bp[test_idx]) - corr_u_full
    delta_R_theta = circ_similarity(theta_km, theta_real, test_idx) - R_theta_full
    # perf drop vs full
    perf_k={str(h):fit_eval(theta_km,h) for h in H}
    delta_perf={}
    for h in H:
        fullA=full_perf[str(h)]["AUC"]
        kA=perf_k[str(h)]["AUC"]
        dA=None if (fullA is None or kA is None) else float(kA-fullA)
        delta_perf[str(h)]={"delta_AUC": dA}
    lobo.append({
        "m": m+1,
        "excluded_gamma_range":[float(gam_ex[0]) if len(gam_ex)>0 else None, float(gam_ex[-1]) if len(gam_ex)>0 else None],
        "delta_similarity":{"delta_corr_u": float(delta_corr_u), "delta_R_theta": float(delta_R_theta)},
        "delta_perf": delta_perf
    })

# Decide localized/distributed based on how much performance drops when leaving out a block and saturation curve
# simple heuristic:
auc_full=np.nanmean([full_perf[str(h)]["AUC"] for h in H if full_perf[str(h)]["AUC"] is not None])
auc_quints=[np.nanmean([q["perf"][str(h)]["AUC"] for h in H if q["perf"][str(h)]["AUC"] is not None]) for q in quintiles]
best_quint=max(auc_quints)
# saturation: auc at j=1..5
auc_cum=[np.nanmean([c["perf"][str(h)]["AUC"] for h in H if c["perf"][str(h)]["AUC"] is not None]) for c in cumulative]
# lobo drops
lobo_drops=[np.nanmean([v["delta_perf"][str(h)]["delta_AUC"] for h in H if v["delta_perf"][str(h)]["delta_AUC"] is not None]) for v in lobo]
max_drop = -min(lobo_drops) if len(lobo_drops)>0 else 0.0  # positive if drop
# classify
if best_quint > auc_full - 0.01 and (auc_cum[0] > auc_full - 0.02):
    loc="localized"
elif max_drop < 0.01 and (auc_cum[0] < auc_full - 0.05 and auc_cum[-1] >= auc_full - 0.01):
    loc="distributed"
else:
    loc="mixed"

conclusion_summary = (
    f"Within the FAST gamma band [{gamma_min:.3f},{gamma_max:.3f}] (K_band={K_band}), "
    f"phase-predictive performance from the zeta-zero reconstruction is not dominated by a single narrow gamma block: "
    f"individual quintiles show weaker or variable AUC than the full-band reconstruction, while cumulative inclusion "
    f"generally improves as more gammas are added. LOBO drops are modest for most blocks, suggesting the lift is "
    f"primarily from multi-mode interference rather than a few isolated modes."
)

out={
 "meta":{"dt":dt,"gamma_min":float(gamma_min),"gamma_max":float(gamma_max),"K_band":K_band,"split":"70/30"},
 "real_phase_perf": real_perf,
 "full_band_zero_perf": full_perf,
 "quintiles": quintiles,
 "cumulative": cumulative,
 "lobo": lobo,
 "conclusion":{
     "is_lift_localized_or_distributed": loc,
     "brief_summary": conclusion_summary
 }
}
# also include similarity for full band? user asked in output only perf; but include? they asked similarity in lobo deltas etc. We'll add in full_band_zero_perf? not requested. We'll add extra keys inside full_band_zero_perf? no.
out["full_band_zero_similarity"]={"corr_u_bp_test": corr_u_full, "R_theta_test": R_theta_full}
out["real_phase_similarity_self"]={"corr_u_bp_test": 1.0, "R_theta_test": 1.0}

json_str=json.dumps(out, indent=None, separators=(",",":"))
len(json_str), json_str[:500]

### Tool (2026-02-22 13:34:48)

(6086,
 '{"meta":{"dt":0.00024594622340595577,"gamma_min":51.09397672521801,"gamma_max":255.46988362609005,"K_band":101,"split":"70/30"},"real_phase_perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6742001435966556,"Brier":0.22950556203714959,"LogLoss":0.6514790516818721},"100":{"pos_rate":0.5063198387983147,"AUC":0.7086426598004022,"Brier":0.21729161375214237,"LogLoss":0.6255255204388528}},"full_band_zero_perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6714092881448086,"Brier":0.22821240057573663,"')

### Assistant (2026-02-22 13:35:24)

out2={
 "meta":out["meta"],
 "real_phase_perf":out["real_phase_perf"],
 "full_band_zero_perf":out["full_band_zero_perf"],
 "quintiles":out["quintiles"],
 "cumulative":out["cumulative"],
 "lobo":out["lobo"],
 "conclusion":out["conclusion"]
}
json_str2=json.dumps(out2, separators=(",",":"))
json_str2[:400], len(json_str2)

### Tool (2026-02-22 13:35:32)

('{"meta":{"dt":0.00024594622340595577,"gamma_min":51.09397672521801,"gamma_max":255.46988362609005,"K_band":101,"split":"70/30"},"real_phase_perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6742001435966556,"Brier":0.22950556203714959,"LogLoss":0.6514790516818721},"100":{"pos_rate":0.5063198387983147,"AUC":0.7086426598004022,"Brier":0.21729161375214237,"LogLoss":0.6255255204388528}},"full_band_ze',
 5911)

### Assistant (2026-02-22 13:35:34)

{"meta":{"dt":0.00024594622340595577,"gamma_min":51.09397672521801,"gamma_max":255.46988362609005,"K_band":101,"split":"70/30"},"real_phase_perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6742001435966556,"Brier":0.22950556203714959,"LogLoss":0.6514790516818721},"100":{"pos_rate":0.5063198387983147,"AUC":0.7086426598004022,"Brier":0.21729161375214237,"LogLoss":0.6255255204388528}},"full_band_zero_perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6714092881448086,"Brier":0.22821240057573663,"LogLoss":0.6480168164255433},"100":{"pos_rate":0.5063198387983147,"AUC":0.6940081215476267,"Brier":0.22174498440272844,"LogLoss":0.637841063224005}},"quintiles":[{"m":1,"gamma_range":[52.97032147771446,91.20920084678595],"K":20,"theta_similarity":{"R_theta":-0.009527577569926349},"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5599726004795228,"Brier":0.2501054843107701,"LogLoss":0.6935667348931817},"100":{"pos_rate":0.5063198387983147,"AUC":0.5897909441337671,"Brier":0.24801209625376407,"LogLoss":0.689176271651294}}},{"m":2,"gamma_range":[92.013795067531,123.25111044876068],"K":20,"theta_similarity":{"R_theta":0.007869331599705096},"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5840394616390244,"Brier":0.24458824657991845,"LogLoss":0.681357561996589},"100":{"pos_rate":0.5063198387983147,"AUC":0.5965136554785385,"Brier":0.24372279979394644,"LogLoss":0.6794730511679372}}},{"m":3,"gamma_range":[125.42102382790763,154.6953957490072],"K":20,"theta_similarity":{"R_theta":0.00216764366669819},"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5658457488153578,"Brier":0.24918087621502727,"LogLoss":0.6915173550489452},"100":{"pos_rate":0.5063198387983147,"AUC":0.5791696652725453,"Brier":0.24692740962324433,"LogLoss":0.6867592373998298}}},{"m":4,"gamma_range":[157.64468021901364,204.87847180053775],"K":20,"theta_similarity":{"R_theta":0.004444823388623759},"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5696268834739292,"Brier":0.2468788499694998,"LogLoss":0.6864341381584453},"100":{"pos_rate":0.5063198387983147,"AUC":0.5919995512435989,"Brier":0.24377730303479317,"LogLoss":0.6795922700276701}}},{"m":5,"gamma_range":[207.5621485193664,254.90044129396044],"K":21,"theta_similarity":{"R_theta":0.00532400236068343},"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5361969005982003,"Brier":0.2515435431732806,"LogLoss":0.6983391416119519},"100":{"pos_rate":0.5063198387983147,"AUC":0.5636332752544726,"Brier":0.24983759700984982,"LogLoss":0.6945865517356293}}}],"cumulative":[{"j":1,"gamma_max_included":91.20920084678595,"K":20,"R_theta":-0.009527577569926349,"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.5599726004795228,"Brier":0.2501054843107701,"LogLoss":0.6935667348931817},"100":{"pos_rate":0.5063198387983147,"AUC":0.5897909441337671,"Brier":0.24801209625376407,"LogLoss":0.689176271651294}}},{"j":2,"gamma_max_included":123.25111044876068,"K":40,"R_theta":-0.003604199116962834,"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6152515638672724,"Brier":0.23908415212577974,"LogLoss":0.6692460024658176},"100":{"pos_rate":0.5063198387983147,"AUC":0.6344560995142363,"Brier":0.23591626646552722,"LogLoss":0.6623837748185383}}},{"j":3,"gamma_max_included":154.6953957490072,"K":60,"R_theta":-0.004950480912536388,"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6415406851140871,"Brier":0.2349304536709171,"LogLoss":0.660098263046212},"100":{"pos_rate":0.5063198387983147,"AUC":0.6656266891854409,"Brier":0.22775996247227343,"LogLoss":0.6449975842517696}}},{"j":4,"gamma_max_included":204.87847180053775,"K":80,"R_theta":0.0015443311047956784,"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6606923247009279,"Brier":0.2307119012982711,"LogLoss":0.6507816153131434},"100":{"pos_rate":0.5063198387983147,"AUC":0.6773125444747223,"Brier":0.22524982313319708,"LogLoss":0.639618602665377}}},{"j":5,"gamma_max_included":254.90044129396044,"K":101,"R_theta":0.006818157882620464,"perf":{"25":{"pos_rate":0.5076614374315943,"AUC":0.6714092881448086,"Brier":0.22821240057573663,"LogLoss":0.6480168164255433},"100":{"pos_rate":0.5063198387983147,"AUC":0.6940081215476267,"Brier":0.22174498440272844,"LogLoss":0.637841063224005}}}],"lobo":[{"m":1,"excluded_gamma_range":[52.97032147771446,91.20920084678595],"delta_similarity":{"delta_corr_u":0.1083692427344554,"delta_R_theta":0.015476308840714233},"delta_perf":{"25":{"delta_AUC":0.012120993115805261},"100":{"delta_AUC":0.012148845764148062}}},{"m":2,"excluded_gamma_range":[92.013795067531,123.25111044876068],"delta_similarity":{"delta_corr_u":-0.018346558507347507,"delta_R_theta":-0.0016033387033954256},"delta_perf":{"25":{"delta_AUC":-0.0035434812969195966},"100":{"delta_AUC":-0.0024023668057140577}}},{"m":3,"excluded_gamma_range":[125.42102382790763,154.6953957490072],"delta_similarity":{"delta_corr_u":-0.025087204202013214,"delta_R_theta":-0.0025546202207828506},"delta_perf":{"25":{"delta_AUC":-0.004013061186794773},"100":{"delta_AUC":-0.004758391284426438}}},{"m":4,"excluded_gamma_range":[157.64468021901364,204.87847180053775],"delta_similarity":{"delta_corr_u":-0.026418296989727178,"delta_R_theta":-0.0005015377383448327},"delta_perf":{"25":{"delta_AUC":0.002823190619049474},"100":{"delta_AUC":0.001683495199500475}}},{"m":5,"excluded_gamma_range":[207.5621485193664,254.90044129396044],"delta_similarity":{"delta_corr_u":-0.03844469178560937,"delta_R_theta":-0.004927819534001683},"delta_perf":{"25":{"delta_AUC":-0.005770636263662433},"100":{"delta_AUC":-0.00507122426555453}}}],"conclusion":{"is_lift_localized_or_distributed":"distributed","brief_summary":"Within the FAST gamma band [51.094,255.470] (K_band=101), phase-predictive performance from the zeta-zero reconstruction is not dominated by a single narrow gamma block: individual quintiles show weaker or variable AUC than the full-band reconstruction, while cumulative inclusion generally improves as more gammas are added. LOBO drops are modest for most blocks, suggesting the lift is primarily from multi-mode interference rather than a few isolated modes."}}

