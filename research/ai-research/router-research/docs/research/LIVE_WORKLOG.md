# Live Worklog

## Latest Update
- Completed `INC-0055` product retrieval-field confirm.
- Result:
  - discrete complex route-key storage in the second `H^4` is evidence-positive
  - candidate fraction dropped from `0.3344` to `0.2672`
  - total runtime dropped from `7.729s` to `7.088s`
  - fallback stayed low at `0.0070`
  - plain product bucket still kept the quality/top-1 lead
- Decision:
  - close `INC-0055` as a positive branch
  - move next to translation/integration of the product complex-key law
  - treat discrete complex/imaginary route-key storage as part of the live `H^4 x H^4` architecture

## Current State
- Latest closed increment: `INC-0055`.
- Next increment: `INC-0056`.
- Current transfer control baseline: `R0`.
- Current operational routed lead: `HOPF_K25_BASE_IT40_P2_STATIC`.
- Current hardware-efficiency routed lead: `HOPF_PHI2_BAND_IT40_P2_STATIC`.
- Current translated retrieval control: matched `DENSE_Q24` / `DENSE_Q32`.
- Current translated retrieval family reference: `HOPF_RET_P1`.

## Current Interpretation
- The cheap routed frontier was strong enough to translate.
- The first translated harness is live, coherent, and still useful as an evaluation path.
- The translated systems branch did not clear operational confirm.
- The next responsible move is product-complex translation, not more local key tuning.

## If Session Gets Cut
1. Read:
  - `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
  - `docs/research/increments/INC_0056_product_complex_translation.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `docs/research/HANDOFF_CURRENT.md`
2. Inspect:
  - `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
  - `docs/governance/gates/gate_20260306_125455.md`
3. Resume with `INC-0056` product complex translation.

## 2026-03-06 (research increment INC-0056)
- Implemented translated discrete complex route-key storage in `tasks/router_retrieval_eval.py`:
  - `route_key_mode=hopf_bucket|hopf_plus_complex`
  - `complex_key_roots`
  - `complex_key_radius_bins`
- Added translated retrieval metrics:
  - `retrieval_bucket_fallback_rate`
  - `retrieval_secondary_key_count`
- 4-seed confirm result:
  - `DENSE_Q24`: `mse=0.004321788`, `top1=0.04867`, `total=16.113s`, `amortized=0.6573s`
  - `HOPF_RET_P1_Q24`: `0.004324992`, `0.04683`, `16.679s`, `0.6685s`, `cand_frac=0.3511`
  - `HOPF_RET_CPX_P1_Q24`: `0.004324266`, `0.04592`, `15.447s`, `0.6129s`, `cand_frac=0.2095`, `fallback=0.0000`
- Reading:
  - the complex key survives translation and materially improves address efficiency
  - the branch slightly improves proxy MSE versus plain Hopf translated retrieval
  - the branch still pays a small top-1 penalty
- Decision:
  - close `INC-0056` as positive
  - move next to hierarchical complex-key backfill (`INC-0057`)

## 2026-03-06 (research increment INC-0057, partial)
- Implemented `complex_backfill_items` in `tasks/router_retrieval_eval.py`.
- Added translated unit coverage proving bounded coarse backfill can recover a coarse neighbor in a controlled case.
- First RR-057 screen attempt showed a pathologically slow naive implementation because the coarse extra pool was recomputed per query.
- Optimized the backfill path by precomputing coarse extra pools per composite key and reran the screen under a v2 config.
- Live observation before stopping the screen:
  - `BF4` remained materially heavier than exact complex addressing even after the optimization.
- Current reading:
  - the next recall-repair step likely needs selective or cached backfill rather than a broad fixed-size coarse augmentation.
