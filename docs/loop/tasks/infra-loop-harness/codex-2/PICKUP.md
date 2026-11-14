# PICKUP: native vector geometry and Q31 CUDA delivery

This is the tracked mirror of `local/codex/PICKUP.md`. The repository is the authority if the two
disagree.

## Premise check

```sh
cd /home/callard/projects/codex-playground/uor-r4-corvuswing
git fetch --prune origin
git rev-parse origin/main
gh pr view 3967 --json state,isDraft,headRefOid,url
git -C local/worktrees/codex-native-space-cycle status --short --branch
git -C local/worktrees/codex-cuda-native-train status --short --branch
```

The previous repair head is `8674c7863`; this tracked note is committed after it. At the note's
parent, `origin/main` is `f40f06abe82447b3174418ffcd2f7245125e5c8f`; PR #3967 is open and draft;
the control-plane branch is `codex/native-space-cycle`; and `codex/cuda-native-train` is at
`01521bbb629c5aae4a910350cf10960172d13c55`. Report any disagreement before acting.

## Off limits

- Do not resume `codex-7` loader symmetry. The owner redirected work to the vector-geometry CUDA
  path.
- Do not touch another lab's branch, worktree, claim or uncommitted files.
- Preserve `/var/tmp/sc-direct-flowmux.iptmjZre/evidence/`, `host-7b224540/`,
  `source-c07.zjcaw7Q7/`, and `source/.mvm-test/target/`.
- Do not change Q31 arithmetic or its closed domain without a reproduced defect.
- Use the RTX 4090, not the 5090.
- Training or evaluation needs its pre-registration/FIT and GPU lease; finite kernel tests do not.
- Model code is Rust plus AOT CUDA. No Python, matmul/GEMM, softmax or squashing.

## Current state

PR #3966 is merged as `f40f06abe82447b3174418ffcd2f7245125e5c8f`. PR #3967 contains the
native-space control plane. Its second review repair added the active ORDER's owner approval,
attributed findings to the failing source, accepted the roadmap lifecycle, aligned the float
exception with D85, and added `crates/uor-r4-discrete/**` plus
`crates/uor-r4-integer/src/discrete_d11.rs` to the next item's scope.

At the repair commit, focused native-space tests passed 7. The full harness package reported
663 passed, 20 failed and 1 ignored because `TMPDIR` was inside the shared checkout; the R10
guard correctly refused synthetic writes. The clean rerun with `TMPDIR` outside the checkout then
passed 684 tests with 0 failures and 1 ignored. Formatting and `git diff --check` pass; Clippy,
full harness check, report check, PR check and a fresh exact-head review remain.

`codex/cuda-native-train` carries the actual Q31 vector operator in
`crates/uor-r4-discrete/src/native_vector.{rs,cu}` and `native_vector_device.rs`, with
`native_vector_qualify.rs` and `native_vector_cuda.rs`. Recorded RTX 4090 evidence: 4,096 Q31
quaternion pairs / 16,384 raw `i64` words bit-identical to the Rust oracle on `compute_89`,
CUDA 13.1; noncommutative Hamilton checks; no float/tensor/matrix/transcendental PTX; ptxas 40
registers and 0 spills. The branch is stacked on the control plane, has no PR, and the kernel is
not wired into training or serving.

## Next task

1. Push the repaired control-plane head and update PR #3967.
2. Rerun the full package with a temp root outside the checkout, for example
   `TMPDIR=$(mktemp -d /var/tmp/uor-harness.XXXXXX)`.
3. Pass formatting, full tests, Clippy, focused/full harness checks, stack-contract and roadmap
   checks, report check, local PR check and a fresh exact-head review.
4. Mark #3967 ready, queue, merge, and verify fresh `origin/main`.
5. Merge `origin/main` into `codex/cuda-native-train`, rerun PTX audit and RTX 4090 parity with
   `UOR_REQUIRE_CUDA=1`, and open its standalone PR.
6. After that merge, wire this same Q31 Hamilton operator into the native trainer and integer
   serving session. That is the immediate project implementation, not more harness bookkeeping.

Cancel if the compiled CUDA source no longer matches the reviewed arithmetic, the 4090 does not
reproduce the Rust words bit-for-bit, or serving needs a numerically different operator.

## Orders already carried out

- Merged and verified #3966 at `f40f06abe82447b3174418ffcd2f7245125e5c8f`.
- Created and repaired PR #3967's exact native-space contract.
- Built and locally qualified the integer Q31 Hamilton CUDA operator on the RTX 4090.
- Made invalid `Q31VectorState` values unconstructible.
- Preserved the ASTRA paths above.

## Standing rules

- `CHARTER.md` wins.
- One persistent native state, one typed operator definition, bit-identical train/serve words
  and lossless serialization (D85).
- Serve integers; floats are reference or diagnostic only.
- Rust model code; CUDA is AOT and has a Rust oracle.
- Deliver through PR, fresh exact-head review, queue, merge, fresh-main verification and cleanup.
- Re-read this note at every commit before changing direction.

## Running

No GPU lease or training run is active. Review session `99896` is stale because it observed a
moving worktree before this tracked repair commit; it is not evidence for the new head.

## Manifest

```text
MANIFEST
- origin/main: f40f06abe82447b3174418ffcd2f7245125e5c8f  check: git rev-parse origin/main
- PR 3967 remote head: e4ae7301b440ec628cca007d23dde41b04832f3b  check: gh pr view 3967 --json headRefOid
- previous repair head: 8674c78631f469df5596d53e67838377b646a4f9  check: git -C local/worktrees/codex-native-space-cycle rev-parse HEAD~1
- focused native-space tests: 7 passed  check: cd local/worktrees/codex-native-space-cycle && CARGO_TARGET_DIR=local/harness-target cargo test --locked -p uor-r4-harness native_space_contract
- full harness package: 684 passed / 0 failed / 1 ignored  check: docs/loop/tasks/infra-loop-harness/codex-2/REPORT.md
- CUDA branch head: 01521bbb629c5aae4a910350cf10960172d13c55  check: git -C local/worktrees/codex-cuda-native-train rev-parse HEAD
- CUDA branch tree: da7b796c1acae62ccf3876e896c37a4d9dbda1cb  check: git -C local/worktrees/codex-cuda-native-train rev-parse HEAD^{tree}
- CUDA PR: none yet  check: gh pr list --state open --head codex/cuda-native-train
- RTX 4090 Q31 parity: 16,384 raw i64 words bit-identical  check: UOR_REQUIRE_CUDA=1 cargo test -p uor-r4-discrete --features cuda --test native_vector_cuda
- next action: finish and merge #3967, then open the Q31 CUDA PR  check: gh pr view 3967 --json state,headRefOid
```
