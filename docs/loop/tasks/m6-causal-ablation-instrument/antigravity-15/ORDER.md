# ORDER: m6-causal-ablation-instrument, antigravity-15

Lab: antigravity. Item: [m6-causal-ablation-instrument](../../../../roadmap/items/m6-causal-ablation-instrument.md). Base: `d7d9d8d61`.

## Goal

The item's done-when reads:

> An ablation instrument registered before any run, able to knock out state transport and addressed memory and report the effect on the frozen panel. Registering the effect-size bar is part of this item (step 4).

## Cancel probe (run first)

Ran on `d7d9d8d61`:
- Checked `crates/uor-r4-integer/src/` (`ls crates/uor-r4-integer/src/`): `discrete_d11.rs` (landed in `#2964`) evaluates D11-exported discrete models (`Route`, `Flock`, `Relation`) and `memory_container.rs` (landed in `#2960`) evaluates `UORMEM01` / `UR4SESS1` exact-addressed memory containers, but no M6 intermediate-state causal ablation instrument (`m6_ablation.rs`) exists yet.
- Checked `docs/harness/panels/m6-reasoning.json`: `"frozen": false`, `"bars": []`, `"noise": "unstated - a bar must not be narrower than it"`.
- Checked `#2863`: open with 0 prior implementation comments.

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `docs/mechanisms/exact-memory.md`
- `docs/mechanisms/geometric-stack.md`
- `docs/model/architecture.md`
- `docs/labs/m6-causal-ablation-instrument/**`
- `docs/roadmap/items/m6-causal-ablation-instrument.md`
- `docs/loop/tasks/m6-causal-ablation-instrument/antigravity-15/**`

## Expected diff size

- `crates/uor-r4-integer/src/m6_ablation.rs`, `crates/uor-r4-integer/src/discrete_d11.rs`, `crates/uor-r4-integer/src/serving_root.rs`: ~450 lines of integer ablation hooks, 4-arm evaluator (`None`, `KnockoutStateTransport`, `KnockoutAddressedMemory`, `KnockoutBoth`), per-family McNemar contrast reporting (`Compositions`, `AlteredConstraints`, `CodeTasks`), and finite CPU control tests.
- `docs/labs/m6-causal-ablation-instrument/README.md`, `docs/mechanisms/exact-memory.md`, `docs/mechanisms/geometric-stack.md`, `docs/model/architecture.md`, `docs/roadmap/items/m6-causal-ablation-instrument.md`: ~160 lines of pre-registration specification, mechanism documentation, and item status update.
- Total expected diff: ~610 lines across 7 files.

## Done when

1. Design note posted on `#2863` and effect-size bar registration decision card posted on `#2028` (CHARTER §5.2).
2. `crates/uor-r4-integer/src/m6_ablation.rs` implements the M6 intermediate-state causal ablation instrument (`AblationArm::{None, KnockoutStateTransport, KnockoutAddressedMemory, KnockoutBoth}`, `M6TaskFamily::{Compositions, AlteredConstraints, CodeTasks}`, `M6AblationRow`, `M6FamilyReport`, `M6AblationReport`, `evaluate_m6_ablation_panel`) hooking into `IntegerDiscreteModel` (state transport at binder-route icosian actions and relation channels) and `IntegerMemoryContext` (exact-addressed `UORMEM01` / `UR4SESS1` reads), with zero softmax/squashing (D23), zero dense matmul (D25), and zero floating-point operations (D32).
3. Finite CPU control tests in `crates/uor-r4-integer/src/m6_ablation.rs` verify:
   - `AblationArm::None` is bit-identical to the unhooked evaluation path;
   - `AblationArm::KnockoutStateTransport` projects binder-route icosian actions to identity (`A_0`) and zeroes relation transport channels (`0`), provably flipping state-transport-dependent composition/constraint rows while preserving pure addressed-memory rows;
   - `AblationArm::KnockoutAddressedMemory` suppresses `UORMEM01` / `UR4SESS1` exact-addressed reads, provably flipping memory-dependent rows while preserving in-window state-transport rows;
   - `AblationArm::KnockoutBoth` knocks out both pathways and reports paired McNemar contrasts `(b, c)` and drop counts per task family.
4. Both the native 44-symbol (`audit_zero_matmul_serving.py`) and WebAssembly 7-symbol (`audit_zero_matmul_wasm.py`) zero-multiply / zero-float audits pass.
5. Pre-registration and documentation updated in `docs/labs/m6-causal-ablation-instrument/README.md`, `docs/mechanisms/exact-memory.md`, `docs/mechanisms/geometric-stack.md`, `docs/model/architecture.md`, and `docs/roadmap/items/m6-causal-ablation-instrument.md` (`status: done`), with `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` green.

## Stop and ask if

- Any file outside `m6-causal-ablation-instrument`'s `scope:` would need to change.
- Any GPU panel-scoring run is needed (GPU evaluation requires the frozen `m6-reasoning` panel from `#2866`, a served M4 trunk, a non-author `TEST FITNESS: FIT` review, and Casey's typed D40 release).
