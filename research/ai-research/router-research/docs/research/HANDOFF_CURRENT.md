# Current Research Handoff

## Latest Closed Increment
- `INC-0057` is complete and closed negative.
- Evidence:
  - `configs/proxy_transfer_inc0057_product_complex_backfill_smallbucket_screen.json`
  - `results/analysis/inc0057_product_complex_backfill_smallbucket_screen.json`
  - `docs/governance/gates/gate_20260306_135217.md`
- Reading:
  - broad and margin-triggered backfill are the wrong recall repair path
  - small-bucket backfill is cheap but almost inert
  - translated complex addressing remains positive
  - the remaining problem should be treated as local ranking, not candidate coverage
- the next live branch is exact-bucket reranking

## Current In-Progress Increment
- `INC-0058` is the active next branch.
- Setup artifacts:
  - `docs/research/increments/INC_0058_product_complex_rerank.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `tasks/router_retrieval_eval.py`
- Branch reading to preserve:
  - `TXH4_W050` is still the tangent Slice A main-objective winner
  - `H4XH4_BUCKET_W025` is the product quality/top-1 reference
  - `H4XH4_CPX13_W025` is the product complex-key efficiency reference
  - `HOPF_RET_CPX_P1_Q24` is the translated complex-key efficiency reference
- Next preferred work:
  - keep the complex key as the primary discrete address field
  - keep the candidate set fixed
  - test whether recall can be repaired cheaply by reranking inside the exact complex bucket
  - use the second `H^4` complex / imaginary field as a local ordering signal, not as a candidate-expansion trigger

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0057_product_complex_backfill.md`
- Active next increment:
  - `docs/research/increments/INC_0058_product_complex_rerank.md`
- Current transfer control baseline:
  - `R0`
- Current operational routed lead:
  - `HOPF_K25_BASE_IT40_P2_STATIC`
- Current hardware-efficiency routed lead:
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`
- Current translated retrieval control:
  - matched `DENSE_Q24` / `DENSE_Q32`
- Current translated retrieval efficiency lead:
  - `HOPF_RET_CPX_P1_Q24`
  - translated complex-key branch with lower total and amortized cost than both plain Hopf translated retrieval and dense exact retrieval in confirm

## What Changed Most Recently
`INC-0057` 2-seed small-bucket screen means:
- `HOPF_RET_CPX_P1_Q24`: `mse=0.00432337`, `top1=0.04767`, `total=14.123s`, `amortized=0.5652s`, `cand_frac=0.20754`
- `HOPF_RET_CPX_SB1_BF2_P1_Q24`: `0.00432294`, `0.04767`, `13.767s`, `0.5482s`, `cand_frac=0.20754`, `trigger=0.0005`
- `HOPF_RET_CPX_SB2_BF2_P1_Q24`: `0.00432261`, `0.04767`, `13.245s`, `0.5261s`, `cand_frac=0.20754`, `trigger=0.0008`

`INC-0057` killed branch notes:
- low-margin backfill over-triggered (`trigger=0.7170` on `M002` seed 0) and exploded amortized cost
- small-bucket backfill stayed almost completely inactive and did not improve top-1
- translated complex recall should be treated as an ordering problem, not a candidate-expansion problem

## Why The Queue Changed
The complex key law now has a translated efficiency signal of its own.
The next question is:
- whether a small coarse-bucket backfill can recover top-1 without undoing the pruning gain
- whether the discrete key should remain a pure address field or become a hierarchical address-plus-recall field

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0056_product_complex_translation.md`
2. read `results/analysis/inc0056_product_complex_translation_confirm.json`
3. read `docs/governance/gates/gate_20260306_131507.md`
4. read `docs/research/increments/INC_0057_product_complex_backfill.md`
5. read `results/analysis/inc0057_product_complex_backfill_smallbucket_screen.json`
6. read `docs/governance/gates/gate_20260306_135217.md`
7. read `docs/research/increments/INC_0058_product_complex_rerank.md`
8. inspect `tasks/router_retrieval_eval.py` for `complex_backfill_mode` and the translated retrieval scoring path
9. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
10. read `docs/research/LEARNED_KNOWLEDGE.md`
11. resume with `INC-0058`
