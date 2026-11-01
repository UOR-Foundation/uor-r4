# Prep sheet: standalone inverted-file CUDA correctness on one RTX 5090, 2026-11-01

- Item: `uor-cuda-inverted-file` (#3475). ORDER: `docs/loop/tasks/uor-cuda-inverted-file/kimi-2/ORDER.md`.
- Source: the head this FIT names, recorded verbatim in the run identity and evidence manifest. Original base: `7d8a0c264`.
- Scope: finite AOT build and CUDA correctness only. No training, model evaluation, performance claim, portability claim, or release decision.

## Question

Does the repaired standalone package compile and admit its emitted PTX/cubin on an explicitly selected RTX 5090 (`sm_120`), then execute all four CUDA Driver API operations with meaningful full-vector CPU-oracle parity and byte-identical repeated row results?

## Pre-lease instrument review and repairs

Three non-author reviews were **NOT_FIT**. They found an unreachable CUDA ABSENT→cell branch, zeroed middle blocks that could not discriminate full-vector ranking, missing deterministic row evidence, and recipe/preservation gaps; then a short member readback and insufficient reinsertion proof; then an incorrect 499-row acceptance count after the corrupt-refusal row made the real count 504. No lease or spend occurred during those reviews.

The repaired host suite now varies all 32 vector elements, retains zero/extreme controls, and requires at least three top-8 changes and one prediction change between varied and zero-middle fixtures. The CUDA fixture uses the same generator, which must stay synchronized with the host fixture. It covers CPU ABSENT/reinsertion/status behavior and requires exact CSR equality with a fresh build after reinsertion. The kernel handles ABSENT before validating the old cell. Buffer capacity is separate from transfer length: the full table is allocated, only actual member bytes are uploaded, and the occupied-member count controls readback. The cubin recipe uses `-arch=$(SM_ARCH)` and explicit CUDA include/stub paths. The new CUDA test syntax-checks on the host and emits 504 deterministic rows per execution with independent result invariants.

## Arms and controls

1. **Identity and recipe.** Run `scripts/pod/uor-pod status`; select exactly `--gpu 5090 --count 1`, never the GPU-type fallback. Record `nvidia-smi` name/driver/compute capability, `nvcc --version`, `make -n cuda`, `make -n cuda-test`, pushed HEAD, clean status, and source hashes. Require compute capability 12.0 and a compatible Toolkit/driver. With `--no-bootstrap`, clone exactly to `/workspace/uor-r4`, record the clone command and `git status --porcelain`, and use host-side lease/status checks plus `uor-pod reap` as the idle backstop. Override `CUDA_HOME` only when the actual CUDA 12.8 installation requires it; this Make variable does not change source identity. Setup is limited to the standalone package and required CUDA toolchain, with no model-crate bootstrap.
2. **Comparator and sensitivity.** Run `make host-test` at the same head on laptop and pod and compare normalized output. Require the host fixture-sensitivity checks on both. A red oracle or sensitivity check is a gate defect, not a CUDA result.
3. **AOT and admission.** Run `UOR_NVCC_ARCH=compute_120 make cuda`. Preserve emitted PTX/cubin and admission output outside `build/` before any clean. Require exactly the four declared entries, admission success, and module hashes. Architecture evidence is `nvidia-smi` compute capability 12.0 plus the recorded `compute_120`/`sm_120` build flags plus successful four-entry Driver API loading on that card; the empty `cuobjdump -arch` output is not used.
4. **Negative admission control.** Add a fifth entry to a scratch PTX copy and require the admission tool to reject it. This proves the gate can fail without changing the retained module.
5. **CUDA correctness.** After `make cuda-test`, run exactly `./build/cuda_parity_test --out /workspace/uor-r4/evals/cuda-release-3/run-<n>.tsv`. `make cuda-test` itself emits no row file and its exit 0 is not the determinism evidence. Cover V ∈ {64,256}, K ∈ {1,8,16}, four query seeds per cell, 64 incremental updates with fresh-build equality, non-root refusal, explicit CPU removal/CUDA reinsertion, one corrupt-CSR update refusal, and extreme signed-integer controls. Every row checks status and, for scans, prediction/top-8/admitted invariants plus CPU equality. Reinsertion also requires exact CSR equality with a fresh build. There are 84 rows per configuration and **504 rows per execution**.
6. **Determinism.** Execute three `--out` tests. Require `wc -l` = 504 for each row file, preserve and compare all three byte-for-byte, and record module hashes before/after each. Preserve the first module before a clean rebuild; compare PTX/cubin hashes after rebuilding. Different compiler bytes with identical rows are a compiler-reproducibility observation, not a semantic failure.

## Decision table

- **Recipe/instrument fault:** SSH/setup/toolchain/include/flag failure, wrong card/capability, missing input, invalid fixture, or a failed archive. No CUDA conclusion. Repair the instrument and rerun only what was affected.
- **Package-source compile error:** the toolchain found the headers and compiled the PTX, but the package's C++ failed to compile. This is a source defect requiring a new head; no CUDA conclusion.
- **Module/source defect:** a matching-architecture admitted module fails to load, or any row has a parity/refusal/status/invariant mismatch with the oracle and sensitivity checks green. Stop before benchmarking; make the smallest host-tested repair and use a new pushed head/run id.
- **Pass:** all 504 rows pass in each of three byte-identical executions; AOT/admission and negative controls pass. This unlocks only the separately reviewed matched full-scan performance instrument and additional-architecture/driver work. It is not a performance or release claim.

## Exact negative scope

This is one card/driver/Toolkit and a fixed grid. The planted offset refusal is driver-side; kernel-side corrupt-CSR rejection is **NOT_RUN**. V ∈ {1,4096}, additional stale/corrupt variants, all-absent CUDA scans, K ∈ {0,17}, concurrent kernels, performance, tuned comparisons, additional architectures/drivers, energy, cross-library comparison, and external replication remain **NOT_RUN**. Single-threaded build/update kernels make the determinism check intentionally weak; it is not a concurrency qualification. Admission runs per build, not once per test execution.

## Resources and preservation

One leased RTX 5090 on Vast, initially one hour at the displayed rate (about $0.563/h; actual provider uptime/rate is reported). Setup, three short executions, negative controls, and archival are projected below 40 minutes; renew a healthy instrument delay rather than canceling it for an estimate. Stop and tear down on a source defect because host repair needs no GPU. No other pod/GPU is touched, and the shared ≤4 pods/≤$8/h caps apply.

Attempts 1 and 2 are sealed read-only at `local/kimi/cuda-release-1/` and `local/kimi/cuda-release-2/`; neither is written again. The guarded retry's original evidence root is `/workspace/uor-r4/evals/cuda-release-3/` on the new pod. Preserve every command, source/module checksum, PTX/cubin, row file, clean-rebuild log/hash, planted negative PTX, its exact command, and rejection log there before teardown. Pull into `local/kimi/cuda-release-3/` and run `shasum -a 256 -c SHA256SUMS`. New attempts use new roots. Local build preparation adds less than 100 MiB and remote evidence is expected below 20 MiB. No generated binary is committed.

## FIT record

**TEST FITNESS: FIT**, 2026-11-01, fifth non-author check (`uor-experiment-designer`). Exact guarded retry source: `18a1c8ad79a468025b0e82e79705c7266abac03c`, with the driver pre-validation, restored kernel bounds, and the driver-side corrupt-CSR test row. Scope: one RTX 5090 AOT/Driver API correctness gate with 504 rows per `--out` execution and three byte-identical row files. Any further change to the kernel, driver, CUDA test, or Make recipe, and any architecture change, requires a new pushed head and another FIT.

`cuda::full_scan` constructs its comparison CSR with the CPU `build(table, index, true)` before running the CUDA full-scan kernel. Probe rows use the GPU-built CSR. This is intentional control structure, not a claim that both scans consume a GPU-built index. The `make cuda-test` smoke run is not counted among the three `--out` determinism executions. CPU and CUDA are separate transcriptions of one algorithm; shared design errors are outside this parity check's detection scope.

## Attempt 1 outcome and retry rule

At `71c84dfa3b083889fb3896886da973354fa87afa`, host oracle and AOT/admission passed with the recorded `compute_120`/`sm_120` build flags, and a planted fifth PTX entry was rejected. The `cuobjdump -arch` output was empty and was not used as architecture evidence. The CUDA test then failed at C++ compilation because three `void* args[]` arrays took addresses of const `count` values. No CUDA parity execution or performance result occurred. Logs and emitted module were pulled to `local/kimi/cuda-release-1/` with SHA-256 verification, the lease was released, and `vast-58641884` was deleted. The retry changes only those three declarations to mutable values, uses a new pushed head and FIT, and writes a new evidence root (`cuda-release-2`).
