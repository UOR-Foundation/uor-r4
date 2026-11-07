# REPORT: one-model-end-to-end, deepseek-19 (checkpoint-parameterized centroid inputs)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `3f41847a3`.

## What did not work, first

- **DeepSeek-18's checkpoint had become stale before it could run.** It was bound to
  Claude-297's n3S step-8,192 pair. Claude-301's b64L step-16,384 recipe then beat D3's
  T2 and unigram at both seeds, and Claude-302's width-1,024 unit stopped at its memory
  and accumulation gates. The width-256 b64L recipe therefore remains the target.
- **A checkpoint identity was embedded in executable seed branches.** The old job repeated
  the archive, checkpoint, file paths and hashes in `case $SEED`; the CPU instruction had
  a second hand-substituted copy. That made a target change an opportunity for the two
  consumers to drift.
- **The normal claim command saw a preserved, already-merged worktree.** DeepSeek-16's
  `.worktrees/deepseek-centroid-cache` still holds ignored local evidence. `loop claim`
  therefore refused this item even though PR #3804 was merged. The worktree was preserved;
  this unit used a new worktree from fresh `main`, with the owner's explicit approval and
  the exception recorded in the ORDER.

## What changed

- Added
  [`checkpoint-inputs.json`](../../../../labs/one-model-end-to-end/centroid-cache-2026-11-06/checkpoint-inputs.json)
  as the single machine-readable checkpoint authority. It pins the source, expected run
  and model config, and each seed's sealed archive, run case, checkpoint directory, run
  config and three checkpoint files.
- The capture job now requires `UOR_CENTROID_SPEC`, validates the schema and safe paths,
  extracts exactly the paths named there, checks every file against both the specification
  and the archive's inner `SHA256SUMS.files`, verifies the b64L run/model configs, and
  records the specification hash and checkpoint path in `status.json`.
- The controller copies and hash-checks both the job and the specification on each pod.
  It derives the source pin from the specification instead of carrying another literal.
- The CPU restore now processes both seeds from the same specification in one loop. It has
  no hand-substituted seed table and checks the captured status and CPU reports against the
  same input identity.
- The sheet now names the current b64L checkpoint and archives, while retaining
  DeepSeek-16 through DeepSeek-18 as the historical n3S records.

Size: see `uor-harness loop pr-check --local`; this is a documentation-and-execution
package change, with no runtime Rust or model arithmetic.

## Measurements

- Seed-1 archive `f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba`.
- Seed-2 archive `1f24f03a4b638f7ad6fafd184f980bca324de758085a62e5b5d0dfb3a4f6ab32`.
- Both archives were downloaded from the project store and their hashes matched the pin.
- Both step-16,384 run configs name seed 1 or 2, D3, `nearest`, `train_as_served: true`,
  width 256, Hamming read 4, batch 64, micro-batch 16 and 16,384 steps.
- Both model configs name width 256, context 192, the six-recurrence pattern, four heads,
  Hamilton gate, SpiralCore/FBS product head and geometric architecture.
- All eight selected files matched both the input specification and the archive's inner
  `SHA256SUMS.files`.
- No model, cache, pod, GPU, CPU evaluator or training process ran.

The final job hash is
`ec2244b6a2f5aee8fb58e48205cac73603c8ece71f46ba27c17ac957b38ca33f`;
the input-specification hash is
`3bfd689fa2d454027392e523ca4ba93af45e1d8856e42ffdc1a2f193dc536910`.

## Checks run at the head

- `bash -n` and `shellcheck -x` on the capture job: passed.
- `--check-spec` for seeds 1 and 2: passed.
- Local archive, inner-manifest, checkpoint-file and run/model-config verification for both
  seeds: passed.
- `bash -n` and `shellcheck` on the sheet's controller block: passed.
- `bash -n` and `shellcheck` on the combined CPU restore, evaluation and aggregate blocks:
  passed.
- `git diff --check`: passed.
- `uor-harness prep-check`: complete and `PENDING`.
- `uor-harness loop report --check --order` and `loop pr-check --local`: passed.

Not run: CUDA tests, pod bootstrap, GPU capture, cache evaluation, panel scoring or any
model computation.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, geometry, delivery and
compute rules were used directly.

## Still open

1. Obtain a fresh non-author `TEST FITNESS: FIT` for the retargeted sheet.
2. Obtain the D67 pre-run review naming the final job/spec hashes and two leases.
3. Run both captures without touching another lab's pod or work.
4. Restore and evaluate both caches on the D84 host.
5. Record the registered result, upload and read back the CPU evidence, and update the
   float-training notebook in a new active result unit.

## Plain summary

The centroid experiment had been tied to an older checkpoint. The project's newer
width-256 human-text model now beats the simple baselines, while the attempted wider model
did not get through its safety gates. This unit moves all checkpoint-specific facts into
one reviewed JSON file. The pod job and the laptop evaluator now read that same file, so
they cannot quietly select different archives or checkpoints. The new target is the current
step-16,384 model at both seeds. Nothing was trained or scored.
