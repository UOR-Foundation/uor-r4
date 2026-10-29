# ORDER: integer-hamming-read, claude-200 (step 4: the export carries the read)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Issue: #3385. Base: `134f7ae48`.

## Goal

The item's step 4. It advances step 2 of PLACE.md's standard path: one snapshot that carries every read a checkpoint holds.
- **Why now:** claude-194's checkpoints hold `hamming_read.safetensors`, and since claude-198 `fbs-snapshot` refuses them. Without this step there is no export of the working model.

1. **`fbs_hamming_snapshot(reads, layers)`**, a `pub fn` in `fbs_export.rs`. It makes an `FbsHammingSnapshot`:
   - `heads` from the margin's length; `depths` from `HAMMING_DEPTHS`;
   - each read's rotations, margin and weight flattened row-major, each through `finite_table`.
   - **It refuses:**
     - a `step_rotation` (the type spin is not ported);
     - a trunk whose layer count is not `HAMMING_READOUT_DEPTH` (claude-199's rule);
     - a shape other than `(width/4, 4)`, `(heads)` or `(2, width)`.
2. **`check_checkpoint_reads`** accounts for `hamming_read.safetensors`.
3. **`fbs-snapshot`** loads the read with `HammingReads::load` on the CPU and sets `snapshot.hamming`.

## Cancel probe (run first)

- **Does the export already carry the read?** No: `fbs_trunk_snapshot` writes `hamming: None`, and `fbs-snapshot` refuses the file (at `134f7ae48`).
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `docs/loop/tasks/integer-hamming-read/**`

## Done when

1. **On the CPU:** a `HammingReads` with random parameters, saved and loaded, snapshots to exactly its tensors, and its JSON reads back equal.
2. **Refused, by text:** a `step_rotation`, a 4-layer trunk, and a non-finite value.
3. **`check_checkpoint_reads`** passes a directory with the read, and still refuses the four other reads.
4. **Negative gates:** each refusal set aside turns its test red.
5. **Checks:** the float suite passes; the stub-nvcc check of `fbs-snapshot` builds; `loop gates --full` is green.
6. A REPORT and a fresh review. The item stays `ready`, at step 5 (parity).

## Stop and ask if

Stop and decide (never wait) if:
- the float and integer layouts of a rotation or weight differ: then fix the layout in this unit, with a test;
- a checkpoint can hold a Hamming read at another depth list: then refuse it by name.
