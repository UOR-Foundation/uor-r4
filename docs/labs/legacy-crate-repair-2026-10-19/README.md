# Legacy crate repair — 19 October 2026 (claude)

**Owner direction (Casey, 19 Oct 2026, charter question Q8).** The packages broken since
[D36](../../integration/DECISIONS.md#d36--github-pages-keep-the-last-good-deploy-live-deploy-job-skipped-until-a-d11-served-wasm-engine-lands)
item 3 are to be **repaired on the current core, not archived**. Their mechanisms
(the R4G1 compiled lookup graph and the September native-geometric engine) stay
**PARKED** under D22: this unit restores buildability and does not change their status,
any model behaviour or any published result. References #2028.

## What was broken and why

#2345 narrowed `uor-r4-core`'s library to `src/native_support.rs`. That surface no
longer compiles `transformerless::{compiler, runtime}` or the pre-#2345
`native_geometric` model API. `uor-r4-graph-compiler` imported both, so it failed with
11 errors (13 with tests). Every package that depends on it failed at the same point:
`graph-certify`, `api`, `proof-model`, `graph-cli`, the root package, and the test
targets of `graph-runtime` and `model-source`.

The retired code cannot simply be compiled again. Four of its paths are banned in every
compiled module (D23/D25/D29/D30):

| Path | Why it stays out |
|---|---|
| `uor-r4-model-source` (the transformer teacher) | its library compiles normalized-exponential attention and dense matrix products, and links `uor-matmul-gemm`; `scripts/native_dependency_boundary.rs` lists both as D29-excluded packages |
| `transformerless::compiler` top-3/top-8 samplers | normalized-exponential sampling of teacher logits |
| `transformerless::runtime` dot assignment (`simd::dot_argmax`) | scores all 256 classes as Σ_d work[d]·W[class][d] with the product done by shift and sign: a dense matrix-vector contraction behind a table wrapper (D30) |
| `src/r4_source_relation_head.rs` | a dense 32×288 affine layer, ReLU and output layer (a dense MLP head) |

D36 item 2 already keeps `transformerless::{compiler, runtime}` retired. Restoring an
R4G1 engine without these paths would be a port (see *Not done*), not a repair.

## Method

The repair applies the #2345 pattern to each package. A new `src/native_support.rs`
becomes the library path; it compiles only the modules that build on the current core
with none of the paths above. **Every historical source file is byte-identical**; this
unit adds six `native_support.rs` files and edits manifests and docs only. Modules are
kept or excluded whole: no function inside a historical module is gated or edited. No
Cargo feature re-enables a historical surface.

Manifest changes:
- the library path points at `src/native_support.rs`;
- `uor-r4-model-source`, `uor-r4-graph-cli` (from `api` and the root) and the
  `uor-matmul-core` dev reference are removed;
- the BLAS features `observation-blas-exception` and `local-inference-accelerate` are
  removed with the links they enabled;
- targets that drive the historical surface are not built (`autotests`/`autobins`/
  `autobenches` off, with the compliant targets listed explicitly);
- the six `[profile.test.package.*]` overrides for the removed packages are dropped.

`Cargo.lock` loses `uor-r4-model-source`, the five `uor-matmul*` packages and `bytemuck`.
None of them remains in the workspace graph.

`graph-cli`'s two kept modules name their error type as
`uor_r4_model_source::SourceUnavailable`, which that crate re-exports unchanged from
`uor-r4-source-interface`. The new surface binds the interface crate under that name
(`extern crate uor_r4_source_interface as uor_r4_model_source;`). They get the identical
type without the teacher link.

## Result per package

| Package | Kept (compiled) | Lines kept / total | Not compiled (preserved) |
|---|---|---|---|
| `uor-r4-graph-compiler` | 22 of 33 modules: planning, semantic state/transitions, stage DAG, executor/jobs/memory budget, pack, residual, reproducibility, monograph, rate-distortion, reference IR, behavioral probes, observation shards, and the three feature-gated modules | 6,090 / 34,424 | `lib.rs` (teacher `compile`/`observe`), `observation`, `observation_text`, `induction`, `recorded_corpus`, `route_fit`, `routing`, `segment_fit`, `skipmix_fit`, `quantum_cover`, `trace_profile`, `probability_calibration` |
| `uor-r4-graph-certify` | 12 of 26: certificate, compiler scaling, corpus-signed transport attention, frame consistency, long context, octeract, prime-route worker canary, route attention, and four feature-gated modules | 4,717 / 25,799 | `score`, `score_runtime`, `fmm`, `certify`, `compare`, `msa_*`, `octeract_trace_screen`, `performance_certificate`, `predictive_sufficiency`, `fairness_provenance`, `route_cost`, `route_fit_report`, `target_operator_certificate` |
| `uor-r4-api` | none of its 12 modules; the surface re-exports the current core's `native_geometric` | 0 / 17,920 | every module: each depends on the R4G1 `engine`/`compile` path or the pre-#2345 model API (`native_geometric::{Model, durable_memory, vsa}`); the `r4-native-chat` and `r4-learned-reference-1102` bins; both examples |
| `uor-r4-proof-model` | 6 of 9: allocation, inference audit, Kani proofs, PDF traceability, proof matrix, range bounds | 936 / 1,970 | `deterministic_topk_proof`, `theorem7_proof` (core's retired `score_q`, `transitions`); `structural_guarantees` (its #161 obligation needs certify's `performance_certificate`, which uses the retired runtime's `OpKernel`) |
| `uor-r4-graph-cli` | 2 of 6: `prime_route_canary`, `recommend_scale` | 1,178 / 20,184 | `lib.rs` (the R4G1 command dispatcher), `cover_sweep`, `convert_r4g1`, `runtime_corpus`, `scenarios` |
| root `uor-r4-wasm-router` | facade re-exports of the current core and router, `r4_source_span_pointer` (a weighted cosine, no weight matrix), `telemetry` | 486 / 75,885 | `lib.rs`, the `r4` binary (`main.rs`, `server.rs`, `chat.rs`, `model.rs`, `r4g1.rs`, `tless_uor.rs`, release bundling), the `r4_*` teacher-trace and reference-generation experiments, the relation head, the native-geometric CLI/service, the wasm bindings |
| `uor-r4-graph-runtime` | library unchanged; 3 of 4 tests, the `vp_tree` bench | — | `r4g1_runtime_test`, the `session_signature` bench |

[`checks/compiled-sources.txt`](checks/compiled-sources.txt) lists every source file the
seven packages compile, read from cargo's dep-info after a clean rebuild. Three entries
(`src/chat.rs`, `src/main.rs`, `src/server.rs`) are read as text by a static
provider-scan test (`include_str!`), not compiled. A banned-token scan of the compiled
Rust finds hits only in comments and in `inference_audit.rs`'s forbidden-opcode lists,
which name instructions to reject. The new files name no banned operation.

## Checks (at the PR head, on the owner's laptop; no model compute)

| Check | Result |
|---|---|
| `cargo check --keep-going -p <pkg> --all-targets`, per workspace member (not `--workspace`: core raises a deliberate `compile_error!` under feature unification) | 24 of 28 members pass, including all seven touched here. `uor-r4-core` 947, `uor-r4-integer` 167 and `uor-r4-router` 40 `error[` lines are pre-existing historical targets; none of them depends on a changed package. `uor-r4-workbench`'s 1 error is core's `compile_error!` reached through its features, as before. |
| `cargo check --all-targets --all-features` for `graph-compiler`, `graph-certify` | pass |
| `cargo test -p <pkg>` | `graph-compiler` 91/91 (95/95 with all features); `graph-certify` 62 pass, 1 fails, 3 ignored (76 pass with all features); `proof-model` 10/10; `graph-cli` 17/17; `graph-runtime` 30 pass, 1 ignored; root 6/6; `api` builds no tests |
| `uor-harness ratchet d23` | 2,253 across 154 files, equal to baseline |
| `uor-harness ratchet d25` | 93 across 26 files, equal to baseline |
| `cargo run -q --locked -p uor-r4-claim-wording -- .` | pass |
| `cargo fmt --check` | pass |

**The one failing test is an instrument effect of the docs cutover, not of this repair.**
`compositional_planning_certification_846::committed_result_binds_the_audit_inputs_and_negative_verdict`
compares CIDs of six source files with the CIDs in the archived
`archive/compositional_planning_certification_846_result.json`. The cutover (#2789)
rewrote one `docs/…` path string to `archive/…` in each of those six files. With their
pre-cutover bytes restored in the working tree, all three tests in that file pass. With
the current bytes, the CIDs differ. The archived result must not be edited, so the test
is left unchanged and reported here (D31: an instrument defect, not a negative). Other
tests that hash source files the cutover rewrote may drift the same way.

## Not done, for the owner

- **R4G1 serving and compilation stay unavailable.** The `r4` CLI, the R4G1 `R4Engine`,
  the compile/observe/score pipeline and the wasm bindings all need a banned path. A
  compliant R4G1 engine would need a port: split `compiler.rs`'s artifact types from its
  teacher, move the `TeacherOracle`/`RepresentationSource` traits out of
  `model-source`, and replace or gate the dense dot-argmax with the Hamming
  (xor + popcount) assignment the runtime also has. That changes the qualified core
  boundary and model behaviour, so it needs its own owner decision and pre-registration.
- **The release pipeline is paused, not restored.** Neither release asset can be built
  under the bans, so `.github/workflows/release.yml` skips all three jobs (`if: ${{ false }}`,
  reason and re-enable condition in-file), as D36 did for `deploy.yml`. Rewriting it for
  the UORLUT01 artifact is the separate M8 task.
- **Proof obligation #161** (`structural_guarantees`) stays out because `OpKernel`, a
  plain operation-count struct, lives in the retired `runtime.rs`. Moving that struct
  would touch the qualified core surface.

## Follow-ups found on the way

- `uor-r4-graph-format` depends on the git `uor-addr` with default features, which
  brings in `uor-prism-tensor` and `uor-prism-numerics`. `scripts/native_dependency_boundary.rs`
  lists both as D29-excluded. Core avoids them by using the vendored `uor-addr` with
  default features off. This is pre-existing and reaches every graph package.
- The D23/D25 ratchets scan `crates/` and lab instruments only, so the root `src/`
  (including `server.rs`) has never been ratchet-tracked.
- The manual QA jobs in `.github/workflows/ci.yml` still name the removed `r4` binary and
  the `bdd` test, and `--workspace --all-features` reaches core's `compile_error!`. They
  were already failing before this unit.
- `uor-r4-core`, `uor-r4-integer`, `uor-r4-router` and `uor-r4-workbench` still fail
  `--all-targets` on historical bins and tests.
