# ORDER: infra-loop-harness, antigravity-78 (reconcile read-binding-on-lawful-read prep sheet headings so uor-harness check passes)

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file and prep-sheet gate maintenance). Base: `616384cacf190b6c174d45b5e79d0b176649b925`.

## Goal

Fix the repository-wide `uor-harness check` failure (`error prep.sheet_shape [docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md]: missing block(s): Base artifact, Data split, Steps, Cost, Minimum test power`) that currently causes the GitHub Actions workflow `"harness rules and tests"` (`harness.yml`) to fail on every PR and causes `uor-harness loop next` to classify open PRs as `GO repair #<pr> — check "harness rules and tests" is FAILURE`:

1. Add `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` to `scope:` in `docs/roadmap/items/infra-loop-harness.md` (`#2904`) and re-render `docs/roadmap/README.md`.
2. Reconcile the section headings of `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` (written in `antigravity-60` for `#2889`) with `REQUIRED_BLOCKS` in `crates/uor-r4-harness/src/prep.rs:30-39` (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, `TEST FITNESS: FIT`) without altering any registered arm, bar, control, or stop rule.

## Cancel probe (run first)

Class: `instrument` (CI gate / `prep.sheet_shape` check at `616384cacf190b6c174d45b5e79d0b176649b925`):
1. `uor-harness loop claim infra-loop-harness --lab-topic fix-read-binding-sheet` refused solely because `PR #3169` (`codex-1`, idle draft) holds the `infra-loop-harness` umbrella. Verified via `gh pr list --state open --json number,files` that neither `#3169` nor `#3225` touches `docs/roadmap/items/infra-loop-harness.md` or `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md` (`0` overlapping files). Claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella exception.
2. Verified via `uor-harness check` at `616384cacf190b6c174d45b5e79d0b176649b925` that `error prep.sheet_shape [docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md]` is the single `error` across all 213 rules (`213 rules, 183 failed (1 error, 181 warn)`).

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md`
- `docs/roadmap/README.md` (shared registration file)
- `docs/harness/prep/read-binding-on-lawful-read-2026-10-25.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-78/**`

## Done when

- `uor-harness check` reports `0 error` across all 213 rules (`prep.sheet_shape` passes on all 64 pre-registration sheets);
- `~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass (`References #2904`, `References #2889`).

## Stop and ask if

- Any file outside the declared scope is needed, or any gate in `uor-harness loop gates --full` fails.
