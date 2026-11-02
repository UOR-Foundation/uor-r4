# ORDER: CPU counterfactual parameter groups, deepseek-2

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `728cf3f6c` from fresh `origin/main`.

## Goal

Extend the CPU shared-vector trainer beyond the tied token table with deterministic exact counterfactual updates for normalization gains, gate/convolution biases and route tables, using the full patched forward as the acceptance oracle.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`, `Cargo.lock`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-2/**`
- `docs/research/notebook/cpu-training.md`

No serving-source edit, corpus run, GPU run, language claim or revival of a rejected objective.

## Cancel probe

Stop a parameter group if its artifact table cannot be changed within the frozen format or if the exact forward cannot expose an improvement/rejection decision. All three table classes are length-addressed public UORLUT sections, so the unit stands. Invalid encoded candidates are skipped or refused; they are never approximated.

## Done when

- A deterministic candidate API patches exactly one named artifact table entry at a time and names the table, index and delta.
- Normalization updates move grid codes only to artifact-valid values; bias, decay and route updates use explicit integer/frame deltas.
- Every candidate reruns the sample through the full changed forward and is accepted only on strict exact-margin improvement with target admission preserved.
- Route candidates preserve exact record/token semantics and are limited to legal state/atom changes.
- The trainer records loss, admission, top-1, top-8, rank, candidate count, accepted move and patched artifact identity.
- A versioned checkpoint binds artifact bytes, group, samples, clocks, cursor and order; deterministic resume reproduces checkpoint bytes and next-step results.
- Finite tests cover accepted and rejected candidates for normalization, bias/decay and route groups, malformed tables, no-op margins, admission loss and resume.
- A release benchmark records candidate evaluations/s and peak RSS on the Ryzen 9 9950X.
