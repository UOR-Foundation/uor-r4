# Current Research Handoff

## Latest Closed Increment
- `INC-0058` is complete and closed negative.
- Evidence:
  - `configs/proxy_transfer_inc0058_product_complex_rerank_screen.json`
  - `results/analysis/inc0058_product_complex_rerank_screen.json`
  - `docs/governance/gates/gate_20260306_140424.md`
- Reading:
  - broad and local translated repairs have both now failed
  - translated complex addressing remains positive as an efficiency signal
  - the next live branch should elevate the second `H^4` into a coupled geometric role, not another local retrieval patch
- the next live branch is coupled `H^4 x H^4` polar flow

## Current In-Progress Increment
- `INC-0059` is the active next branch.
- Setup artifacts:
  - `docs/research/increments/INC_0059_h4x4_polar_flow.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
- Branch reading to preserve:
  - `TXH4_W050` is still the tangent Slice A main-objective winner
  - `H4XH4_BUCKET_W025` is the product quality/top-1 reference
  - `H4XH4_CPX13_W025` is the product complex-key efficiency reference
  - `HOPF_RET_CPX_P1_Q24` is the translated complex-key efficiency reference
- Next preferred work:
  - keep the complex key as evidence, not as the final form
  - couple the two `H^4` factors more directly
  - let the second factor act as a real flow / retrieval manifold
  - prefer a geometrically meaningful coupled score or diagnostic over more local retrieval heuristics

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0057_product_complex_backfill.md`
- Active next increment:
  - `docs/research/increments/INC_0059_h4x4_polar_flow.md`
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
`INC-0058` 2-seed rerank screen means:
- `HOPF_RET_CPX_P1_Q24`: `mse=0.00432337`, `top1=0.04767`, `total=13.074s`, `amortized=0.5233s`, `cand_frac=0.20754`
- `HOPF_RET_CPX_R025_P1_Q24`: `0.00432341`, `0.04767`, `13.402s`, `0.5347s`, `cand_frac=0.20754`
- `HOPF_RET_CPX_R050_P1_Q24`: `0.00432431`, `0.04750`, `12.599s`, `0.5028s`, `cand_frac=0.20754`
- `HOPF_RET_CPX_R075_P1_Q24`: `0.00432388`, `0.04783`, `17.449s`, `0.7014s`, `cand_frac=0.20754`

`INC-0058` killed branch notes:
- keeping candidate fraction fixed was necessary but not sufficient
- the simple complex-plane rerank did not rescue top-1 cleanly
- translated local repair looks exhausted enough to justify returning to the deeper coupled-geometry branch

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
6. read `docs/research/increments/INC_0058_product_complex_rerank.md`
7. read `results/analysis/inc0058_product_complex_rerank_screen.json`
8. read `docs/governance/gates/gate_20260306_140424.md`
9. read `docs/research/increments/INC_0059_h4x4_polar_flow.md`
10. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
11. read `docs/research/LEARNED_KNOWLEDGE.md`
12. resume with `INC-0059`
