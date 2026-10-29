# REPORT: integer-hamming-read, claude-188 (the artifact carries the read)

Lab: claude. Order: [ORDER.md](ORDER.md). The item is not done: this is step 1 of its six (claude-187's design). It has no GitHub issue; claims and merges are noted on #2028.

**Speed.** No serving change. An artifact with a read is refused, and a read-free one is served as before.

## What did not work, first

- **The read's description could not go in `StackShape` without breaking 25 struct literals.**
  - Two of them are in `crates/uor-r4-integer/tests/memory_shape.rs`, outside the item's scope.
  - It goes in the artifact's header instead, beside claude-181's optional `transport_snap`. `StackShape` is unchanged.
- **My first command to set the format aside for the negative gate** wrote a scratch copy to `local/format.rs.keep`. The guard refused it, since `local/<name>` reads as another lab's folder. The copy went under `local/claude/claude-188/`.
- **A stray `git stash list` in one command** was refused by the guard; this loop forbids stash commands, even read-only ones. Nothing ran, and the command was rerun without it.

## What changed

- **`crates/uor-r4-integer/src/fbs_snapshot.rs`:** `FbsTrunkSnapshot` gains `hamming: Option<FbsHammingSnapshot>`. It is `serde(default, skip_serializing_if = "Option::is_none")`, so an old `snapshot.json` parses and a read-free one serializes with no `hamming` key.
  - `FbsHammingSnapshot` holds `heads`, `depths` and `reads`.
  - `FbsHammingRead` holds the query, key and value rotations, `margin` and `weight`, as `hamming_read.safetensors` stores them.
- **`crates/uor-r4-integer/src/stack/format.rs`:**
  - `STACK_FBS_HAMMING_SCHEMA = "uor-r4.lut-stack/6"`.
  - **`StackHamming`,** the header record: `heads`, `depths`, `kept` (16), and the exponents of the rotations (`−24`), margins (`−16`) and weights (`−24`). Its `check` refuses:
    - a non-FBS stack;
    - an odd head count, or one that does not divide the width into 4-lane blocks of at most 64 coordinates;
    - depths that are empty, out of order, or outside `1..=layers`;
    - zero kept positions;
    - other exponents.
  - **The header** gains an optional `hamming`. The loader expects `/6` exactly when it is present, and checks it.
  - **The FBS exporter** writes `hamming.d{depth}.{query_rotation, key_rotation, value_rotation, margin, weight}` as `i32` at those exponents. It refuses a wrong length or a value that does not fit, and a margin that is not positive, since a weight is `max(0, m − d)/m`. Only then does it add the `hamming` key and choose `/6`.
- **`crates/uor-r4-integer/src/stack/hamming.rs` (new):** `read_hamming_uorlut01` and `HammingImage` / `HammingReadImage`. These read the reads back with every length checked, for the serving unit.
- **`crates/uor-r4-integer/src/stack/session.rs`:** `IntegerStackModel::parse` refuses a `/6` artifact ("this engine does not serve a Hamming read yet"), so a read is never silently dropped.
- **`crates/uor-r4-integer/src/stack/synthetic.rs`:** `synthetic_hamming`, with random rotations whose lane 0 is the zero quaternion, positive margins, and random weights.
- **`crates/uor-r4-float-train/src/fbs_export.rs`:** `hamming: None`. Reading `hamming_read.safetensors` is a later unit.
- **Four tests** (`crates/uor-r4-integer/src/stack/tests.rs`):
  - `read_free_fbs_exports_are_unchanged_by_the_hamming_format`;
  - `hamming_reads_round_trip_in_lut_stack_6`;
  - `malformed_hamming_reads_are_refused`;
  - `snapshots_without_hamming_reads_are_unchanged`.

## Measurements

- **Read-free bytes pinned at the base `a368b78ef`,** before any change: the synthetic trunk's snapped export is `e377d68f6b3c112ed267a9e3c09ef2ef2210e8f1c9bc4619ae26dd4c68d18e5c`, and its both-raw export is `ec5ca212c257a2b2cd73c337029ff8556b454810377801d81addca75c4105965`. Both still match at the head.
- **Negative gate:** writing the `hamming` key always (as `null` when absent) changes the snapped export to `cf7929e2…0ba3`, and the pin fails. Restored, it passes, and both builds recompiled.
- **The round trip, snapped and both-raw:**
  - every rotation, margin and weight reads back as `round(v · 2^−exp)`;
  - the zero quaternion reads back as `[0, 0, 0, 0]`;
  - the header says `/6`;
  - the session refuses it;
  - the same trunk without the read is served.
- **All nine malformed reads are refused:**
  - an odd head count;
  - a head count of zero;
  - depths out of order;
  - a depth beyond the layers;
  - a depth of zero;
  - a missing read;
  - a short rotation;
  - a zero margin;
  - a weight that does not fit.

## Checks run at the head

At `340560e59`, and after one test-label fix:
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`:** 555 passed, 35 failed, 4 ignored. The 555 are the 551 of `main`, plus this unit's 4. All 35 failures panic on a missing local input (a bundle, 32, or `tokenizer.json`, 3): the same messages as `main`'s.
- **`cargo test --locked -p uor-r4-float-train --no-fail-fast`:** 122 passed.
- **The serving audit** (`audit_zero_matmul_serving.py --stack` on the release `uor-r4-stack`): FULL PASS. The multiplies / divides are 47 / 1 and 65 / 9, unchanged.
- **The GPU evaluator's host tests** (`docs/labs/d11-exporter-float-trunk/instrument`, which links this crate; stub nvcc, `--target aarch64-apple-darwin`): parity 5 passed.
- **`cargo fmt --check`,** on both crates: clean.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **The test-label fix:** `malformed_hamming_reads_are_refused`'s case "a head wider than one word" set `heads = 0`, which is refused for being below 2. It is renamed "a head count of zero". With width 32, no head can exceed one word. The test passes.

## Skills used

- **holding-the-goal.** The format is built so that nothing served today changes: read-free bytes are pinned at the base, and a negative gate shows the pin can fail. The session refuses the new schema, rather than serving part of it.

## Still open

- **The item's steps 2 to 6:**
  2. the copy heads served, with synthetic controls against an exact reference;
  3. the type heads;
  4. the exporter reading `hamming_read.safetensors`, which must refuse a `step_rotation`;
  5. the float parity on claude-146's checkpoints (GPU, under a prep sheet);
  6. the speed.
- **The mechanism page** `docs/mechanisms/lut-serving.md` (`infra-loop-harness`'s scope) does not mention `lut-stack/6` yet.

## Plain summary

The integer model file can now hold the memory lookup's learned settings: its rotations, its distance margins and its weights. It stores them under a new format version, so older software refuses the file instead of silently ignoring the lookup.

Files without the lookup come out byte for byte as before; a test pins this, and a deliberately broken version shows the test can catch a change. The integer engine doesn't run the lookup yet, so for now it refuses a file that has one. Running it is the next step.
