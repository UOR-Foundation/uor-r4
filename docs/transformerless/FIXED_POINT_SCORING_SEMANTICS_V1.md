# Fixed-point scoring semantics v1 (normative)

Version: `score-semantics-v1`  
Status: active for compiler-emitted integer scoring paths.

## Definition

- Canonical score carrier: `ScoreQ(i32)` Q16.16.
- Runtime scoring inputs are compiler-emitted quantized residuals only.
- Runtime arithmetic is integer-only accumulation (`saturating_add`/`saturating_sub`) with deterministic table reads and integer comparisons.
- Runtime scoring performs no coefficient multiplication, division, softmax, floating normalization, or runtime rescaling.

## Objective

Freeze one reproducible semantic contract for score accumulation, ordering, sentinels, and overlap accounting so scalar and optimized kernels can be differentially tested bit-for-bit.

## Guarantee (Structural)

### G1 — Residual taxonomy

Typed contribution kinds are:

1. transition residuals
2. emission residuals
3. goal rewards
4. constraint penalties
5. uncertainty penalties

`crates/uor-r4-graph-runtime/src/scoring.rs` is the canonical no_std type surface for this taxonomy.

### G2 — Canonical accumulation order and saturation

- Contributions are sorted canonically by `(kind, evidence_id)`.
- Accumulation is a single left-to-right pass over canonical order.
- Arithmetic uses `ScoreQ` saturating operations at `i32::MIN/MAX`.
- Duplicate `evidence_id` entries are rejected to prevent double counting.

### G3 — Sentinel semantics

Ordered score domain:

`NoScore < SaturatedLow < Real(ScoreQ) < SaturatedHigh`.

This ordering is explicit in `OrderedScore` and is used by deterministic selection paths.

### G4 — Deterministic tie-breaking

Every top-1 selection point uses:

1. higher score first
2. on ties, lower `TokenId` first

This matches the deterministic top-k proof contract and is exposed by `select_best`.

### G5 — Overlap residualization (no double counting)

Each evidence contribution must be represented by exactly one canonical `evidence_id` in a candidate’s contribution list. Repeated IDs are invalid and rejected by the reference accumulator.

### G6 — Reference-vs-optimized equivalence target

`accumulate_reference` is normative for fixed-point accumulation semantics. Any optimized kernel must be equivalent to this order/saturation/tie-break contract.

## Empirical criterion

Quantization error is measured at compile time and recorded in score reports by residual kind. Runtime scoring never computes quantization error.
