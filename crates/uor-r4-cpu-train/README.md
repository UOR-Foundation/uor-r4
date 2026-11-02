# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

`CpuFbsForward` executes the SpiralCore/FBS trunk and reproduces the serving query at Q12 with both whole-table nearest and native-cell writer heads. `from_artifact` loads Hamming copy/type reads, `/7` opening/reply memory and the `/8` point-key fallback; `mark_reply_start`, `clear_reply_start` and `reset` control the memory path. `from_model` remains the no-read, no-memory entry point.

`CpuRankMarginTrainer` is the first finite learning unit on that path. It applies a deterministic frozen-query Q12 subgradient coordinate step to the target and hardest admitted rival rows of the single tied token table, reparses the patched artifact immediately to refresh Hamming codes, reports exact admission/top-1/top-8/rank readings, and round-trips a versioned checkpoint. Routes, gates and normalization remain unchanged; this is not yet a complete end-to-end trainer.

## Checks

```sh
cargo test -p uor-r4-cpu-train
cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-rank-margin-bench
```

The forward benchmark prints read-free, copy-only, copy-plus-type and full Hamming-plus-memory cases with hashes and process peak RSS. Its readings are component throughput, not training or language evidence.

The rank-margin benchmark times one repeated synthetic sample through the Hamming-read forward, exact ranking, accepted table edit and artifact rebuild. Its steps/s are a component ceiling, not corpus learning or language evidence.
