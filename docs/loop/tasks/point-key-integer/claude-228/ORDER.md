# ORDER: point-key-integer, claude-228 (the run's instruments: done-when 2)

Lab: claude. Item: [point-key-integer](../../../../roadmap/items/point-key-integer.md) (#3450). Base: `cd328d3af`.

**Written by hand,** because `loop claim` refuses on the false overlap with #3436's `crates/*/README.md`.

## Goal

Build the three run-time instruments of the item's done-when 2, so the run's sheet (the next unit) can rely on them.

Done-when 2 reads: "The instrument holds, exactly, on every panel run. Any one of these failing is a fault, and the unit stops":
- (i) at every served opening, the integer search's name equals a test-side reference scan of the same integer query against the same integer table;
- (ii) the table read back from the artifact equals the one built, byte for byte;
- (iii) no served name is BOS.

## Cancel probe (run first)

- At the base, no run-time check does (i) or (ii).
- (iii) holds at serve time: `ServedPointKey::name` refuses a BOS name, so a served BOS fails the step.
- The run's snapshots exist (claude-227's store folder), so nothing else blocks the run once the instruments exist.
- No open PR touches the scope.

If any fails, stop and report.

Run on 31 Oct at the base:
- `panel_reply` records `last_point_key()` and compares it with nothing.
- `fbs-export --point-key` parses the `/8` stack it writes, but never compares the table it reads back with the one it built.
- `name()` returns "the point key named no token, or BOS" on BOS.
- `results/claude/integer-retention-rates-parity-2026-10-31/out/full/snapshot-s{1,2,3}.json` are in the store.
- No open PR touches the scope.
- `loop claim` refused, as above.

Result: **go**.

## The design

1. **(i) The reference scan.**
   - `ServedPointKey::reference_name(query)` is a second, plain implementation of the rule:
     - every row's L1 distance at Q12 into a list;
     - the least of them;
     - a count of the targets at that distance in an ordered map;
     - the largest count, ties to the smaller id.

     It shares no code with `name()`.
   - `panel_reply` checks every served opening against it, with the step's own `query_q12`, the buffer `name()` read. A mismatch is an error, so `fbs-toks --replies` stops.
   - `fbs-toks` prints how many served openings were checked.
2. **(ii) The read-back.** `fbs-export --point-key` reads the table back from the `/8` bytes it is about to write (`point_image`), and refuses unless it equals the built `PointKeyImage`.
3. **(iii)** stays where it is (`name()`). A test pins that a BOS name is refused at the step.
4. **`cargo fmt`** on `bin/fbs-export.rs` and `bin/fbs-toks.rs`, claude-224's drift, since this unit touches both.
5. **The item's "The units"** records this unit, and that the snapshots are in the store.

## Files in scope

- `crates/uor-r4-integer/src/stack/{point_key.rs, session.rs, mod.rs, tests.rs}`
- `crates/uor-r4-integer/src/bin/{fbs-export.rs, fbs-toks.rs}`
- `docs/roadmap/items/point-key-integer.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/point-key-integer/claude-228/**`

## Done when

- **Tests,** each with a negative gate:
  - the reference scan names as `name()` does, over random tables with ties;
  - a served opening whose name differs from the reference's is refused;
  - a table that does not read back as built is refused;
  - a BOS name is refused at the step.
- **The suites:** `cargo test --release --locked -p uor-r4-integer --features serving-wasm --lib` passes. So do the serving audit, the WASM build and `cargo fmt --check -p uor-r4-integer`.
- **The REPORT** gives the negatives first, and a fresh review runs on the diff.

## Stop and ask if

The check needs a file outside scope, or the audit refuses the step's code.
