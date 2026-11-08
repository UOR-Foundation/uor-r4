# REPORT: one-model-end-to-end, deepseek-33

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Preparation at base `208a29ce5`; no
parity reading yet.

## What did not work, first

- The earlier packet-parity unit built the common reader but did not run it on a
  checkpoint.
- Comparing the separately produced float and integer aggregate top-1 counts
  would use different renderings and contexts; it is not same-scorer parity.
- `m2-reply512` remains unavailable, so this run uses `m2-fresh256` only for
  parity diagnostics and makes no M2 claim.

## What changed

- Added one closed parity job for the sealed content-ratio source.
- It re-verifies the complete source seal, then binds selected step 512 and
  final step 2,048 checkpoint files, their integer artifacts, the tokenizer and
  all four panel files by SHA-256.
- It runs both `per-turn/v1` and `one-user-turn/v1` through the merged
  `packet-parity` binary for both checkpoints.
- Reports, source subset, binary and CPU/GPU identities are sealed, uploaded,
  read back and checked before the owned pod can be deleted.

## What ran

Only local shell syntax, ShellCheck and diff checks. No pod or model forward has
run yet.

## Measurements

Pending the GPU run. The source run is:

`results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88/`.

## Evidence

Pending. The source seal, checkpoint and artifact hashes are pinned in
`job.sh`.

## Checks run at the head

- `bash -n` and ShellCheck on the parity job: passed.
- `git diff --check`: passed.
- Local CUDA binary tests could not run because this machine has no `nvcc`; the
  pod build is the executable check.

## Skills used

No system-provided Codex skill applied. The repository's D22, D37, GPU, serving
and delivery rules were used directly.

## Still open

- Run the four parity comparisons and record the actual mismatch and rank
  readings.
- `m2-reply512` recovery remains a separate unresolved gap.

## Plain summary

The merged parity instrument now has a closed job for the actual trained
checkpoints; the numbers remain to be measured.
