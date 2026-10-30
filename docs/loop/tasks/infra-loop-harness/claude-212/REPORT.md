# REPORT: infra-loop-harness, claude-212 (M1's close, recorded)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Order: [ORDER.md](ORDER.md).

## Tokens per second

The closed model serves at a median 10,876 / 10,566 decode tok/s on an Apple M1 laptop (claude-211's M1 bar (f), `docs/labs/m1-heldout-phrasing-transfer/evidence/toks-s{1,2}.json`). This unit measures nothing new.

## What did not work, first

- **Three records lag until the owner acts.** `AGENTS.md` still says "0 of 8 milestones (M1–M8, issues #2029–#2036) are complete". `docs/integration/DECISIONS.md` has no row for the close, and `CHARTER.md` none either. All three are owner paths. The proposal is in the owner's queue on #2028 (comment `6515151539`).
- **#2846, the W4 float-credit sheet,** is marked not needed for M1's close but not re-homed to M2. Moving it to another milestone is a scope change for its own unit.

## What changed

- **`PLAN.md`:**
  - the goal now says "1 of 8 milestones complete (M1, 30 Oct 2026 …)";
  - the M1 premise row is `closed`, with its answer;
  - the M1 path paragraph records the close and its limit;
  - steps 8 and 9 read `done`;
  - #2846 is noted as not needed for the close.
- **`docs/roadmap/items/m1-heldout-phrasing-transfer.md`** is `status: done`, and `docs/roadmap/README.md` is re-rendered by `cargo xtask roadmap render`.

## Measurements

None new. The evidence for M1's six bars is in #2029's acceptance criteria, its owner log (comment `6515151261`), and claude-211's REPORT in `docs/loop/tasks/m1-heldout-phrasing-transfer/`.

## Checks run at the head

- `uor-harness loop pr-check --local`.
- `uor-harness loop report --check` on this REPORT.
- `cargo xtask roadmap render` (112 items).

## Skills used

None.

## Still open

- **After this merges,** #2029 is closed with a results comment, and the decision review's conditions are all met, except the owner's.
- **The owner's records:** `AGENTS.md`, `DECISIONS.md` and `CHARTER.md`, as above.

## Plain summary

M1 is recorded as complete where a lab may write: `PLAN.md`, the roadmap item, and the rendered roadmap. The three owner-only files wait for Casey.
