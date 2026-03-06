# Current Research Handoff

## Latest Closed Increment
- `INC-0052` confirm is complete.
- Evidence:
  - `configs/proxy_transfer_inc0052_retrieval_amortization_confirm.json`
  - `results/analysis/inc0052_retrieval_amortization_confirm.json`
  - `docs/governance/gates/gate_20260306_115931.md`
- Reading:
  - the amortized translated retrieval crossover did not survive 4-seed confirm
  - plain Hopf still preserves useful pruning signal under translation
- the next live branch is deeper dynamic geometry, not more translated packaging

## Current In-Progress Increment
- `INC-0050` remains active, but Slice A is complete.
- Slice A artifacts:
  - `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
  - `docs/research/LEARNED_KNOWLEDGE.md`
  - `tasks/dynamic_h4_state_eval.py`
  - screen config: `configs/proxy_transfer_inc0050_dynamic_h4_screen.json`
  - confirm config: `configs/proxy_transfer_inc0050_dynamic_h4_confirm.json`
  - screen analysis: `results/analysis/inc0050_dynamic_h4_screen.json`
  - confirm analysis: `results/analysis/inc0050_dynamic_h4_confirm.json`
  - gate note: `docs/governance/gates/gate_20260306_122733.md`
- Slice A result:
  - `TXH4_W050` wins on proxy MSE and runtime over static `H^4`
  - `H4XH4_W025` stays alive because it improves top-1 more strongly than the static branch
- Next preferred work:
  - `INC-0054` tangent-flow route law pilot
  - `INC-0055` product `H^4 x H^4` retrieval-field pilot

## Exact Current State
- Latest closed increment:
  - `docs/research/increments/INC_0052_retrieval_amortization_confirm.md`
- Active next increment:
  - `docs/research/increments/INC_0050_dynamic_h4_state.md`
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
`INC-0050` Slice A 4-seed confirm means:
- `STATIC_H4`: `mse=0.004314443`, `top1=0.02758`, `total=8.569s`
- `TXH4_W050`: `0.004303599`, `0.03200`, `8.458s`
- `H4XH4_W025`: `0.004305430`, `0.03767`, `8.454s`

## Why The Queue Changed
The first dynamic-state confirm is now positive enough to split the branch.
The next question is:
- how to turn the tangent-flow win into a real route-law or retrieval-law pilot
- whether the product branch is better treated as a retrieval/discrete-decision field than a main proxy-MSE branch

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0052_retrieval_amortization_confirm.md`
2. read `results/analysis/inc0052_retrieval_amortization_confirm.json`
3. read `docs/governance/gates/gate_20260306_115931.md`
4. read `docs/research/increments/INC_0050_dynamic_h4_state.md`
5. read `docs/research/MATH_REVIEW_DYNAMIC_H4_STATE_20260306.md`
6. read `docs/research/LEARNED_KNOWLEDGE.md`
7. resume with `INC-0054` first unless the user explicitly wants to prioritize the product branch
