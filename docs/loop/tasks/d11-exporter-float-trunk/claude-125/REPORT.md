# REPORT: d11-exporter-float-trunk, claude-125

Lab: claude. Order: [ORDER.md](ORDER.md). Pre-run review (D67): #2820 comment `6409947018`, PRE-RUN: GO.

References #2820.

## What did not work, first

- **One pre-registered bar missed, on the single-token query gap.** The D11-mode GPU's largest query gap to the quantized reference over the 256 single tokens is 0.766. The bar is 1e-3.
  - **Classification:** an instrument miss under the ORDER's pre-registered rule: at most 6 positions, each traced to a near-tie.
  - **The trace:** only 2 positions exceed the bar, tokens 32 and 240, and every token agrees. `fbs-compare` now traces each position over the bar against all three computations: the f32 GPU in D11 mode, the f64 reference, and the integer artifact's own Q12 query.

    | Token | GPU vs reference | GPU vs integer | Integer vs reference | Which one split off |
    |---|---|---|---|---|
    | 32 | 0.766 | agree | 0.766 | the f64 reference |
    | 172, 104, 177 | agree | 1.363, 0.742, 0.360 | same gaps | the integer artifact |
    | 240 | 0.179 | 0.179 | agree | the f32 GPU |

    At each position exactly one of the three splits off, and a different one each time: each precision breaks a tie its own way. A mapping fault would put one computation off systematically. The planted faults show what that looks like: unrounded tables put all 256 positions over the bar, and the transition snap turned off puts 55 of 64 sequence positions over.
  - Over the 64 sequence positions, all three computations agree to one Q12 step (2.4e-4).
  - This trace came after the run, from the stored GPU outputs, with an extended `fbs-compare`. The classification rule itself was fixed before the run.
- **My first D11 mode rounded the gates to Q12, which the integer path does not.** I caught it before the run, by reading the reference: its `INTEGER` snaps do not round. claude-124's attribution shows that rounding alone moves queries by up to 0.536. The D11 mode now snaps the gates directly, through the `HamiltonIcosianHard` gate path, and the ORDER records the change.
- **My first rounding test could not fail for the gains.** A fresh trunk's gains are ones, already exact grid values, so skipping the grid rounding passed the test; a set-aside showed it. The test now gives every table random off-grid values first.
- **Not pinned:**
  - The GPU snap maps an all-zero 4-vector to root index 1 of its own order. The integer reference maps it to index 0 of `H4_ROOT_COEFFICIENTS`. No zero block occurs here, so this edge is unmeasured.
  - `with_d11_mode` restores the hook on an error, not on a panic, as `with_d11_export_gates` does.
- **Still a random-init trunk.** No trained model ran and no export gap was measured; that is Stage 4b.

## What changed

Size against `origin/main`: code +539/−160 in 7 files, including in-file test modules; tests +54 in `stack/tests.rs`; docs: the ORDER, this REPORT and the notebook row.

- **`uor-r4-integer`:**
  - `fbs_snapshot` (ungated) now holds the exporter's roundings: `grid_code`, `grid_value`, `fixed_value`, `q12_value` and `FBS_FIXED_EXP`.
  - `stack::encode_grid_code`, `decode_grid_code` and the reference's `ref_fixed` delegate to them, so there is one implementation.
  - `FbsReadoutMode` (`plain`, `export`, `d11`) replaces the readout's `export_mode` flag.
  - `fbs-compare`:
    - takes a D11 readout, and reports it against the quantized reference and against the integer session's tokens and own Q12 query;
    - every pairing now counts its positions over the 1e-3 bar and lists the worst.
- **`uor-r4-float-train`:**
  - `hamilton_sheet::with_d11_mode` snaps both gates directly and turns on a new transition-snap flag, restoring the whole hook afterwards. `composed_recurrence` snaps the transition to the 120 icosians when the flag is set (`icosian_snap_quaternions`, CUDA only).
  - `fbs_export::round_tables_as_exported` rounds the trunk's tables in place to the artifact's values: grid codes for the gains and taps, Q16 for the biases and decay, and Q12 for the embedding.
  - `fbs_float_readout(model, FbsReadoutMode)`: in D11 mode it rounds the final query to Q12 before the writer.
  - `fbs-snapshot` takes `[READOUT [EXPORT_READOUT [D11_READOUT]]]` and rounds the tables before the D11 readout.

## Measurements

On an RTX 5090, random-init trunk (vocabulary 256, width 32, 2 layers), at `c1bfd1b78897f1eb4084743ee7e351ce044d1a36`. The bars were fixed in the ORDER before the run.

| Check | Single tokens | Sequence positions | Bar |
|---|---|---|---|
| Rounded trunk's artifact | `cmp`: the same bytes as the original's | | same bytes |
| D11 GPU tokens against the quantized reference | 256/256 | 64/64 | ≥ 250 and ≥ 60 |
| D11 GPU tokens against the integer session | 256/256 | 64/64 | ≥ 250 and ≥ 60 |
| D11 GPU largest query gap to the quantized reference | 0.766 (2 positions over, both near-ties: an instrument miss under the rule) | 2.4e-4 | ≤ 1e-3 |
| D11 GPU largest query gap to the integer session's query | 1.363 (4 positions over, near-ties) | 2.4e-4 | (reading) |
| Set-aside 1, tables unrounded | 232/256, all 256 over the bar | 56/64, all 64 over | must break: **broke** |
| Set-aside 2, transition snap off | 256/256 (no transition acts at one position) | 39/64, 55 over | must break: **broke** |
| Plain and export parity (claude-124's pairings) | 256/256 | 64/64 | unchanged |

**Host checks:**

| Check | Result |
|---|---|
| Rounding test | each rounded table is its own exported rounding; every table changes and the routes do not; set-asides of the grid, Q16 and Q12 roundings each fail it |
| Transition flag | set only inside `with_d11_mode`, also after an error; its set-aside fails the test |
| Exporter roundings | the ungated roundings reproduce the quantized reference and export to the same bytes |
| Suites | float-train 33; integer serving-build library 321; no-feature library 55; `m5_full_path_cost_controls` 3/3; `memory_shape` 3/3; `fbs-compare`'s own test |
| Zero-multiply audit | `FULL PASS`: rlib 57 symbol ranges and 41 functions; `uor-r4-stack` 44 and 90 |
| GPU cost | one RTX 5090 for about 2 minutes of job time; `vast-57777264` deleted |

Evidence is in the HF store under `results/claude/d11-fbs-d11mode-2026-10-26/`, with a `SHA256SUMS`:
- `fbs-d11/` holds the run;
- `traced/` holds the traced comparisons, run on the laptop from the stored outputs.

## Checks run at the head

See the exact-head review record on the PR:
- the suites above;
- the stub-nvcc `cargo check --features cuda --all-targets`;
- clippy;
- `cargo fmt --check`;
- both audits;
- `loop report --check`, `loop pr-check --local` and `loop gates --full`.

## Skills used

- **holding-the-goal.** The aim was a mode Stage 4b can score the artifact with on the GPU. That is why the gate rounding was corrected before the run rather than reported afterwards.
- **writing-meaningful-tests.** Each piece was set aside to show its test can fail. One test could not fail at first, and was fixed.

## Still open (Stage 4b)

- **The Stage 4 sheet,** with:
  - a fresh non-author FIT;
  - a two-seed retrain saved with `StackModel::save`;
  - the export gap, scored on the GPU: the plain float trunk against the D11 mode on the sheets' evaluator, with the two-seed noise band as the band;
  - laptop tok/s, with decode and context reported separately.
- **The zero-block snap edge,** if a trained trunk ever produces one.
- **Serving front ends and the pages,** as before.

## Plain summary

The float model can now run in a "D11 mode": on the GPU, it computes what the integer file that ships to devices computes. It rounds the same rotations to the same fixed set of 120, and reads its numbers rounded the way the file stores them.
- On a test model, this mode picks the same next token as the integer file on every input tried, 256 single tokens and 64 positions in sequences.
- Its internal numbers agree with the integer file's to one rounding step, except at a few places where three slightly different kinds of arithmetic break a near-tie differently.
- With this, the next step can measure how much accuracy a trained model loses when it is shipped as integers, using the GPU as the project's rules require.
