# ORDER: CPU shared-vector forward parity, deepseek-1

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `d8f4282cf` from fresh `origin/main`.

## Goal

Implement an independent CPU execution of the current no-Hamming-read SpiralCore/FBS trunk and prove complete forward parity against `IntegerStackSession::step_native`.

## Scope

- `crates/uor-r4-cpu-train/**`
- `Cargo.toml`, `Cargo.lock`
- `docs/loop/tasks/cpu-shared-vector-forward/deepseek-1/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-shared-vector-forward.md`

No edit to `crates/uor-r4-integer` or another serving crate. No training, corpus run, GPU run, artifact publication or deployment.

## Cancel probe

Stop if `FbsImage` cannot expose the no-read forward tables needed for a complete step. It exposes route maps, gains, biases, the shared writer table, roots and kept-cell count, so the unit stands.

## Done when

- The CPU crate rebuilds the no-Hamming-read FBS state from `FbsImage`.
- Exact CPU arithmetic reproduces the serving Q12 query, nearest/writer token and kept cells on one, three and six layer synthetic artifacts.
- A deliberate route-sign fault changes the full query, proving the test compares more than a final token.
- A release benchmark records tokens/s on the Ryzen 9 9950X.
- Hamming reads and exact turn memory remain explicitly open; no training claim is made.
