# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

`CpuFbsForward` executes the SpiralCore/FBS trunk and reproduces the serving query at Q12 with both whole-table nearest and native-cell writer heads. `from_artifact` loads Hamming copy/type reads, `/7` opening/reply memory and the `/8` point-key fallback; `mark_reply_start` and `clear_reply_start` control the memory path exactly as the serving session does. `from_model` remains the no-read, no-memory entry point.

## Checks

```sh
cargo test -p uor-r4-cpu-train
cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench
cargo run --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench
```

The forward benchmark prints read-free, copy-only, copy-plus-type and full Hamming-plus-memory cases with hashes and process peak RSS. Its readings are component throughput, not training or language evidence.
