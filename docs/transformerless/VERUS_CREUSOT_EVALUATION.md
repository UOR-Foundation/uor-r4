# Verus & Creusot Evaluation for R⁴ Graph Compiler

As part of Phase 10 (Formal Verification), we evaluated extending our formal verification suite from Kani (which successfully covers bounded model checking, checked arithmetic, and fixed-capacity invariant testing) to heavier deductive verification tools like Verus or Creusot.

## Goal
The objective was to evaluate whether Verus or Creusot would be suitable for proving step-level properties of the prediction hot path and topological routing in `uor-r4-graph-runtime` and `uor-r4-core`.

## Findings

### Verus
- **Pros:** Excellent support for proving complex invariants and properties in Rust code using an integrated SMT solver (Z3).
- **Cons:** High annotation burden. The transformerless prediction loops heavily rely on tightly packed byte slices, array lookups, and masked-Hamming distance operations over borrowed `GraphView` payloads. Verifying these bounds in Verus requires pervasive ghost code and loop invariants which clutter the allocation-free logic in `engine.rs`.

### Creusot
- **Pros:** Translates Rust into WhyML, leveraging Why3 for proving properties, which handles Rust's ownership well.
- **Cons:** Still nascent and can struggle with the specific bitwise and pointer-arithmetic-like slicing operations used in decoding `PackedNode` and `PackedEdge` directly from `[u8]` buffers. The `uor-r4-graph-format` decoding relies extensively on little-endian reads that are difficult to model symbolically in Why3 without significant abstraction overhead.

## Conclusion
For the R⁴ system, **Kani** strikes the perfect balance. It automatically verifies our absence of panics, checked arithmetic (`ScoreQ::saturating_add`), and out-of-bounds array access in our `RuntimeState` updates without requiring invasive ghost code annotations.

We will stick to Kani for bounding invariants and the `ProofStatusMatrix` for tying empirical/differential tests to executable specifications. We will avoid adopting Verus or Creusot to maintain a readable, zero-overhead prediction hot path.
