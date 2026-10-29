# REPORT: integer-hamming-read, claude-199 (the scope step 4 needs)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3385. The item stays `ready`.

**Speed.** No serving change: a scope decision.

## What did not work, first

- **Step 4 could not start as code.** The session guard enforces an item's scope as it stands on `main`, and `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs` was not in this item's scope. That binary is where the read must be loaded and carried. So the scope lands first, as claude-197 did for `turn-memory-integer`.
- **claude-194's checkpoints still cannot be exported.** Since claude-198, `fbs-snapshot` refuses them by name, because they hold `hamming_read.safetensors`. That refusal stays until step 4's code lands.

## What changed

- **`docs/roadmap/items/integer-hamming-read.md`:**
  - **`scope:`** adds one path, `crates/uor-r4-float-train/src/bin/fbs-snapshot.rs`, to make 15 paths;
  - **step 4** of "The order of units" now names three things:
    - the binary;
    - where the conversion lives: a `pub fn` in `fbs_export.rs`, because the binary is a separate crate and cannot read `HammingRead`'s `pub(crate)` fields;
    - the depth rule: the export refuses a trunk whose layer count is not `HAMMING_READOUT_DEPTH` (6).
  - **The depth rule was narrowed by the review.** The float readout read always runs on the readout. The integer engine takes the read whose depth equals the layer count as the readout. The first wording mapped depth 6 to the layer count, and on a 2-layer trunk that would serve the stream read on the readout.
- **No other file is needed.** `HammingRead`'s rotations, margin and weight are `pub(crate)`, and `heads` follows from the margin's length. So `fbs_export.rs`, already in scope, can read them without changing `flock_read_hamming.rs`.
- No code.

## Measurements

None: a decision.

## The decision review

Fresh context, at `3bc4cfb8c`. It read:
- the item and the ORDER;
- the export path: `fbs-snapshot.rs`, `fbs_export.rs`, `flock_read_hamming.rs` and `fbs_snapshot.rs`;
- CHARTER §5.1 and §5.2, and `owner-paths.json`;
- the other items' scopes and the open PRs.

- **DECISION REVIEW: PASS.**
  - **Necessary:** the binary is the only caller that holds the checkpoint directory.
  - **Sufficient:** `HammingReads::load` is `pub`, and the fields are `pub(crate)`.
  - **Nothing else needed:** `FbsHammingSnapshot`, the integer reader and `fbs-export.rs` are already in scope.
  - **Nothing is the owner's call.**
  - **No collision:** `turn-memory-integer` also lists `fbs-snapshot.rs`, but no ORDER is open on it.
- **DIFF REVIEW: APPROVE WITH NITS.** Two nits, both made at the next commit:
  - the depth rule's 2-layer case, above;
  - that the conversion lives in `fbs_export.rs`.
- **Its note:** a later `turn-memory-integer` unit and step 4 would both edit `fbs-snapshot.rs`. Whoever claims second checks the other item's open ORDERs first.

The owner log is on #3385.

## Checks run at the head

- `cargo run -p xtask -- roadmap render`: it rewrites `docs/roadmap/README.md` with no change.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at this REPORT's commit.

## Skills used

- None was invoked this unit.

## Still open

- **Step 4's code,** the next unit:
  - `fbs_export.rs` reads `HammingReads` into `FbsHammingSnapshot`;
  - `check_checkpoint_reads` accounts for `hamming_read.safetensors`;
  - `fbs-snapshot` loads and carries the read;
  - a round trip through `fbs-export` gives `/6`, or `/7` beside a memory;
  - the depth rule is checked.
- Then parity (step 5, GPU under a prep sheet) and speed (step 6).

## Plain summary

The program that turns a trained model into the file the integer engine reads can now be changed to carry the model's look-back read. Until that change lands, the program refuses such a model rather than packing it without the read.
