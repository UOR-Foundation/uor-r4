# REPORT: one-model-end-to-end, codex-6

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Order: [ORDER.md](ORDER.md). Result: [PARITY-RESULT.md](PARITY-RESULT.md).

## What did not work, first

- The first Route trace attempt returned 32 zero words per row: `discrete_forward` still passed
  the trace pointer only for active-leaf modes. Passing the same pointer in Route mode fixed it.
- The first implementation changed the independently qualified bucket-writer source hash even
  though the trace was a read-only observation. The traced Route helper was moved after the
  bucket generator's `// Statuses added` boundary; the ordinary helper/writer prefix and the
  committed bucket receipt now remain byte-identical.
- The full CPU-training suite reports two failures in FBS ask/carry parity. Claude-371 already
  recorded both failing at the parent base with its change set aside. They are outside this
  trace unit; all other CPU-training tests pass.
- The exact-head review at `1c925165e` found two P2 issues: an untraced Relation read still
  allocated and copied trace-only relation scores, and the active-slot control asserted
  non-vacuity without pinning the measured 126 words. Both are fixed at the head recorded in the
  Git history; the delta is re-reviewed before delivery.

## What changed

- `IntegerDiscreteModel::evaluate_prefix_traced` exposes the exact Route and active-leaf words
  produced by the same serving read. `evaluate_prefix` still takes the untraced path.
- `CpuDiscreteProposalScorer::query_traced` and the public Route/Flock/Relation readers expose the
  CPU oracle words.
- `Device::forward_traced` now supports Route as well as Flock/Relation.
  `Forward::read_trace` returns the mode-specific raw words; ordinary `forward` allocates none.
- Route header and stage words pin the configured frame and selected port. Active-leaf levels
  trace selection even at gate zero, matching CUDA.
- The 39-prefix contract compares every trace word through CUDA, CPU and serving. Fault controls
  retain the output-only counts and add direct trace counts.

## Measurements

Device and build:

- RTX 4090, compute capability 8.9, driver 595.99.02.
- CUDA 13.1, AOT `compute_89`.
- 39 prefixes; two fixtures; CPU debug test build.

Parity:

- Route and Flock: every read vector, admitted cell, top-8, token and trace word matched across
  CUDA, CPU and serving.
- Route and Flock: all 624 carry words and all 32 counterfactual losses matched.
- Route candidate 7 and Flock candidate 1 applied, epoch `0 -> 1`.
- Untraced and traced forward returned identical public words; the untraced result held no trace.

Fault visibility:

- Route frames: direct trace `252 / 252`; output-only K1 remains `108 seen / 144 missed`.
- Flock gates: direct trace `8 / 8`; output-only K1 remains `3 seen / 5 missed`.
- Active trace: 126 kept-node words and 126 transport-action words over the fixture prefixes.
- One planted carry word and stale proposal reuse remained detected.

No training run, capability, language result, throughput or performance claim is made.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train
  native_space_cuda_parity_contract_runs_the_same_words --features cuda --lib` - passed on the
  RTX 4090.
- `cargo test --locked -p uor-r4-cpu-train
  native_space_parity_contract_catches_one_planted_fault_per_category --lib` - passed.
- `cargo test --locked -p uor-r4-cpu-train
  native_space_trace_contract_exposes_active_leaf_kept_slots --lib` - passed.
- Full `uor-r4-discrete --lib`: **355 passed, 0 failed, 1 ignored**.
- Full `uor-r4-cpu-train --lib`: **507 passed, 2 failed, 5 ignored**. Both failures are the
  parent-base FBS ask/carry tests named above. With those two named tests filtered:
  **507 passed, 0 failed, 5 ignored**.
- `bucket_source::tests::committed_gpu_qualification_receipts_pass_b1_b2_b3_and_match_across_builds`
  - passed; the bucket helper/writer source and its committed receipt are unchanged.
- `rustfmt --check` on the changed Rust files - passed.
- `cargo clippy --no-deps` for `uor-r4-cpu-train` (with `cuda`), `uor-r4-discrete` and
  `uor-r4-integer` - passed; repository warnings remain.
- `UOR_LAB=codex uor-harness check` - **0 errors, 314 warnings**.
- `loop pr-check --local --base e978e53ef` - **PASS**, with the repository's scope, commit-order
  and FILEMAP advisories.
- Fresh-context exact-head review: the first pass returned CHANGES with the two P2 findings above;
  the repaired head is submitted for delta re-review. External PR publication requires owner
  approval under the repository AGENTS instruction.

## Skills used

- `uor-research-architect`: kept the unit finite, artifact-bound and separate from language
  qualification.
- `working-discipline`: read the existing state first, compared the shipped paths, retained row
  words and reported the parent-base failures separately.
- `compaction`: recovered the interrupted CUDA-parity state without reconstructing measurements
  from memory.

## Still open

- Loader symmetry (roadmap step 2).
- Shared typed operator definitions (step 3).
- Table/Route/Gate update-status parity (step 4).
- Generation and stop state (step 5).
- Exact copy and turn-memory carry (step 6).

## Plain summary

The model now shows which Route frame and port it chose, and which nodes and transport actions each
active leaf kept, as the same raw words from the RTX 4090, the CPU oracle and integer serving. The
previously invisible 144 route faults and five gate faults are now directly observable. This is
still finite construction parity, not training or language capability.
