# uor-r4-cpu-train

Isolated CPU development trainers authorized by D84. GPU remains the qualification authority, and this crate is deliberately outside the workspace's default-member set.

The first unit provides `CpuWriterCore`, an independent implementation of exact cell admission followed by full-vector token ranking. `WriterReading` exposes one row's top-8 and target outcomes; `WriterSummary` aggregates top-1, top-8 and target admission. These are component oracles, not a training result.

## Checks

```sh
cargo test -p uor-r4-cpu-train
cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings
cargo run --release -p uor-r4-cpu-train --bin cpu-writer-bench
```
