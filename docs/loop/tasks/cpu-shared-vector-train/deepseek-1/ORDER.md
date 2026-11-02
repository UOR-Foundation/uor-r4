# ORDER: CPU tied-table rank margin, deepseek-1

Lab: deepseek. Item: [cpu-shared-vector-train](../../../../roadmap/items/cpu-shared-vector-train.md) (#3505). Base: `6a2d26249` from fresh `origin/main`.

## Goal

Establish the first deterministic CPU learning unit for the serving-aligned shared-vector path: a lawful rank-margin update of the single tied token table, evaluated by rerunning the exact forward and exact writer decisions.

## Scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-shared-vector-train/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`

No serving-source edit, corpus-scale run, GPU run, training or language claim.

## Cancel probe

Stop if an accepted table edit cannot refresh every table-derived Hamming key/code without changing serving semantics. The CPU crate can rebuild those caches from the public Hamming image, so the unit stands. If that rebuild cannot be made exact, refuse Hamming artifacts rather than train a stale model.

## Done when

- A sample's writer score is exact displacement to the target and hardest wrong token, and the rank margin is finite and geometric.
- One accepted update is a bounded ±1 Q12 subgradient coordinate step on the target and rival rows of the shared table; ties and coordinate order are deterministic.
- Every accepted update reruns the full CPU forward, refreshes table-derived read keys, and is retained only when it improves the exact margin and does not reduce the configured exact admission set.
- Route, gate and normalization tables remain byte-identical in this first unit.
- Training reports loss, admission, top-1, top-8, target rank and exact writer cells per step.
- A versioned checkpoint binds the table, clocks, cursor, config and state; two deterministic resumes reproduce identical checkpoint bytes and next-step readings.
- Finite tests cover no-update margin, target/rival movement, bounds, admission rejection, Hamming cache refresh, malformed checkpoints and resume identity.
- No squashing, dense matrix product, floating training loop or dependency edge to `uor-r4-training` is introduced.
