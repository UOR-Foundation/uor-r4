# ORDER: integer-retention-rates, claude-227 (parity of the retained trunk on the GPU)

Lab: claude. Item: [integer-retention-rates](../../../../roadmap/items/integer-retention-rates.md) (#3456). Base: `bd9660fa7`.

**Written by hand,** because `loop claim integer-retention-rates` refuses on the false overlap with #3436's `crates/*/README.md`.

## Goal

The item's done-when 3 and 5: `fbs-compare` on (r-drop) s1, the retained artifact's integer readout against the float trunk's D11 readout, under a prep sheet with a fresh FIT and a pre-run review. Then one notebook row and the REPORT. claude-226 met done-when 1, 2 and 4.

## Cancel probe (run first)

- (r-drop)'s step-16,384 checkpoints for seeds 1–3 are in the HF store, with the run's `config.json`.
- `fbs-snapshot` at the base refuses readouts beside `turn_memory.safetensors` (`check_readout_reads`), so the parity needs a copy of the trunk without the memory.
- No open PR touches the scope, and the pod caps have room.

If any fails, stop and report.

Run on 31 Oct at the base:
- `hf_fs ls` shows the five files in each `s{1,2,3}/r-drop-s{n}/models/step-016384`. The trunk is 4,457,344 bytes, and `turn_memory_reply` 48,200,600.
- `check_readout_reads` refuses "the float readouts do not apply the turn memory".
- No open PR touches the scope.
- `uor-pod status`: 0 pods running, 5090s offered from $0.949/h.
- `loop claim` refused, as above.

Result: **go**.

## The design

1. **One pod job on one leased 5090**, from the pushed head:
   - the `fbs_export::` GPU tests;
   - the serving build of `fbs-export` and `fbs-compare`;
   - s1's checkpoint, every file checked against the run's `SHA256SUMS`.
2. **The trunk alone:** a directory holding `config.json` and `model.safetensors`, with the run's `config.json` two levels up, as `fbs-snapshot` reads it.
   - `fbs-snapshot` writes the snapshot and the plain, export and D11 readouts, with retention on from the config.
   - Without the Hamming read and the memory, the parity is the trunk's recurrence alone, where the reading lives.
3. **The decay control:** the same snapshot with `"retain_rates":false`, exactly what an engine before claude-226 served.
4. **Both are exported** and compared with `fbs-compare --parity-only` against the same three readouts. Each artifact's sha256 must equal the comparison's `artifact_sha256`.
5. **A by-product, with no reading taken:** `fbs-snapshot` on each seed's full checkpoint (the memory and the Hamming read, with no readouts). These are the snapshots point-key-integer's run (#3450) exports on the laptop. If they fail, they are reported and the bar is unaffected.
6. **Archive** to the store with a `SHA256SUMS`, then pod down.

The bar and its decision table are registered in the sheet before the run.

## Files in scope

- `docs/harness/prep/integer-retention-rates-parity-2026-10-31.md`
- `docs/labs/integer-retention-rates/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/integer-retention-rates.md`, `docs/roadmap/README.md`, `docs/roadmap/issues.json`
- `docs/loop/tasks/integer-retention-rates/claude-227/**`

## Done when

- **The sheet:** a non-author FIT of FIT, and a pre-run review passed.
- **The run:** archived with its sums, and the bar read from the stored comparisons.
- **One notebook row** in `m4-serving.md`, and the lab record.
- **The item:** set to done when the bar passes. On a FAIL, the decision table names the next unit.
- **The REPORT** gives the negatives first, and a fresh review runs on the diff.

## Stop and ask if

The run needs a code change: that is a new unit with its own scope. Or the pod caps are full.
