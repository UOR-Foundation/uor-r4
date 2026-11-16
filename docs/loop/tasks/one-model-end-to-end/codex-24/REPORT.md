# REPORT: one-model-end-to-end, codex-24 (the verdict rule as a command)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-23](../codex-23/REPORT.md).

## What did not work, first

- **The instrument crate does not build on `origin/main`.** `src/lib.rs` declares only
  `reply_control_controller`, while `evidence.rs` and `carry_geometry.rs` sit in `src/` unregistered;
  `m2-evidence` and `real-carry-geometry` import them from the crate and fail to resolve. This is a
  pre-existing break, not this unit's: the fix is two feature-gated `pub mod` lines, and both bins
  compile again.
- **The verdict's serde tag nested the answer** (`{"verdict":{"verdict":"memorized"}}`); it is a
  plain lowercase string now, with `undetermined` carrying its reason.
- **No protocol input exists yet.** The probe family (ext-s1/s2) is not the protocol's
  keyed/unkeyed/new-template panel, so the one run of `protocol-row` at this head is a **format
  fixture** whose numbers are the rule's unit-test fixture. It proves the plumbing and prints
  `memorized`; it is not a reading.
- The evidence report's strata are `memory-key:keyed/unkeyed` and a prompt-template *census share*,
  not a third reading: mapping a report to the three strata is the caller's next unit, and the
  instrument does not pretend the mapping exists.

## What changed

- `docs/labs/one-model-end-to-end/instrument/src/lib.rs`: `pub mod verdict;` and, under
  `cpu-parity`, `pub mod carry_geometry;` and `pub mod evidence;`.
- `docs/labs/one-model-end-to-end/instrument/src/verdict.rs` (new): `Stratum`, `ArmStrata`,
  `VerdictInput`, `Verdict`, `VerdictReport`, `Gains`, `share_ppm`, `verdict()`. The rule is the
  item's, and its only thresholds are the two the sheet must fix (`mostly_training_ppm`, default one
  half, and `novelty_band_ppm`).
- `docs/labs/one-model-end-to-end/instrument/src/bin/protocol-row.rs` (new) and the `Cargo.toml`
  entry; the README documents the row.
- The rule refuses to choose: an empty stratum or a missing novelty share is `Undetermined` with
  the reason, because `Generalizes` cannot be read without DEV's share.

## Measurements

- Eight unit tests on the rule: memorized; generalizes (gains over both controls on unkeyed and
  new-template, novelty within the band); mixed on far-below-dev novelty; mixed with no gains; mixed
  with keyed gains but few TRAIN replies; undetermined on an empty stratum; undetermined on missing
  novelty; the integer shares with a zero whole.
- `m2-evidence` and `real-carry-geometry` compile under `cpu-parity` at this head.
- `protocol-row --input FIXTURE` prints `"verdict": "memorized"` for the fixture, with every
  reading in the report.

## Checks run at the head

- `cargo test --offline --lib verdict` in the instrument — **8 passed**.
- `cargo check --offline --features cpu-parity --bin m2-evidence --bin real-carry-geometry` —
  clean.
- `cargo run --offline --bin protocol-row -- --input FIXTURE` — the report above.

## Still open

1. The report-to-strata mapping: which evidence fields become `keyed`, `unkeyed` and
   `new_template`, and where the per-row new-template stratum comes from.
2. The sheet's two numbers: "mostly" and the novelty band, fixed before the run.
3. The run itself (I6), and the `/17` contract that names the read.
