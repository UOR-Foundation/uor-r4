# Current Research Handoff

## Latest Closed Increment
- `INC-0055` confirm is complete and closed.
- Evidence:
  - `configs/proxy_transfer_inc0055_product_h4x4_retrieval_field_confirm.json`
  - `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
  - `docs/governance/gates/gate_20260306_125455.md`
- Reading:
  - plain product bucket remains the quality/top-1 reference
  - product complex route keys reduce candidate fraction and runtime with low fallback
  - the result is positive for retrieval/discrete-key efficiency, not for main MSE
- the next live branch is translation/integration of the product complex-key law

## Current In-Progress Increment
- `INC-0056` is the active next branch.
- Setup artifacts:
  - `docs/research/increments/INC_0056_product_complex_translation.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `tasks/dynamic_h4_state_eval.py`
- Branch reading to preserve:
  - `TXH4_W050` is still the tangent Slice A main-objective winner
  - `H4XH4_BUCKET_W025` is the product quality/top-1 reference
  - `H4XH4_CPX13_W025` is the product complex-key efficiency reference
  - `INC-0055` says the complex-key law should be translated into the retrieval harness next
- Next preferred work:
  - translate the product complex-key law into the retrieval harness
  - use the second `H^4` as a retrieval / imaginary field candidate
  - keep discrete complex storage as the keying law under test

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
- Active next increment:
  - `docs/research/increments/INC_0056_product_complex_translation.md`
- Current transfer control baseline:
  - `R0`
- Current operational routed lead:
  - `HOPF_K25_BASE_IT40_P2_STATIC`
- Current hardware-efficiency routed lead:
  - `HOPF_PHI2_BAND_IT40_P2_STATIC`
- Current translated retrieval control:
  - matched `DENSE_Q24` / `DENSE_Q32`
- Current translated retrieval fast candidate:
  - `HOPF_RET_P1`
  - translated pruning-positive family, but not operationally promoted

## What Changed Most Recently
`INC-0055` 4-seed confirm means:
- `H4XH4_BUCKET_W025`: `mse=0.004318471`, `top1=0.03333`, `total=7.729s`, `cand_frac=0.3344`
- `H4XH4_CPX13_W025`: `0.004336934`, `0.03167`, `7.088s`, `0.2672`, `fallback=0.0070`
- `STATIC_BUCKET`: `0.004327840`, `0.02558`, `7.794s`, `0.3344`

## Why The Queue Changed
The product branch now has a positive efficiency signal of its own.
The next question is:
- whether the complex key law survives translation into the more model-like retrieval harness
- whether the discrete key should be treated as a pure address field or a mixed address-plus-ordering field

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
2. read `results/analysis/inc0055_product_h4x4_retrieval_field_confirm.json`
3. read `docs/governance/gates/gate_20260306_125455.md`
4. read `docs/research/increments/INC_0056_product_complex_translation.md`
5. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
6. read `docs/research/LEARNED_KNOWLEDGE.md`
7. resume with `INC-0056`
