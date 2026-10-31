# ORDER: integer-hamming-read, claude-234

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md) (#3385). Base: `5ed1018fe`.

## Goal

Make step 5, the read's float parity, traceable, so the next unit can run it. Casey chose this step first (31 Oct 2026, in chat, verbatim): "Trace attention first".

claude-233 (#3465) measured the gap. On (r-drop) step 16,384, the trunk alone agrees with its float D11 forward at s1 and s3 (d 5 and 11 of 1,536 sequence positions). With the Hamming read, the trunk does not (694 and 823).

Two leads come from the code, and the trace must tell them apart:
- **The tie rule.** The float `choose_sorted` (`flock_read_hamming.rs`) keeps the first 16 of a GPU arg-sort, whose tie order is not specified. The integer `nearest` (`stack/hamming.rs`) keeps ascending distance, then ascending position.
- **The read's parameters.** The float D11 readout loads `hamming_read.safetensors` at full f32 (`fbs-snapshot.rs:97`). `round_tables_as_exported` rounds only the trunk's tables. The artifact holds the rotations and weights at 2^-24 and the margins at 2^-16.

A per-head trace needs the float read's own codes and kept positions. Those exist only inside `flock_read_hamming.rs`'s `apply`, which is not in this item's scope. The harness reads an item's scope from origin/main, so the scope change merges alone first, as claude-214 (#3422) did for one-model-end-to-end.

This unit:
1. adds `crates/uor-r4-float-train/src/flock_read_hamming.rs` to the item's scope;
2. records step 5's trace method in the item, so the next unit's sheet starts from it.

## Cancel probe (run first)

The item's cancel condition reads: "The integer runtime already has a Hamming read on main, or the float read is withdrawn."
- The read is on main (claude-189, claude-194, claude-195, claude-200), but its done-when 2, parity, is not met: claude-233's row R. The condition was written before the port, so it is stale wording, not a cancel.
- The float read is not withdrawn: (r-drop) trains with it.

Result: not cancelled. Class: instrument (the stale wording is amended here).

## Files in scope

- `docs/roadmap/items/integer-hamming-read.md`
- `docs/loop/tasks/integer-hamming-read/claude-234/**`

## Done when

1. The item's `scope:` includes `crates/uor-r4-float-train/src/flock_read_hamming.rs`. The cancel condition names parity, not the read's existence.
2. The item's step 5 records the trace:
   - the float read's codes and kept sets, captured inside `apply`;
   - the integer read's, from the same artifact;
   - the per-(position, head, read) kept-set share, split by cause;
   - the two leads above.
3. `cargo run -p xtask -- roadmap render` leaves no diff; the REPORT is written; a fresh review approves; the PR is merged and verified.

## Stop and ask if

- the render or a harness check needs a file outside this scope: decide by decision review, and keep the change docs-only;
- the read's design in the item turns out to disagree with `flock_read_hamming.rs` on main: record what main does, and carry it to the next unit.

(Stop and decide; never stop and wait.)
