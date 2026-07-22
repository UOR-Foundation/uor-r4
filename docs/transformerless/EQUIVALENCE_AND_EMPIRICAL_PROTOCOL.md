# Behavioral Equivalence and Empirical-Claims Protocol (Issue #33)

Status: normative Phase-0/3 specification for plan §9 item 23; feeds decision D2.

This document defines:

1. **Byte reproducibility** for certificate-bearing artifacts.
2. **Behavioral graph equivalence** (PDF §15) for non-canonical compile modes.
3. The **confidence-bounded statistical protocol** required for *all empirical claims*.

## 1. Reproducibility classes (D2)

### Class BR — byte reproducible

An artifact is **byte reproducible** iff two independent compiles with identical pinned inputs:

- source model CID + revision,
- tokenizer CID,
- corpus CIDs and split declaration (D3),
- compiler version and canonical deterministic mode,
- seed set and declared compile options,

produce byte-identical artifact and certificate payloads (equivalently, identical CIDs).

Class BR is mandatory for certificate-bearing artifacts used to satisfy Gate E.

### Class BE — behaviorally equivalent

Two artifacts are **behaviorally equivalent** when BR is not expected (for example, platform FP/SIMD
local-iteration compiles), but both satisfy the same declared behavioral contract on the same
evaluation distribution:

- identical runtime semantics and fallback policy class,
- identical declared metric set and acceptance thresholds,
- pairwise metric deltas within predeclared equivalence margins (PDF §15 framing),
- confidence bounds that stay within those margins per §2.

BE supports local iteration and cross-platform comparison, but does not replace BR for
certificate-bearing release artifacts.

## 2. Confidence-bounded empirical-claims protocol (Gate K)

Every empirical claim MUST ship a claim record containing:

- distribution identifier and corpus CIDs (D3),
- unit of analysis and sample size `n`,
- metric definition and direction (higher/lower is better),
- estimator, uncertainty method, and confidence level,
- acceptance threshold (or equivalence margin),
- random seeds, stopping rule, and slice definitions.

### 2.1 Required estimators and intervals

- **Rates/proportions** (agreement, recall, abstention): Wilson interval (or exact binomial for very
  small `n`).
- **Means** (bits/token, latency, bytes-read/token): bootstrap percentile interval (paired bootstrap
  when comparing systems on the same examples).
- **Comparative deltas** (graph vs baseline): paired delta CI on the same held-out examples.

Default confidence level is 95% (`alpha = 0.05`) unless a stricter level is declared in advance.

### 2.2 Claim acceptance rules

- Lower-bounded claim (`metric >= t`): accepted iff CI lower bound `>= t`.
- Upper-bounded claim (`metric <= t`): accepted iff CI upper bound `<= t`.
- Equivalence claim (`|delta| <= epsilon`): accepted iff CI for `delta` is fully contained in
  `[-epsilon, +epsilon]`.
- Non-inferiority claim (`delta >= -epsilon`): accepted iff CI lower bound `>= -epsilon`.

Point estimates without confidence bounds are descriptive only and MUST NOT be used as gate-passing
evidence.

### 2.3 Multiple slices and repeated looks

- If a gate depends on multiple metrics/slices, control family-wise error with Holm correction (or
  stricter predeclared rule).
- Repeated evaluation peeks require a predeclared stopping rule; otherwise, report as exploratory and
  non-gating.

## 3. Reporting contract

Certificates and reports MUST separate:

- **Structural claims** (by construction / by witness), and
- **Empirical claims** (by measurement under this protocol).

Every empirical table/statement must include `distribution`, `n`, `CI`, and acceptance decision.
Claims that fail protocol requirements are marked **unsupported**.
