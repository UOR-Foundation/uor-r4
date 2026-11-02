# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

`CpuFbsForward` executes the SpiralCore/FBS trunk and reproduces the serving query at Q12 with both whole-table nearest and native-cell writer heads. `from_artifact` loads Hamming copy/type reads, `/7` opening/reply memory and the `/8` point-key fallback; `mark_reply_start`, `clear_reply_start` and `reset` control the memory path. `from_model` remains the no-read, no-memory entry point.

`CpuRankMarginTrainer` is the first finite learning unit on that path. It applies a deterministic frozen-query Q12 subgradient coordinate step to the target and hardest admitted rival rows of the single tied token table, reparses the patched artifact immediately to refresh Hamming codes, reports exact admission/top-1/top-8/rank readings, and round-trips a versioned checkpoint. Routes, gates and normalization remain unchanged; this is not yet a complete end-to-end trainer.

`CpuCounterfactualTrainer` extends the same exact full-forward acceptance loop to normalization gains, convolution/gate biases, decay offsets and all six FBS route tables through bounded BVG-K coordinate blocks. `CpuPanelEvaluator` grades a finite panel and records target admission, top-1/top-8, rank margin and component-wise CPU-versus-serving query/token/kept-cell gaps. `panel_admission_gate` refuses checkpoint admission loss or any serving mismatch; these are component controls, not a language result.

## Checks

```sh
cargo test -p uor-r4-cpu-train
cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-rank-margin-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-counterfactual-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-panel-export-bench
```

The forward benchmark prints read-free, copy-only, copy-plus-type and full Hamming-plus-memory cases with hashes and process peak RSS. Its readings are component throughput, not training or language evidence.

The rank-margin benchmark times one repeated synthetic sample through the Hamming-read forward, exact ranking, accepted table edit and artifact rebuild. The counterfactual and panel benchmarks cover coordinate evaluation and exact CPU/serving panel parity. Their rates are component ceilings, not corpus learning or language evidence.
