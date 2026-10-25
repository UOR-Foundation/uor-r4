# ORDER: float-training-module, antigravity-72

Lab: antigravity. Item: [float-training-module](../../../../roadmap/items/float-training-module.md). Base: `7c6f1f4ad`.

## Goal

The item's done-when reads:

> A new module builds CPU-only on the laptop with CPU tests, passes the banned-token scan under its own scanned entry, and carries the 24-kernel extract with no dependency on `crates/uor-r4-training`. The CUDA kernels are built ahead of time by nvcc on the GPU pod (CHARTER §3.4); on the laptop they get a stub-nvcc type check only.

Deliver the standalone scanned D37 offline float training crate `crates/uor-r4-float-train` (`uor-r4-float-train`) in the workspace, carrying the 24-kernel geometric stack extract (`cuda_stack_kernels.cu` -> `cuda_stack_kernels.ptx`, `geometric_stack_native`, and ahead-of-time CUDA wrappers) with zero dependency on `crates/uor-r4-training` (`cargo tree -p uor-r4-float-train` verified by an integration test), its own explicit scanned D37 allowlist entry in `crates/uor-r4-harness/src/ratchet.rs` (`uor-harness ratchet d23|d25|d32` passing with `0` banned sites), finite CPU controls for all 24 kernels, stub-`nvcc` `--features cuda` type check, design note in `docs/labs/float-training-module/README.md`, and notebook row in `docs/research/notebook/float-training.md`.

## Cancel probe (run first)

1. **Umbrella lock & no-overlap check (`run-antigravity.txt` Part 3 Step 2):** `uor-harness loop claim float-training-module --lab-topic float-training-module` refused with:
   `uor-harness: loop claim: refused: scope docs/roadmap/items/float-training-module.md overlaps docs/roadmap/items/*.md of item infra-loop-harness (PR #3169 (docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md))`
   Inspected open PRs via `gh pr list --state open --json number,headRefName,files`: PR `#3169` (`codex/d63-finite-probe-scope`, idle draft since `2026-10-25T01:07:47Z`) touches only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/{ORDER.md,PLAN.md,REPORT.md}`, and PR `#3203` (`claude/carrier-hard-selection`) touches only `crates/uor-r4-discrete/**` and `carrier-credit-hard-selection` docs. Zero files overlap `float-training-module`.
2. **Repository state check at `7c6f1f4ad` (class: `instrument`, task stands):**
   - `ls crates/uor-r4-float-train` returns `No such file or directory`: no standalone D37 float training crate exists in `Cargo.toml`.
   - `crates/uor-r4-training/src/cuda_stack_kernels.cu` and `geometric_stack_native.rs` still reside only inside `crates/uor-r4-training`, which carries historical gated dense/sigmoid modules.
   - `crates/uor-r4-harness/src/ratchet.rs` has no dedicated D37 scanned crate allowlist check verifying that `crates/uor-r4-float-train` is scanned with zero `D23`/`D25` hits while exempted from `D32` (`crates/uor-r4-discrete` integer-only).

## Files in scope

- `crates/uor-r4-float-train/**`
- `Cargo.toml`
- `Cargo.lock`
- `crates/uor-r4-training/**`
- `crates/uor-r4-harness/src/ratchet.rs`
- `docs/harness/prep/float-training-module*`
- `docs/labs/float-training-module/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/float-training-module.md`

## Done when

1. Design note posted on `#2823` and committed at `docs/labs/float-training-module/README.md` naming the crate (`uor-r4-float-train`), all 24 kernels in the extract, the zero-dependency-on-`uor-r4-training` invariant, and the `ratchet.rs` D37 scanned entry.
2. `crates/uor-r4-float-train` builds CPU-only (`cargo build -p uor-r4-float-train`) and type-checks with stub `nvcc` under `--features cuda`, passes finite CPU control tests across all 24 kernels plus the `cargo tree -p uor-r4-float-train` exclusion test and D29 PTX scan, and passes `uor-harness ratchet d23`, `uor-harness ratchet d25 --list`, and `uor-harness ratchet d32 --list`.
3. `docs/research/notebook/float-training.md` records the finite qualification row, `docs/roadmap/items/float-training-module.md` adds `notebook: docs/research/notebook/float-training.md` and `status: done` (`Closes #2823`), and `docs/start/FILEMAP.md` adds `crates/uor-r4-float-train` and `docs/labs/float-training-module`.

## Stop and ask if

- Any file in `crates/uor-r4-float-train` introduces a dependency on `crates/uor-r4-training`, softmax/squashing (D23), dense matmul/GEMM (D25/D30), or NVRTC (D29).
- Any edit falls outside the declared scope or modifies another crate's `Cargo.toml` dependencies outside `uor-r4-float-train` and `uor-r4-training`.
