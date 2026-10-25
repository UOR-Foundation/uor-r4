# REPORT: m2-pin-train-mask-sheet, claude-100

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #2851. References #2030.

## What did not work, first

- **The item stayed open after its done-when was met.** Its discrete-law branch was met once two things were on record:
  - the discrete rung-0 sheet (13 Oct, FIT at Amendment 2), carrying the pins;
  - the owner's answer (24 Oct).

  Nothing recorded the item as done, so `loop next` kept offering it as GO, ranked as unblocking 8 items. claude-99 (#3196) recorded the owner's answer but read the discrete-law sheet as "not yet written". This unit finds it is the 13 Oct discrete rung-0 sheet, which names itself the Amendment 6 record's successor.
- **The discrete sheet is longer than one page,** and is titled M1. It is the rung-0 sheet on the discrete law, which the old sheet's Amendment 7 asked for. Its FIT (design) is from 14 Oct. The decision review judges whether that still meets "a one-page sheet that is ready for an owner hold-release decision".
- **No notebook row.** This unit records a status from existing records and measures nothing.

## What changed

- **`docs/roadmap/items/m2-pin-train-mask-sheet.md`:** `status: done`, and a "Done (25 Oct)" note before "Done when". It names:
  - the owner's answer;
  - the discrete-law sheet, with the pins it carries (parent tar, mask member, count, source, CLI);
  - its FIT;
  - that the D37 sheet goes on separately, its FIT after #2823, #2820 and its fresh slice.
- **`docs/roadmap/README.md`:** re-rendered (done 38 → 39).

## Measurements

None.

## Checks run at the head

- **The discrete sheet's pins** match the ones re-verified on 25 Oct (claude-99): the tar sha256 `b1d22113…473f`, the mask `b5e7352d…49e3` at 7,512,797 bytes, and the count 2,425,925 (lines 47-50 and 80-86). Its Verdict (line 369 on) reads `TEST FITNESS: FIT (design), Amendment 2`, from Claude and Codex as non-authors.
- `cargo xtask roadmap check`: 92 items OK.
- The fresh review, which is also the decision review for the status change (D68), is in the PR's review record and the owner log on #2851.

## Skills used

- holding-the-goal: an item whose done-when is already met is closed with its evidence, so the loop's ranking moves on to work that is really open.

## Still open

- **The D37 float sheet** needs `float-training-module` (#2823), then `d11-exporter-float-trunk` (#2820), then its sealed fresh-transfer slice, then a non-author FIT.
- **`m2-train-grounded-replies`** still depends on `m2-donor-corpus-generation`, `m2-fresh-heldout-panel` (done), `m1-close-saved-model` and `m1-d11-exporter-discrete`.

## Plain summary

The M2 mask-sheet item was finished in substance: Casey had answered its question, and a reviewed discrete-training sheet already carried its pins. Nothing had marked it done. Now it is, so the loop stops sending labs to it.
