# ORDER: point-key-integer, claude-229 (the run, on the laptop)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Base: `c71e85788`.

**Written by hand,** because `loop claim` refuses on the false overlap with #3436's `crates/*/README.md`.

## Goal

The item's done-when 3–6, under a prep sheet with a fresh FIT and a pre-run review.
- (r-drop) step 16,384 at seeds 1–3, exported to the integer artifact. `/7` is the integer greedy arm, and `/8` adds the served point key.
- Both are decoded on the panels p-x (fresh), p-w, p-v and DEV.
- The grade reads form C's bar on p-x, with parity against claude-220's float key reported, and tok/s.

All of it runs on the laptop's bit-exact integer engine, under the owner's 31 Oct allowance for point-key-integer's scoring. No code changes: a needed change is a new unit.

## Cancel probe (run first)

- claude-228's instruments are on main.
- (r-drop)'s three full snapshots are in the store (claude-227).
- Seed 2131 is used by no panel.
- No open PR touches the scope.

If any fails, stop and report.

Run on 31 Oct at the base:
- `c71e85788` holds `check_served_point` in `panel_reply` and `check_point_key_read_back` in `fbs-export --point-key`.
- The store lists `results/claude/integer-retention-rates-parity-2026-10-31/out/full/snapshot-s{1,2,3}.json`.
- `seed=2131` appears in no sheet or lab page; the seeds used are 1000, 2026, and 2051 to 2121 in steps of 10.
- No open PR touches the scope.
- `loop claim` refused, as above.

Result: **go**.

## The steps

1. **The panel p-x:** `heldout-panel out=p-x cell=dev,dev conversations=1200 seed=2131 protocol=2`.
   - Stored with its `SHA256SUMS` before any model reads it.
   - Its census is `train-in-space --census 1`, a count with no model, with Fresh V2 counted from its preservation archive as claude-202's census did.
2. **The sheet,** `docs/harness/prep/point-key-integer-run-2026-10-31.md`.
   - It registers done-when 4's rows before p-x is read: carries at all three seeds, at some, or at none. Row 0 is an instrument fault.
   - It also gives the parity classes, the cap, the power, the commands, the costs, a fresh FIT and a pre-run review.
3. **The artifacts.**
   - `fbs-export` of each snapshot gives `/7`, with `rec_gates` checked as `geometric-retention`.
   - `fbs-export --point-key` over TRAIN gives `/8`, with 212,968 rows expected.
   - Each printed sha256 is checked against the file every later step loads (claude-228's review).
4. **The prompts** (`--panel-prompts`), then **24 decodes** (`fbs-toks --replies`, 2 arms × 4 splits × 3 seeds), then **the grade** (`--integer-grade`).
5. **Tok/s:** decode against context, with the table search counted, and the resident memory.
6. **The records:**
   - the outputs sealed to the store with sums;
   - the lab record;
   - one notebook row (`float-training.md`, the item's notebook);
   - the REPORT;
   - the item's status by the bar's row.

## Files in scope

- `docs/harness/prep/point-key-integer-run-2026-10-31.md`
- `docs/labs/point-key-integer/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/point-key-integer.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/point-key-integer/claude-229/**`

## Done when

- **The sheet:** FIT, and a pre-run review passed, before p-x is read.
- **The run:** every done-when 2 instrument holds, or the run stops as row 0. The outputs are sealed with sums, and the bar is read from the stored grade.
- **The records:** the notebook row, the lab record and the REPORT, with the negatives first and tok/s first. A fresh review runs on the diff.

## Stop and ask if

The run needs a code change (a new unit), or an instrument stops a decode or an export (row 0: stop, report, repair in a new unit).
