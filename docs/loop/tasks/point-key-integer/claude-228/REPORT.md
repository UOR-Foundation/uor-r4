# REPORT: point-key-integer, claude-228 (the run's instruments: done-when 2)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Order: [ORDER.md](ORDER.md), written by hand, since `loop claim` refuses on #3436's false glob overlap.

This unit builds the item's done-when 2 on the laptop, with finite tests and no run. The run is the next unit's, under its own sheet.

## Tokens per second

No measurement.
- The served step is unchanged: the new checks run in `panel_reply` and in `fbs-export --point-key`, off the step's path.
- The serving audit's verdict is unchanged.

## What did not work, first

- **Two of done-when 2's three instruments did not exist at the base.**
  - Nothing compared a served name with a reference.
  - `fbs-export --point-key` parsed the `/8` stack it wrote, but never compared the table it read back with the one it built.
  - The run's plan (claude-225) had assumed them, so this unit comes before the run.
- **The wiring has no failing test.**
  - The check in `panel_reply`, and the read-back call in `fbs-export`, are each one line. Each is read in the diff, and no test fails with it removed.
  - The checks themselves are pinned (gates 3 and 4). `name()` and the reference scan read one table, so no honest stack makes them disagree. Pinning the wiring would take a fault injected into the search.
- **The WASM library build warns that `stack_whole_table_nearest` is never used.**
  - That function is used only on a path the WASM target does not build (`session.rs:3428`), and this diff does not touch it.
  - claude-226's REPORT called the WASM build "clean" because its filter matched only `warning: unused`, which hid this warning.
- **The audit counts 116 reachable functions**, where claude-226 reported 123. The difference is not traced; the verdict is FULL PASS.

## What worked

- **(i) The reference scan.**
  - `ServedPointKey::reference_name` is a second, plain implementation of the rule: every row's L1 distance into a list, the least, the targets at it counted in a `BTreeMap`, then the largest count, ties to the smaller id. It shares no code with `name()`.
  - `panel_reply` checks every served opening against it (`check_served_point`), with the opening step's `query_q12`, the buffer `name()` read. A mismatch stops the decode.
  - `fbs-toks --replies` prints `reference_checked`: the keyed count, since a finished decode checked every keyed opening.
- **(ii) The read-back.** `fbs-export --point-key` refuses unless the table read back from the `/8` bytes it writes equals the built `PointKeyImage` (`check_point_key_read_back`).
- **(iii) BOS.** `name()` already refused a BOS name at the step. A test now pins it, from a table that skipped the image's check.
- **`cargo fmt --check -p uor-r4-integer` is clean.** `bin/fbs-export.rs` and `bin/fbs-toks.rs` are formatted (claude-224's drift).

## What changed

- **`crates/uor-r4-integer/src/stack/point_key.rs`:**
  - `reference_name`, `check_served_point` and `check_point_key_read_back`;
  - the check in `panel_reply`;
  - two tests.
- **`stack/session.rs`:** `point_key_reference`.
- **`stack/mod.rs`:** the export.
- **`stack/tests.rs`:** two tests.
- **`bin/fbs-export.rs`:** the read-back, and formatting.
- **`bin/fbs-toks.rs`:** `reference_checked`, and formatting.
- **`docs/roadmap/items/point-key-integer.md`:** "The units" records this unit and the run after it. That run needs no pod: the snapshots are in the store (claude-227).

## Measurements

- **The integer crate's library tests** (`--features serving-wasm`): 390 pass and 1 is ignored, against 386 at the base.
- **The serving audit** (`audit_zero_matmul_serving.py --stack`): FULL PASS, with 0 forbidden instructions across 44 symbol ranges and 116 reachable functions.

## Checks run at the head

- **Four negative gates,** each failing at its own assertion with its rule set aside, and passing when restored:
  - **G1:** the reference's tie rule (`>=` for `>`), at `point_key.rs:462`;
  - **G2:** the BOS refusal in `name()`, at `point_key.rs:494`;
  - **G3:** `check_served_point`'s comparison, at `tests.rs:5482`;
  - **G4:** `check_point_key_read_back`'s comparison, at `tests.rs:5497`.
- **The random test** covers 3,200 queries over 400 random tables. More than 100 of them tie in counts at the least distance, which the test asserts.
- **The release builds** of `fbs-export` and `fbs-toks`, and the WASM library check, all finish.
- `xtask roadmap check`: 117 items OK. The sync dry run plans only #3450's body update, which is applied after the merge.

## Skills used

None.

## Still open

- **The run** (unit 4), on the laptop, under a prep sheet with a fresh FIT and a pre-run review. Its steps:
  - export the three stored snapshots to `/7`, checking `retain_rates` in each;
  - `--point-key` over TRAIN (212,968 rows expected);
  - draw p-x with its census;
  - the prompts;
  - the 24 decodes;
  - the grade;
  - tok/s.

## Plain summary

Before the scored run, this unit adds three checks:
- that the model's "nearest earlier opening" lookup gives the same answer as a second, plainly written lookup, every time it is used;
- that the lookup table saved in the model file reads back exactly as it was built;
- that the lookup never names the start-of-text marker as an answer.

If any check fails during the run, the run stops.
