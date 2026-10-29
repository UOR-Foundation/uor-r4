# REPORT: integer-hamming-read, claude-200 (step 4: the export carries the read)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3385. The item stays `ready`, at step 5 (parity).

**Speed.** No serving change: the integer session already serves the read (claude-194, claude-195). This unit lets a trained checkpoint reach it.

## What did not work, first

- **No real checkpoint was exported in this unit.** `fbs-snapshot` needs the GPU, because the trunk's routes are read back through CUDA, and claude-194's checkpoints are in the HF store. The conversion is tested on the CPU with saved random reads. A real export comes with step 5's parity run, under its prep sheet.
- **The float and integer layouts were matched by reading, not by a run.**
  - Each rotation reshapes to `(1, width)`, in the points' own lane layout (`flock_read_hamming.rs`, `rotate`).
  - The integer side stores `width / 4` quaternions, row-major (`fbs_snapshot.rs`, `FbsHammingRead`).
  - Step 5's parity is what proves the two agree end to end.
- **The snapshot does not refuse a margin that is not positive.** The integer export already refuses one by name (`format.rs`, "a margin is not positive"). So such a checkpoint fails at `fbs-export`, not at `fbs-snapshot`.
- **The type spin (claude-151's `step_rotation`) cannot be exported.** It is refused by name until the integer engine serves it. claude-146's and claude-194's checkpoints do not have it.

## What changed

- **`fbs_hamming_snapshot(reads, layers, width)`** (`fbs_export.rs`) builds an `FbsHammingSnapshot`:
  - `heads` comes from the margin's length, and `depths` from `HAMMING_DEPTHS` (`[2, 6]`);
  - each read's rotations, margin and weight are flattened row-major, each through `finite_table`.
- **It refuses, by text:**
  - a trunk whose layer count is not `HAMMING_READOUT_DEPTH`, under claude-199's rule;
  - a read with `step_rotation`;
  - a table whose shape is not `(width/4, 4)`, `(heads)` or `(2, width)`;
  - a value that is not finite.
- **`CHECKPOINT_ACCOUNTED`** now includes `hamming_read.safetensors`. A checkpoint with the Flock, content or opening read, or the prompt register, is still refused by name.
- **`fbs-snapshot`** loads the reads with `HammingReads::load` on the CPU, and sets `snapshot.hamming`. `fbs-export` then writes `/6`, or `/7` when a turn memory is also present.

## Measurements

None: an export path and its checks.

## Checks run at the head

- **Done-when 1** (`saved_hamming_reads_snapshot_as_they_load`): 4 heads at width 32, with random parameters, saved and loaded.
  - The snapshot equals the one taken before the save, and holds exactly the tensors' values, row-major.
  - `heads` is 4, and `depths` is `[2, 6]`.
  - The JSON reads back equal.
- **Done-when 2:** each refusal fires, by text:
  - a 4-layer trunk ("this trunk has 4 layers");
  - width 64 against 32-wide tensors ("hamming_read.d2.query_rotation has shape");
  - `with_type_spin` ("the type spin (step_rotation)");
  - a NaN in `d6.weight[3]`.
- **Done-when 3** (`a_checkpoint_read_the_export_does_not_carry_is_refused`): a directory with `hamming_read.safetensors` beside the trunk, the optimizer and the memory passes. The four other reads and an invented sixth are each refused by name.
- **Done-when 4, the negative gates.** Each check was set aside, its test turned red, and each was restored:
  - H1: the depth refusal;
  - H2: the type-spin refusal;
  - H3: the shape check;
  - H4: the read taken out of the allow-list.
- **Done-when 5:**
  - `cargo test --locked -p uor-r4-float-train --no-fail-fast`: 139 passed, 0 failed.
  - The stub-nvcc check, `CUDARC_CUDA_VERSION=12080 NVCC=<stub> UOR_NVCC_ARCH=compute_120 cargo check --offline --locked -p uor-r4-float-train --features cuda --bin fbs-snapshot --lib --tests`: it builds.
  - `uor-harness loop gates --full`, at `b472da448` with this REPORT: green, 11 gates GREEN.

## Skills used

- None was invoked this unit.

## Still open

- **Step 5, parity, on the GPU, under a prep sheet.** Export claude-194's (r) checkpoint at both seeds through `fbs-snapshot` and `fbs-export`, then score the integer artifact against the float forward on DEV:
  - the share of kept sets equal to float's;
  - top-1 agreement.

  The prep sheet should come after the whole-project assessment that Casey asked for on 29 Oct. It decides what that run must also measure.
- **Step 6: speed** with the read.

## Plain summary

The program that packs a trained model for the integer engine now packs the model's look-back read too. Before, it refused such a model. A model with the read can now be exported. Whether the integer copy matches the float one is the next test.
