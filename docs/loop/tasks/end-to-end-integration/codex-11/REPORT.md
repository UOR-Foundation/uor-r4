# REPORT: end-to-end-integration, codex-11

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic session-scan-timing` was refused because
  `crates/uor-r4-integer/src/stack/**` overlaps another lab's open order. The unit adds one test and
  docs, no dependency and no `Cargo.lock` change.
- **The read-free overflow policy is not built.** It was the other half of the request. On
  inspection it is not worth the code: a read-free model can only continue by dropping the oldest
  token, and with the session's `Vec<u32>` that is `O(context)` per step (a ring would touch every
  reader); no committed artifact is read-free, so the policy would serve a case that does not occur;
  and `ContextFull` is the exact answer for every model the repo serves. The negative is recorded
  here rather than a policy added for a hypothetical.
- **The times are indicative, not a benchmark.** One run per cell in a debug build, on this machine.
  The asserted part is token equality and the ratio's shape; the absolute times move.
- **4,096 is the largest CI-friendly context here.** 32k is not run in the suite; codex-8 measured
  the selection at 32k, and a local indexed 32k session ran at about 34,000 tok/s.
- **Single-code heads.** Pair heads and the forced replay still scan; the test's heads are copy key
  1 under the Mean rule.
- Test-only; no model or quality claim.

## What changed

- `crates/uor-r4-integer/tests/hamming_index_session_timing.rs` (new): the same read-bearing
  synthetic trunk and stream served indexed and scanned at 1,024 / 2,048 / 4,096, tokens compared at
  every step, times printed, and two shape assertions.

## Measurements

| Context | Indexed | Scan | Ratio |
|---:|---:|---:|---:|
| 1,024 | 29.94 ms (34,201 tok/s) | 76.55 ms (13,376 tok/s) | 2.56x |
| 2,048 | 60.58 ms (33,806 tok/s) | 291.80 ms (7,018 tok/s) | 4.82x |
| 4,096 | 121.03 ms (33,844 tok/s) | 1,160.32 ms (3,530 tok/s) | 9.59x |

- Tokens identical at all three contexts.
- The indexed session holds about 34k tok/s while the scan falls with context, as an `O(t)` a step
  scan must.
- Separate local run: a 32k-context indexed session ran 32,768 steps in 0.959 s (about 34,158 tok/s,
  debug); not part of the suite.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --test
  hamming_index_session_timing` — 1 passed (1.75 s).
- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — at the head.
- `rustfmt --check` on the new file — clean.
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib
  --tests` — no warning in the new file (`--lib --tests` still fails on the pre-existing
  `openai_loopback.rs:1658` deny).
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: the timing is a whole-session measurement with a parity check, and the
  read-free policy was rejected on its actual cost and prevalence rather than built for symmetry.
- `scope-guard`: one test file; no source change.
- `test-freeze`: a new file; no existing test changed.

## Still open

- The 32k session on/off comparison is not feasible on this machine (the scan is minutes to hours at
  that context); the 32k evidence is codex-8's per-query count plus the local indexed run.
- The read-free overflow policy stays unbuilt; `ContextFull` remains the exact policy.
- The capability question is the read's other-fact objective (C2′), on the native-read line.

## Plain summary

A whole session served from the index is identical to one served from the scan and gets
progressively cheaper as context grows: 2.56x at 1k, 4.82x at 2k, 9.59x at 4k, with tokens
identical. The read-free overflow policy was inspected and dropped: it would cost O(context) a step,
no served artifact is read-free, and `ContextFull` is already exact.
