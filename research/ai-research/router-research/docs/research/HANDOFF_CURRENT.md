# Current Research Handoff

## Latest Closed Increment
- `INC-0056` confirm is complete and closed.
- Evidence:
  - `configs/proxy_transfer_inc0056_product_complex_translation_confirm.json`
  - `results/analysis/inc0056_product_complex_translation_confirm.json`
  - `docs/governance/gates/gate_20260306_131507.md`
- Reading:
  - translated complex route keys reduce candidate fraction materially with zero fallback
  - online and amortized translated retrieval cost improve versus plain Hopf translated retrieval
  - MSE improves slightly, but top-1 drops slightly
  - the result is positive for translated discrete-key efficiency, not yet a complete recall solution
- the next live branch is hierarchical complex backfill

## Current In-Progress Increment
- `INC-0057` is the active next branch.
- Setup artifacts:
  - `docs/research/increments/INC_0057_product_complex_backfill.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `tasks/router_retrieval_eval.py`
  - `configs/proxy_transfer_inc0057_product_complex_backfill_screen.json`
  - `configs/proxy_transfer_inc0057_product_complex_backfill_screen_v2.json`
- Branch reading to preserve:
  - `TXH4_W050` is still the tangent Slice A main-objective winner
  - `H4XH4_BUCKET_W025` is the product quality/top-1 reference
  - `H4XH4_CPX13_W025` is the product complex-key efficiency reference
  - `HOPF_RET_CPX_P1_Q24` is the translated complex-key efficiency reference
- Next preferred work:
  - keep the complex key as the primary discrete address field
  - add a small coarse-bucket backfill path to recover top-1
  - preserve the pruning gain while testing whether recall can be repaired cheaply
  - prefer selective or cached backfill over naive global coarse augmentation

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0056_product_complex_translation.md`
- Active next increment:
  - `docs/research/increments/INC_0057_product_complex_backfill.md`
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
`INC-0056` 4-seed confirm means:
- `DENSE_Q24`: `mse=0.004321788`, `top1=0.04867`, `total=16.113s`, `amortized=0.6573s`
- `HOPF_RET_P1_Q24`: `0.004324992`, `0.04683`, `16.679s`, `0.6685s`, `cand_frac=0.3511`
- `HOPF_RET_CPX_P1_Q24`: `0.004324266`, `0.04592`, `15.447s`, `0.6129s`, `cand_frac=0.2095`, `fallback=0.0000`

`INC-0057` current partial state:
- implemented `complex_backfill_items` in `tasks/router_retrieval_eval.py`
- added unit coverage proving backfill can recover a coarse neighbor in a controlled case
- first screen attempt showed a pathologically slow naive implementation because coarse extras were recomputed per query
- optimized that path by precomputing coarse extra pools per composite key
- reran screen under `configs/proxy_transfer_inc0057_product_complex_backfill_screen_v2.json`
- live observation before interrupt:
  - `BF4` remained materially heavier than exact complex addressing even after the optimization
  - no summary artifact was emitted yet, so `RR-057` should be treated as in-progress, not concluded

## Why The Queue Changed
The complex key law now has a translated efficiency signal of its own.
The next question is:
- whether a small coarse-bucket backfill can recover top-1 without undoing the pruning gain
- whether the discrete key should remain a pure address field or become a hierarchical address-plus-recall field

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
2. read `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
3. read `docs/governance/gates/gate_20260306_125455.md`
4. read `docs/research/increments/INC_0056_product_complex_translation.md`
5. read `results/analysis/inc0056_product_complex_translation_confirm.json`
6. read `docs/governance/gates/gate_20260306_131507.md`
7. read `docs/research/increments/INC_0057_product_complex_backfill.md`
8. inspect `tasks/router_retrieval_eval.py` for `complex_backfill_items`
9. inspect `configs/proxy_transfer_inc0057_product_complex_backfill_screen_v2.json`
10. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
11. read `docs/research/LEARNED_KNOWLEDGE.md`
12. resume with `INC-0057`
