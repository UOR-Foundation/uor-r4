# DECISIONS

Append-only. Each entry is an owner decision with its rationale and scope.
Agents may draft entries; only the owner ratifies them.

---

## D0 — What "no matmul at serving" means

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

**Decision: D0-a — the multiplier-free-kernel reading.**

The project's objective is to eliminate *wasted compute*. The serving path must
execute no multiplier and no floating-point arithmetic, and must not perform a
dense contraction that touches every parameter for every token. Offline training
is unrestricted.

### Permitted at serving

- integer add, subtract, shift, rotate, bitwise AND/OR/XOR/NOT, popcount, compare
- bounded index arithmetic and bounds checks
- exact fixed-point arithmetic in Q1.15 / Q1.30 (add and shift only)
- **table and array reads of learned values** — the value was learned offline; the
  serving operation is a selected read
- bit-serial / LUT accumulation of low-bit (ternary or ≤4-bit) weights, where the
  multiplier circuit is absent by construction (T-MAC-class kernels)
- exact selected reads, writes and copies over addressed memory

### Excluded at serving

- IEEE `f32` / `f64` arithmetic and `libm` transcendental functions on the hot path
- the integer `*`, `/`, `%` operators on the hot path (permitted only through
  documented shift-and-add idioms such as `square_u64` / `div_small_positive`)
- any GEMM or matrix-vector product that multiplies and accumulates over a dense
  weight matrix
- any tabulated implementation of a **dense** contraction: if serving touches all
  entries of a learned weight store per token, it is a matrix product regardless of
  the opcode used

### Unrestricted offline

Floating point, gradients, `uor-matmul`, Adam and any other optimizer. Training may
freely compute linear maps; the constraint applies only to the served computation.

### The distinguishing test

**Per-token parameter sparsity.** For each token, count the fraction of the learned
parameter store that is read. Sparse selected access ⇒ compliant. Dense access ⇒
non-compliant, whatever arithmetic is used. This test, not the opcode, decides.

### Consequences

1. The lane tables (`discrete_tables[l]`, 120×120 trained by Adam) are
   **compliant**: reading one 120-entry row is a sparse selected read.
2. Two current serving operations are **not** compliant and must be repaired or
   removed: the JEPA projection in `learner/jepa_trainer.rs::predict_jepa_step_q30`
   (literal `i64` multiplies) and the `f64` softmax sampler in
   `native_capability_api.rs` (reachable whenever `temperature > 0.001`).
3. A ternary linear map trained offline and served by LUT accumulation **is**
   compliant. This is the precedent-backed path to competitive quality.
4. The root `README.md` and `AGENTS.md` wording ("no mathematical matrix products,
   including … tabulated lookup/add contractions") is **narrower than this decision**
   and currently conflicts with the shipped implementation. Updating that wording is
   a stable-goal change and must go through owner-directed protected delivery; until
   it does, this file is the normative statement of intent and the conflict is known.

---

## I1–I5 — Serving efficiency invariants

Recorded with D0. These are measured, not asserted, and are reported together.

| ID | Invariant | Measurement |
|---|---|---|
| **I1** | Sparse per-token parameter access | fraction of the learned store read per token |
| **I2** | Low bytes/token | bytes streamed per token; bits per stored weight |
| **I3** | Bounded candidate scoring | candidates scored per token (target ≪ vocabulary) |
| **I4** | No multiplier in the kernel | static AST check + dynamic op census |
| **I5** | Exact addressed memory, no recomputation | provenance/version reads per token |

I3 and I5 are the properties a dense transformer cannot match at equal quality and
are the basis of any efficiency claim.

---

## D1 — Falsification before extension

Owner (human): Casey · Drafted by: Zed (agent) · Date: 2026-09-19 · **Signed: Casey 2026-09-19**

Method is Socratic and falsify-first. Before any new geometric mechanism is added,
every existing mechanism must show a **measured** contribution against a matched
control, or be retired from the critical path. A mechanism that cannot be shown to
change held-out bits-per-byte is not carried forward on grounds of elegance,
correctness of its mathematics, or architectural priority.

Corollary: the per-mechanism ablation table (Card P7) is a precondition for
Stages 3–6, not an optional diagnostic.
