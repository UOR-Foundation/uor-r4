# EVIDENCE_SUMMARY

This document tracks what has been empirically explored so far.

## 1. Routing Experiments

Initial routing experiments explored geometric compatibility routing
versus dense attention baselines.

Observed: - structured routing behavior emerges - compatibility kernels
produce sparse activation

Unknown: - long-range stability - scaling behavior

## 2. Embedding Stability

Experiments evaluated embedding stability within curved spaces.

Observed: - hierarchical datasets embed naturally in negatively curved
space - local neighborhoods remain stable

Risks: - training instability at larger scales

## 3. Sparse Event Routing

Threshold routing experiments tested sparse activation behavior.

Observed: - strong sparsity patterns possible - compute savings in
prototype systems

Unknown: - training convergence - gradient propagation stability

## 4. Phase Transport

Preliminary experiments suggest phase alignment may encode useful
relationships between routed states.

Status: - exploratory - not yet proven necessary

Prime admissibility transport note:

- the exact finite-depth admissibility state is currently interpreted as a
  layered torus-valued phase-fiber state
- a compressed quotient in `C^2` appears to capture a reusable transport
  backbone across wheel scales
- a shared canonical law trained on `W = 2310, 30030, 510510` generalized to
  unseen `W = 9699690` with only small degradation
- the current conservative framing is `z_{t+1} ≈ A_* z_t + epsilon_t`, where
  `A_*` is the reusable backbone and `epsilon_t` is unresolved local residual
  structure
- this is evidence for reusable compressed transport structure, not evidence of
  exact closure or a prime oracle

## 5. Hardware Implications

Prototype results suggest possible compute savings if sparse routing
scales.

However: - routing overhead must remain small - memory locality must be
preserved

------------------------------------------------------------------------

Current evidence is promising but insufficient to confirm the central
hypothesis.
