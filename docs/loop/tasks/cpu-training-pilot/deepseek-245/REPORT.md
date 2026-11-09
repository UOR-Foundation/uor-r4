# REPORT: cpu-training-pilot, deepseek-245

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- No Claude-330 checkpoint exists on `main` or in the open draft, so the real
  trained-offset reading is `NOT_RUN`. The binary is ready but has only been
  exercised on a controlled synthetic checkpoint.
- `loop claim` refused this unit because another open order's broad CPU glob
  includes one pointer-driver file. This unit touches no file from that order;
  it used an isolated worktree and will let the local policy check prove the
  actual changed-path scope.
- The first clippy pass found one unnecessary borrow in the new audit. It was
  removed; the new files now add no clippy warning.
- The audit measures relation-key sign codes, not the full B2 carry or a
  trained model's final answer. It is a diagnosis of whether the learned offset
  moves the question geometry in the intended direction.

## What changed

- Added `TrainedAskOffsets` and `cpu-ask-offset-audit`, which read
  `embedding.weight` from `model.safetensors` and every `d{depth}.ask` plus
  `d{depth}.key_rotation` from `hamming_read.safetensors`.
- The audit binds both file hashes, requires ask tables at every Hamming depth,
  rejects the type-spin/ask combination, validates finite F32 values and the
  head/width geometry, and writes only to a new report path.
- For all 16 relation/query pairs and every ask head/depth, it reports zero and
  trained Hamming distance to the target key, bit improvement, target rank
  among the 16 relation keys, nearest-key status and offset magnitude.
- Added tests for a controlled improvement/regression fixture, missing ask
  tables, non-finite values, partial depths, exact report generation, input
  hashes and argument parsing.
- Added an ask-head serving design gate naming the proposed Q12 integer table,
  artifact marker/exponent, i32 pre-rotation sum, refusal invariants and the
  float/artifact/CPU/integer parity sequence. No schema or serving code changed.
- Changed 2 tracked source files and added 2 source files, plus the task,
  serving design, resource ledger and one notebook row. The code is about 880
  lines; the ORDER gave no line estimate.

## Measurements

The controlled fixture uses a two-head, width-8, one-depth checkpoint. For
relation 16's unchanged question, zero and trained offsets both measure
`0` bits and the target remains nearest. For relation 17's question, a trained
`-2` offset on every coordinate changes the target distance from `0` to `4`
bits, records `-4` improvement and moves the target from nearest to
non-nearest. This checks direction and failure detection, not learning.

The full synthetic report test took `2.95 s` wall with 227,928 KiB maximum RSS.
The release binary SHA-256 is
`e677b3dff0a15cd47cf4be4300cb8ff461d11bcbebeaead7a0974f0ba7c1ea4b`.
Real Claude-330 seed runs, offset values and target-nearest counts remain
unavailable.

Resources are recorded in the
[D84 ledger](../../../../labs/cpu-training-pilot-ask-offset-audit-2026-11-09/resource-ledger.json).
The serving contract is in
[ASK_HEAD_SERVING_DESIGN.md](ASK_HEAD_SERVING_DESIGN.md).
The previous admitted total `103,967.677487699 s` plus this unit's
1,800-second projection gives `105,767.677487699 s`.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --lib
  ask_offset_audit::tests`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-audit`: 3 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 464 passed,
  4 ignored.
- `cargo check --offline --locked -p uor-r4-cpu-train --all-targets`: passed.
- `cargo build --release --offline --locked -p uor-r4-cpu-train --bin
  cpu-ask-offset-audit`: passed.
- `cargo clippy --offline --locked --no-deps -p uor-r4-cpu-train --bin
  cpu-ask-offset-audit`: passed with only pre-existing warnings in untouched
  files; the new files add none.
- Targeted `rustfmt --check` and `git diff --check`: passed.

## Skills used

No system-provided Codex skill applied. The CPU-research, serving-design and
loop-delivery instructions were followed directly.

## Still open

- Run the audit on both Claude-330 B2 checkpoints when their exact paths and
  `model.safetensors` / `hamming_read.safetensors` hashes exist.
- Compare its trained-target-nearest counts with B2's `R_ask` and
  `hamming_pull_distance` outcome before naming training, carry or payload as
  the next failure.
- Implement the integer artifact and serving path only if B2 is positive. The
  new design intentionally changes no schema.

## Plain summary

The question-keyed model learns one small vector offset for each token. The
question word is added to that offset, rotated, and compared with earlier key
words. A GPU run is about to produce checkpoints containing those offsets.

This unit gives the workstation a way to open the checkpoint directly and ask
whether the learned offsets do the intended geometric work. For each of the 16
question/key relations, it compares the code before and after the offset,
measures Hamming distance to the correct key, and checks whether the correct
key is the nearest of all 16 relation keys. It also records how much the offset
moved and at which depth/head the move helped or hurt.

A second document checks in the future serving design without building it. It
proposes an explicit Q12 integer offset table, shows exactly where the offsets
enter the query, and lists the fail-closed and parity checks that must pass
before an ask-head model can be exported and served. Nothing is added to the
serving format until B2 demonstrates that the mechanism works.
