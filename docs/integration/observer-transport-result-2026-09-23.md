# Transported-state recovery and independently weighted lexical residual, September 23, 2026

## Result
The claim that `prev` is absent from the retained state is contradicted by a coordinate-aware observer. On the legacy restricted 4,228-position population, training-free centered decoding recovers `prev` at **75.90%** in `olx-form-2` (original observer **2.06%**) and **59.22%** in `olx-form-4` (original **1.87%**). No model weights changed. The observer receives the frozen state and known causal event phase; labels enter only evaluation or explicitly named oracle comparisons.

The mechanism is to decode `cur`, subtract its embedding and known event/fact offset, then compare the residual with candidate embeddings transported through the actual artifact recurrent map, including the previous-event offset. This does not assume an inverse, orthogonality, or a geometric group action. The exact state SHA matches the original probe. Both final artifact decodes reproduce the first-run results exactly.

## Matched predictive comparison
All losses use the same 5,376 targets, 24 documents, and original artifact Stop probability. Copy remains illegal.

| Offline predictor | Bits per target |
| --- | ---: |
| Original artifact | 6.684104957 |
| Centered decoded context addressing retained counts | 5.365482250 |
| Exact causal context addressing retained counts | 5.121723622 |
| Original convex artifact/count blend | 5.096542701 |
| Calibrated counts, exponent 1.10 | 5.079035181 |
| Independent residual: count exponent 1.00, artifact strength 0.15 | **5.041423387** |

The centered decoded-context readout improves all 24 documents, but adds fitted count-table capacity; this is not an equal-byte comparison or a trained successor. The independent residual improves 19/24 documents versus calibrated counts: **-0.037611793 bits/target**, paired-document 10,000-draw interval **[-0.05725, -0.01646]**. An independent server-side 50,000-draw interval is **[-0.05710, -0.01659]**. The winning configuration and full loss report reproduce exactly in a fresh process.

## Selection and controls
The original convex blend `A^(1-beta) C^beta` selected beta 0.90 on 8,512 tune positions. A stronger count-temperature baseline selected exponent 1.10 and had a better development point estimate. The follow-up therefore decoupled the two strengths: `C^alpha A^gamma`, normalized over Generate, with alpha in `{0.9,1.0,1.1,1.2,1.3}` and gamma in `{0,0.05,...,1}`. The global minimum across all 105 pairs was selected from tune loss alone, yielding `(1.0,0.15)`. The calibrated count endpoint reproduces the independent temperature run. Original-scorer parity is within 3.56e-15 bits per position.

## Interpretation and limits
The retained readout contributes beyond the tested calibrated-count baseline when its weight is independent of the prior exponent. This does not isolate long-range recurrence from local features or token-specific calibration. The proposed next served design is a strong addressed local prior plus a separately weighted learned correction, preserving exact ownership and the Generate/Copy/Stop contract.

All experiments are offline floating-point diagnostics on repeatedly inspected development data. Tune-only coefficient selection does not remove adaptive study-design effects. Bootstrap intervals condition on these 24 documents. No fresh-source generalization, useful free generation, geometric advantage, equal-byte/latency gain, energy advantage or complete served D0-b qualification is established. The transported map is the retained ternary recurrence, not evidence that H4 or zeta phases caused the gain. No successor model artifact was promoted.

## Source, evidence and execution
Base: `34bbe2ae952763a0431b5b9dba3e0b2ead098a52`. Primary diagnostics: `be40da9be6d047957212792084a7651d77c2e56e`. Residual diagnostics: `0c1b715b9cdd75ce031d8ba8f21019bec2b1ac8d`.

Evidence: [primary summary](../evidence/observer-transport-summary-2026-09-23.json) and [residual summary](../evidence/observer-residual-summary-2026-09-23.json). Full sealed roots, binaries, per-position records, projections and replay logs remain in `/Users/casey.allard/uor-r4-investigations/observer-transport-20260923`. The final focused Rust suite passes 14/14 tests. Release builds, formatting and diff checks pass; this is not a full-repository-suite claim. The underlying model implementation and both frozen artifacts are unchanged.

All 83 main commit bodies since September 18, 2026, 18:52:45 UTC were read; critical source and changed-file history were inspected. Initial all-ref titles numbered 293 and include duplicated branch history. Full patches were retained, not exhaustively read line by line. Work was isolated from the owner's checkout. Code has not been pushed or merged.

## Independent next workstreams
- Native integer prior-plus-correction readout with byte/latency and legal-action parity checks.
- Transport-aware state access across depth and event/context interventions.
- Matched independent-coefficient controls for unigram, cur-only and reordered-state alternatives.
- Warm-started readout and genuine optimizer/schedule tests under matched grounded-retention budgets.
- Fresh-source generated text, grounded sessions and geometric/non-geometric comparisons after design freeze.
