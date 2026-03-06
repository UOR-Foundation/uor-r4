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
`INC-0052` 4-seed amortization confirm means:
- `DENSE_Q24`: `mse=0.004321788`, `amortized=0.5051s`
- `HOPF_RET_P1_Q24`: `0.004324992`, `amortized=0.5938s`, `cand_frac=0.3511`
- `DENSE_Q32`: `mse=0.004321788`, `amortized=0.5586s`
- `HOPF_RET_P1_Q32`: `0.004324992`, `amortized=0.6544s`, `cand_frac=0.3511`

## Why The Queue Changed
The first translation path is now timing-decomposed and confirmed as non-promotable operationally.
The next question is:
- whether the next meaningful gain requires dynamic geometry rather than more translated-systems tuning
- whether `H^4 + T_xH^4` or `H^4 x H^4` is the right next object

## Resume Rule
Default resume path is:
1. read `docs/research/increments/INC_0052_retrieval_amortization_confirm.md`
2. read `results/analysis/inc0052_retrieval_amortization_confirm.json`
3. read `docs/governance/gates/gate_20260306_115931.md`
4. read `docs/research/increments/INC_0050_dynamic_h4_state.md`
5. reopen the next dynamic geometry branch
