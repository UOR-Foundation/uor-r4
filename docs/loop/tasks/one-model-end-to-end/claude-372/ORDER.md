# ORDER: one-model-end-to-end, claude-372 (CUDA leg of the native-space parity contract)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `fccad269b8e81100a82d21082e7eae4aa9055874`.

## Goal

**Run the CUDA training kernels and the CPU training oracle from byte-identical bytes on `wick`'s RTX 4090, and compare raw words at each step of one training transition: read vectors, the sixteen counterfactual loss words per candidate, the chosen proposal, and the artifact bytes after apply.** No training run, no corpus, no capability or speed claim.

**Why now.** claude-371 (PR #3962) wrote the inventory and the CPU-to-serving leg. Its REPORT names this leg as the smallest next training dependency: until it passes, a CUDA training run's updates are not known to be the ones the CPU and serving compute.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a test that runs `discrete_forward`, `discrete_counterfactual`, `discrete_choose` and `discrete_apply` and the CPU scorer, chooser and applier on the same bytes and compares their words.
- **Result (at base):** none. The CPU tests copy CUDA's rules by hand (`apply_status_order_and_noop_match_cuda`, `discrete_route.rs`). Device tests in `uor-r4-discrete/src/device.rs` compare against host witnesses. No test loads one artifact on both.

## Claim

Written by hand, as claude-359 to claude-371 were. No CUDA-to-CPU parity test exists at this order's commit.

## Design

- **Start:** `Device::new` / `new_with_read` builds the model on the GPU; `snapshot` then `export_d11` writes `UORLUT01` bytes; the CPU scorer, chooser and applier load those bytes. Route and Flock fixtures as in claude-371.
- **Place:** a `#[cfg(all(test, feature = "cuda"))]` module in `uor-r4-discrete`, because `Device::apply` is `pub(crate)` (the training hold). `uor-r4-cpu-train` becomes a dev-dependency; the two sides exchange bytes and integer words only.
- **Compare:** forward vectors against `CpuDiscreteProposalScorer::query`; `candidate_losses` against `score().losses` for every Route, VectorBlock and Gate parameter; the proposal observation against `choose`; after one `apply` on each side, exported bytes against `artifact_bytes`.
- **Plant:** one fault per CUDA-side step, each caught by the named comparison.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-372/**`
- `crates/uor-r4-discrete/Cargo.toml` (one dev-dependency), `crates/uor-r4-discrete/src/lib.rs` (one test-only module line), one new test-only module file
- the discrete line's notebook, one row

## Done when

1. The test runs on `wick` (container `uor-r4-claude`, `UOR_NVCC_ARCH=compute_89`) and either every compared word is equal, or each difference is reported with its step, parameter and row.
2. Each planted fault is caught by its named comparison.
3. A REPORT, negatives first, with the backend identities (GPU, driver, nvcc, kernel sha) and the next dependency.

## Stop list

- No training run, corpus, pod or rented GPU. `wick` only, inside Claude's own container; no other lab's process or file touched.
- No change to any kernel, serving code, export or artifact format, or to the public training API.
- No harness, loop-kit or bookkeeping change.
