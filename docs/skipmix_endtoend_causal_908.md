# Skip-mix end-to-end R4Engine causal run (#908, follow-up to #897/#904/#906)

Empirical record. The S1-redesign injected-candidate skip-mix lane cleared its
then-described deployed **fidelity** spot-check in #906 (58/87,
`docs/skipmix_candidate_injection_906.md`). Fidelity is not promotion: per the
#822 §6-4 re-entry decision the frozen **20‰ end-to-end causal floor** is the
S1 promotion gate. This run measures the lane's causal effect in the exact
`R4Engine` harness against that floor; the #933 correction below classifies
that harness as reference/off-serving.

- **Status:** S1 (#822) child #908. Evidence for the maintainer S1 promotion
  verdict; this record does not itself promote the stage.
- **Historical execution-scope declaration (#908; corrected below):** offline
  fit + emit, plus the then-described **deployed** `R4Engine` admission and
  prediction path. **Teacher-free** — base vs skip-mix selected tokens, both
  scored against the corpus's recorded `t_argmax` labels. The #933 audit later
  classified that exact selector as reference/off-serving, not the normative
  production token selector. No live teacher pass.
- **Historical conformance declaration (#908; corrected below):** the run was
  recorded as binding existing deployed-path capabilities. The #933 audit
  supersedes that scope. No `model/ids.toml` row and no `CONFORMANCE.md` change
  landed under #908; RF-31 registration remained a separate,
  promotion-conditional step.
- **Claim class and status:** **Empirical Criterion. Status: Empirical.**

Harness: `crates/uor-r4-api/tests/skipmix_endtoend_causal_908.rs` (`--ignored`;
exercises the exact `R4Engine` path, now classified reference/off-serving).
Machine result:
`docs/skipmix_endtoend_causal_908_result.json` (result_cid
`blake3:e32e4e33…`).

## Hypothesis

**Empirical Criterion. Status: Empirical. Historical declared execution scope:
the #908 `R4Engine` harness; corrected to reference/off-serving below.**

The injected-candidate skip-mix lane (`predict_decision_candidates_with_skipmix`),
fit + emitted through the real release emitter and consumed by the deployed
`R4Engine`, improves deployed held-out top-1 by **≥ 20‰** over the deployed base
(no SKMX/PSIB sections), with the paired 95% lower bound clearing the floor. The
off-serving phase-0 figure (+56.2‰ vs a *toy suffix-rate bigram base*,
`docs/skipmix_confirm_897_result.json`) is expected to **overestimate** the
deployed delta, because the deployed base is far stronger than the toy base.

## Method (apples-to-apples; isolates exactly the lane's contribution)

One-time: fit the lane tables (`skipmix_fit::fit_skipmix_tables`) and recompile
the graph sections from the bundle's TRAIN split, exactly as #906's harness does.
Then emit **three** graphs from the SAME recompiled sections, differing only in
the two optional lane sections:

- **base** — empty SKMX/PSIB. `predict_decision_candidates_with_skipmix` is
  byte-identical to plain base (absent-section identity, asserted).
- **skip** — the real fitted SKMX/PSIB.
- **null** — SKMX/PSIB fitted on a corpus whose TRAIN targets are
  deterministically rotated by `n/2 + 1` (window↔target association broken,
  target multiset preserved) — the conditioning-specificity null.

Replay every held-out position through each engine and compare served tokens to
`corpus.t_argmax`. Statistics use the phase-0 convention (`ci95_permille`,
`paired_delta_permille`; normal-approx 1.96·σ).

## Run contract and cheap-instrument gate

The run contract is on #908 (issue body). The AGENTS.md binding cheap instrument
— a reachability probe on a 6,000-position subsample — ran FIRST and **passed**:
reachability ceiling 549.2‰ (≫ 20‰), subsample net delta +35.67‰ [25.76, 45.58]
(lower bound already clears the floor). Per the pre-declared gate the full run
was then launched.

## Results — attested bundle `smollm2-360m-broad-clean`

**Empirical Criterion. Status: Empirical. Execution scope: the exact #908
`R4Engine` harness, now classified as reference/off-serving.**

`corpus.meta` CID `blake3:aa9d1767…`; train / held-out 288,794 / 72,130.
skip_artifact_cid `blake3:19eb04d7…`; base_artifact_cid `blake3:aaf98b68…`.

| arm | top-1 | rate (permille) |
|---|---|---|
| base (no SKMX/PSIB) | 19,372 / 72,130 | 268.57 [265.34, 271.81] |
| **skip (injected lane)** | **21,424 / 72,130** | **297.02 [293.68, 300.35]** |
| null (label-shuffle) | 2,281 / 72,130 | 31.62 [30.35, 32.90] |

| paired vs base | delta (permille) | 95% CI |
|---|---|---|
| **skip − base** | **+28.45** | **[25.57, 31.32]** |
| null − base | −236.95 | [−240.33, −233.56] |

**Reachability decomposition.** The lane changes the base top-1 on 39,360 /
72,130 positions (545.68‰ ceiling): 6,651 base-wrong→skip-right (toward), 4,599
base-right→skip-wrong (away), 28,110 both-miss (neutral). Net = 6,651 − 4,599 =
2,052 = +28.45‰, matching the paired delta.

## Read

**Floor CLEARED; null COLLAPSES.** The deployed causal delta is **+28.45‰
[25.57, 31.32]** — the paired 95% lower bound (25.57‰) clears the frozen 20‰
end-to-end promotion floor. The conditioning-specificity null collapses to
−236.95‰ (decorrelated tables inject wrong candidates that displace correct base
predictions), so the improvement is **lane-specific**, not a generic
"injection helps" artifact. As predicted, the deployed +28.45‰ is roughly half
the off-serving +56.2‰ (toy base), confirming the toy baseline overestimated the
deployed delta — but the lowered lane still clears the floor against the real
deployed base.

**Recommendation: PROMOTE.** The S1 promotion verdict itself is a maintainer
decision (#822); this run supplies the evidence and the recommendation. On a
PROMOTE verdict, the promotion-conditional follow-on is RF-31 registration
(`model/ids.toml` → tagged Gherkin → failing marker/behavior test →
implementation → regenerated `CONFORMANCE.md`) and wiring the lane onto the
serving path. A negative (sub-floor) outcome would have been recorded as REVISE /
retain-dormant; it was not reached.

## Reproduction

`R4_SKIPMIX_PHASE=full R4_CAUSAL_BUNDLE=<broad-clean> cargo test -p uor-r4-api
--release --test skipmix_endtoend_causal_908 -- --ignored --nocapture`. The
base/skip/null top-1 counts (19,372 / 21,424 / 2,281), the changed/toward/away
decomposition (39,360 / 6,651 / 4,599), the floor-cleared verdict, and the
null-collapse verdict are pinned as regression assertions; the emit and the
deployed engine are deterministic (no RNG/clock/HashMap-order dependence), so the
counts reproduce byte-for-byte. Wall clock ~122 s (teacher-free), not the
~24 h teacher-forced `evaluate-report` path.

## Scope / conformance

The skip-mix lane remains **dormant** under this issue — no `model/ids.toml`
serving row, no `CONFORMANCE.md` change, no wire-format/compile-time change and
no recompile of any released bundle. This record measures what the lane *would*
do if turned on; turning it on (RF-31) is a separate, promotion-conditional step
gated on the maintainer S1 verdict.

## Append-only execution-scope correction — 2026-08-24 (#933)

**Empirical Criterion. Status: Empirical. Execution scope:
reference/off-serving.** The counts, paired interval, reachability
decomposition, null collapse, CIDs, and deterministic reproduction above remain
valid for the exact harness. The harness exercised
`R4Engine::predict_decision_candidates_with_skipmix`, which ADR-0001 scopes as
reference/certifier plus D4 policy—not as the normative production token
selector. Descriptions above that call this a “deployed” served-token result are
therefore superseded in scope, not erased: the exact result is **29.702% top-1,
+28.45‰ [25.57, 31.32], reference/off-serving**.

#910 subsequently routed `R4Engine::predict_decision` through this lane and
emitted SKMX/PSIB, but did not make `R4G1Runtime` consume those sections and did
not bind a normative full-census report to the release artifact. RF-31 is thus
NOT ESTABLISHED at normative deployed-serving scope. #933 must reproduce or
limit this effect through `R4G1Runtime` itself, with absent-section identity,
planted reachability, common sampled/greedy/beam candidates, typed report
binding, and the predeclared teacher-free decision gates. The frozen +20‰ floor
is unchanged.
