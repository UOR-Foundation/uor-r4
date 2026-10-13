# D29 vendor boundary repair — 13 October 2026

Status: source repair checkpoint; composed CUDA build and finite qualification **NOT_RUN**.
Training remains **HELD**. This record does not declare rung0 ready or change a model-quality headline.

## Problem and scope

Claude's project-module/AOT split alone leaves prohibited dependency contents:
Candle CUDA constructors eagerly create a cuBLAS handle; lawful elementwise
operations load UNARY PTX containing squashing kernels; REDUCE includes softmax;
the kernel build also creates a tensor-core MOE archive. Core and float8 unify
cudarc's NVRTC/BLAS features. The prebuilt loader itself was gated on NVRTC.
Candle's CPU and NN implementations add a separate compiled contraction/squash
surface even when CUDA is selected.

Codex owns the vendor repair and root dependency wiring. Claude retains the
training crate and project kernels. Direct review also found the unrestricted
learned 4×4 contraction in Claude's provisional selected_potential module;
that module requires exclusion/refusal or a reviewed geometric replacement.
The native-context module also scores 120/33 classes against independently
learned four-coordinate coefficient banks; a fixed H4 query does not turn that
dense output projection into a constrained geometric operator. Both findings
were sent directly to Claude. No peer worktree or source is modified by this delivery.

## Implementation

- Vendored cudarc separates a prebuilt PTX carrier/driver loader from runtime
  compilation and fails compilation if NVRTC, cuBLAS/cuBLASLt or cuDNN features
  are enabled. Invalid PTX metadata returns a driver error before binding.
- Vendored candle-kernels builds an explicit list of nine lawful tensor CUDA
  modules with nvcc. Historical MOE, conv and quantized sources, removed unary/
  reduction functions and the old build script remain inactive evidence.
- Candle's CUDA backend removes BLAS initialization and contraction/compiler
  implementations. Unsupported operations return recoverable errors. Legal
  tensor entry points retain their formulas and names.
- Float8 retains CUDA DeviceRepr support without enabling compiler/BLAS features.
- CPU/NN computational implementations are excluded, retaining storage and
  serialization. Prohibited backend features fail closed before activating
  vendor dependencies. No complete vendor claim precedes the composed build.

The per-crate UOR-PATCH.md files describe the boundaries. This is a dependency
and implementation repair, not a newly measured learning mechanism.

## Validation and qualification

Observed in this source preparation:

- Prebuilt PTX metadata tests: **4 passed**, no device/context/model arithmetic.
- Host storage and refusal contracts: **4 passed** (two core storage/callback
  controls, one quantized output-preservation control, one NN admission control).
  They use fixed host operands and require refusal before model arithmetic;
  they are not CPU numerical references. The initial test command was rejected
  because historical dev-dependencies remained registered on an excluded vendor
  package; unused historical target/dev-dependency registrations were removed,
  with original source preserved, and the named library controls then executed.
- Standalone Rust vendor checker: compiled; planted forbidden-token self-test
  passed. Whole-tree check correctly refuses the old training manifest's direct
  NVRTC feature until Claude's AOT source is integrated.
- Targeted offline lock update and resolved CUDA feature inspection completed;
  no vendor NVRTC/BLAS edge remains in the reviewed provisional graph. Final
  feature graph is re-recorded after all vendor and training changes.
- Focused host compilation of candle-core and candle-nn passes (3.40 seconds
  for the corrective incremental attempt, two jobs). The first attempt found
  one incorrectly named binding in a refusal stub; it was corrected. This
  builds source only and performs no CPU model arithmetic. CUDA remains unbuilt.
- CUDA compilation, generated PTX census, executable import checks and all
  26 finite device controls: **NOT_RUN**.

[Prospective sheet](../../harness/prep/m1-d29-vendor-finite-2026-10-13.md)
reuses the existing stored controls with unchanged expectations. FIT and exact
combined source/executable pins are pending. No CPU model/reference, optimizer
update, corpus evaluation, training, pod or rental was executed by this repair.
The latest Rust-only tooling direction also requires a compliant admitted pod
path; the stock provider/setup tooling still calls Python. Disk was observed at
29 GiB, below the 30 GiB local floor. Neither tooling nor resource blockers are
model negatives.

## Preservation and upstream identity

Upstream registry archive SHA256 values from the original Cargo.lock:

| Package | Version | SHA256 |
|---|---|---|
| candle-kernels | 0.9.2 | b8455f84bd810047c7c41216683c1020c915a9f8a740b3b0eabdd4fb2fbaa660 |
| candle-nn | 0.9.2 | 3045fa9e7aef8567d209a27d56b692f60b96f4d0569f4c3011f8ca6715c65e03 |
| cudarc | 0.19.10 | 7359bf1de037ddada72729c3acec9b040f3468400c6e9e740b01ba0cd2e89562 |
| float8 | 0.6.1 | 719a903cc23e4a89e87962c2a80fdb45cdaad0983a89bd150bb57b4c8571a7d5 |

Licenses and upstream provenance are retained. Removed implementations are
inactive historical source; they grant no execution permission. All earlier
model evidence remains at its original source/configuration scope.

## Delivery and next dependency

Protected delivery is pending. After Claude's corrected module split and this
vendor repair integrate, freeze source/PTX/executables and execute only the
FIT-reviewed fixed CUDA controls through the admitted runner. Publish the exact
verdict directly to Claude and Kimi; the owner alone releases training.

Line: existing complete geometric learning/output integration; no headline
change, no count reset, no language or fresh-transfer result.
