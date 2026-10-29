# REPORT: infra-loop-harness, claude-183 (item maintenance: the exporter directions, done)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904. The umbrella item stays open; its done-when does not move.

**Speed.** No serving change. The figures quoted are from claude-177 (decode 3,029 / 3,041 → 4,829 / 4,827 tok/s) and claude-182 (4,217–4,320 decode tok/s on the closing artifacts).

## What did not work, first

- **The decision review's first round rejected the record. The status values stood, but I had stated a false premise.**
  - I wrote that no discrete-path model had been exported through D11, so the return to ACTIVE rested on claude-182's float trunk alone. I also added a sentence to the board's ACTIVE definition to cover that.
  - In fact `m1-export-gap-and-toks` (antigravity-96, #2839) exported the four Stage T discrete checkpoints through `lut-discrete/1` on 26 Oct, with an export gap of 0. It is in `m4-serving.md`.
  - My cancel probe checked `lut-serving.md`'s table for rows after 25 Oct. It did not check the notebook for exports outside the item I came from.
  - The fix cites antigravity-96 as the first export through D11 and claude-182 as the second, adds its row, and drops the definition sentence.
- **My R6 line implied that the passing models used the fixed-order backward.** dual-c131 was trained before #3367, with the old `atomicAdd` backward. The only fixed-order configuration, bfixed-c179, misses. PLAN and the page now say so, and R6's "with repeats" is listed as open.
- **The first draft also had smaller errors:**
  - "7 of 7 WebAssembly" without its bulk-memory condition;
  - the snap localisation credited to claude-132 to claude-136, when it is claude-180's. My first rewording then called the export "faithful", which claude-136's "not cleared" contradicts;
  - two leftover READY sentences on the page.

## What changed

- **`PLAN.md`:**
  - The two 28 Oct owner directions read `done`, each with what met it:
    - D80's: the audits allow the integer multiply (claude-175), and the kernels use it, at 1.59× the speed (claude-177).
    - D81's: `integer-mul-div-serving` is done, and `d11-exporter-float-trunk` is done at DEV scope (claude-179 to claude-182).
  - The trunk line's list gains R6, done, with its open part.
- **`docs/mechanisms/lut-serving.md`:**
  - **Status:** ACTIVE by D72's own condition, met first by antigravity-96 (26 Oct, discrete) and then by claude-182 (29 Oct, float trunk).
  - **"What has been tried":** four new rows, for the discrete export, the FBS export path, the snapped export gap, and R6 with the unsnapped contract.
  - **The never-tested and frozen sections** match.
- **`docs/mechanisms/README.md`:**
  - the row moves to ACTIVE;
  - the counts are 10 / 7 / 11;
  - the tree's label and class move, and so do the coverage markers;
  - the "Where the audit questions a status" list notes the return to ACTIVE.

## Measurements

None: no run. Every figure is quoted from a REPORT or a notebook row, and each names its source.

## Checks run at the head

- **`uor-harness loop gates --full`:** 11 of 11 green.
- **`uor-harness loop pr-check --local`:** PASS. Its only earlier FAIL was this REPORT, missing.
- **The board's rows,** counted per section: ACTIVE 10, READY 7, PARKED 11. They match the headings and the meaning table.
- **The decision review (D68), fresh:**
  - **Round 1:** REJECT on the record (one High, one Medium, five Lows), with the values approved.
  - **Round 2:** it confirmed every round-1 finding fixed, and every number against the notebook. It rejected on one new sentence: "claude-132 to claude-136 showed that the export is faithful". That overstated the record, since claude-136 found the exporter "not cleared".
  - **The sentence now uses round 2's own wording,** which I checked against claude-132's and claude-136's notebook rows ("at most 8 in net"; "the exporter is not cleared, but it cannot change these verdicts"). No other text changed after round 2.

## Skills used

- **holding-the-goal.** The unit records what met each direction, and what did not:
  - R6's repeats are open;
  - the passing models predate the fixed-order backward;
  - the float trunk's fresh split is unread.

## Still open

- **The confirming run** on new dual-c131 seeds trained with the fixed-order backward. It is R6's "with repeats", a GPU training unit under `d11-exporter-float-trunk`'s successor. It needs its own item.
- **R7** (`docs/model/pipeline-map.md`): a text path for the FBS artifact in `uor-chat --stack`. It has no item yet.
- **Pages that were already stale before this unit,** found by the decision review and out of this ORDER's scope:
  - `README.md:89`, `docs/README.md:87`, `docs/faq.md:101` and `docs/transformer-vs-geometry.md:19,69` give the 19 Oct board counts;
  - `docs/transformer-vs-geometry.md:38,47` still call D11 unexported;
  - `docs/mechanisms/spiralcore-fbs.md`'s READY line says "Never trained".

## Plain summary

Two of the owner's instructions from 28 October are now finished, and the plan says so. One was to allow the integer multiply instruction, which made the served model about 1.6 times faster. The other was to allow divide and fix the exporter, which now brings one trained configuration's integer copy within the allowed margin of the trained model.

The page for the integer serving engine goes back to "active". The owner had set it to "ready until a lawful trained model is exported through it", and that has now happened twice: a discrete model on 26 October with no difference at all, and a float-trained model on 29 October.

The first draft missed the 26 October export. A fresh review caught it, and the record is corrected.
