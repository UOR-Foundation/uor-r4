# ORDER: integer-nearest-readout, claude-232 (end-to-end token parity against the trained model)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `62308fcbc`.

**Written by hand,** because `loop claim` refuses on the false overlap with #3436's `crates/*/README.md`.

## Goal

The item's done-when 3 and 5: integer greedy of (r-drop) step 16,384, nearest head and memory on, against claude-220's stored float greedy replies on DEV and p-w at seeds 1–3, under a prep sheet with a fresh FIT and a pre-run review. Then one notebook row and the REPORT.

Everything runs on the laptop: the integer side is product serving (CHARTER §3.6) under the owner's 31 Oct allowance, and the float side is stored GPU output. No code changes; a needed change is a new unit.

## Cancel probe (run first)

- claude-231's nearest head, `fbs-export --run-config` and `serve_step` are on main.
- claude-220's float greedy replies are stored for DEV (`samples-memory.json`) and p-w (`samples-p-w-greedy-memory.json`) at seeds 1–3.
- No open PR touches the scope.

If any fails, stop and report.

Run on 31 Oct at the base:
- `62308fcbc` holds `head_nearest`, `--run-config` and `serve_step`;
- the six float files are in the store and on the laptop. They are 64 tokens a reply, with DEV at 310 and p-w at 1,202;
- no open PR touches the scope;
- `loop claim` refused, as above.

Result: **go**.

## Files in scope

- `docs/harness/prep/integer-nearest-readout-parity-2026-10-31.md`
- `docs/labs/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-nearest-readout.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/integer-nearest-readout/claude-232/**`

## Done when

- **The sheet:** FIT, and a pre-run review passed, before any decode is read.
- **The run:** sealed with sums, and the bar read from the stored files.
- **The records:** one notebook row, the lab record, and the item's status by the bar's row (done only on a pass). The REPORT gives the negatives first, and a fresh review runs on the diff.

## Stop and ask if

The comparison needs a code change (a new unit), or the decode refuses.
