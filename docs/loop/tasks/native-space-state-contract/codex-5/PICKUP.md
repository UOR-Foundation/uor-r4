# PICKUP: shared Q31 model state to integer serving

## Premise check

```sh
cd /home/callard/projects/codex-playground/uor-r4-corvuswing
git fetch --prune origin
git -C local/worktrees/codex-cuda-native-train status --short --branch
git -C local/worktrees/codex-cuda-native-train rev-parse HEAD
gh pr view 3968 --json state,isDraft,headRefOid,baseRefName,url
```

Expected: clean `codex/cuda-native-train` at
`ebeca85cb6c126c559df91c14ca2d6ea04c20b6b` plus this tracked pickup commit; PR #3968 is a draft
stacked on `codex/native-space-cycle` until PR #3967 merges. The repository wins over this note.

## Current model path

- `Q31HamiltonModel` owns one gate, step shift and epoch in
  `crates/uor-r4-discrete/src/native_vector.rs`.
- `NativeVectorEngine::train_model_step` runs the AOT CUDA per-item credit and commits the shared
  four-word state.
- RTX 4090: 4,096 training records match Rust word-for-word; 64 accepted shared steps lower
  aggregate L1 loss from `17,179,869,184` to `13,373,147,056`.
- CPU/CUDA gate and epoch match after every shared step; state round-trips through 48 bytes.
- The aggregation is currently host-side. No language, serving or throughput claim is made.

## Next task

Make the integer serving session consume the same serialized `Q31HamiltonModel` state. The next
order must define the shared operator boundary before editing `crates/uor-r4-integer`, preserve
the existing serving integer conventions, and prove a loaded trained gate produces the same raw
words in serving and Rust oracle.

## Off limits

- Do not resume `codex-7` loader symmetry or add more harness layers.
- Do not change Q31 arithmetic/domain without a reproduced defect.
- Do not use another model output or a float master to manufacture the served state.
- Do not touch other labs' work or the protected ASTRA paths.
- Use the RTX 4090 for CUDA checks.

## Standing rules

- `CHARTER.md` wins.
- One persistent native state, one typed operator definition, bit-identical train/serve words,
  lossless serialization (D85).
- Rust model code; AOT CUDA with a Rust oracle; no matmul/GEMM, softmax or squashing.
- Deliver through PR, fresh exact-head review, queue, merge and fresh-main verification.

## Manifest

```text
MANIFEST
- CUDA branch head: ebeca85cb6c126c559df91c14ca2d6ea04c20b6b  check: git rev-parse ebeca85cb
- shared model evidence: codex-5/REPORT.md  check: sed -n '/## Measurements/,/## Checks/p' docs/loop/tasks/native-space-state-contract/codex-5/REPORT.md
- CUDA integration tests: 4 passed  check: UOR_REQUIRE_CUDA=1 cargo test -p uor-r4-discrete --features cuda --test native_vector_cuda
- discrete library tests: 367 passed / 1 ignored  check: cargo test -p uor-r4-discrete --lib
- CUDA PR: 3968 draft stacked on PR 3967  check: gh pr view 3968 --json state,isDraft,baseRefName
- next: integer serving consumes the shared Q31 model state  check: top of this file
```
