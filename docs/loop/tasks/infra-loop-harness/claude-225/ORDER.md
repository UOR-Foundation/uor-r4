# ORDER: infra-loop-harness, claude-225 (record the integer-retention-rates item)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `18c4c3da7`.

Setup row: 2 (item-file maintenance; a records PR, as in claude-221 and claude-223).

**Written by hand.** `loop claim infra-loop-harness` refuses while Kimi's #3436 holds `PLAN.md`. This order touches none of that order's files.

## Goal

Record the gap that blocks `point-key-integer`'s run as its own item, `integer-retention-rates`, and make the run depend on it.

**The gap,** found reading the code before any run:
- **The float trunk:** with `retain_rates` on (`geometric_stack_native.rs`: "On (`in_space_long`, 27 Oct 2026): a lane carries `1 - 2^-(k+1)`"), each recurrence lane carries 1 − 2^−(k+1) of its state, with keep = √(1 − λ²). Every one-model-end-to-end checkpoint since 27 Oct trained so, (r-drop) included (its run's `config.json`: `"retain_rates": true`).
- **The integer artifact:** its geometric recurrence (`stack/session.rs`, about line 3637) serves only the off reading, λ = 2^−(k+1). The exporter (`fbs_export.rs`, `fbs-snapshot`) carries no flag.
- **So** an export of such a checkpoint serves a different recurrence from the one trained.

## Cancel probe (run first)

- No item or report mentions serving retention in the integer artifact (`grep -i retention` over `docs/roadmap/items` and the REPORTs).
- The integer recurrence has no retained form, and the snapshot no flag.
- The roadmap sync's dry run plans only #3440's and #3450's body updates.

If any fails, stop and report.

Run on 31 Oct at the base:
- no item mentions retention;
- the three REPORTs that do (claude-142, -151, -152) concern training only;
- the integer recurrence and the snapshot have no retained form or flag;
- the dry run plans exactly #3440's and #3450's body updates.

Result: **go**. The probe reads the setting, so it is an instrument check.

## Files in scope

- `docs/roadmap/items/integer-retention-rates.md` (new) and `docs/roadmap/items/point-key-integer.md` (`depends_on`, and a line in "The units")
- `docs/roadmap/issues.json` and `docs/roadmap/README.md` (written only by `roadmap sync --apply` and `render`)
- `docs/loop/tasks/infra-loop-harness/claude-225/**`

## Done when

- `roadmap check` passes, and `render --check` is clean.
- **The sync:** its dry run plans one create (the new item), #3450's body (the new `depends_on`, plus claude-223's deferred update) and #3440's body. Then it is applied.
  - **#3440 is decided here, not left:** its body catches up with DeepSeek's own item file on main (`d2f0354a9`). The sync writes only text generated from files already on main, so no lab's content changes. Leaving it would block every later sync.
- No other item, owner path, bar or code changes. The REPORT records the commands and the sync's writes.

## Stop and ask if

The sync plans anything else, or the item needs a decision row rather than an item file.
