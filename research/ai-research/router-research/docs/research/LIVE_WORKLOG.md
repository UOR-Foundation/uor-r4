# Live Worklog

## Latest Update
Completed `INC-0058` translated complex rerank screen.
Result:
  - exact-bucket complex-plane reranking kept candidate fraction fixed
  - small lambda reranks were effectively neutral or slightly regressive
  - only the largest lambda nudged top-1 upward, but at unacceptable cost
  - exact translated complex addressing remains the live efficiency reference
Decision:
  - close `INC-0058` negative
  - move next to coupled `H^4 x H^4` polar flow instead of more translated local repair
  - keep discrete complex / imaginary routing as part of the live `H^4 x H^4` architecture

## Current State
- Latest closed increment: `INC-0058`.
- Next increment: `INC-0059`.
- Current transfer control baseline: `R0`.
- Current operational routed lead: `HOPF_K25_BASE_IT40_P2_STATIC`.
- Current hardware-efficiency routed lead: `HOPF_PHI2_BAND_IT40_P2_STATIC`.
- Current translated retrieval control: matched `DENSE_Q24` / `DENSE_Q32`.
- Current translated retrieval family reference: `HOPF_RET_CPX_P1_Q24`.

## Current Interpretation
- The cheap routed frontier was strong enough to translate.
- The first translated harness is live, coherent, and still useful as an evaluation path.
- The translated systems branch did not clear operational confirm.
- The next responsible move is coupled product geometry, not more translated local repair.

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

## 2026-03-06 (research increment INC-0057)
- Implemented `complex_backfill_items` in `tasks/router_retrieval_eval.py`.
- Added translated unit coverage proving bounded coarse backfill can recover a coarse neighbor in a controlled case.
- First RR-057 screen attempt showed a pathologically slow naive implementation because the coarse extra pool was recomputed per query.
- Optimized the backfill path by precomputing coarse extra pools per composite key and reran the screen under a v2 config.
- Added selective/gated backfill controls:
  - `complex_backfill_mode`
  - `complex_backfill_max_exact`
  - `complex_backfill_margin_threshold`
- Added translated metrics:
  - `retrieval_backfill_trigger_rate`
  - `retrieval_backfill_extra_candidates_mean`
- Completed the selective small-bucket screen:
  - exact complex addressing stayed the efficiency reference
  - `SB1/SB2` backfill triggered at about `0.0005-0.0008`
  - top-1 stayed unchanged at `0.04767`
  - candidate fraction stayed unchanged at about `0.20754`
- Killed low-margin backfill after live seed-0 evidence:
  - `trigger=0.7170`
  - amortized cost exploded
- Current reading:
  - translated recall repair should be treated as an in-bucket ordering problem, not a candidate-expansion problem

## 2026-03-06 (research increment INC-0058)
- Implemented no-expansion translated rerank controls in `tasks/router_retrieval_eval.py`:
  - `complex_rerank_mode`
  - `complex_rerank_lambda`
- Added direct unit coverage proving the complex-plane rerank can flip an in-bucket ordering without changing candidate count.
- Ran `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`.
- 2-seed screen result:
  - `HOPF_RET_CPX_P1_Q24`: `top1=0.04767`, `mse=0.00432337`, `amortized=0.5233s`
  - `R025`: `0.04767`, `0.00432341`, `0.5347s`
  - `R050`: `0.04750`, `0.00432431`, `0.5028s`
  - `R075`: `0.04783`, `0.00432388`, `0.7014s`
- Reading:
  - the obvious rerank law is too weak at useful cost points
  - the second `H^4` remains important, but it likely needs a more coupled geometric role
