# REPORT: exact CPU Hamming reads, deepseek-3

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `c411f64e4`.

## What did not work

- **Exact turn memory remains outside this slice.** `from_artifact` refuses a `lut-stack/7` model by name rather than executing its trunk and silently dropping the memory. Memory state, reply-start signaling and point-key fallback require the next separately testable unit.
- **Parity is synthetic, not trained-artifact evidence.** The fixtures exercise exact artifact bytes and the serving integer oracle, but no trained checkpoint or captured model query set is bound here.
- **No Hamming-code fault changed every possible query.** The negative control therefore erases a read's copy codes instead of toggling one entry: a single changed vocabulary code can leave the kept set and weighted mean unchanged. The stronger control verifies that the complete query changes when the read data are genuinely corrupted.
- **Peak RSS is a whole-process high-water mark.** It includes artifact loading, both benchmark cases and the type-head cache; it is not isolated per model.

## What changed

- Added `CpuFbsForward::from_artifact`, which parses the model, reads the public `lut-stack/6` Hamming image, strips only the read from a second parsed model to recover the FBS trunk view, and attaches the CPU read state.
- Added an independent CPU Hamming port in `crates/uor-r4-cpu-train/src/hamming.rs`:
  - vocabulary copy and type key codes;
  - Q31 weight margins and Hamming tie ordering;
  - copy means over the points after earlier matches;
  - type query codes, value-rotated payloads and per-position caches;
  - stream and readout hooks with their exact exponent transitions and half-away-from-zero sums.
- Pushed each token before the named-layer stream hooks, ran a depth equal to the layer count only at the readout hook, and applied readout before the final Q12 query and writer head.
- Kept `from_model` as the no-read constructor and preserved all prior forward parity and admission controls.
- Extended the release benchmark to read-free, copy-only and copy-plus-type cases with stable checksums and process peak RSS.

## Measurements

- `cargo test --locked -p uor-r4-cpu-train`: 27 passed, 0 failed.
- Six configurations compare every step's complete `query_q12`, token and kept-cell vector against `IntegerStackSession::step_native`: zero weight, copy only, type only, combined, depth-1 stream only and depth-equal-to-layers readout only.
- Signed half-ties cover the one-rounding stream and readout sums.
- The payload-cache constructor is tested at exactly 4 GiB and one element past it.
- Erasing all copy codes in an otherwise valid read changes the complete query; zero read weights reproduce the read-free trunk.
- Release benchmark on the Ryzen 9 9950X:

| Read | Timed steps | Elapsed | Tokens/s | Checksum |
|---|---:|---:|---:|---:|
| none | 64 | 19.788 ms | 3,234.218 | `11743905789448238222` |
| copy | 64 | 20.146 ms | 3,176.879 | `16079281126448246411` |
| copy + type | 64 | 20.602 ms | 3,106.451 | `11401164151804504729` |

Process peak RSS was 25,912 KiB. The benchmark includes the 4,096-token table, six 256-wide layers, two Hamming depths and the type payload cache; it is not training throughput.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train`: PASS, 27 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS; dependency warnings were emitted without `--no-deps`.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench`: PASS; readings above.
- `cargo fmt --package uor-r4-cpu-train`: PASS.
- `uor-harness ratchet d23`, `d25` and `d32`: PASS with zero growth.
- `uor-harness loop gates --full`: PASS, 11/11.

Exact-head adversarial review is recorded in the PR after it runs.

## Still open

- Exact turn memory and its restore/reset continuation.
- Trained-artifact parity and captured-query parity.
- Shared-vector training gradients, deterministic checkpoints and exact-export evaluation.
- The item remains `ready`; this PR references #3504 and does not close it.

## Plain summary

The CPU shared-vector forward now executes the current Hamming copy and type reads exactly, including payload state and both artifact hooks. Its complete Q12 query, token and kept-cell outputs match serving across every supported read combination. Exact turn memory is the next forward slice; this is still infrastructure rather than a training or language result.
