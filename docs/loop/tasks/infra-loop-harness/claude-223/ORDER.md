# ORDER: infra-loop-harness, claude-223 (point-key-integer's scope for its grade, and its unit order)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `f4beaaf79`.

Setup row: 2 (item-file maintenance; a records PR, as in claude-221).

**Written by hand.** `loop claim infra-loop-harness` refuses while Kimi's docs-status-refresh order (#3436) holds `PLAN.md`. This order touches none of that order's files.

## Goal

Amend [`point-key-integer`](../../../../roadmap/items/point-key-integer.md) (#3450) so its next units can run inside its scope:
- **The scope:** add `crates/uor-r4-float-train/src/in_space_long.rs` and `crates/uor-r4-float-train/src/bin/train-in-space.rs`, where the integer arms are graded.
  - The grade's reference reading (templates, exact replies, loops, the census and `generation`) lives there, beside claude-220's `served_grade`.
  - The float crate depends on `uor-r4-integer` without `serving-wasm`, so it cannot decode the integer stack itself. The decode is a `fbs-toks` mode, which the item already lists.
- **The progress:** unit 1 is done (claude-222, #3452, `f4beaaf79`).
- **The unit order:**
  1. done;
  2. the panel decode and the grade, on the laptop, with finite tests;
  3. the (r-drop) snapshots on a pod (`fbs-snapshot` needs CUDA), the export and the TRAIN pass on the laptop, the fresh panel p-x, and the run, under a prep sheet with a fresh FIT and a pre-run review.

## Cancel probe (run first)

- `point-key-integer`'s scope at the base lacks both float files.
- No open order holds `in_space_long.rs` or `train-in-space.rs`.
- claude-222 is merged as `f4beaaf79`.

If any fails, stop and report.

Run on 31 Oct at the base:
- the item's scope names no float crate file (its one `float-train` match is the notebook, `float-training.md`);
- no open PR touches either file;
- claude-222 is merged.

Result: **go**. The probe reads the setting, so it is an instrument check.

## Files in scope

- `docs/roadmap/items/point-key-integer.md`
- `docs/roadmap/issues.json` and `docs/roadmap/README.md` (written only by `roadmap sync --apply` and `render`)
- `docs/loop/tasks/infra-loop-harness/claude-223/**`

## Done when

- `roadmap check` passes, and `render --check` is clean.
- The sync's dry run shows exactly one update (#3450's body) and nothing else, then applies it.
- No other item, owner path, bar or code changes. The REPORT records the commands and the sync's writes.

## Stop and ask if

A file outside scope is needed, the sync plans anything but #3450's update, or the amendment would change the item's bar.
