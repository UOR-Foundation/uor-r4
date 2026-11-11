# REPORT: one-model-end-to-end, deepseek-276 (E8 first-test sheet)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). ORDER: [ORDER.md](ORDER.md). Base branch: `deepseek/e8-audit`
at `d792c77f2`, the E8 draft plus current `main`.

**Status:** the E8 branch integrates cleanly with current `main`, its host and
stub-CUDA checks pass, and its first registered test is written. The sheet is
`TEST FITNESS: PENDING`; no compute is cleared and no pod was launched.

## What did not work, first

- The first full host-suite invocation omitted `CARGO_TARGET_DIR`. One unrelated
  save/load test correctly refused with "owned artifact directory required."
  Repeating the same suite with `CARGO_TARGET_DIR=local/e8-host` passed all 388
  tests. The first invocation was an invalid environment, not a code failure.
- The E8 tensor code still has never run on a GPU. The host tests prove the
  plain-Rust address and wall formulas and the stub compiler proves the tensor
  paths type-check; neither proves GPU arithmetic.
- T0's no-memory baseline predates this branch. Config equality is registered
  as a check, but a byte-identical full rerun is not claimed.

## What changed

- [The E8 first-test sheet](../../../harness/prep/one-model-end-to-end-e8-address-2026-11-11.md)
  registers T0's 1,000-dialogue set, the sealed no-memory negative, four address
  arms at seeds 1 and 2, 4,096 steps, T0's scoring rules, fixed bars, a
  top-down outcome map, eight named GPU preflights, cost and stop rules.
- No production source changed. The integration commit merges current `main`
  into the audit branch; it does not alter Claude's `claude/e8-address` branch.

The outcome map distinguishes the E8 arm passing, an E8 decode-only miss, a
control-only pass, and all arms missing. A mixed-seed result cannot close a
configuration.

## Measurements

| Check | Result |
|---|---|
| E8 unit tests | **10 passed, 0 failed** |
| `train-in-space` tests | **12 passed, 0 failed** |
| Float-train release host suite | **388 passed, 0 failed** |
| Stub-`nvcc` CUDA type-check | exit 0 |
| Harness sheet shape/readability | **3 passed, 0 failed** |
| Harness `prep-check` | sheet found, `PENDING`, not cleared for compute |

The registered GPU preflight is 17 tests under `e8_address` plus the address
decode-cache test, one test: **18 total**. Those are not reported as run.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-float-train --lib e8_address`
- `cargo test --release --locked -p uor-r4-float-train --bin train-in-space`
- `CARGO_TARGET_DIR=local/e8-host cargo test --release --locked -p uor-r4-float-train --lib`
- stub-`nvcc` `cargo check --offline --locked -p uor-r4-float-train --features cuda --profile test --lib --bin train-in-space`
- `cargo test -p uor-r4-harness --lib -- the_real_sheets`
- `uor-harness prep-check` and `git diff --check`

## Skills used

`working-discipline`, `solution-integrity`, `scope-guard` and `decide-first`
(branch integration, measured host baseline, bounded sheet-only unit, and the
choice of E8 preparation over another place-read run).

## Still open

- An independent non-author must review the sheet before it can become `FIT`.
- The 18-test GPU preflight must pass before a long run starts.
- No owner approval exists for the registered four-pod run.
- No integer runtime or export path exists for the address rows yet.
- The E8 branch is local/test evidence until it is pushed and reviewed.

## Plain summary

The E8 branch now sits on current `main` and all CPU checks pass. Its first real
test is pre-registered against the same small memorization set that the
no-memory model failed. Four address forms, two seeds and three matching
controls are specified, but the sheet is deliberately not marked FIT and
nothing was run.
