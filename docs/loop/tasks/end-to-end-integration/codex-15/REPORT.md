# REPORT: end-to-end-integration, codex-15

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Every k-means site ADR-0007 names is uncompiled.** `addressing_arms` and `joint_admission` are
  `#[cfg(any())]` in `crates/uor-r4-training/src/lib.rs` (lines 26-27 and 126-127). The
  transformerless compiler is in `transformerless/mod.rs`, which the lib target does not mount: the
  target is `src/native_support.rs`, whose `transformerless` is `native_mod.rs` (tokenizers only,
  and its header calls `compiler.rs` "inactive evidence"). The graph compiler's `induction.rs` has
  the same shape: `src/native_support.rs` and "No Cargo feature re-enables the historical surface."
  So "attach e-means at the four sites" cannot be done without reviving a parked line.
- **The only live codebook is not a k-means fit.** A8 address cells (`uor-r4-discrete`, ACTIVE)
  place tokens by a greedy, quota-constrained weighted-L1 rule. e-means on the same task is
  **2.152x worse** on the fixture's weighted cell-L1 (31,588 vs 14,676). The arm is opt-in and is
  **not promoted**; the default placement is unchanged.
- **The fitter is parity at this scale, not a win.** 4 e-means epochs vs Lloyd 16 passes:
  MSE 5.118431 vs 5.104009 (1.003x); vs Lloyd at 4 passes: 5.110207 (1.002x). The paper's
  large-scale advantage is not reproduced at `n = 2,048`, `k = 16`, `dim = 8`, which is the ADR's
  own caveat about adopting its scaling laws.
- **A pre-existing library test fails on clean `origin/main` too**:
  `tests::apply_calls_are_confined_to_bounded_fixture_and_admitted_transition`
  (`crates/uor-r4-discrete/src/lib.rs:425`). It is not introduced by this unit.
- No model or quality claim; no GPU, no float training, no serving change.

## What changed

- `crates/uor-r4-discrete/src/e_means.rs` (new, live through `lib.rs`): the fitter, its k-means++
  initialization, `mse`, and two tests (the parity gate and a bad-shape refusal).
- `crates/uor-r4-discrete/src/address_cells.rs`: `AddressCellsMode::EMeansV1 { seed }`, its label,
  the `e_means_cells` placement, and a determinism/validity/measurement test.
- `crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`: `e-means` and `e-means:SEED`.
- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`: Amendment 2 records the
  uncompiled sites and the negative measurement; it changes no rule.

## Measurements

- Fitter, synthetic 16 clusters / 2,048 points / 8 dims: e-means 4 epochs **1.003x** Lloyd 16 and
  **1.002x** Lloyd 4 (MSE as above); byte-identical centroids across two runs.
- A8 fixture (512 tokens, 4 blocks, quota 2): greedy **14,676**, e-means **31,588** weighted
  cell-L1 (**2.152x**); the e-means table is injective (512 distinct) and occupies all 256 cells at
  the exact quota; two runs are identical.
- `cargo test -p uor-r4-discrete --lib`: **358 passed, 1 failed (pre-existing), 1 ignored**.

## Checks run at the head

- `cargo test --locked -p uor-r4-discrete --lib e_means` — 3 passed.
- `cargo test --locked -p uor-r4-discrete --lib` — 358 passed, the pre-existing failure, 1 ignored.
- `cargo check --locked -p uor-r4-discrete --bin address-cells-codebook` — clean.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --lib --bin address-cells-codebook` — no
  warning in the changed lines.
- `rustfmt --check` on the three changed files — clean.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at
  the head.

## Skills used

- `decide-first`: the arm's gate (parity at 4 epochs, deterministic table) was declared before the
  measurement; the placement loss is reported rather than tuned away.
- `solution-integrity`: the default path is untouched and the new arm is opt-in; no existing
  assertion is weakened.
- `scope-guard`: one crate, one fitter, one mode, one ADR amendment; no serving or artifact change.

## Still open

- I3 needs a genuine live k-means target, e.g. a fitted read-key codebook in the integer path, or a
  revived site with its reopen condition met. The greedy A8 placement is not that target.
- The A8 arm's promotion would need a frozen panel and its control; none is claimed here.
- D86 step 6 still gates any serving use of a fitted codebook.

## Plain summary

The e-means fitter now exists as live code and passes its parity gate, but the only live codebook
places tokens by a greedy quota rule, where e-means is 2.15x worse. The arm is opt-in and unpromoted,
the default is unchanged, and ADR-0007 now records that its four named fit sites are uncompiled.
