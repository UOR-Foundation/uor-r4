# REPORT: exact CPU turn memory, deepseek-4

Lab: deepseek. Item: [cpu-shared-vector-forward](../../../../roadmap/items/cpu-shared-vector-forward.md) (#3504). Base: `c70ab1fe1`.

## What did not work

- **Saved-session restart is not established.** The CPU forward can clear its state and restart fresh with the same loaded tables, but this unit does not serialize recurrence, Hamming payloads and memory anchors into a resumable checkpoint. Serving still refuses `restore_state` for a turn-memory artifact.
- **Parity is synthetic, not trained-artifact evidence.** The tests construct exact artifact tables with known opening and reply hits, but no trained checkpoint or captured model panel is bound.
- **The point-key fixtures are synthetic.** They verify a unique target, the commonest target, and the smaller-id tie; a trained point table and captured-query panel remain for the trained-artifact gate.
- **No serving behavior changed.** `fbs_image` still returns `None` when Hamming or memory is present. The new `fbs_trunk_image` is explicitly a trunk-only evaluator view and must accompany separate read/memory execution.

## What changed

- Added additive `IntegerStackModel::fbs_trunk_image`, with `fbs_image` delegating to it only after retaining its existing Hamming/memory refusal.
- Added an independent CPU turn-memory port:
  - opening and reply key construction with BOS/EOS anchors and key-window boundaries;
  - prefix-truncating exact table lookups and stored-token moves;
  - Q12 writer-point promotion and complete-sum half-away-from-zero rounding;
  - point-key L1 naming, commonest-target ties to the smaller id, and opening-miss token override.
- Integrated memory after the Hamming readout, before final Q12 conversion and the writer head, matching serving order.
- Added `mark_reply_start`, `clear_reply_start` and fresh-stream `reset` to the CPU forward.
- Extended the full-path release benchmark to a Hamming-plus-memory artifact where the opening and every timed reply row hit.

## Measurements

- `cargo test --locked -p uor-r4-cpu-train`: 37 passed, 0 failed.
- Exact opening and reply moves match serving for writer and nearest heads.
- Fractional positive and negative weights match serving; a direct primitive test covers the positive and negative half-away ties.
- A missed exact opening uses the point key and names token 99 exactly as serving does.
- Point fallback tests cover a two-row commonest target and an equal-count smaller-id tie.
- A short key window crosses blocks, restarts from EOS, drops the prompt turn after leaving its block, and matches serving at every row.
- Combined Hamming plus memory matches for writer and nearest configurations.
- Without a reply signal, and after clearing one, every step matches a memory artifact's un-signalled serving session.
- CPU reset reproduces a fresh memory stream with the same table; malformed memory and point dimensions are refused.
- Release benchmark on the Ryzen 9 9950X:

| Path | Timed steps | Elapsed | Tokens/s | Checksum |
|---|---:|---:|---:|---:|
| no read/memory | 64 | 19.844 ms | 3,225.180 | `11743905789448238222` |
| copy | 64 | 20.197 ms | 3,168.736 | `16079281126448246411` |
| copy + type | 64 | 20.632 ms | 3,101.953 | `11401164151804504729` |
| copy + type + memory | 64 | 20.659 ms | 3,097.849 | `17326088280257995008` |

Process peak RSS was 32,792 KiB. The full path includes the 4,096-token table, six 256-wide layers, two Hamming depths and a 63-entry reply table. It is not training throughput.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train`: PASS, 37 tests.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: PASS.
- `cargo run --locked --release -p uor-r4-cpu-train --bin cpu-fbs-forward-bench`: PASS; readings above.
- `cargo fmt --package uor-r4-cpu-train -- --check`: PASS.
- `uor-harness ratchet d23`, `d25` and `d32`: PASS with zero growth.
- `uor-harness loop gates --full`: PASS, 11/11.

Exact-head adversarial review is recorded in the PR after it runs.

## Skills used

No repository skill was invoked. A fresh-context subagent will perform the required exact-head review before merge.

## Still open

- Deterministic saved-session/process restart and reset parity in #3503.
- Stored-GPU/trained-artifact and captured-query parity in #3502.
- Shared-vector gradients and checkpoint/export evaluation in #3505.
- The item remains `ready`; this PR references #3504.

## Plain summary

The CPU forward now executes the current artifact's Hamming reads, opening and reply memory, and point-key fallback, and matches serving on complete final vectors and decisions in the tested configurations. This closes the unported forward path but is not yet a trained-model or training result.
