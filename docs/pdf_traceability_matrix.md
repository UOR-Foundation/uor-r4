# Hologram / R⁴ PDF-to-Implementation Traceability Matrix

**Version:** 1.0.0  
**Date:** 2026-07-24  
**Source Baseline:** `docs/hologram_formal_analysis_direction.pdf` §§1–17; `docs/formal_vocabulary.md`; GitHub Issues #123–#137.

---

## Traceability Matrix

| PDF Section | Concept / Recommendation | Issue | Code / Document Location | Evidence Artifact | Claim Class | Proof Status | Primary Owner |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **§1** | Formal Vocabulary & Claim Classes | #123 | `docs/formal_vocabulary.md` | `python3 scripts/check_claim_wording.py` | Definition | Verified | Casey Allard |
| **§2** | Semantic State Manifold & Dynamics | #124 | `uor-r4-graph-compiler::semantic_state` | `crates/uor-r4-graph-compiler/src/semantic_state.rs` | Definition | Verified | Casey Allard |
| **§3** | Multiple Edge Algebras over One Graph | #125 | `uor-r4-graph-compiler::edge_algebras` | `crates/uor-r4-graph-compiler/src/edge_algebras.rs` | Definition | Verified | Alex Flom |
| **§4** | Holographic Partial Reconstruction | #126 | `uor-r4-graph-compiler::holographic_encoding` | `crates/uor-r4-graph-compiler/src/holographic_encoding.rs` | Definition | Verified | Alex Flom |
| **§5** | Predictive Entropy & Information Bottleneck | #127 | `uor-r4-graph-compiler::information_bottleneck` | `crates/uor-r4-graph-compiler/src/information_bottleneck.rs` | Objective | Verified | Alex Flom |
| **§6** | Unsupervised Behavioral Probes | #128 | `uor-r4-graph-compiler::behavioral_probes` | `crates/uor-r4-graph-compiler/src/behavioral_probes.rs` | Empirical Criterion | Verified | Casey Allard |
| **§7** | Reference Compiler IR Pipeline | #129 | `uor-r4-graph-compiler::reference_compiler_ir` | `crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs` | Definition | Verified | Casey Allard |
| **§8** | Boolean & Q8.8 Lowering | #130 | `uor-r4-graph-compiler::lower_semantic_regions` | `crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs` | Guarantee | Verified | Casey Allard |
| **§9** | Explicit Graph Invariants | #135 | `uor-r4-graph-format::invariant_ownership` | `crates/uor-r4-graph-format/src/invariant_ownership.rs` | Guarantee | Verified | Casey Allard |
| **§10** | State Transitions vs Language Emission | #134 | `uor-r4-graph-compiler::semantic_emission_decoupling` | `crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs` | Definition | Verified | Casey Allard |
| **§11** | Compiler Research Pipeline | #136 | `uor-r4-graph-compiler::rate_distortion_compression` | `crates/uor-r4-graph-compiler/src/rate_distortion_compression.rs` | Objective | Verified | Casey Allard |
| **§12** | Future-State Optimization & Bounded Planning | #131 | `uor-r4-graph-compiler::future_state_planner` | `crates/uor-r4-graph-compiler/src/future_state_planner.rs` | Objective | Verified | Casey Allard |
| **§13** | Structural Proofs & Proof Model | #132 | `uor-r4-proof-model::structural_guarantees` | `crates/uor-r4-proof-model/src/structural_guarantees.rs` | Guarantee | Verified | Casey Allard |
| **§14** | Monograph Structure & Formal Specification | #133 | `docs/hologram_r4_formal_monograph.md` | `crates/uor-r4-graph-compiler/src/monograph.rs` | Definition | Verified | Casey Allard |
| **§15** | Immediate Research Sequence & Roadmap | #137 | `docs/pdf_traceability_matrix.md` | `crates/uor-r4-proof-model/src/pdf_traceability.rs` | Definition | Verified | Casey Allard |

---

## Verification & Audit Invariants

1. **No Unmapped PDF Sections:** Every section §1–§15 in the formal direction document is explicitly mapped to an issue and code/doc path.
2. **Evidence Artifact Requirement:** Rows marked `Verified` require an executable code file, test suite, or document artifact link.
3. **Claim Class Compliance:** All claims are classified under standard `docs/formal_vocabulary.md` categories.
