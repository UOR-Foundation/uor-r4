# REPORT: infra-loop-harness, kimi-21 (scope glob for the m4-c1 draft sheet)

Lab: kimi. Order: [ORDER.md](ORDER.md). Item `infra-loop-harness` (#2904). Base: `7080fd6cf`.
Unblocks PR #3052 (deepseek-47): `References #2904`.

## What did not work, first

- Nothing failed; this is the requested one-glob widening. The mismatch it fixes: the item's
  `scope:` covered `docs/harness/prep/m4-flock-coupling-prereg*` while the canonical draft is
  `docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md`, so `loop pr-check` refused the
  sheet the deepseek-47 NOT_FIT verdict belongs in (deepseek's request on #2028, 2026-10-23
  10:18Z; they reverted the sheet edit and held it).

## What changed

| File | Change |
|---|---|
| `docs/roadmap/items/m4-flock-coupling-prereg.md` | `scope:` gains `docs/harness/prep/m4-c1-flock-coupling*`; nothing else |
| this folder | ORDER (committed first) and this REPORT |

## Measurements

- Cancel probe at the base: `grep -n m4-c1 docs/roadmap/items/m4-flock-coupling-prereg.md` matched
  only prose line 23; the scope line had no `m4-c1` entry. Task stood.
- `xtask roadmap render` re-wrote `docs/roadmap/README.md` byte-identically (scope lines are not
  rendered), so the diff is the one scope line plus this folder.

## Checks run at the head

`loop report --check`, `loop pr-check --local`; results below before the PR opens.

## Skills used

- `scope-guard`: one glob, one file; the canonical path is the one the item's own text names.

## Still open

- After this merges: #3052's pr-check scope rule passes at its merge base and its review completes.

## Plain summary

- Added the missing `m4-c1-flock-coupling*` prep glob to the item's scope so the canonical draft
  sheet is claimable; docs metadata only, no GPU, $0.
