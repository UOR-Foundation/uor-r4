# REPORT: infra-loop-harness, kimi-24 (card #3041: the last TEST FITNESS verdict wins)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Card: #3041.
Base: `15d5728c7`. ORDER: [ORDER.md](ORDER.md), committed first.

## What did not work, first

- `prep::test_fitness` (`crates/uor-r4-harness/src/prep.rs:97`) returned on the FIRST anchored verdict line. The re-review flow appends a new verdict instead of editing history, so a sheet that earned NOT_FIT and later FIT still parsed NOT_FIT and stayed silently blocked at the FIT gate (D22). Live path: #3052 recorded a NOT_FIT on the m4 flock-coupling sheet on 23 October; its later FIT will be appended.
- First probe attempt: none needed beyond the source read — the `return` on the first match is visible at the base, and the existing test module had no two-verdict case (gap confirmed by grep).

## What changed

| Commit | What |
|---|---|
| ORDER commit | The ORDER, alone. |
| this commit | `prep.rs`: `test_fitness` now scans all lines and the LAST anchored verdict wins (ABSENT if none); anchoring, case and NOT_FIT-before-FIT prefix rules unchanged. Regression test `the_last_verdict_wins`: NOT_FIT-then-appended-FIT reads FIT, FIT-then-appended-NOT_FIT reads NOT_FIT, appended PENDING reads PENDING, unrelated later prose does not erase the verdict. |

## Measurements

- Call-site audit (the ORDER's stop-and-ask): `preprules.rs:120` (`fit_before_run` warns on a stale non-FIT verdict) and `main.rs:1407` (reports a written body's fitness) both want the sheet's CURRENT verdict; no caller depends on first-wins.
- `cargo test -p uor-r4-harness --lib prep`: 31 passed, 0 failed (includes the new test).
- `cargo test -p uor-r4-harness` (full): results at the head in the PR body.
- Class **instrument** (harness unit fix; no GPU, no model compute).

## Checks run at the head

- `cargo test -p uor-r4-harness`, `uor-harness loop report --check`, `loop pr-check --local`, `loop gates --quick`; results in the PR body. `prep.rs` entered the umbrella scope with kimi-23 (#3055).

## Skills used

- `solution-integrity`: the fix is one behavioral change (first → last) with the regression pinned both directions, verified by running the suite rather than by reading the diff.

## Still open

- Card #3042 (`render_fit` trailing space + leaked indentation in the emitted block) is separate and claimable by any lab; not touched here.
- The m4 flock-coupling sheet's re-review FIT (after the five §7 answers) will be the first production use of the fixed path.

## Plain summary

The fitness gate read a sheet's oldest verdict; it now reads its newest, so a re-reviewed sheet is no longer stuck at its first NOT_FIT.

Closes #3041
References #2904
