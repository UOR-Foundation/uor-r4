# REPORT: infra-loop-harness, claude-156

Lab: claude. Order: [ORDER.md](ORDER.md). Doc-only: no code, no run, no GPU.

The owner's 28 Oct pivot is now on the roadmap. Three items are `ready`, and PLAN records the owner's three decisions verbatim.

## What did not work, first

- **The map could not be revised in place.** `uor-harness loop claim pipeline-map` refused because the item is `done` ("not ready"). So the revision is a new item, `pipeline-map-revision`, depending on `pipeline-map`, as claude-147 did when it added `pipeline-map` through this umbrella.
- **`uor-harness check` exits 1** at this head, on one error that is not this unit's: `prep.sheet_shape` in `docs/harness/prep/m1-close-saved-model-2026-10-26.md`. No rule names a file this unit wrote.

## What changed

- **`docs/roadmap/items/fbs-cost-profile.md`** (new). One training step and one served token, profiled by stage, each summing to the measured time within 10%, with the top three costs named. A sheet with a fresh FIT comes before its GPU part.
- **`docs/roadmap/items/pipeline-map-revision.md`** (new). The pipeline map revised with:
  - claude-149 to claude-155;
  - the storage diagnosis;
  - cost per token first;
  - the archived prime and angular router, read in full;
  - a re-ranked order of work.
- **`docs/roadmap/items/route-plus-memory.md`** (new). The decisive run. The turn anchor's first `k` ids (or a learned code of the bag) key an exact table built from TRAIN, and the stored token's point moves the readout through a zero-start weight. It is barred against (a), a no-memory control (L-010) and claude-155's bag, at two seeds.
- **`PLAN.md`:**
  - the three 28 Oct owner decisions, verbatim, newest first, each `open`;
  - the three items listed in order under the side lines, as the trunk line's pivot.

## Checks run at the head

- **`uor-harness check`:** no rule names this unit's files. The one error is in another lab's sheet, and it predates this unit.
- **`uor-harness loop report --check`:** RESULT.
- **A fresh in-unit review of the doc change:** RESULT.

## Done when

1. **Met.** Three items, each with the owner's words, what is missing with sources, a done-when, a scope and `status: ready`.
2. **Met.** PLAN carries the three decisions and lists the three items.
3. **Met on merge.**

## Plain summary

Casey voted to pivot to the roadmap diagnosed in claude-155, and to keep training in float with an exact integer export, judging every piece by its cost per token. This unit writes that roadmap as three items in order:
- a cost profile of training and serving;
- a revision of the pipeline map, including the old prime and angular router;
- one decisive run that connects the trunk to an exact memory.
