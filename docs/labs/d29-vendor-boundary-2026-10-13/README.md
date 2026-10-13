# D29 vendor and native dependency boundary repair — 13 October 2026

Status: the repaired **native-only library and ordinary train/evaluate CLI compile on the host**, with model execution disabled in that check. The successor CUDA build and all **31 mandatory finite controls are NOT_RUN**. The earlier pod source `b7a4f01bbbc49da54838edf678a815ff67786a60` reached **SETUP_ONLY** and is **BLOCKED** by prohibited compiled Rust dependencies; its setup binaries do not qualify the successor. Global training and separate M2 training/rental holds remain **HELD**. No model-quality headline, language result or fresh-transfer result changes.

The current repair is based on `fa9294a5afafea12ecb2a4bd779d1c9217ccf362`; the exact composed successor source/executable/PTX identities must be frozen before CUDA qualification. Claude's project contraction quarantine is integrated as [#2347](https://github.com/UOR-Foundation/uor-r4/pull/2347). The [prospective finite sheet](../../harness/prep/m1-d29-vendor-finite-2026-10-13.md) is **FIT** with A1–A4 and unchanged expectations: 15 library controls, 11 CLI controls and five module controls, totalling **31**, with exactly three project PTX modules plus nine vendor tensor PTX modules. FIT does not release an owner hold.

## Problem and scope

The project-module split alone left prohibited dependency contents. Candle CUDA constructors eagerly created cuBLAS handles; UNARY PTX contained squashing kernels, REDUCE contained softmax, and the kernel build emitted a tensor-core MOE archive. Core/float8 activated cudarc's compiler/BLAS features, while prebuilt driver loading was gated on NVRTC. Candle CPU/NN implementations added compiled contraction and squash bodies even with CUDA selected.

The first composed pod build then exposed additional Rust graph edges: `model-source → uor-matmul → uor-matmul-gemm`, optional-constructor code through `microlp → sprs → ndarray → matrixmultiply`, the retired LUT normalized-exponential runtime, and `uor-addr`'s Prism dependencies. These were **setup/compliance blockers**, not failed model controls or mechanism negatives. Runtime refusal alone could not remove their compiled numerical implementations.

Codex owns this dependency isolation, vendor repair and Rust pod tooling. Claude retains the project mechanism and historical geometric-stack source. Review found freely learned contractions in `selected_potential.cu` and `native_context.cu`; Claude quarantined both in #2347. Their source remains inactive evidence. No peer worktree is modified.

## Implementation

- Vendored cudarc separates the prebuilt PTX carrier/driver loader from runtime compilation. NVRTC, cuBLAS/cuBLASLt, cuDNN, cuTENSOR, cuSOLVER/cuSOLVERMg and cuSPARSE features fail closed. Invalid PTX metadata returns a driver error before binding.
- Vendored candle-kernels builds nine explicit lawful tensor modules with nvcc. Historical MOE, conv, quantized, removed unary/reduction bodies and unused RoPE stay inactive. Candle removes eager BLAS, GEMM/compiler implementations and CPU/NN numerical bodies, retaining host storage/serialization and lawful GPU tensor operations. Prohibited operations return recoverable errors. Float8 retains DeviceRepr without compiler/BLAS activation.
- Core and integer select native geometry/storage/tokenizer metadata facades. Source-ingest enums and availability reports move to a shared metadata crate; historical model-source reexports their canonical types. Model-source numerical runtimes are outside the native dependency graph.
- Training selects the native recurrence/FBS/shared-writer modules and excludes the preserved historical model, teacher, bank and exporter modules. The model-source and LUT dependency edges are removed. Historical reference and CPU/Metal features fail closed; automatic historical bin/example/test/bench discovery is disabled. Optional `microlp` is outside the admitted native feature set.
- `geometric_stack_native.rs` retains the recurrence-only FBS projection, geometric rate/Hamilton operators, native vector head, optimizer and fixed controls. The original `geometric_stack.rs` remains unchanged and inactive. `native_vector_channel.rs` extracts the exact existing shared channel/store types and helpers, reexported under the established namespace; it introduces no replacement writer or fabricated outputs.
- The registered `geometric-stack` example selects `examples/geometric-stack-native.rs`. Its ordinary train/evaluate path retains the pinned native arguments, one-stream seeded window draws, original warmup/cosine schedule, unmasked total training margin, CUDA clipping/update, checkpoint/resume, synchronized step timings and healthy-run estimate extension. Initial/dev/final evaluation uses the same masked TRAIN reference with a separate DEV oracle. Host sampling and tensor diagnostics refuse.
- New native checkpoint lineage schema `/2` binds settings, input content, TRAIN mask/reference policy/counts, tokenizer, DEV panel and initialization. Missing/stale reference bindings and incompatible historical lineages refuse; no DEV-as-TRAIN fallback is introduced. Independent review additionally found that matching state lineage could coexist with a swapped valid model config or optimizer settings. The native CLI now compares loaded model config and optimizer decay/clip to the requested settings before evaluation/update; the existing argument control includes metadata-only mismatch fixtures without adding or relaxing a finite control. Original CLI source remains inactive evidence.
- LUT is a format/grid metadata facade with its old engine, sampling and stack source preserved under `docs/history/d29-excluded-lut-2026-10-13/`. Vendored uor-addr preserves exact canonical JSON/NFC/BLAKE3 label behavior without Prism/model edges; it does not fabricate the excluded Prism witness.
- A canonical `native_starling_spec` metadata type is exposed through the established `starling_native` namespace. The model's typed S1 getter/setter and artifact metadata seam retain that canonical identity; attempted runtime/load activation returns a recoverable refusal pending Claude's actual murmuration operator integration. The admitted rung0 model remains recurrence-only. This repair does not qualify a read mechanism.

The per-crate provenance files and indexed restoration patch retain the original bytes. Exclusion and compile admission are implementation results, not evidence that the model has learned useful language or satisfies D11 serving.

## Validation and qualification

Observed source and metadata checks:

- Prebuilt PTX metadata: **4 passed**; host storage/refusal contracts: **4 passed**. No context/model arithmetic ran. The first vendor test command was blocked by historical dev-dependency registrations; their removal and subsequent passing named controls are recorded.
- Rust vendor checker compiled; planted forbidden-token and exact source-derived PTX export-inventory self-tests passed. The earlier nine vendor modules emitted at compute_120 passed that frozen export/instruction check. Successor module bytes must be verified separately.
- Candle core/NN host compilation: **PASS**, corrective incremental attempt 3.40 seconds. An earlier incorrectly named refusal binding was corrected; no CPU model arithmetic ran.
- Core/integer native facade check: **PASS**, final composed check 1.54 seconds. Address metadata controls: **3 passed**, compilation 0.91 seconds, execution 0.00 seconds. They test canonical bytes, streaming hash/framing and invalid JSON only, not a model or Prism runtime.
- Selected native training library plus registered CLI host check: **PASS, 9.11 seconds**. Retained test-module host typecheck: **PASS, 1.77 seconds** (`local/native-cut-test-check.log`). No test, forward, evaluation or optimizer executed in these commands. A duplicate independently started host compile completed in 13.98 seconds before its interrupt arrived; it is recorded as duplicate compilation, not another validation result.
- After the final metadata seam and reviewed resume admission correction, the selected retained test-module typecheck passes again in **2.40 seconds** (`local/native-cut-final-test-check.log`), without executing tests or a model. Native CLI SHA-256 at this corrected snapshot is `3c9f084c4c563d249a1cd904825c778466486c88a9d6c5bcab1bff1e191e74d7`; it is source evidence, not a qualified CUDA executable identity.
- Original pod setup at `b7a4f01bb`: nvcc12.8.93, compute_120, Rust1.97.1; twelve PTX modules and module-test/library-test/CLI-test/normal-CLI binaries built. A parser first counted ancillary Cargo binaries; the repaired parser selected the target and resumed the same cache. Five setup/argument/dependency metadata fixtures passed. The compiled Rust dependency findings then **blocked qualification**. No finite numerical CUDA control or model compute ran at this source.
- The successor's **31 mandatory controls remain NOT_RUN**. Their count, identities and expectations are unchanged. The existing writer artifact/control fixture now explicitly chooses geometric recurrence and Hamilton gate metadata so it does not request the excluded historical initializer. This is fixture admission repair, not a new model control or relaxed bar.

`local/native-cut-receipt.md` and the build logs preserve setup mistakes: a first core check without the required target directory was interrupted; missing `libm`/address plumbing and stale-lock checks were corrected; an address-preservation command used a wrong relative prefix; a nested vendor workspace needed exclusion; and the duplicate host compilation above completed before interruption. None is a model negative. No new Python execution was used.

The admitted provider is the Rust Vast adapter (six protocol/admission fixture tests passed); Rust setup/control helpers bypass the historical Python/HF bootstrap. Pod `55777904` was admitted at $0.848/h for one RTX5090 with `nvidia/cuda:12.8.1-devel-ubuntu24.04`, Ubuntu24.04.1 and nvcc12.8.93 observed. This record makes no new live-status or invoice claim. Root owns its process/resource and preservation receipts. No training, optimizer update or corpus evaluation is authorized by setup or finite FIT.

## Preservation and upstream identity

Original registry identities from Cargo.lock:

| Package | Version | SHA-256 |
|---|---|---|
| candle-kernels | 0.9.2 | b8455f84bd810047c7c41216683c1020c915a9f8a740b3b0eabdd4fb2fbaa660 |
| candle-nn | 0.9.2 | 3045fa9e7aef8567d209a27d56b692f60b96f4d0569f4c3011f8ca6715c65e03 |
| cudarc | 0.19.10 | 7359bf1de037ddada72729c3acec9b040f3468400c6e9e740b01ba0cd2e89562 |
| float8 | 0.6.1 | 719a903cc23e4a89e87962c2a80fdb45cdaad0983a89bd150bb57b4c8571a7d5 |

LUT originals are indexed by `docs/history/d29-excluded-lut-2026-10-13/SHA256SUMS` (SHA-256 `4716d47ff5dccd58d3ea3fed2187e091f5fd389521986efd87915a127f0f042c`). The exact uor-addr package from Git revision `165b51e3e2113ee5d032730cde709335d4fe9b60` is preserved under `third_party/uor-addr-0.2.0/historical/upstream/`, with `UPSTREAM-SHA256SUMS` (SHA-256 `678d9122df6768bea93992375d371f27ad5436282dff92c487b06d93a4bf9d36`). Licenses and original source remain evidence, never execution permission.

The [reverse patch](../../history/branch-archive/d29-native-dependency-originals-20261013.patch) for twelve changed original core/integer/training/model-source extraction paths is sealed and indexed in [branch archive](../../history/branch-archive/INDEX.md), SHA-256 `c7c35799e9e0ab43ab48a920a225dc2d5d4a574ad8c1c1005915eb7de0ed2f43`. It restores those paths to `fa9294a5afafea12ecb2a4bd779d1c9217ccf362`; new native files and root Cargo/lock are excluded. Applying the patch in an isolated Git index and comparing the resulting index to that parent both **PASS**, proving the named original tracked bytes without modifying the working tree or shared index. The original historical model/CLI bodies remain on disk unchanged. Earlier source/executable/PTX setup receipts do not qualify the repaired successor.

## Delivery and next dependency

Protected delivery and successor CUDA qualification are pending. Freeze the composed graph, lock, source and PTX inventory; obtain Claude's confirmation; then execute only the FIT-reviewed 31 controls through the admitted Rust runner. Preserve exact source/executable/module identities and publish the verdict to Claude and Kimi. S1 runtime integration remains Claude-owned. The owner alone releases training; full M2/shared intent and D11 serving remain unqualified.

Line: existing complete geometric learning/output integration; no headline change, count reset, model-quality verdict or fresh-transfer result.
