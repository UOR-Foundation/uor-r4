# ORDER: exact CPU turn memory, deepseek-4

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `c70ab1fe1` from fresh `origin/main`.

## Goal

Extend the CPU FBS forward to execute the `/7` turn memory and `/8` point-key fallback exactly against `IntegerStackSession::step_native`, so a complete shared-vector artifact has no unported forward operation.

## Scope

- `crates/uor-r4-cpu-train/**`
- `crates/uor-r4-integer/src/stack/session.rs` for one additive trunk-only view only
- `docs/loop/tasks/cpu-shared-vector-forward/deepseek-4/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-forward.md`

No serving behavior, format or existing public-method change; no Claude-owned PR edit, corpus run, GPU run, training, or language claim.

## Cancel probe

Stop if the public memory reader or point table is incomplete. `read_memory_uorlut01` exposes both, so the unit stands. `fbs_image` intentionally refuses a memory artifact; add an explicitly named trunk-only view and leave `fbs_image` unchanged.

## Done when

- The CPU forward loads opening/reply tables and the optional point key from artifact bytes.
- Explicit reply-start signaling reproduces opening hits and misses, reply hits, anchor formation, exact move rounding and point-key fallback.
- Deterministic fixtures compare complete Q12 query, token and kept cells against `IntegerStackSession::step_native`, including a combined Hamming-plus-memory artifact and both output heads where supported.
- Clearing/resetting memory state and malformed or unsupported states are tested.
- `fbs_image` retains its old refusal while the additive trunk view serves the CPU evaluator.
- A release benchmark records memory-path throughput and peak RSS on the Ryzen 9 9950X.
- One REPORT and notebook row state every remaining limitation; finite parity is not training or language evidence.
