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
  two-target corpus. Its 44 ms wall measures local search throughput only, not
  quality on a trained checkpoint.
- The first validation pass covered malformed pull corpora but did not yet pin
  the malformed and non-finite checkpoint cases required by this order. Those
  controls, deterministic metadata serialization and output-path collision
  refusal were added before delivery.
- Fresh exact-head review then found four material gaps: two-file output was not
  transactional under failures, custom pull corpora were identified only by
  path, the evaluation cap omitted candidate scoring, and the accepted
  checkpoint inventory was only the subset needed by this solver. A public
  helper also had unchecked token addition. All four are repaired and covered
  by tests at this head.
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
  multiple targets per question. Its raw bytes are SHA-256 bound in the report;
  `--expected-pulls-sha256` can require a pre-registered value.
- For each question/depth, candidates are the zero offset, every distinct
  exact target offset and a weighted coordinate median. Up to four best starts
  run a deterministic coordinate search at steps
  `16384,4096,1024,256,64,16,4,1`; an exact zero objective stops early.
- The objective is weighted XOR/popcount Hamming distance over the ask half of
  the heads after `hamilton_gate_f32` rotation and sign packing. There is no
  dense matrix multiplication or softmax in this path.
- Independent question/depth cells run through the existing Rayon pool.
  Ordered parallel collection and explicit sorting produce stable readings;
  tensor names are sorted before serialization.
- `--max-evaluations` now bounds every objective call per question/depth. A
  reading records candidate, coordinate and total objective evaluations,
  search starts and whether the budget stopped candidate selection or search.
- The output is a new complete `hamming_read.safetensors`: all original
  tensors and metadata are preserved, existing ask tables are replaced and
  every solved depth receives a full F32 `(vocab,width)` ask table. Safetensors
  header metadata is canonicalized for deterministic fresh-process bytes.
- The checkpoint reader now requires the complete Claude-330 per-depth
  query/key/value rotations, margins and weights, plus valid scalar or mixed
  copy keys. Empty or zero-weight pulls, out-of-vocab tokens, bad shapes,
  invalid head geometry, partial depths, checked-overflow failures, non-finite
  values, type-spin mixing and invalid copy-key records fail closed.
- Output names are normalized through their real parent directories so aliases
  collide before solving. Candidate and report bytes are written to synced
  sibling temporaries and promoted without replacement. If the second
  promotion fails, the first promoted output is removed.
- The CLI reopens the candidate with `TrainedAskOffsets`, audits the fixed B2
  relation map, independently re-audits every selected pull from the serialized
  candidate and writes candidate/report hashes plus per-cell
  zero/solved/improvement readings.
- No serving, artifact-schema, training-default, GPU or Claude-owned file was
  changed.
- Size: `pr-check` reports 11 changed paths: code 4 files `+1,977/-0`,
  documentation 3 files `+241/-0`, and other/evidence/claim files 4 files
  `+1,026/-0` (`+3,244/-0` total). The ORDER gave no line estimate. Inline unit
  tests are counted in the code class.

## Measurements

The retained synthetic checkpoint has 64 tokens, width 8, two Hamming depths
and one copy/ask head pair. Over 16 questions and two depths it reduced total
weighted search bits from **8 to 0**, improved **2/32** readings and used **96**
total objective evaluations, all during candidate selection; no cell exhausted
its budget. The selected-corpus audit reproduced the same 8-to-0 result across
all 32 readings. The existing relation audit then measured **32/32** exact
codes and **32/32** target-nearest cells, versus **30/32** target-nearest for
the zero offsets. A fresh-output repeat was byte-identical.

- Synthetic report SHA-256:
  `dbcb43538888810d52aad1386c86e519501cf6200dbe047f6aaf0c1f8ccdb2e8`
- Candidate Hamming sidecar SHA-256:
  `ac67ea16710d09d8d98c0391ab549f166f93304c40a401f2f1a7350d42929d35`
- Synthetic model input SHA-256:
  `2a7a8ed21c2a3b5d07c3a4c9e2a35a30c53fa91f0c261bb8da18fde546b49549`
- Synthetic Hamming input SHA-256:
  `ae901772f76ed6ee3a0d5b19cd4775d315ff283c905ff8778b3abb5c73e581c7`
- Default pull corpus SHA-256:
  `5b0b129803110a3efb90d063ca6c84bcdc9ac6ac0079ad2ccf733a09c61a80e0`
- Binary SHA-256 recorded by the retained report:
  `3228f4997e9e6f4a1eae31a581ba086b2fa6f88d345444226e871c25fa58adb7`

The isolated release control used width 256, two depths, 16 questions and two
targets per question. It completed **393,344** total objective evaluations in
**44 ms** across 16 workers, with no exhausted cell. Maximum resident set size
was 56,736 KiB for the benchmark and 236,160 KiB for the evidence test. This is
a throughput measurement, not a trained-model or capability result.

Resources are recorded in the
[D84 ledger](../../../../labs/cpu-training-pilot-ask-offset-solver-2026-11-09/resource-ledger.json).
The previous admitted total `105,767.677487699 s` plus this unit's 1,800-second
projection gives `107,567.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib ask_offset_solver::tests
  -- --skip release_width_256_two_depth_solver_benchmark`: 7 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver`: 4 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 472 passed,
  5 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `RAYON_NUM_THREADS=16 cargo test --release --offline --locked -p
  uor-r4-cpu-train --lib release_width_256_two_depth_solver_benchmark --
  --ignored --nocapture`: passed at 393,344 total evaluations and 44 ms.
- The evidence-retaining binary test described above reproduced the retained
  report and candidate sidecar in 5.90 s, peak RSS 236,160 KiB.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-offset-solver`: passed with only pre-existing warnings in untouched
  files.
- Targeted `rustfmt --check` and `git diff --check`: passed.
- `uor-harness loop report --check`, `loop pr-check --local` and
  `loop gates --full`: passed.
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
