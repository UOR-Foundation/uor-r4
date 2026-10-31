# uor-r4-discrete

The active M1 geometric learning crate: integer 8D carrier blocks, 64 signed
SpiralCore transport actions, a 16-root carrier orbit, ordered ternary FBS context,
Route/Flock/Relation reads and one shared input/carry/output vector table and writer.
Four blocks give width32. See the [architecture source map](../../docs/architecture.md)
[PLAN.md](../../PLAN.md) and rendered [STATE.md](../../STATE.md) for evidence and release scope.

## Read the source

| Source | Responsibility |
|---|---|
| [build.rs](build.rs), [metadata.rs](src/metadata.rs) | Generate and expose exact finite actions, carrier roots and transport tables. |
| [metadata.rs](src/metadata.rs), [discrete.cu](src/discrete.cu) | Ordered context plans; causal reads, shared writer and exact hinge objectives. |
| [device.rs](src/device.rs) | CUDA snapshot, proposals, apply and state boundaries. |
| [rows_source.rs](src/rows_source.rs), [rows_device.rs](src/rows_device.rs) | Opt-in row-parallel exact counterfactual proposals. |
| [training.rs](src/training.rs), [controller.rs](src/controller.rs), [attention_learning.rs](src/attention_learning.rs) | Pinned corpus/schedule/checkpoint lineage and scoped admission controllers. |
| [prime_zeta.rs](src/prime_zeta.rs) | Offline optional codebook construction: C⁴ per block, realification into R⁴ + R⁴, normalization/Q30, permutation and carrier allocation. |
| [main.rs](src/main.rs) | `discrete-stack` command dispatch; CUDA execution requires the `cuda` feature. |

## Build and validation boundaries

The repository pins Rust1.97.1. From its root, source/host checks are:

```sh
cargo check --locked -p uor-r4-discrete --lib
cargo test --locked -p uor-r4-discrete --lib
```

These are intended for metadata and construction tests; the default feature set
has no CUDA model execution. The `cuda` feature requires CUDA12.8-compatible build
and driver tooling. Device qualification, evaluation and learning are GPU-only,
under the admitted runner, exact sheet and the lab's pre-run review (D67). Commands
above are not permission to launch a dose. D41's donor pass is the named owner-at-dispatch exception.

The selected core dependency uses `native_support.rs`, not historical `lib.rs`;
integer selects `native_geometry.rs`. See their [manifests](../uor-r4-core/Cargo.toml)
and [integer manifest](../uor-r4-integer/Cargo.toml). Do not substitute old model CLI examples.

## Evidence to start with

- [D33 pilot](../../archive/labs/a2-pilot-2026-10-14/README.md): 1,640 batches, 236 accepted updates; mixed DEV measurements.
- [Original paired attention dose](../../docs/labs/attention-dose1-strategic-stop-2026-10-18/README.md): owner-stopped before Q1, partials preserved.
- [Rows parity attempt3](../../docs/labs/counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json): 1,085 finite cases, zero updates; proposal timing is non-gating (`qualified=false`).
- [M1 close](../../docs/labs/m1-close-saved-model/README.md): the saved 19-table model passes bars (a)–(e) at two seeds.
- [M1 bar (f)](../../docs/loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md): p-f 7,850 / 7,872 against T2 7,375, band 81, closing all six bars. On p-f, 62.5% of reference replies are verbatim TRAIN replies: reference-data overlap, not a generated-copy rate or new reply kinds. Apple M1 decode is 10,876 / 10,566 tok/s; useful conversation remains unqualified.

D37 permits offline float training without matmul/GEMM or softmax/squashing, while
this discrete implementation stays integer. D11 integer serving (the hardware integer multiply and divide are allowed, D80/D81; floats stay banned) has exact table-native exports for the M1 pair and a separate FBS trunk. The accepted M4 full-path qualification remains open.
