# Discrete SpiralCore/FBS context, shared vector writer and learning proposals

**Source implemented; eleven CUDA controls observed PASS, required admission pending.** Candidate source `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523` builds and all eleven exact construction controls returned PASS/exit0. Both attempts ran **OUT_OF_REQUIRED_REVIEW_ORDER**: the PM required Claude's prospective FIT, while the available FIT came from Codex. The observations are preserved, but do not qualify the milestone. Claude FIT and a newly admitted exact eleven-control repeat remain pending; no further numerical work is authorized before that review. Protected [PR #2379](https://github.com/UOR-Foundation/uor-r4/pull/2379) delivery is pending. Three local Rust metadata tests pass. No optimizer or geometric update has run. **Global training and separate M2 holds remain HELD.**

The owner requires a discrete, non-floating-point successor. This unit implements a standalone complete bounded integer path from a supplied context through FBS routing and actual SpiralCore transport to a shared vector writer. It does not activate the old floating-point trunk, implement Claude's murmuration/read, qualify full M2/D11, or establish useful conversation, language quality or energy savings. Previous floating-point source/results and unfinished fixes remain preserved by [PR #2370](https://github.com/UOR-Foundation/uor-r4/pull/2370), not promoted by this successor.

Coordination: [Kimi's ownership sequence](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154849555), [initial proposed law](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154897103), [Claude's interface review](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155013108) and [PM consolidation of both reviews](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155105001). The revised law below incorporates geometric admission, total move ordering, explicit absence eligibility and variable valid row lengths. These are source/review coordination, not training release or measurement. The milestone headline and negative count are not reset; PM/root owns merged-unit accounting.

## Implemented geometry and learning law

`crates/uor-r4-discrete` uses the existing exact **64 signed-axis SpiralCore actions and one 16-root E8 anchor orbit**. The compiler preserves its actual generator order, composition/inverse tables, FIFO BFS actions and ordered ternary FBS children/frontier. This is not full 240-root E8 coverage, full-orbit learnability or the complete marked external FBS object.

One integer codebook has shape `[vocabulary, blocks, 8]`. Each block is a root of that admitted orbit. The last token of each supplied causal window supplies a query; FBS leaves retain the entire window. Learned categorical actions frame each block/stage query. Coordinate L1 displacement orders route ports, coordinate Hamming resolves equal displacement, and retained port identity resolves a remaining tie. Actual BFS actions compose on the left and transport the selected earlier carrier. No averaging, continuous projection or floating-point surrogate occurs.

The same CUDA table encodes tokens, supplies carried/store vectors, ranks output tokens and scores targets. Each token's product cell derives from its **current first/last codebook root indices**; no token-number/hash distance defines admission. Every forward/counterfactual recomputes occupied cells from current or virtually changed geometry on CUDA. Empty cells are skipped; crowded cells retain every identity through an explicit bounded full-vocabulary scan (at most8192), with no occupancy truncation. Targets are scored even if omitted and never participate in admission. Smallest exact token identity resolves equal token distances; vector collisions preserve token/occurrence/record/version identities and need not invert uniquely through nearest-vector emission.

The declared integer loss numerator is `8 * token_hinge + width * address_hinge`, over common denominator `8 * width`. Token hinge uses margin `width` with one best admitted non-target token; address hinge uses margin `8` with one best admitted non-target cell. An absent comparison has zero hinge and an explicit token/address rival count; zero is not evidence of learning when eligibility is zero. Proposal eligibility is fixed by the baseline: skip rows without a baseline token rival, refuse a wholly ineligible batch, and make a candidate that removes a required rival noncompetitive. A move cannot improve by dropping its supervised comparison. Loss remains integer; this is a new law, not old multi-rival objective equivalence.

For one route action or codebook block, `propose` evaluates **identity plus 15 actual bivector moves** over the same baseline-eligible batch. Codebook counterfactuals change every repeated input occurrence and all candidate/target reads, including cells, together. Move order is minimum exact cost, identity first on ties, then lowest resulting port identity (frame index for a route, root index for a vector block), then fixed enumeration index. Proposals do not mutate state. The exact fixture's 640→544 virtual improvement was observed in the second attempt; it is a proposal witness, not a committed learning result or admitted milestone qualification. `apply` implements the left action/composition and stale/foreign refusal; it is **NEVER_RUN under the hold**. `Parameter::Gate` reserves kind3 and returns a typed unimplemented error before CUDA; no invented gate law is supplied.

## Shared carrier and Claude read interface

`Device::carry(&[Identity])` encodes arbitrary nonempty token inventories through the shared device table. `carry_windows(&[Vec<Identity>])` accepts1–64 caller-bounded prefixes, each length1..context, and binds explicit per-row lengths and ordered FBS plans. Physical padding copies the row's first token, while its plan contains only valid atoms; the query is the last valid token. Original logical identities remain separate from padding, and fixed full-length batches use the same row-plan ABI. Plain partial `carry` records remain store-capable but refuse model evaluation until assembled into windows. Carrier binding includes model/source/geometry and epoch; record versions are not epochs or document IDs. This is a vector bridge, not a trained intent compiler or full exact-store pipeline.

The read interface is an opaque carried integer batch plus exact identities and valid lengths/plans. It returns device vectors, emitted token IDs, integer losses, token/address rival counts and admitted cells (unused slots are ABSENT) from the same writer. Targets are declared next tokens and cannot change routed vectors or admission. Claude may design a discrete read against these carrier/epoch semantics; this unit does not implement murmuration. **BOS/document segmentation remains the caller's responsibility**; the API excludes its structural padding but does not infer document boundaries from record IDs.

Snapshots bind schema, spec, initializer seed, model epoch, codebook/routes, exact geometry kappas, generated PTX and complete device/kernel/metadata/compiler source. Reload admits shapes/bindings before allocation and validates every vector block and route state on CUDA. Reloads have fresh live-model identities, so foreign carried buffers/proposals cannot become valid by reloading identical bytes. Storage and finished-result readbacks do not implement a host model.

The runtime uses cudarc's driver-only loader and one ahead-of-time `discrete.ptx`. The core geometry compiler is a build dependency, excluded from the model runtime. There is no Candle/autodiff or CPU forward/score/transport fallback. Serving efficiency and a full executable integer-instruction census await the actual qualification receipts.

## Qualification scope and current evidence

Local metadata-only Rust tests: configuration/training-hold admission, all admitted contexts' ordered atom coverage, and three/nine-atom child closure — **3 PASS**. These do not execute a model.

Eleven exact CUDA controls in `src/qualification.rs` — **all observed PASS in attempt2; required admission pending**:

| Control | Intended construction question |
| --- | --- |
| `exact_writer` | Literal integer ranking/loss and one-leaf identity transport |
| `virtual_proposals` | Nonvacuous full-path proposal, identity tie and zero mutation |
| `context_transport` | Actual earlier FBS child reaches output via signed BFS transport |
| `isolated_windows` | Another supplied row/future window leaves earlier output unchanged |
| `collision_and_omitted_target` | Exact identities survive vector collisions; omitted target cannot alter admission |
| `nonvacuity` | Zero hinge has explicit zero eligibility; wholly ineligible learning proposal refuses |
| `artifact_and_identity` | Exact artifact output round-trip, foreign handles and non-geometric/old-schema refusals |
| `inventory_bounds` | Partial windows, token/parameter bounds and training hold |
| `variable_length_prefixes` | Unequal valid lengths retain their own final queries, with padding omitted |
| `target_independent_admission` | Target changes preserve routed vectors and admitted cells |
| `token_relabelling_preserves_geometry` | Relabelled exact identities preserve vector-derived cell behavior |

Attempt1 at `c9673359c708f9ed53ad55e1d42e14c49c2d2662` produced one PASS (`exact_writer`), then a CUDA out-of-resources hard stop in `virtual_proposals`; nine controls were NOT_RUN. The host launch used generic 1024-thread blocks. Candidate `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523` changes only the host launch to at most128 threads, preserving the kernel, controller, law, fixtures and bars. Its rebuild passed in0.73s; attempt2 observed all11 PASS/exit0. The first failure is a launch blocker, not a mechanism negative. No corpus optimizer, `apply`, CPU reference or training campaign belongs to these controls.

The available [Codex FIT receipt](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6155334135) and [amendment](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6155366841) did not satisfy the [PM's required Claude review](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6155342262). Root recorded the coordination error on [M4](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155397253), [M1](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6155397730) and [PR #2379](https://github.com/UOR-Foundation/uor-r4/pull/2379#issuecomment-6155398713). Preserve both reports unchanged. Formal milestone qualification awaits Claude FIT followed by a new admitted report with the same exact pins and eleven controls.

Attempt2 executable SHA256: `c9879778a1b0187db3d473c4041886cae14cd245f2544867e721527bd834548e`; generated PTX SHA256: `e50304bf73b7b81cdc876af5687ea65df3749a9dcd8c43197c794b8d6649c878`. Final compiler/driver identity, elapsed/memory and complete storage receipts remain pending root's collection.

Root reports owned Vast `55810308`, one RTX5090 at quoted `$0.892/h`, Ubuntu24.04.3. Actual compiler/driver identity, setup/control cost and invoice remain pending; the quote is not an invoice. Preserve source/restoration bundles, setup failures, exact binaries/PTX hashes, control logs and verified store round-trips before cleanup. Protected merge proof and pod-closeout absence proof are **PENDING**.

## Commands and next dependency

The source CLI exposes `schema`, `init SPEC.json NEW_ARTIFACT.json`, `infer ARTIFACT REQUEST`, `propose ARTIFACT REQUEST route|vector INDEX`, `list-controls`, and `qualify NEW_REPORT_DIR`; model commands require CUDA. Requests accept variable-length windows1..context and one valid next-token target per row. Inference reports rival counts/admitted cells and valid lengths so empty comparison cannot masquerade as a learned zero loss. `train` always returns TrainingHeld, `propose` reports `applied:false`, and reserved `gate INDEX` refuses before loading a model. No trainer/update CLI is enabled.

Next: obtain the required Claude FIT, then repeat the exact eleven controls in an admitted report before claiming milestone qualification; preserve all attempts and finish protected delivery. Root's source/read-interface review requests are on [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155255690) and [M1 #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6155255999). Kimi owns the successor training sheet and Claude owns the discrete read design. Neither finite FIT nor source delivery releases an owner hold.

## Preservation checkpoint

Before the assigned-review wait, both raw attempts, source patch, executable, PTX, generated exact geometry, setup/build/job logs and session receipts were stored with verified cloud-store round-trip at `icloud:UOR-R4/results/codex/discrete-preserved-20261013-2124.tar`: 12,246,528 bytes, MD5 `9945a68f2571af9f1b4e52c8b813127c`. This is a preservation checkpoint, not a corrected admission order or a claim that the pending merge is complete. The future admitted attempt, if authorized, receives its own new report and preservation receipt.
