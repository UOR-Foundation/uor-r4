# ORDER: integer-nearest-readout, claude-252 (Q5's trace instrument: T's side)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `ee534711f`. Rule: claude-247's [decision card](../claude-247/DECISION.md).

## Goal

Build T's side of Q5's three-way trace. F's side merged in claude-251 (#3514, `7c229614`), and R's in claude-249. Casey, 1 Nov: "Finish Q5 first" (#3462, `6564833770`).

**What T is (the card):** F's path, with F's build and arguments, on an evaluation copy of the seed's checkpoint. In that copy, every parameter the artifact holds is rounded as `fbs-export --raw-gates --raw-transition` rounds it. So T's parameters are R's, and only its arithmetic is float.

**The copy, `fbs-snapshot --round-as-exported MODEL_DIR OUT_MODEL_DIR`,** a form beside `--rebind-fbs-backward`:
- **`model.safetensors`:** `round_tables_as_exported` (grid codes, Q16, Q12).
- **`hamming_read.safetensors`:** `round_hamming_as_exported` (2^−24, 2^−16, 2^−24).
- **`turn_memory*.safetensors`:** the opening and reply weights at 2^−24 (`MEMORY_WEIGHT_EXP`). No helper rounds them yet. The keys and tables are exact and are copied as they are.
- **Copied byte for byte:** the step's `config.json` and the run's config.
- **A marker, `rounded-as-exported.json`,** written last. It holds each source file's sha256 and the roundings applied.
- **Refusals:** an existing output, a step file it does not know, and whatever the export refuses. The copy is written with no partial output, and reloaded with every check.

**The side:** `score_rows` writes side `T` when the checkpoint holds the marker, and `F` otherwise. So T's command is F's, unchanged.

## Cancel probe (run first)

- **Do the raw flags store any gate or transition value?** If not, T needs only the float path's own normalization. The scout read `format.rs:1588-1589` and found no such table. Confirm on `score_rows`' forward: it has no D11 snap.
- **Are the memory weights the only group no helper rounds?**
- **Are I-Twin's inputs in the HF store?** Found, 1 Nov:
  - R's sealed `laptop/R/a7/a7r-sN.uorlut`;
  - claude-227's `out/full/snapshot-sN.json`;
  - the checkpoints, `training-460fe46e/sN/r-drop-sN/models/step-016384`, which hold 5 files.
- **Classed:** instrument.

## Files in scope

The item's scope. This unit edits:
- `fbs_export.rs` and `bin/fbs-snapshot.rs`: the copy;
- `flock_read_memory.rs`: the weights' accessors;
- `in_space_long.rs` and `bin/train-in-space.rs`'s docs: the side;
- this folder and the item.

## Done when

1. **The copy** holds every rounding above. Re-rounding it changes nothing, and each refusal is tested.
2. **The side.** On a copy, `score_rows` writes side `T`. F's and R's files are unchanged.
3. **Tests.** Host tests for:
   - the memory rounding: exact, idempotent, and refusing a value past i32 at its step;
   - the marker's side;
   - the copy's refusals.

   Each has a negative gate. The crate also passes a stub-nvcc `cargo check --features cuda`, with no clippy finding on added lines.
4. **I-Twin, at seeds 1–3, on a pod.**
   - The path: the copy, then `fbs-snapshot COPY SNAP` with no readouts, then `fbs-export --raw-gates --raw-transition --run-config CONFIG SNAP OUT`.
   - It must give R's sealed artifact's sha256 at each seed.
   - If it does not, T is repaired before anything is read.
5. **T's smoke, outside Q5's set.** F's command on T's s1 copy, tracing DEV s1's 308 windows:
   - side `T`;
   - its windows align with F's and R's;
   - the rows are byte-identical with the trace on and off.

   T's agreement with F′ is reported as a reading, with no bar. Times are recorded.
6. **A fresh exact-head review passes,** in slices of at most 600 lines. The REPORT puts the negatives first. No notebook row.

## Stop and decide (never wait) if

- I-Twin fails at any seed: repair T first, for example by reading T's parameters from R's artifact;
- the artifact holds a parameter the copy cannot round to an exact f32.
