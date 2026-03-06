# Current Research Handoff

## Latest Closed Increment
- `INC-0054` screen is complete and closed.
- Evidence:
  - `configs/proxy_transfer_inc0054_tangent_flow_route_law_screen.json`
  - `results/analysis/inc0054_tangent_flow_route_law_screen.json`
  - `docs/governance/gates/gate_20260306_124322.md`
- Reading:
  - static Hopf bucket keys cut candidate fraction to about `0.34` with zero fallback
  - tangent flow partially repaired same-bucket quality loss, but not enough to beat the global baseline on MSE
  - product `H^4 x H^4` stayed strongest on top-1 under bucketed retrieval
- the next live branch is `INC-0055`, not a confirm on `INC-0054`

## Current In-Progress Increment
- `INC-0055` is the active next branch.
- Setup artifacts:
  - `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `tasks/dynamic_h4_state_eval.py`
- Branch reading to preserve:
  - `TXH4_W050` is still the tangent Slice A main-objective winner
  - `H4XH4_W025` is still the product branch reference because it improves top-1
  - `INC-0054` says the product branch is a better next retrieval/discrete-key branch than the tangent branch
- Next preferred work:
  - `INC-0055` product `H^4 x H^4` retrieval-field pilot
  - use the second `H^4` as a retrieval / imaginary field candidate
  - explicitly test whether route keys want discrete complex storage in that field

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0054_tangent_flow_route_law.md`
- Active next increment:
  - `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
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
`INC-0054` corrected 2-seed screen means:
- `STATIC_GLOBAL`: `mse=0.004315002`, `top1=0.02400`, `total=7.674s`, `cand_frac=1.0000`
- `STATIC_BUCKET`: `0.004329264`, `0.02317`, `7.196s`, `0.3408`
- `TXH4_BUCKET_W050`: `0.004320435`, `0.02717`, `7.275s`, `0.3408`
- `H4XH4_BUCKET_W025`: `0.004318685`, `0.03300`, `8.855s`, `0.3408`

## Why The Queue Changed
The tangent-flow route-law pilot did not beat the global dynamic baseline on MSE.
It did prove that:
- same-bucket Hopf retrieval is a real locality structure
- the product branch keeps the strongest top-1 under routed locality
The next question is:
- whether the second `H^4` should be treated as a retrieval / imaginary field
- whether route keys should be stored discretely in a complex field attached to that second factor

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0054_tangent_flow_route_law.md`
2. read `results/analysis/inc0054_tangent_flow_route_law_screen.json`
3. read `docs/governance/gates/gate_20260306_124322.md`
4. read `docs/research/increments/INC_0055_product_h4x4_retrieval_field.md`
5. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
6. read `docs/research/LEARNED_KNOWLEDGE.md`
7. resume with `INC-0055`
