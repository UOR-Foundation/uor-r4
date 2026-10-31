# ORDER: integer-nearest-readout, claude-233 (locate the parity gap: trunk and Hamming read on the GPU)

Lab: claude. Item: [integer-nearest-readout](../../../../roadmap/items/integer-nearest-readout.md) (#3462). Base: `4195ce2bf`.

**Written by hand,** because `loop claim` refuses on the false overlap with #3436's `crates/*/README.md`.

## Goal

claude-232's parity missed, and the sheet's MISS row says to locate the gap layer by layer. This unit runs claude-227's GPU parity (`fbs-snapshot` readouts, then `fbs-compare --parity-only`) on four variants of (r-drop) step 16,384:
- the trunk alone at s1 (repeating claude-227) and at s3;
- the trunk with its Hamming read at s1 and at s3.

The integer trunk's and trunk-plus-read's queries and writer tokens are compared with the float model's, under a prep sheet with a fresh FIT and a pre-run review.

No code changes. It reads which layer differs; it does not fix it.

## Cancel probe (run first)

- `fbs-compare` exports a snapshot with its Hamming read (`/6`) and serves it; nothing refuses it.
- `fbs-snapshot` takes readouts from a checkpoint without `turn_memory.safetensors`, and applies the read to them (claude-200).
- The pod caps have room, and no open PR touches the scope.

If any fails, stop and report.

Run on 31 Oct at the base:
- `fbs-compare` checks only the readout's shape (`check_readout`);
- `check_readout_reads` refuses only beside `turn_memory`, and `fbs-snapshot` calls `set_hamming_reads` before its readouts;
- `uor-pod status`: 0 pods running, 5090s from $0.563/h;
- no open PR touches the scope.

Result: **go**.

## Files in scope

- `docs/harness/prep/integer-nearest-readout-locate-2026-10-31.md`
- `docs/labs/integer-nearest-readout/**`
- `docs/research/notebook/m4-serving.md`
- `docs/loop/tasks/integer-nearest-readout/claude-233/**`

## Done when

- **The sheet:** FIT, and a pre-run review passed.
- **The run:** archived with sums, and its row read from the stored comparisons.
- **The records:** one notebook row and the lab record. The REPORT gives the negatives first, and a fresh review runs on the diff.

## Stop and ask if

The run needs a code change (a new unit).
