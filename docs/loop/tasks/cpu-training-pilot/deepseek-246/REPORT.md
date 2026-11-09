# REPORT: cpu-training-pilot, deepseek-246

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- No Claude-330 checkpoint is available in this branch, so fitting a real
  B2-trained ask table, measuring held-out pulls or making any carry/capability
  claim is `NOT_RUN`.
- The synthetic B2-style fixture reaches a zero objective because it contains
  the exact target-key offset as a candidate. That 8-to-0 bit change verifies
  the objective, candidate construction, sidecar and audit path; it is not
  evidence of learned generalization.
- The release control uses synthetic identity rotations and a deterministic
  two-target corpus. Its 47 ms wall measures local search throughput only, not
  quality on a trained checkpoint.
- The first validation pass covered malformed pull corpora but did not yet pin
  the malformed and non-finite checkpoint cases required by this order. Those
  controls, deterministic metadata serialization and output-path collision
  refusal were added before delivery.
- Repository-wide `cargo fmt --all -- --check` is already red across many
  untouched files. The two new Rust files and their edited module are
  targeted-format clean; this unit did not reformat unrelated code.
- A strict `clippy -D warnings` run sees ten pre-existing warnings in
  `prompt_reply_rank.rs` and `suffix_arbitration.rs`. The normal scoped Clippy
  run passes, and neither new file adds a warning.

## What changed

- Added `AskOffsetSolveBasis`, `AskPullCorpus` and `cpu-ask-offset-solver`.
  The solver reads `embedding.weight`, `heads` and every
  `d{depth}.key_rotation`; it needs no trunk forward.
- The default corpus is the fixed B2 relation map `key -> key + 16`, tokens
  `16..32` to `32..48`. An optional weighted JSON corpus accepts repeated and
  multiple targets per question.
- For each question/depth, candidates are the zero offset, every distinct
  exact target offset and a weighted coordinate median. Up to four best starts
  run a deterministic coordinate search at steps
  `16384,4096,1024,256,64,16,4,1`; an exact zero objective stops early.
- The objective is weighted XOR/popcount Hamming distance over the ask half of
  the heads after `hamilton_gate_f32` rotation and sign packing. There is no
  dense matrix multiplication or softmax in this path.
- Independent question/depth cells run through the existing Rayon pool.
  Source-order collection produces stable readings and tensor order.
- The output is a new complete `hamming_read.safetensors`: all original
  tensors and metadata are preserved, existing ask tables are replaced and
  every solved depth receives a full F32 `(vocab,width)` ask table. Safetensors
  header metadata is canonicalized for deterministic fresh-process bytes.
- Both input files are SHA-256 pinned. Empty or zero-weight pulls, out-of-vocab
  tokens, bad shapes, invalid head geometry, partial depths, finite overflow,
  non-finite values, type-spin mixing, output replacement and colliding output
  paths fail closed.
- The CLI reopens the candidate with `TrainedAskOffsets`, audits the fixed B2
  relation map and writes candidate/report hashes plus per-cell
  zero/solved/improvement readings.
- No serving, artifact-schema, training-default, GPU or Claude-owned file was
  changed.
- Size: `pr-check` reports 10 changed paths: code 4 files `+1,399/-0`,
  documentation 3 files `+217/-0`, and other/evidence 3 files `+545/-0`
  (`+2,161/-0` total including this final REPORT revision). The ORDER gave no
  line estimate. Inline unit tests are counted in the code class.

## Measurements

The retained synthetic checkpoint has 64 tokens, width 8, two Hamming depths
and one copy/ask head pair. Over 16 questions and two depths it reduced total
weighted search bits from **8 to 0**, improved **2/32** readings and used **32**
counted search evaluations. The existing audit then measured **32/32** exact
codes and **32/32** target-nearest cells, versus **30/32** target-nearest for
the zero offsets. A fresh-output repeat was byte-identical.

- Synthetic report SHA-256:
  `c074fc98bf85fd1345e3fa0fa27391e34ddf62c685e68641297c2e25e788ef2f`
- Candidate Hamming sidecar SHA-256:
  `bbe3df9be314b1e41675a8900e793b592f07a87ff7402f5f0987d17edc08b0f7`
- Synthetic model input SHA-256:
  `2a7a8ed21c2a3b5d07c3a4c9e2a35a30c53fa91f0c261bb8da18fde546b49549`
- Synthetic Hamming input SHA-256:
  `f553318d6491bbf33e2910507500aae096ddd825b7ef5068b37f8fe8e89a3f90`
- Binary SHA-256 recorded by the retained report:
  `bd204cbc928dc5b05b67a522458c41c202a541580fe321484ab6c4f3c7d6dd8a`

The isolated release control used width 256, two depths, 16 questions and two
targets per question. It completed **393,312** counted evaluations in **47 ms**
across 16 workers. This is a throughput measurement, not a trained-model or
capability result.

Resources are recorded in the
[D84 ledger](../../../../labs/cpu-training-pilot-ask-offset-solver-2026-11-09/resource-ledger.json).
The previous admitted total `105,767.677487699 s` plus this unit's 1,800-second
projection gives `107,567.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib ask_offset_solver::tests
  -- --skip release_width_256_two_depth_solver_benchmark`: 5 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 470 passed,
  5 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `RAYON_NUM_THREADS=16 cargo test --release --offline --locked -p
  uor-r4-cpu-train --lib release_width_256_two_depth_solver_benchmark --
  --ignored --nocapture`: passed at 393,312 evaluations and 47 ms.
- The evidence-retaining binary test described above reproduced the retained
  report and candidate sidecar in 6.04 s.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver`: passed with only pre-existing warnings in untouched
  files.
- Targeted `rustfmt --check` and `git diff --check`: passed.
- `cargo fmt --all -- --check`: failed on pre-existing formatting outside this
  unit.

## Skills used

No system-provided Codex skill applied. The repository CPU-research,
task-loop and delivery instructions were followed directly.

## Still open

- Run the solver against the exact Claude-330 B2 checkpoints when their paths
  and hashes exist, then compare held-out target-nearest and pull distance to
  the zero-offset checkpoint.
- Use the optional weighted corpus for real prompt-conditioned pulls; the
  built-in B2 relation map is a deterministic initializer, not a learned
  conversational objective.
- Treat the output only as a checkpoint-compatible ask-table candidate. B2
  must pass before implementing the Q12 ask marker/exponent and serving parity
  sequence already described in `deepseek-245`.
- If B2 is positive, run the same candidate through the trained-offset audit,
  CPU/integer serving construction and a held-out reply panel. None of those
  steps ran in this unit.

## Plain summary

This unit gives the workstation a fast way to synthesize Claude-330's learned
question offsets from token geometry alone. For each question token and
Hamming depth, it tests the zero offset, exact offsets to requested target
tokens and a robust median, then improves the best starts by exact weighted
Hamming distance over rotated sign codes. It writes a complete checkpoint-like
sidecar rather than mutating the source checkpoint.

The synthetic run proves that a valid candidate can be made, preserved through
safetensors, reopened by the existing audit and repeated byte-for-byte. It does
not prove that a real checkpoint learns well, that B2 carries the required
information or that a reply becomes conversational. Those remain gated on a
real Claude-330 checkpoint and the project's separate training/serving bars.
