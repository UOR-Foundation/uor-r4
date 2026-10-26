# M1 Stage 3 Trunk Assembly and Training (`m1-trunk-assembly-and-training`, `#2844`)

- **Date**: 2026-10-25
- **Lab**: `antigravity` (`antigravity-89`, PR `#3258`)
- **Milestone**: M2 (`#2030`), Effective Milestone M1 (`#2029`)
- **Roadmap Item**: [`docs/roadmap/items/m1-trunk-assembly-and-training.md`](../../roadmap/items/m1-trunk-assembly-and-training.md) (`#2844`)
- **Pre-Registration Sheet**: [`docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md`](../../harness/prep/m1-trunk-assembly-and-training-2026-10-25.md)
- **Implementation**:
  - [`crates/uor-r4-discrete/src/trunk.rs`](../../../crates/uor-r4-discrete/src/trunk.rs)
  - [`crates/uor-r4-discrete/src/bin/trunk-run.rs`](../../../crates/uor-r4-discrete/src/bin/trunk-run.rs)

## 1. Overview

This lab implements `TrunkAssemblyV1` (`crates/uor-r4-discrete/src/trunk.rs`) and its CLI runner (`trunk-run`), connecting:

1. **Causal Read + Position + Normalization (GPU)**: `Relation { kept: 8, sweeps: 1 }` (`T16_rel`) and `FrozenZeroV1` (`T16_fz`) with `PositionHolonomyV1` (`#2843`) and `OrbitRootSnapNormV1` (`#2842`).
2. **Causal Quaternion/Clifford Prefix-Holonomy Recurrence (`DiscreteQuaternionRecurrenceV1`)**: 64-entry whole-number `PositionStepTable` (`blocks = 4`) transporting predecessor (`L - 2`), mid-window (`(L - 1) / 2`), and window-start (`0`) roots by exact multiscale relative holonomy ($F_{L-1,b} \circ F_{p,b}^{-1}$ and $F_{L-1,b}$) in $\mathrm{Cl}(0,6)$ and projecting onto $\mathcal{O}_{16}^4$ via scale-matched `OrbitRootSnapNormV1` (`multiplicity = 4`, `NormFrameTable`).
3. **Non-Linear Geometric Feed-Forward Gate (`DiscreteHamiltonGateV1`) + Unmodified Single-Vector `OuterL1V1` Writer**: Rotates the 32-int GPU read vector $q^{\text{read}}(x) \in \mathcal{O}_{16}^4$ by the exact transitive $\mathrm{Cl}(0,6)^4$ frame action `geometry::BFS[r_b * 16 + t_b]` into a single 32-int trunk output vector $q^{\text{trunk}}_L(x) \in \{-2, 0, 2\}^{32}$, verified on the canonical 16-root $E_8$ orbit and scored by the unmodified `OuterL1V1` writer on both `ft/dev` (`14,287` rows) and the sealed fresh evaluation panel `fresh_v2` (`14,287` rows).
