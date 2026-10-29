# REPORT: turn-memory-integer, claude-197 (the scope the export needs)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3392. The item stays `ready`.

**Speed.** No serving change: a scope decision.

## What did not work, first

- **The unit set out to write the export's code, and could not.** The session guard enforces the item's scope as it stands on `main` (or as recorded at claim time), not an amendment on the unit's own branch. Its edit to `crates/uor-r4-integer/src/fbs_snapshot.rs` was refused.
  - So this unit is the scope decision alone, and the code is the next unit's, claimed once this one merges.
  - The friction is logged: a unit that needs new files must land its scope decision as a PR of its own first.
- **The export silently drops two reads today.** That is why the scope is needed, and it is the next unit's first job:
  - `fbs-snapshot` loads a checkpoint with `StackModel::load`, which reads neither `turn_memory*.safetensors` nor `hamming_read.safetensors`;
  - `fbs_trunk_snapshot` writes `hamming: None` (`fbs_export.rs:282`);
  - the exporter's memory refusal checks `config.memory`, a different field;
  - so claude-194's checkpoints would export as a trunk without the memory it depends on, with no error.

## What changed

- **`docs/roadmap/items/turn-memory-integer.md`:**
  - **`scope:`** adds three files:
    - `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, the only path from a checkpoint to a snapshot, and the only holder of the checkpoint directory that `TurnMemory::load` needs;
    - `crates/uor-r4-integer/src/fbs_snapshot.rs`, where the snapshot types live (`deny_unknown_fields`);
    - `crates/uor-r4-integer/src/stack/synthetic.rs`, which builds the snapshot by struct literal and needs the new field.
  - **The order of units:** the parity unit now waits on `integer-hamming-read`'s step 4. claude-194's checkpoints hold a Hamming read, and the export will refuse a read it does not carry.
- **The ORDER** is amended to the decision alone, with the plan for the next unit's code kept in it.
- No code.

## Measurements

None: a decision.

## The decision review

Fresh context, at `f602bbdcc`. It read the item, the export path (`fbs-snapshot.rs`, `fbs_export.rs`, `fbs_snapshot.rs`, `flock_read_memory.rs`), CHARTER §5.1 and §5.2, `owner-paths.json`, and the other items' scopes and open claims.
- **Verdict: PASS.** Done-when 1 cannot be met without the two files, and two exact paths are the narrowest fix.
- Nothing is the owner's call, and there is no active collision: `integer-hamming-read` already lists `fbs_snapshot.rs`, and no one has claimed it.
- **Its notes:**
  - the parity dependency on `integer-hamming-read`'s step 4, now in the item;
  - `integer-hamming-read` will need `fbs-snapshot.rs` in its own scope for that step. That is left to that item's unit.

`synthetic.rs` was added after the review, for the reason above. The PR's fresh review checks the final diff. The owner log is on #3392.

## Checks run at the head

- `uor-harness loop report --check` and `loop pr-check --local`, before the review record.

## Skills used

- None was invoked this unit.

## Still open

- **The next unit, unit 1's code,** as this ORDER's work list:
  - `FbsMemorySnapshot`;
  - the float side's accessors and `fbs_memory_snapshot`;
  - `fbs-snapshot` carrying the memory and refusing a Hamming read;
  - the integer export's sections, record and schema;
  - the reader;
  - the session's refusal.

## Plain summary

Before the model's memory can be packed into the integer file, two programs that sit between training and packing have to be allowed to change. This unit records that decision and why. The packing itself comes next.
