# Full-path cost run: width-576 dialogue bundle (one run, not a qualification)

**Date**: 2026-09-28
**Author**: Lab 3 (Anti-Gravity); corrected by Lab 1 per the owner's direction (PR #1452 review)
**Track**: T3 (mission runtime and measured efficiency)
**Status**: one unsealed run; **not a qualification**
**References**: #963, #820, #964, #962, PR #1450, PR #1452

---

## 1. Status

The harness ran once, at 2026-09-28 05:38:25 UTC, in a single test thread under unknown machine load, and wrote no sealed report root. This record is therefore not a qualification. The run does **not** meet the declared alpha ceilings:

- the mean step was 7.315 ms and p90 13.237 ms, against ≤ 4 ms for each;
- the cold load was 1,530.49 ms, against ≤ 250 ms.

An earlier version of this record reported other figures (67.137 ms cold load, 3.163 ms mean step, 304.3 tok/s, 3.445 µs/token encode, and others) and marked them PASS. No committed run produced them. The release packager returned them as literal defaults because it read keys that the harness never writes. They are withdrawn, together with every PASS and "qualified" statement they supported.

## 2. Recorded values

Source: [`docs/evidence/full-path-m1-cost-dialogue576-2026-09-28.json`](../evidence/full-path-m1-cost-dialogue576-2026-09-28.json), written by the harness `crates/uor-r4-integer/tests/full_path_m1_cost.rs` as it stood before this correction.

| Metric | Recorded value | Declared ceiling | Meets |
|---|---:|---:|:---:|
| Cold bundle load | 1,530.49 ms | ≤ 250 ms | no |
| Tokenizer encode (4 prompts) | 43.93 µs/token | — | — |
| Prompt ingestion | 7.788 ms/token | — | — |
| Step latency, mean | 7.315 ms | ≤ 4 ms | no |
| Step latency, p50 | 5.170 ms | — | — |
| Step latency, p90 | 13.237 ms | ≤ 4 ms | no |
| Step latency, p95 / p99 / max | 18.483 / 38.317 / 139.054 ms | — | — |
| Throughput (2,526 steps / replay wall time) | 130.6 tok/s | — | — |
| Session save / restore (50-turn session) | 0.165 / 0.322 ms | — | — |
| Peak process RSS (maximum of `ps` samples) | 22.36 MB | — | — |
| Replay parity | 38 requests, 58 turns, 1,433 decisions, 0 departures | exact | yes |
| SoC energy | UNAVAILABLE | — | — |

## 3. What the run established, and what it did not

- **Build and machine.** The run measured the library in-process under `cargo test --release`. It used an uncommitted working tree between `698bdda` and `a8d59d31`: the run time, 01:38:25 EDT, precedes the commit `a8d59d31` at 01:41:21 EDT. The source commit, compiler flags, machine and machine load were not recorded. The JSON's `hardware.platform` and `threads` fields are literals in the harness, not measurements.
- **Not the frozen binary.** The harness did not execute the frozen `uor-chat` binary, so that binary's SHA-256 and instruction audit do not describe this run.
- **Parity.** The harness compared every greedy decision with `responses-integer.json` and stopped at the first departure. The completed run therefore had 0 departures over 1,433 decisions.
- **Step timing.** Each step was timed around `IntegerModel::step_conversational`, which also copies the step's output buffers into a new `IntegerStep`.
- **RSS.** The peak is the largest of the `ps` samples taken after prompt ingestion and after each replayed turn, in the test process. The earlier RSS series (2.31, 24.19, 25.78 and 29.50 MB) and the 23.22 MB "live REPL" figure were not recorded by the harness and are withdrawn.
- **Save and restore.** Only the save and restore durations were timed. The restored session was never compared with the original, so the earlier "roundtrip parity verified" statement is withdrawn. A width-576 restore does not in fact preserve the session:
  - `uor-r4.integer-session/1` does not serialize the 576-wide value stores (`persistent_values_576`, `dialogue_values_576`, `l2_pages_576`);
  - `ChatSession::from_serialized` rebuilds those stores empty;
  - the corrected harness and the ignored capability-API continuation test therefore find the restored session diverging at its first step.

## 4. Bytes touched per token: analytic, not measured

The harness computes 1,821,872 bytes (1.737 MiB) per token by hand. The count covers:

- the 4-bit vocabulary projection;
- one 64 × 576 read projection;
- 256 memory keys and values;
- the state, zeta and Hopf coordinates.

It is not measured. It also omits the recurrent input and state matrices, the read key and value matrices and the update matrix of `JointConfig::shapes()`. At width 576 these hold 3,022,848 signed-4-bit weights, 1,511,424 bytes packed. It omits the product tables as well. The count therefore understates per-token reads. It supports no claim about cache residency or energy.

## 5. Numerical contract

This run measures latency only; it does not audit instructions. The instruction audit of the frozen `uor-chat` binary is recorded in the [release-bundle record](release-bundle-dialogue576-2026-09-28.md). That binary predates this PR's source changes.

## 6. Re-running the harness

The harness is now ignored by default, and it fails when the fixture is missing:

```bash
UOR_R4_M1_COST_REPORT_ROOT=/new/attempt/dir UOR_R4_SOURCE_COMMIT=<sha> \
  cargo test -p uor-r4-integer --release --test full_path_m1_cost -- --ignored --nocapture
```

- **Report root.** With the variable set, the harness claims the directory exclusively (`report_output::claim`) before the bundle loads. It writes `m1-cost.json` there, then seals and verifies the root. Without the variable it writes nothing, and it never writes a tracked file.
- **Run conditions.** The report records the source commit supplied in `UOR_R4_SOURCE_COMMIT`, the CPU brand string and the load averages before and after the run.
- **Ceilings.** The report records each declared ceiling beside its measured value, with a computed `meets` flag. Ceilings are not asserted.
- **Restore.** The report records whether a restored session continues with the same output distributions as the original. It does not assert this, because width-576 restores are known to diverge (section 3).
- **Assertions.** The harness asserts replay parity and the replay counts: 38 requests, 58 turns, 1,433 decisions and 2,526 steps.

A qualification needs a fresh sealed run that meets the ceilings on a machine whose load is recorded.
