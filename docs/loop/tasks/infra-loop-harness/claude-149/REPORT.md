# REPORT: infra-loop-harness, claude-149

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: item-file maintenance, no model run.

## What did not work, first

- **Only two of the map's eight remaining units get items here.** U4 and U6–U9 depend on what U2 and U3 find, so their items are written after those results, not now.
- **U5 gets no new item,** by design. The deterministic FBS backward is already the open item `d11-exporter-float-trunk`'s next unit (claude-140's REPORT: "make the FBS backward reduction deterministic"), inside that item's scope.
- **The first draft made `integer-hamming-read` depend on `d11-exporter-float-trunk`,** which is open, so `loop next` would have held the new item as blocked. It now depends on `float-training-module` and `spiralcore-fbs-offline-float-sheet`, both done, and its body says why the exporter's open export gap is not its gate.

## What changed

- **`docs/roadmap/items/one-decoder.md` (new), unit U2:** make the served decoder the measured one. The writer and whole-table nearest disagree on one checkpoint (345 against 7,813); `voronoi`'s satisfiable margin gave the best writer score (7,906 at 16,384 steps), and the writer's collapse after step 16,384 is untested under it. Done when a pre-registered `voronoi` run past step 16,384, at two seeds, reads both decoders at every checkpoint and the REPORT records which decoder serves.
- **`docs/roadmap/items/integer-hamming-read.md` (new), unit U3:** serve claude-146's Hamming read from the integer artifact on the laptop. Done when the D11 runtime runs the read, matches the float forward's picks and ids on DEV within the D11 forward's tolerance on a stored checkpoint, and `fbs-toks` gives decode tokens/s with and without it on this M1.
- **`docs/roadmap/README.md`:** re-rendered with the two rows.

## Measurements

None: no run. Every figure in the two items is the map's or its source's, cited in each item.

## Checks run at the head

- `cargo xtask roadmap check`: 99 items OK.
- `cargo xtask roadmap render --check`: the README is current.
- `uor-harness loop report --check` and `loop pr-check --local`: see the review record.

## Skills used

- **holding-the-goal.** The unit follows the owner's open 27 Oct direction through the map's own order of work, and writes items only for the units the map says run first.
- **root-cause-debugging.** The dependency that would have blocked `integer-hamming-read` was found by reading the dependency's status before the render, not after a refused claim.

## Still open

- Items for U4 and U6–U9, after U2's and U3's results.
- The owner direction stays `open` in PLAN until the map's order of work has carried the missing steps.

## Plain summary

The project's new end-to-end map listed the next pieces of work in order. Two of them had nowhere in the task system to be picked up from. This adds them: making the model's served decoder the same one its scores are measured with, and running the newly working look-back on the laptop's integer engine. A third piece, making training runs repeatable, already had a home and needed nothing new.
