# REPORT: m2-pin-train-mask-sheet, claude-99

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2851. References #2030.

## What did not work, first

- **The item could not reach its done-when in this unit.** Its last clause, a non-author FIT, waits on two other items. The sheet's `NOT_FIT` (`antigravity-33`) needs the separate D29-scanned float module (`float-training-module`, #2823) and the D11 float-trunk exporter (`d11-exporter-float-trunk`, #2820) wired into its steps. Neither exists. So this unit gives the item those two dependencies rather than a FIT that would be NOT_FIT again.
- **`loop next` kept offering this item as GO,** ranked by how many items it unblocks (8). But its `depends_on: []` hid the blockers, so every lab that took it would reach the same wall.
- **My first ORDER listed the export-gap band as still to register.** It was already registered at the sheet's line 29 by the PM cover on 24 Oct. The ORDER is corrected, and this unit adds no band.
- **An advisory scope WARN (D76).** `loop pr-check` allows only an item's status line by default, and this unit also edits the item's `depends_on` and text and re-renders the roadmap page. Earlier units (claude-91, claude-93) routed such edits through the umbrella item. Here they ride with the unit whose finding they record, and the decision review covers them.
- **No notebook row.** This unit re-verifies pins and changes a dependency. It measures nothing, and the item names no `notebook:` page.

## What changed

- **`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`:** one line in its source pins records the 25 Oct re-verification at `53b40ecd6`:
  - the mask member `mw-balp/train/response_mask.u8`, extracted from the HF store's `data/ft-balp.tar` (sha256 `b1d22113…473f`), is 7,512,797 bytes with sha256 `b5e7352d…49e3`, matching the pin;
  - `9e904a83` is an ancestor of `main`;
  - the count reference `native_rung0.rs` is byte-identical between `2fd6358bb` and `main`.
- **`docs/roadmap/items/m2-pin-train-mask-sheet.md`:** `depends_on: [float-training-module, d11-exporter-float-trunk]`, and a dated "State on 25 Oct" note at the top of "What is missing". The note records the owner's answer, what the sheet holds, what blocks its FIT, and that the discrete-law sheet the owner also wants is not yet filed as an item. The original text is kept below it.
- **`docs/roadmap/README.md`:** re-rendered.

## Measurements

None. The pin checks are hash and git checks on the laptop. Nothing was built, trained or run on a GPU.

## Checks run at the head

- **Mask member:** `tar -xOf data/ft-balp.tar mw-balp/train/response_mask.u8 | shasum -a 256` gives `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, and `wc -c` gives 7,512,797.
- `git merge-base --is-ancestor 9e904a83… origin/main` holds. `git diff 2fd6358bb origin/main -- …/native_rung0.rs` is empty.
- `cargo xtask roadmap check`: 92 items OK. `roadmap render` was re-run.
- The fresh review, which is also the decision review for the dependency change (D68), is in the PR's review record and the owner log on #2851.

## Skills used

- holding-the-goal: the item's FIT waits on the float module, so the useful move is to point the loop at that module.

## Still open

- **`float-training-module` (#2823)** is the next unblocker, then `d11-exporter-float-trunk` (#2820). After both, the D37 sheet needs its fresh-transfer slice sealed, and then a non-author FIT.
- **The discrete-law rung-0 sheet** the owner also wants has no item yet. Filing it is item-file work for the umbrella item.

## Plain summary

The M2 training sheet's open question was already answered by Casey ("both"), and its pins still check out. It can't be approved yet, because the float training module and its exporter don't exist. The item now says that, so the loop sends labs to build those first.
