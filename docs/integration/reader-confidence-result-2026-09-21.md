# Read-confidence influence interface: the parent rule is preserved, the class is feasible, text is not improved

September 21, 2026. Executed from reviewed parent `e0d6296c` (PR #1332). Prospective
[design](reader-confidence-design-2026-09-21.md); [principal review](policy-obstruction-review-2026-09-21.md);
[constructive prompt](deepseek-reader-confidence-step-2026-09-21.md). Delivered root
`.uor-models/realtext-prior-2026-09-20/reader-confidence-3` (sealed, verified, 0 unlisted, manifest
`d81bda9e3dbff98a69ba5020631527698fbad83f3359611c1a15c5f3d6b0255f`); superseded `reader-confidence-1`
and `-2` are retained and never reused as the delivered root.

## Decision

The parent-preserving interface is **verified**, the confidence-extended influence class is
**feasible and optimal** on development (the coarse 32-address class is not), and it preserves useful
answers and absence while cutting most of the parent's text harm. It does **not** obtain a useful
(negative) text gain, so **no positive influence claim is made**. The interface and its witness are
retained as a reusable component; the bounded integration ends here.

## Witness: exact parity with the retained scored reader

`D = max_strength strength_score(candidate, relation, strength, parent_bucket) - noread_score(parent_bucket)`
in widened `i64`; `parent_bucket` is the selector's own `bucket_of`. The rule
`empty pool -> NoRead; otherwise the ungated top source at eight nats iff D > 0` reproduces the
frozen parent's own served action on **2,969 development positions with 0 mismatches** for both the
H4 (`relational_ctx`) and categorical scored parents. This is the cheap implementation prerequisite,
not a model-quality claim.

## The confidence address restores feasibility

Address `= utility_bucket * 2 + 1[D > 0]`, at most 64 opcodes, same four actions. Fit populations:
197 final-present, 23 final-absent, 1,952 text tokens (944 candidate-bearing).

| Arm | Solve | Supported | Text b/t | Present correct | Present Δ bits | Absent reads | Absent Δ bits |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| parent reference | — | — | +0.4532* | 149 | −1931.32 | 7 | +3.497 |
| **H4 confidence (selected)** | `OPTIMAL` exhaustive | 28/64 | **+0.0228** | 148 | −1926.64 | 7 | +3.497 |
| **categorical confidence (selected)** | `OPTIMAL` exhaustive | 27/64 | **+0.0036** | 158 | −2032.89 | 5 | +1.416 |
| witnessed parent rule (H4) | — | — | +0.5724 | 149 | — | 7 | — |

\*parent text value is the previously measured held-out figure; the others are the development fit
text stream. The learned tables satisfy every behavioral constraint **and** the +0.05 bits/token text
screen, while the raw witnessed parent rule would fail the screen (+0.5724). The 64-address search
finishes exhaustively (about 1.7–2.6M nodes), unlike the coarse class, which was proven infeasible.

## Fresh evaluation (one declared draw)

Fresh construction seed `0x5C0F_C0DE` (140 sequences, 117 final-present / 23 final-absent) and the
**four remaining eligible `Dev` reader documents** (976 positions, disclosed as a small honest final
population; the other 32 are development history).

| Arm | Fresh present emitted | Fresh absent reads | Fresh text Δ bits/token |
| --- | ---: | ---: | ---: |
| local / NoRead | 0/117 | 0/23 | 0.0000 |
| `relational_ctx_parent` | 73/117 | 10/23 | +0.4346 |
| old coarse `h4_policy` | 57/117 | 23/23 | +0.0885 |
| `categorical_policy` | 80/117 | 23/23 | +0.1251 |
| fixed one-nat | 0/117 | 23/23 | −0.0115 |
| **`h4_confidence`** | **71/117** | **11/23** | **+0.0379** |
| **`categorical_confidence`** | **79/117** | **7/23** | **+0.0201** |

Required present preservation is parent − 2 (margin translated to 117): H4 confidence keeps 71 versus
the parent's 73, so **present answers are preserved**; its absent reads rise 10 → 11, which **fails the
no-increase absence criterion by one read**. The categorical confidence arm preserves both (79 present,
7 absent reads) and keeps text inside the screen. Both confidence arms lift text from the parent's
+0.4346 to within +0.05, but neither reaches a *useful* negative gain; the constant one-nat reader is
the only negative text arm and it emits **zero** correct present answers. Copying an answer is not
reasoning and this is not a prose result.

## Diagnosis

The sign-of-D bit resolves the **present/absent** collision inside a bucket, which is exactly why the
coarse class was infeasible and this class is feasible. It does not separate **natural-text** positions
from **present-query** positions, which still share the five-bit bucket; those addresses must read at
eight nats to satisfy the present constraint, and that shared dose is the residual text harm. The
remaining text harm is therefore a stream-mixing effect at the address, plus the low copy reliability
of the text stream itself, not an absent/present confusion and not a geometry-capacity limit.

## Interface repairs delivered in this run

- **Source-hash completeness**: the receipt now hashes `relational.rs`, `policy_feasibility.rs` and
  `competitive-reader.rs`, written after the final source edit.
- **Actual revision recorded**: `base_revision` and `running_source.git_rev` are both `e0d6296c`
  (dirty snapshot recorded) rather than a hard-coded stale revision.
- **Verified predictor used**: the one-nat comparator is now the predictor returned by the
  expected-manifest loader, not a separately loaded equal selector.
- **Expected-manifest loader**: 0 failures across `h4_policy`, `categorical_policy`, `h4_one_nat` and
  the two confidence artifacts; four rejections exercised (drifted thresholds, wrong byte hash, wrong
  fit-input identity, truncated bytes).
- **Neighbour accounting**: the deliberately edited occurrence is excluded from neighbour-change
  counts (140 → 20 changed), with neighbours added (0), removed (16), lost admission (0), still-admitted
  (140/140) and NoRead (0) reported separately; the disabled-condition invariant holds 140/140.
- **Identity**: the counterfactual score-space identity holds to `1.42e-13` bits over 5,442 positions.

## Retained and next

Retained: the verified parent-preserving confidence interface, the witnessed parent rule as fallback,
the two compact tables, and all superseded sealed roots. The evidenced successor is to separate the
**text stream from the present-query stream at the address** — a prospectively declared query-type /
role observation, or an explicit per-position reliability signal — rather than adding persistent state
or dimensions. Structural persistence, dependent composition, broader language/executed Rust and
qualified scale remain the ordered responsibilities in [project-track](project-track.md).

Development selection plus one declared fresh evaluation; not generalization. Physical energy is
UNAVAILABLE and whole-path D0-b is not claimed. Resources and charges are in the
[resource ledger](resource-ledger-2026-09-19.md).
