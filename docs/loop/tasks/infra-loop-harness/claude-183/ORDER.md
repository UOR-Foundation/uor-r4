# ORDER: infra-loop-harness, claude-183 (item maintenance: the exporter directions, done)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `b8309a64b`.

Setup row: 2 (item-file maintenance: PLAN's owner-direction rows, and the mechanism-page row the exporter item's scope note assigns to this umbrella).

## Goal

`loop next` gives GO on PLAN's owner direction "allow divide, hold on floats for now as you have recommended and fix the exporter", which is still `open`. Both of its conditions are now met on `main`, and so is the D80 direction's:
- **The integer multiply and divide:** `integer-mul-div-serving` is `done`. claude-177 measured decode at 3,029 / 3,041 → 4,829 / 4,827 tok/s, against claude-169's 3,042–3,044.
- **The FBS export gap:** `d11-exporter-float-trunk` is `done` (claude-182, #3370). dual-c131 under `raw_both` is inside the export-gap bar on both seeds, at DEV scope.

**This unit, under one decision review (CHARTER §5.1, D68):**
1. **PLAN:**
   - the two 28 Oct directions, D80's and D81's, go to `done` with what met them;
   - the trunk line's list gains R6, done.
2. **`docs/mechanisms/lut-serving.md`:**
   - the 26–29 Oct FBS export-gap rows (claude-132 to claude-182), which the item's scope note left to this umbrella;
   - its status, READY → ACTIVE, by D72's own condition: "READY until a lawful trained model is exported through it".
3. **`docs/mechanisms/README.md`:** the board's row, its counts and its tree label, to match.

## Cancel probe (run first)

Run at `b8309a64b`:
- **The two directions are `open`** in PLAN.md.
- **Their items are `done`:** `integer-mul-div-serving` and `d11-exporter-float-trunk`.
- **No open PR touches** `PLAN.md` or `docs/mechanisms/lut-serving.md`. The only open PR is antigravity's #3323.
- **`lut-serving.md` has no row after 25 Oct.**

All four held. **Not cancelled.** Class: instrument (none needed).

## Files in scope

- `PLAN.md` (the two direction rows and the trunk-line list)
- `docs/mechanisms/lut-serving.md`
- `docs/mechanisms/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-183/**`

## Done when

1. **PLAN's two directions read `done`,** each with the records that met it, and the trunk-line list names R6 as done.
2. **`lut-serving.md`** has the export-gap rows, each with its record. Its status reads ACTIVE under D72, with the scope (DEV) stated.
3. **The board matches the page.**
4. **A fresh decision review passed,** reading the whole chain. Its owner log is posted on #2028 and on #2904.
5. **`loop pr-check --local` and `loop gates --full` pass.** The PR is merged and verified.

## Stop and ask if

Each of these is a stop-and-decide, never a wait:
- **the decision review finds that a direction's condition is not met:** that row stays `open`, with the gap named;
- **the review reads D72's condition as unmet:** the status stays READY, and the page records the result without moving it.

No step waits on Casey.
