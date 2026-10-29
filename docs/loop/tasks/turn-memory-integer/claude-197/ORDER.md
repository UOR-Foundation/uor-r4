# ORDER: turn-memory-integer, claude-197 (unit 1: the export carries the memory, or refuses)

Lab: claude. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md). Issue: #3392. Base: `48cc5fe83`.

## Goal

Unit 1 of the item's five (claude-196): the export of both memory tables, their weights and the key window. It advances step 2 of PLACE.md's order, and the owner's direction of 29 Oct (18:50Z): "a valid (and somewhat coherent and standardized) training/serving path".

**The standard this unit sets:** a float checkpoint exports through one snapshot that carries every read the checkpoint holds, or the export refuses. Today the export silently drops two reads:
- `fbs-snapshot` loads the checkpoint with `StackModel::load`, which reads neither `turn_memory*.safetensors` nor `hamming_read.safetensors`;
- `fbs_trunk_snapshot` writes `hamming: None` (`fbs_export.rs:282`);
- its memory refusal checks `config.memory`, a different field.

So claude-194's checkpoints would export as a trunk without its memory. That trunk falls to about 4,000 DEV hits (claude-194's REPORT).

**First, a scope decision** (CHARTER §5.1, D68), with its own fresh decision review logged on #3392. The export runs through two files outside the item's scope, which the item's own done-when 1 needs. They are added to the item's scope and to this ORDER's:
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, the binary that loads a checkpoint;
- `crates/uor-r4-integer/src/fbs_snapshot.rs`, where the snapshot types live.

**Then the work:**
1. **`FbsMemorySnapshot`** in the snapshot, `serde(default, skip_serializing_if)` like `hamming`, so every existing snapshot and read-free export is unchanged. It holds:
   - `k` and the key window (192);
   - the opening table (keys, padded with `u32::MAX`, and values) and its weight;
   - the reply table and its weight, if any.
2. **The float side:**
   - accessors on `TurnMemory` (`flock_read_memory.rs`), and `fbs_memory_snapshot` (`fbs_export.rs`);
   - `fbs-snapshot` loads the memory from the checkpoint directory and carries it;
   - `fbs-snapshot` refuses a checkpoint that holds `hamming_read.safetensors`, since the Hamming read's export is integer-hamming-read's step 4, so nothing is dropped.
3. **The integer export** (`format.rs`, `export_fbs_trunk_uorlut01_with`):
   - sections `memory.opening.{keys,values,weight}` and `memory.reply.{keys,values,weight}`, keys and values as `u32`;
   - weights as `i32` at a fixed exponent, rounded as the exporter rounds;
   - a header record (`k`, the key window, the counts, the weight exponent);
   - a schema of its own, so an engine that does not serve the memory refuses it.
4. **A reader** (`read_memory_uorlut01`), checking:
   - every section's length;
   - keys sorted and unique;
   - every id below the vocabulary or a pad;
   - values below the vocabulary.
5. **The session refuses a memory** until units 2 and 3 serve it, by its text.

## Cancel probe (run first)

- **Does the export already carry the memory?** No. At `origin/main` `48cc5fe83`, `fbs_export.rs` and `fbs-snapshot.rs` never read `turn_memory*.safetensors`.
- **Is the item withdrawn?** No.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-integer/src/stack/memory.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs` (added by this unit's decision)
- `crates/uor-r4-integer/src/stack/synthetic.rs` (added by the same decision: its struct literal of the snapshot needs the new field)
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` (added by this unit's decision)
- `docs/harness/prep/turn-memory-integer*`
- `docs/loop/tasks/turn-memory-integer/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/turn-memory-integer.md`

## Done when

1. **The scope decision** passes its fresh review and is logged on #3392. The item's scope lists the two files.
2. **A round trip.** A snapshot with a memory (synthetic tables and weights, with and without a reply table) exports, reads back every key, value and weight at its exponent, and its record and schema check out.
3. **Unchanged without a memory.** A snapshot without a memory serializes with no `memory` key, and its exports keep claude-188's pinned sha256s.
4. **The refusals, by their text:**
   - the session, on an artifact with a memory;
   - the reader, on a malformed section (a short section, unsorted keys, an id past the vocabulary);
   - `fbs-snapshot`'s check, on a checkpoint with `hamming_read.safetensors`.
5. **The float side:**
   - `fbs_memory_snapshot` of a `TurnMemory` equals its tables and weights, round-tripped through `save` and `load` on the CPU;
   - the snapshot's JSON reads back equal.
6. **Negative gates:** each named check set aside fails its test.
7. **Checks:**
   - `cargo test -p uor-r4-integer` shows no new failure against `main`;
   - `cargo test -p uor-r4-float-train` passes;
   - the stub-nvcc CUDA check of `fbs-snapshot` builds;
   - `loop gates --full` is green.
8. A REPORT. The item stays `ready`.

## Stop and ask if

Stop and decide (never wait) if:
- carrying the memory needs a change to the trunk's own sections: then that is its own unit;
- the scope decision's review fails on a point that needs the owner: then post once on #2028, and park the item.
