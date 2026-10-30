# REPORT: infra-loop-harness, claude-209 (D82: M1's held-out-phrasing bar)

Lab: claude. Order: [ORDER.md](ORDER.md). Setup row: 2. References #2028 and #2029.

**Result: the owner's decision is recorded.** M1 adds bar (f): on a fresh held-out-phrasing panel, the same saved model's integer top-1 must beat both the unigram table and the T2 successor table, on each of two seeds, with bar (e)'s paired band. It is written in:
- CHARTER §4 item 8;
- the D82 row;
- the change log;
- PLAN.md (the owner direction, the M1 bar, and the M1 path's step 9);
- a new roadmap item, `m1-heldout-phrasing-transfer`, that measures it.

**Speed.** None measured; this unit is a record.

## What did not work, first

- **M1 is no longer one step from closing.** antigravity-110 (`m1-close-saved-model`) reported `PASS_M1_ALL_FIVE_BARS` on its saved two-seed discrete artifact, and M1 (#2029) is still open. Bar (f) keeps M1 open until that saved model also passes on a held-out-phrasing panel. This is the decision's direct consequence. It was named to the owner in the question that allowed the owner-file edits, and it is written into each record.
- **The first edit was refused.** The loop guard refuses lab edits to owner paths (`CHARTER.md`, `docs/integration/DECISIONS.md`, `docs/rules/changes.md`). The owner's permission for D80 did not extend to D82, so he was asked again, and chose "Yes, write them (Recommended)". The edits went in as D80's did:
  - copies were edited under `local/claude/d82/`;
  - one patch, [`d82-owner-paths.patch`](d82-owner-paths.patch), was applied with `git apply`;
  - a fresh review checks it line by line against his words.
- **Bar (f) cannot be measured yet.** It needs:
  - a new panel and its census against the saved model's own TRAIN (sha `3dee858f…`, whose size is in question, claude-202);
  - `fresh-score` to read an unpinned panel;
  - a GPU run.

  The new item names each.

## What changed

- **`CHARTER.md`, §4 item 8:** bar (f), after D39's text, tagged `[D82, owner 29 Oct]`. It says only what the owner chose: a fresh held-out-phrasing panel, drawn by `heldout-panel` at a new seed with its own census, the unigram and T2 tables, two seeds, and bar (e)'s band.
- **`docs/integration/DECISIONS.md`:** the D82 row, with:
  - the owner's words verbatim;
  - the alternatives he did not choose;
  - its reversal cost: remove (f), and close on (a)–(e).
- **`docs/rules/changes.md`:** the dated entry, newest first.
- **`PLAN.md`:**
  - the owner direction's row, `done`, with both answers verbatim;
  - the M1 bar paragraph;
  - step 9 in "The path to M1".
- **`docs/roadmap/items/m1-heldout-phrasing-transfer.md`** (new, `ready`, `needs_gpu`).
  - **Its steps:** the panel at a new seed (claude-204 used 2041–2043), the census against the saved model's TRAIN with a `train,train` control, and the GPU scoring with bar (e)'s band `max(24, ⌈2√(b+c)⌉, |M(1) − M(2)|)`.
  - **Its scope:** the scorer instrument, its own prep, lab and task folders, and the notebook.
- **`docs/roadmap/README.md`,** re-rendered: 112 items.
- **This task folder:** the ORDER, with the owner's words and the `## Owner approval` section; the patch; and this REPORT.

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 112 items OK. `cargo xtask roadmap render`: written.
- `loop report --check`, `loop pr-check --local` and `loop gates --full`: below, before review.

## What it unlocks

- **The item `m1-heldout-phrasing-transfer`,** which any lab may claim. When it passes, M1 can close on (a)–(f).
- **P-dev stays reserved** for one-model-end-to-end's training-run decisions.

## Skills used

None was invoked this unit.

## Still open

- **The item's issue:** `cargo xtask roadmap sync --apply` after merge.
- **Measuring bar (f).**

## Plain summary

Casey decided that the first milestone must also show the model handles questions worded in ways the training data never used, not only new names and numbers inside familiar wording. That rule is now in the project's rule book, its decision list, its change log and its plan, with his words. A new task describes how to test the finished milestone model against it. Until that test passes, the first milestone stays open, even though the model already passed the earlier five checks.
