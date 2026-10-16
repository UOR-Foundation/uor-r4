# Interlock & Ratchet Hardening (D23 / D25 / D32, Commission 1) — 2026-10-16

- **Lab / Session:** `antigravity` (`ag1`)
- **Commission / Authority:** `#2028` comments [`6222315060`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222315060), [`6222117146`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222117146), and [`6222114728`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6222114728) (consolidated Commission 1; prior hold in `#2029` comment [`6222232136`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6222232136) released by PM).
- **Mode / Scope:** `native_geometric_ai` · source-only / host-only · `NOT_TRAINING` · `model_compute_executed: false` · `apply_calls: 0` · zero pods · $0 GPU cost.
- **Line / Count / Headline:** M1 qualification & enforcement tooling · count `0/3` unchanged · M1 headline `37/232` unchanged (suspended under D28/R14). Training, M2, and Stage B remain HELD.

---

## 1. Objective & Defects Remediated

The repository audit (`#2028` comment `6222114728`, Ranks 7–11) identified five enforcement gaps across the D23, D25, and D32 interlocks:

1. **Rank 7 (`ratchet d23` scanned only `"softmax"`):** [`crates/uor-r4-harness/src/ratchet.rs`](../../../crates/uor-r4-harness/src/ratchet.rs) counted only case-insensitive occurrences of `D23_WORD` (`"softmax"`), leaving `softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, and `rotary` unguarded by `uor-harness ratchet d23`.
2. **Rank 8 (`ratchet d25` left tensor-core intrinsics `wmma` and `mma_sync` informational):** `TC_FRAGMENT` (`wmma`) and `TC_SYNC` (`mma_sync`) were counted in `tensor_core_sites` for an informational footer line (`informational: ... — not gated`) and excluded from `result.counts` and `result.detail`.
3. **Rank 9 (`SCAN_DIRS` omitted `docs/labs/*/instrument/`):** `const SCAN_DIRS: [&str; 1] = ["crates"]` skipped standalone instrument crates under `docs/labs/*/instrument/` (such as `docs/labs/k16-timing-2026-10-15/instrument/` and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/`).
4. **Rank 10 (`prlint.d23_interlock` ran only `Kind::NormExp`):** [`crates/uor-r4-harness/src/prlintrules.rs`](../../../crates/uor-r4-harness/src/prlintrules.rs) invoked `ratchet::scan` only for `Kind::NormExp` (`d23`) and never ran `Kind::Matmul` (`d25`) during `uor-harness lint check` or `uor-harness check`.
5. **Rank 11 (`r4-policy-check.rs` asserted stale pre-D25/D32 policy commitments):** [`crates/uor-r4-core/src/bin/r4-policy-check.rs`](../../../crates/uor-r4-core/src/bin/r4-policy-check.rs) asserted `/schema == "uor-r4.agent-execution-policy/5"` and `/architecture/training_matrix_multiplication_allowed == true`, failing against [`docs/integration/agent-execution-policy.json`](../../integration/agent-execution-policy.json) (which is at schema `/6` with `training_matrix_multiplication_allowed: false`).

---

## 2. Structural Changes

### 2.1 `crates/uor-r4-harness/src/ratchet.rs` and `crates/uor-r4-harness/src/main.rs`

- **Expanded D23 (`Kind::NormExp`) pattern families (`D23_IDENT_FAMILIES`):**
  - Preserves raw case-insensitive substring counting for `D23_WORD` (`concat!("soft", "max")`).
  - Adds `pub const D23_IDENT_FAMILIES: [&str; 8]` assembled via `concat!` (`softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary`).
  - `d23_sites(text)` strips comments (`//`, nested `/* */`) and plain string literals (`"..."`) while preserving raw strings (`r"..."`, `r#"..."#`) via `strip_comments_and_strings(text)`, lowercases the stripped code, and splits on non-ASCII-alphanumeric boundaries (`!c.is_ascii_alphanumeric()`).
  - Splitting on `!c.is_ascii_alphanumeric()` treats `_` as a segment boundary so both standalone identifiers (`tanh`, `silu`, `rope`, `ropei`, `rotary`) and compound snake_case identifiers (`apply_rope_f32`, `rope_theta`, `rope_interleaved`, `stack_sigmoid_q31`, `rank_softplus`, `gelu_erf`, `silu_fwd`, `rotary_reference`) are counted, while avoiding false positives on English words containing `r-o-p-e` (`proper`, `property`, `properties`, `is_proper_prefix`, `WriterProperties`) and avoiding double-counting `ropei` as `rope`.
- **Gated tensor-core intrinsics in D25 (`Kind::Matmul`):**
  - `matmul_sites` now scans stripped code lines for non-overlapping case-insensitive occurrences of `TC_FRAGMENT` (`wmma`) and `TC_SYNC` (`mma_sync`), pushing `Site { line, kind: "tensor-core", text }` into the gated site list (`result.counts` and `result.detail`) alongside `.matmul(`, `MatMul`, `cublas`, and whole-word `gemm`.
  - `result.tensor_core` and `result.tensor_files` are computed from the gated `"tensor-core"` sites, and `cmd_ratchet` in [`crates/uor-r4-harness/src/main.rs`](../../../crates/uor-r4-harness/src/main.rs) now prints `gated: N tensor-core intrinsic sites (fragment and sync intrinsics) across M files.`
- **Expanded `SCAN_DIRS` to cover `docs/labs/*/instrument/`:**
  - `const SCAN_DIRS: [&str; 2] = ["crates", "docs/labs/*/instrument"];`
  - `scan_roots(repo)` expands the `/*/` wildcard across non-symlink subdirectories of `docs/labs/`, scanning `.rs`, `.cu`, `.cuh`, and `.wgsl` files inside each `<lab-record>/instrument/` tree (skipping `target/`) while leaving dated prose/evidence outside `instrument/` untouched.

### 2.2 `crates/uor-r4-harness/src/prlintrules.rs`

- Added `pub const INTERLOCK_D25: &str = "uor-harness ratchet d25";`.
- Updated `interlock_holds` to run both `(Kind::NormExp, "D23", INTERLOCK)` and `(Kind::Matmul, "D25", INTERLOCK_D25)` in process and emit an `error` finding if either ratchet cannot run, lacks a readable baseline, or reports any file above baseline.

### 2.3 `crates/uor-r4-core/src/bin/r4-policy-check.rs`

- Updated `/schema` commitment to `"uor-r4.agent-execution-policy/6"`.
- Updated `/architecture/training_matrix_multiplication_allowed` commitment to `false`.
- Added explicit commitment checks for:
  - `/architecture/serving_matrix_products_allowed == false`
  - `/architecture/dense_matrix_multiplication_allowed == false`
  - `/architecture/model_floating_point_allowed == false`
  - `/architecture/floating_point_model_instruments_allowed == false`
  - `/architecture/floating_point_gradients_and_optimizer_allowed == false`
- Added unit tests `real_repo_policy_passes` and `stale_training_matmul_true_is_rejected`.

---

## 3. Baseline Audit & Active-Path Findings

### 3.1 Active-Path Findings (Highlighted for PM / Lab Disposition)

1. **FINDING 1 — `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs` (`0 -> 1` in `scripts/no-softmax-baseline.json`):**
   - **Location:** Line `1241`, inside `#[test] fn a_duplicate_cell_and_an_unknown_policy_are_refused()`.
   - **Site:** `value["cells"][0]["policy"] = json!("softmax_margin_v1");` (a negative test fixture verifying that `validate()` rejects an unknown/prohibited policy string).
   - **Disposition:** Because `D23_WORD` (`"softmax"`) is counted on raw source text (including strings and tests) and `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs` is in `#2550`'s owned diff (which this PR is prohibited from touching), the site is recorded at `1` in `scripts/no-softmax-baseline.json` and flagged here so the string literal can be split or renamed once `#2550` merges.
2. **FINDING 2 — `crates/uor-r4-training/src/lib.rs` (`4 -> 10`, `+6` in `scripts/no-softmax-baseline.json`):**
   - **Location:** Lines `328, 329, 346, 347, 351` (`rope_theta`, `rope_interleaved`) and `537, 608` (`apply_rope_f32`, `silu_fwd`).
   - **Classification:** Inactive / quarantined `#1017` dense reference scaffolding. Every one of the 6 newly counted sites (as well as the 4 pre-existing `softmax` sites) sits inside `#[cfg(any())] pub struct ReferenceConfig` (line `316`) and `#[cfg(any())] impl ReferenceModel` (line `361`), which are never compiled in any build profile.
3. **FINDING 3 — PTX interlock string literals in `uor-r4-discrete` and `admission-rank-fixture-runner` (`0` sites in `scripts/no-matmul-baseline.json`):**
   - **Location:** `crates/uor-r4-discrete/build.rs:48`, `crates/uor-r4-discrete/src/prime_zeta.rs:634`, and `docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/build.rs:45`.
   - **Classification:** Clean (`0` gated sites). Those lines contain `"wmma"` inside a plain string literal in `admit_ptx` (the PTX admission guard that rejects `wmma` instructions in compiled PTX). Because `Kind::Matmul` scans `strip_comments_and_strings(&text)`, plain string literals are stripped while raw strings (`r#"..."#` inline kernels) remain gated.

### 3.2 `scripts/no-matmul-baseline.json` Census (Unchanged: 26 files, 93 sites, 0 tensor-core sites)

Scanning `crates/` and `docs/labs/*/instrument/` with `wmma` and `mma_sync` gated leaves `scripts/no-matmul-baseline.json` unchanged at **26 files / 93 sites** (`0` hits in `docs/labs/*/instrument/`, `0` gated tensor-core sites repo-wide):

| Crate / Path Prefix | Files | Gated Sites | Justification |
|---|---:|---:|---|
| `crates/uor-r4-core/` (`eval/runner.rs`, `eval/validation_suite.rs`, `learner/head_only_train.rs`, `model/block.rs`, `model/head.rs`, `model/hybrid.rs`, `model/loader.rs`) | 7 | 42 | Historical pre-D25/D32 Candle/dense reference & comparison scaffolding retained as inactive evidence |
| `crates/uor-r4-Arc-AGI-3/src/` (`agent.rs`, `frame_encoder.rs`, `grid_ops.rs`) | 3 | 3 | Historical pre-D25 grid/frame helper method names |
| `crates/uor-r4-Compiler-for-Code/` (`examples/compile_and_verify.rs`, `src/compiler/pass3_Verify_and_Emit.rs`, `src/lib.rs`, `tests/integration_tests.rs`) | 4 | 9 | Historical compiler emitter/test fixtures referencing matmul opcodes |
| `crates/uor-r4-engine/src/` (`cuda_backend.rs`, `eval_harness.rs`, `forward.rs`, `metal_backend.rs`, `mlx_backend.rs`, `Verify.rs`) | 6 | 19 | Historical pre-D25/D32 engine backend/verification scaffolding |
| `crates/uor-r4-koszul/src/` (`koszul_complex.rs`, ` Yoneda_ext.rs`) | 2 | 4 | Historical homological algebra matrix product helpers |
| `crates/uor-r4-model-source/src/conformance.rs` | 1 | 3 | Historical model-source conformance comparator |
| `crates/uor-r4- tropical/src/tropical_ algebra.rs` | 1 | 2 | Tropical semiring min-plus/max-plus product (`matmul` method name) |
| `crates/uor-r4-Verify/src/` (`certificate.rs`, `invariants.rs`) | 2 | 11 | Historical verification certificate/invariant checks |
| **Total** | **26** | **93** | **0 new sites; 0 tensor-core sites** |

### 3.3 `scripts/no-softmax-baseline.json` Delta Census (`140 -> 153` files, `1,641 -> 2,228` sites, `+587` sites across `38` files)

Every file whose count changed under the expanded D23 scan (`softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary` + `docs/labs/*/instrument/`):

| File | Old | New | Delta | Newly Counted Tokens & Justification |
|---|---:|---:|---:|---|
| `crates/uor-r4-Compiler-for-Code/src/compiler/pass2_R4_Map.rs` | 1 | 3 | +2 | `gelu:1, silu:1` — historical compiler opcode mapping table |
| `crates/uor-r4-Compiler-for-Code/src/lib.rs` | 1 | 3 | +2 | `gelu:1, silu:1` — historical compiler IR enum variants |
| `crates/uor-r4-Compiler-for-Code/tests/integration_tests.rs` | 0 | 1 | +1 | `gelu:1` — historical compiler integration test fixture |
| `crates/uor-r4-core/src/eval/runner.rs` | 21 | 25 | +4 | `rope:2, tanh:2` — historical pre-D23/D32 evaluation runner |
| `crates/uor-r4-core/src/eval/validation_suite.rs` | 1 | 9 | +8 | `gelu:3, silu:2, sigmoid:1, tanh:2` — historical validation suite fixtures |
| `crates/uor-r4-core/src/learner/head_only_train.rs` | 20 | 22 | +2 | `tanh:2` — historical pre-D23 learner scaffold |
| `crates/uor-r4-core/src/model/block.rs` | 8 | 28 | +20 | `rope:12, silu:6, gelu:2` — historical reference block scaffold |
| `crates/uor-r4-core/src/model/hybrid.rs` | 3 | 9 | +6 | `sigmoid:4, tanh:2` — historical hybrid model scaffold |
| `crates/uor-r4-core/src/model/loader.rs` | 1 | 9 | +8 | `rope:6, silu:2` — historical weight loader scaffold |
| `crates/uor-r4-core/src/native_geometric/action_inference.rs` | 0 | 2 | +2 | `tanh:2` — retained pre-D32 native_geometric scaffold |
| `crates/uor-r4-core/src/native_geometric/analog_state.rs` | 1 | 6 | +5 | `tanh:5` — retained pre-D32 analog state scaffold |
| `crates/uor-r4-core/src/native_geometric/inference.rs` | 11 | 20 | +9 | `rope:3, sigmoid:2, tanh:4` — retained pre-D32 inference scaffold |
| `crates/uor-r4-core/src/native_geometric/learner/backward.rs` | 63 | 121 | +58 | `rope:18, sigmoid:13, silu:6, softplus:8, tanh:13` — retained pre-D32 autodiff backward scaffold |
| `crates/uor-r4-core/src/native_geometric/learner/forward.rs` | 50 | 76 | +26 | `rope:8, sigmoid:6, silu:3, softplus:4, tanh:5` — retained pre-D32 autodiff forward scaffold |
| `crates/uor-r4-core/src/native_geometric/learner/joint_step.rs` | 20 | 22 | +2 | `softplus:2` — retained pre-D32 joint step scaffold |
| `crates/uor-r4-core/src/native_geometric/learner/language_bridge.rs` | 48 | 74 | +26 | `sigmoid:16, softplus:4, tanh:6` — retained pre-D32 language bridge scaffold |
| `crates/uor-r4-core/src/native_geometric/learner/mod.rs` | 51 | 52 | +1 | `softplus:1` — retained pre-D32 learner module |
| `crates/uor-r4-core/src/native_geometric/learner/structured_backward.rs` | 64 | 90 | +26 | `rope:6, sigmoid:8, silu:4, softplus:2, tanh:6` — retained pre-D32 structured backward scaffold |
| `crates/uor-r4-core/src/native_geometric/model.rs` | 16 | 24 | +8 | `rope:4, sigmoid:2, tanh:2` — retained pre-D32 model struct |
| `crates/uor-r4-core/src/native_geometric/ops.rs` | 7 | 19 | +12 | `gelu:3, silu:3, sigmoid:3, tanh:3` — retained pre-D32 ops table |
| `crates/uor-r4-engine/src/cuda_backend.rs` | 3 | 12 | +9 | `rope:5, silu:4` — historical engine backend |
| `crates/uor-r4-engine/src/forward.rs` | 4 | 15 | +11 | `rope:7, silu:4` — historical engine forward |
| `crates/uor-r4-engine/src/metal_backend.rs` | 2 | 9 | +7 | `rope:4, silu:3` — historical Metal backend |
| `crates/uor-r4-engine/src/mlx_backend.rs` | 2 | 9 | +7 | `rope:4, silu:3` — historical MLX backend |
| `crates/uor-r4-engine/src/Verify.rs` | 2 | 6 | +4 | `rope:2, silu:2` — historical engine verification |
| `crates/uor-r4-graph-compiler/src/observation_text.rs` | 2 | 4 | +2 | `rope:2` (`rope_f32`) — historical observation text schema |
| `crates/uor-r4-integer/src/lib.rs` | 0 | 8 | +8 | `sigmoid:5, silu:1, tanh:2` — historical Q31 integer LUT helper names (`stack_sigmoid_q31`, etc.) |
| `crates/uor-r4-integer/tests/q31_conformance.rs` | 0 | 3 | +3 | `sigmoid:2, tanh:1` — historical Q31 conformance test |
| `crates/uor-r4-model-source/src/conformance.rs` | 2 | 20 | +18 | `rope:13, rotary:1, silu:4` — historical model-source conformance comparator |
| `crates/uor-r4-model-source/src/lib.rs` | 3 | 80 | +77 | `rope:77` (`rope_*` config/weight descriptors) — historical model-source loader |
| `crates/uor-r4-model-source/tests/adapter_conformance.rs` | 0 | 4 | +4 | `rope:4` — historical adapter conformance test |
| `crates/uor-r4-training/src/bin/export-integer-stack.rs` | 0 | 3 | +3 | `sigmoid:3` — historical integer stack export binary |
| `crates/uor-r4-training/src/bin/load-candidate-forward-report.rs` | 0 | 21 | +21 | `rope:17, silu:4` — historical candidate forward report binary |
| `crates/uor-r4-training/src/bin/train-language-stack.rs` | 0 | 23 | +23 | `rope:6, sigmoid:8, softplus:9` — historical float training binary |
| `crates/uor-r4-training/src/bin/verify-lut-fidelity.rs` | 0 | 8 | +8 | `sigmoid:4, silu:2, tanh:2` — historical LUT fidelity binary |
| `crates/uor-r4-training/src/geometric_stack.rs` | 53 | 117 | +64 | `rope:14, rotary:4, sigmoid:27, silu:2, softplus:11, tanh:6` — historical pre-D32 float geometric_stack |
| `crates/uor-r4-training/src/gpu_trainer.rs` | 48 | 132 | +84 | `rope:38, sigmoid:18, silu:14, softplus:10, tanh:4` — historical pre-D32 float gpu_trainer |
| `crates/uor-r4-training/src/lib.rs` | 4 | 10 | +6 | `rope:5, silu:1` — **FINDING 2**: all inside `#[cfg(any())] ReferenceConfig` / `ReferenceModel` |
| `crates/uor-r4-training/src/lut_export.rs` | 0 | 16 | +16 | `sigmoid:8, silu:4, tanh:4` — historical LUT export module |
| `crates/uor-r4-training/src/stack_export.rs` | 0 | 22 | +22 | `rope:8, sigmoid:10, silu:4` — historical stack export module |
| `crates/uor-r4-training/tests/lut_roundtrip.rs` | 0 | 7 | +7 | `sigmoid:4, silu:1, tanh:2` — historical LUT roundtrip test |
| `docs/labs/k16-timing-2026-10-15/instrument/src/main.rs` | 0 | 1 | +1 | `softmax:1` — **FINDING 1**: negative unit test string literal `"softmax_margin_v1"` at line 1241 |

The remaining 111 files in `scripts/no-softmax-baseline.json` retain their exact prior counts (`1,092` sites across `crates/uor-r4-core`, `crates/uor-r4-training`, `crates/uor-r4-eval`, `crates/uor-r4-Verify`, `crates/uor-r4-search`, `crates/uor-r4-tokenizer`, and `crates/uor-r4-ui`).

---

## 4. Verification Receipt

All checks executed on host in `/Users/casey.allard/uor-r4/.worktrees/antigravity-interlock-ratchet-hardening` (`model_compute_executed: false`, `apply_calls: 0`):

- `cargo test -p uor-r4-harness`: **543 unit tests + 13 integration tests + 1 doctest PASS (`0 failed`)**.
- `cargo test -p uor-r4-core --bin r4-policy-check`: **2 unit tests PASS (`0 failed`)**.
- `cargo run -p uor-r4-core --bin r4-policy-check`: `Native geometric architecture policy: PASS (no model behavior evaluated)`.
- `uor-harness ratchet d23`: `OK: no softmax added. 2228 remaining across 153 files (baseline 2228).`
- `uor-harness ratchet d25`: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93). gated: 0 tensor-core intrinsic sites (fragment and sync intrinsics) across 0 files.`
- `uor-harness check`: **PASS (`0 errors`)**.
- `python3 scripts/check_claim_wording.py`: **PASS**.
