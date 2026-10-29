# ORDER: integer-hamming-read, claude-199 (the scope step 4 needs)

Lab: claude. Item: [integer-hamming-read](../../../../roadmap/items/integer-hamming-read.md). Issue: #3385. Base: `5520524bb`.

## Goal

A scope decision for the item's step 4, "the exporter reading `hamming_read.safetensors`". It advances step 2 of the standard path in PLACE.md: one snapshot that carries every read a checkpoint holds, or refuses it by name. It also serves the owner's direction of 29 Oct: "a valid (and somewhat coherent and standardized) training/serving path".

**Why a decision unit.** The session guard enforces an item's scope as it stands on `main`, so a scope change must merge before the code that needs it (claude-197's finding).

1. **The one file to add:** `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`.
   - It is the only path from a checkpoint directory to a snapshot, and the only holder of the directory that `HammingReads::load` needs.
   - Since claude-198 (`5520524bb`) it refuses `hamming_read.safetensors` by name through `check_checkpoint_reads`. Step 4 must make it load the read and carry it.
2. **No other file is needed:**
   - `HammingRead`'s rotations, margin and weight are `pub(crate)`, and `heads` follows from the margin's length, so `fbs_export.rs` (already in scope) can read them without changing `flock_read_hamming.rs`;
   - `CHECKPOINT_ACCOUNTED` and the snapshot types are in `fbs_export.rs` and `fbs_snapshot.rs`, both already in scope.
3. **The item text:** step 4 of "The order of units" names the binary, and how the float readout depth maps to the trunk's layer count.

## Cancel probe (run first)

- **Is the file already in the item's scope?** No. At `origin/main` `5520524bb`, the `scope:` of `docs/roadmap/items/integer-hamming-read.md` lists 14 paths, none of them under `crates/uor-r4-float-train/src/bin/`.
- **Does the export already carry the read?** No. `fbs_trunk_snapshot` writes `hamming: None` (`fbs_export.rs`), and `fbs-snapshot` refuses the file.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/integer-hamming-read.md`
- `docs/loop/tasks/integer-hamming-read/**`

## Done when

1. **The item's `scope:`** lists `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, and nothing else is added.
2. **Step 4's text** names the binary, and the depth rule: the float readout read sits at `HAMMING_READOUT_DEPTH`, and the export maps it to the layer count or refuses a trunk of another depth.
3. **A fresh decision review passes.** It reads the item, the export path (`fbs-snapshot.rs`, `fbs_export.rs`, `flock_read_hamming.rs`), CHARTER §5.1 and §5.2, `owner-paths.json`, and the other items' scopes and open claims. The owner log goes on #3385.
4. **`loop report --check` and `loop pr-check --local` pass.** The item stays `ready`.

## Stop and ask if

Stop and decide (never wait) if:
- the review finds that step 4 needs another file outside the scope: add it here, named, or split it off;
- another open ORDER holds `fbs-snapshot.rs`: then take that up with its lab on #2028, and take other work.
