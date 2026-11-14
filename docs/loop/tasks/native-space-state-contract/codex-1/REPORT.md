# REPORT: native-space-state-contract, codex-1

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- The first Q31 CPU fixture stored `2^31` in `i32`. Rust correctly exposed that this wraps to
  the negative minimum instead of representing positive one. The ABI now stores Q31 words as
  signed 64-bit values whose admitted coordinate range is `-2^31..=2^31`.
- The first cudarc launch passed the item count by value; this cudarc version requires a
  scalar argument by reference. The launch now passes `&count`.
- The CUDA integration test initially saw `CUDA_ERROR_NO_DEVICE` inside the filesystem
  sandbox even though `nvidia-smi` identified the RTX 4090. Rerunning the finite test with
  device access passed.
- The full `uor-r4-discrete` library suite has one unrelated existing failure:
  `controller::completion::baseline_authentication::tests::authentication_publication_is_immutable_and_resume_fails_closed`
  returns `No such file or directory`. All focused native-vector tests pass; the unrelated
  failure was not changed or masked.

## What changed

One native vector transition now exists as an integer-only Q31 operator:

`NativeVectorState (i64x4) -> q31 Hamilton rotate -> NativeVectorState (i64x4)`

The Rust oracle forms sixteen signed 32x32-to-64-bit products, accumulates modulo `2^64`,
rounds by `2^30`, and shifts right by 31. The CUDA implementation uses the same order. The
serialized carrier is four consecutive little-endian signed 64-bit words.

The new `native-vector.cu` module is compiled ahead of time by nvcc for `sm_89`, inspected by
the existing PTX admission, assembled into a cubin, and loaded only through
`NativeVectorEngine`. There is no NVRTC or PTX fallback. The operator contains no float,
tensor, transcendental or matrix-multiply instruction.

Files:

- `crates/uor-r4-discrete/src/native_vector.rs`: state type, Rust oracle and serialization.
- `crates/uor-r4-discrete/src/native_vector.cu`: AOT CUDA integer kernel.
- `crates/uor-r4-discrete/src/native_vector_device.rs`: admitted cubin loader and batch launch.
- `crates/uor-r4-discrete/tests/native_vector_cuda.rs`: 4,096-item raw-word parity and a
  planted shift-change control.
- `crates/uor-r4-discrete/examples/native_vector_qualify.rs`: retained finite qualification
  receipt.

Size: 698 added/deleted lines across ten files, about 1.55 times the ORDER's 450-line upper
estimate. The overrun is the retained qualification example, exact serialization tests and the
delivery record. Size is reported, not capped (D48).

## Measurements

Device: NVIDIA GeForce RTX 4090, compute capability 8.9, driver 595.99.02.

Finite qualification:

- 4,096 Q31 quaternion pairs.
- 16,384 raw `i64` output words compared.
- CUDA and Rust outputs: bit-identical.
- Optimized-build wall time including host allocation, H2D, launch and D2H:
  `3,099,351 ns`.
- This is not a kernel-throughput result and is not used as a speed claim.

Generated image:

- Entry: `native_q31_hamilton_rotate`.
- Target: `sm_89`.
- PTX instruction inventory included 16 `mul.lo.s64`, 15 `add.s64`, 4 `sub.s64` and
  4 `shr.s64`; the admission found no float/tensor/transcendental instruction.
- ptxas: 40 registers, 0 bytes stack frame, 0 spill stores, 0 spill loads.
- Source/PTX/cubin/report SHA-256:
  `7b0bbe18ce484c85d7278618706a185c200f26abdfc5f0c6aff549dce6ddae1e`,
  `22af89ff9e4e86ca4817b1b05818cc1329b9c1a70b8faa3e93b08e0c334ee2ee`,
  `7fcc74069bbeb90cd030b26ec1970d7fe9d35751826088492a8d86a8481318ef`,
  `3284751e4416eef371e642cb94627181493db1d8ee93f05c7d3c2e32a4ea69a5`.

The normalized source/PTX/cubin/report identity was identical in debug and release builds.

## Checks run at the head

- `cargo fmt --package uor-r4-discrete -- --check` - passed.
- `CARGO_TARGET_DIR=target/cuda-native cargo test --locked -p uor-r4-discrete native_vector
  --lib` - 3 passed.
- `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1
  CARGO_TARGET_DIR=target/cuda-native cargo test --locked -p uor-r4-discrete --features cuda
  --test native_vector_cuda` - 2 passed on the RTX 4090.
- The retained qualification example - passed and emitted the identity above.
- `cargo clippy --locked -p uor-r4-discrete --features cuda --test native_vector_cuda` - passed.
- `uor-harness check` - 346 rules, 0 errors and 312 existing warnings.
- `git diff --check` - passed.
- Full `cargo test --locked -p uor-r4-discrete --lib` - 356 passed, one unrelated existing
  failure and one ignored, as recorded above.

## Skills used

- `uor-research-architect`: kept the unit finite, artifact-bound and separate from a training
  campaign.
- `working-discipline`: measured the shipped kernel, retained the instrument and reported the
  unrelated suite failure.
- `solution-integrity`: found the Q31 width error before parity and added a planted scale fault.
- `native-space-cycle`: required one declared state, exact words, lossless serialization and no
  float master.

## Still open

- The existing integer serving session does not dispatch through this new module yet. The next
  unit must replace its private Hamilton oracle with this shared native transition or prove one
  source-level operator path across both.
- No trainer optimizer or full model step uses the kernel yet.
- No throughput claim is made from the development-build wall time.
- A training run remains out of scope until the operator is wired into the training and serving
  transition.

## Plain summary

The project now has a real integer CUDA training kernel for rotating a four-word geometric
vector in Q31 space. It uses no floating point, runs on the 4090, and returned exactly the same
16,384 words as the Rust reference. The next task is to make the existing served transport use
the same operator so the training carry and served carry are the same state rather than two
parallel implementations.
