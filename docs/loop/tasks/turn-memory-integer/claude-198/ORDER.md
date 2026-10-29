# ORDER: turn-memory-integer, claude-198 (unit 1's code: the export carries the memory, or refuses)

Lab: claude. Item: [turn-memory-integer](../../../../roadmap/items/turn-memory-integer.md). Issue: #3392. Base: `9e889300c`.

## Goal

Unit 1 of the item's five (claude-196), under the scope claude-197 added: the export of both memory tables, their weights and the key window. It advances step 2 of PLACE.md's order, and the owner's direction of 29 Oct (18:50Z): "a valid (and somewhat coherent and standardized) training/serving path".

**The standard:** a float checkpoint exports through one snapshot that carries every read the checkpoint holds, or the export refuses it by name. Today the export silently drops the memory and the Hamming read (claude-197's REPORT).

1. **`FbsMemorySnapshot`** (`crates/uor-r4-integer/src/fbs_snapshot.rs`), `serde(default, skip_serializing_if)` like `hamming`, so every existing snapshot and read-free export is unchanged. It holds:
   - `k` and `key_window` (192);
   - the opening table, as entries (key, value) in ascending key order, and its weight;
   - the reply table and its weight, if any.

   `synthetic.rs` gains `memory: None`.
2. **The float side:**
   - read-only accessors on `TurnMemory` (`flock_read_memory.rs`);
   - `fbs_memory_snapshot(memory, key_window)` (`fbs_export.rs`);
   - `check_checkpoint_reads(dir)` (`fbs_export.rs`), which refuses a directory holding `hamming_read.safetensors`. That read's export is `integer-hamming-read`'s step 4;
   - `fbs-snapshot` runs the check, loads the memory with `TurnMemory::load`, and carries it.
3. **The integer export** (`format.rs`, `export_fbs_trunk_uorlut01_with`):
   - **sections:**
     - `memory.opening.keys` (`u32`, `n × k`, padded with `u32::MAX`), `memory.opening.values` (`u32`) and `memory.opening.weight` (`i32` at `2^-24`);
     - `memory.reply.keys` (`n × 20`, the float `MAX_REPLY_KEY`), `memory.reply.values` and `memory.reply.weight`;
   - **a header record** `memory` (`StackTurnMemory`): `k`, `key_window`, `reply_key_len`, the counts and `weight_exp`, checked at parse;
   - **the schema** `uor-r4.lut-stack/7` for a stack with a memory, so an engine that does not serve it refuses it.
4. **The reader** (`crates/uor-r4-integer/src/stack/memory.rs`, `read_memory_uorlut01`), checking:
   - every section's length;
   - keys strictly ascending as unpadded rows, the float `save`'s order;
   - opening keys of at most `k` ids, each below the vocabulary;
   - reply keys of the form `turn (≤ k ids), MAX, length (≤ key_window), MAX, last id`;
   - values below the vocabulary.
5. **The session refuses a memory** until units 2 and 3 serve it, by its text.

## Cancel probe (run first)

- **Does the export already carry the memory?** No. At `origin/main` `9e889300c`, `fbs_export.rs` and `fbs-snapshot.rs` never read `turn_memory*.safetensors`.

- **Is the item withdrawn?** No; claude-197 added the scope this unit uses.

- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-integer/src/stack/memory.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/harness/prep/turn-memory-integer*`
- `docs/loop/tasks/turn-memory-integer/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/turn-memory-integer.md`

## Done when

1. **A round trip.** A snapshot with a memory exports and reads back every key, value and weight at its exponent, with its record and schema. The synthetic memories:
   - with and without a reply table;
   - an empty table.
2. **Unchanged without a memory.** A snapshot without a memory serializes with no `memory` key, and its exports keep claude-188's pinned sha256s.
3. **The refusals, by their text:**
   - the session, on an artifact with a memory;
   - the reader, on a short section, unsorted keys, an id past the vocabulary, and a malformed reply key;
   - the exporter, on a weight that does not fit;
   - `check_checkpoint_reads`, on a directory with `hamming_read.safetensors`.
4. **The float side, on the CPU:**
   - `fbs_memory_snapshot` of a `TurnMemory` equals its tables and weights, after `save` and `load`;
   - its JSON reads back equal.
5. **Negative gates:** each named check set aside fails its test.
6. **Checks:**
   - `cargo test -p uor-r4-integer` shows no new failure against `main`;
   - `cargo test -p uor-r4-float-train` passes;
   - the stub-nvcc check of `fbs-snapshot` builds;
   - `loop gates --full` is green.
7. A REPORT. The item stays `ready`.

## Stop and ask if

Stop and decide (never wait) if:
- carrying the memory needs a change to the trunk's own sections: then that is its own unit;
- the reply table's JSON proves too large for the snapshot handoff (about 574,000 keys): then the snapshot carries it as a sidecar file, and the ORDER is amended first.
