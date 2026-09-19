# Card P7 — Per-mechanism falsification sweep — RESULT

Card: [`P7-falsification-sweep.md`](P7-falsification-sweep.md) · Date: 2026-09-19
Executed by: Zed (agent) · **Status: executed, unsealed diagnostic run (not a fresh draw)**

Raw receipt: [`docs/evidence/native_geometric_p7_ablation_2026-09-19.txt`](../../evidence/native_geometric_p7_ablation_2026-09-19.txt)

## Method

Each mechanism is ablated **in isolation** in an in-memory copy of the shipped artifact
and the held-out BPB is re-evaluated. No training, no fitting, no new draw. Numbers are
the **discrete scorer** (`ExportedGeometricModel::score_context_candidate_with_vsa`)
normalised over the full 4,096-token vocabulary with the serving sampler form
`exp((s − max) / (temperature · 8192))` at `temperature = 1.0`. Deltas are **paired** on
identical positions, with a 1,000-resample bootstrap 95% CI.

## Result

Artifact `native_geometric_prose_model.rgm` (2,691,950 B, vocab 4096, 4 lanes,
`flags = 0x000f`); 2,112-token held-out slice, 2,048 teacher-forced positions.
Baseline BPB **1.8055**.

| Ablation | ΔBPB | Paired 95% CI | Verdict | Artifact bytes |
|---|---:|---|---|---:|
| `jepa` | **+0.3743** | [+0.3519, +0.3973] | CONTRIBUTES | 64 |
| `s2_readout` | **+0.3377** | [+0.3204, +0.3565] | CONTRIBUTES | 40,960 |
| `engram` | **+0.2228** | [+0.1919, +0.2544] | CONTRIBUTES | 495,632 |
| `bias` | **+0.1482** | [+0.1366, +0.1604] | CONTRIBUTES | 16,384 |
| `lattice_fine` | **+0.0955** | [+0.0894, +0.1017] | CONTRIBUTES | 57,600 |
| `lattice` (both tiers) | **+0.0927** | [+0.0853, +0.1000] | CONTRIBUTES | 1,851,400 |
| `vsa` | −0.0007 | [−0.0010, −0.0005] | **INERT/HARMFUL** | 168,022 |
| `lattice_coarse` | −0.0066 | [−0.0101, −0.0033] | **HARMFUL** | 1,728,000 |
| `lanes` | −0.0071 | [−0.0082, −0.0059] | **HARMFUL** | 115,232 |

## Findings

1. **The VSA attention layer is inert.** Ablating it *improves* BPB by 0.0007
   (CI excludes 0). This confirms the source analysis: the four heads bind a **fixed
   random** token codebook (`vsa/codebook.rs`) that is disconnected from the learned
   120-root assignment (`jepa_trainer.rs:1119`), so the only recoverable signal is
   `a == b` and the mechanism duplicates the exact n-gram/induction paths. It is not a
   learned attention over the model's representation.

2. **The coarse lattice tier is net-negative and is 64 % of the artifact.**
   `coarse_trigram` is 1,728,000 B of 2,691,950 B (64.2 %) and its ablation improves BPB
   by 0.0066. Removing it would cut bytes/token by ~64 % while *improving* quality — a
   direct I2 (bytes-per-token) win. The fine cluster residual (+0.0955) carries the
   lattice's contribution.

3. **The learned lane tables are net-negative.** Zeroing four Adam-trained 120×120
   i16 maps improves BPB by 0.0071. They are 115,232 B of the artifact.

4. **Geometry-carried state prediction is the most valuable mechanism per byte.**
   The JEPA block is **62 bytes** (31 × i16, padded to 64) and its ablation costs more
   BPB than any other single mechanism (+0.3743). The S2 readout is next (+0.3377,
   40,960 B). The count mechanisms (engram, lattice) contribute, but at three to five
   orders of magnitude worse bytes-per-BPB.

5. **The metric/serving divergence is now quantified.** The training-time continuous
   figure is 1.2372 BPB; the discrete artifact that actually serves scores **1.8055 BPB**
   under full-vocabulary normalisation — a 0.57 BPB gap between the number that was
   reported and the model that ships. Any quality claim must state which scorer it used.

## Attention repair applied in the same change

`vsa/attention.rs` bound **independent** random roles into query and key
(`r_query != r_key`), so two occurrences of the same token gave
`d_H = d_H(r_query, r_key) ≈ 2048 = THRESHOLD` — not strictly below the threshold — and
received **zero** rectified weight, while mismatching pairs fluctuating below the
threshold received weight. The head attended to noise and skipped its own matches. The
old `test_head_1_induction_circuit` could not detect this (a single comparison against an
out-of-window candidate, decided by the codebook seed), and
`test_multi_head_roles_orthogonality` *enshrined* the broken configuration.

Fixed by sharing one `r_role` between query and key. Because XOR is self-inverse this
makes the binding algebraically a no-op (`d_H(x⊕R, x_j⊕R) = d_H(x, x_j)`), so identical
tokens now give `d_H = 0`. Replacement tests: a controlled multi-seed induction test with
a predecessor-order control and a no-repeat arm, a value-role orthogonality test, and a
regression guard asserting zero distance for identical tokens.

## Limitations

- 2,048 positions from a single already-open development slice; not a fresh draw.
- **Ablated terms are not orthogonal.** Zeroing the JEPA weights also changes the fiber
  the S2 readout consumes, so `jepa` and `s2_readout` overlap; deltas do not sum.
- Full-vocabulary normalisation, not the 64-candidate serving shortlist: an upper bound
  on served quality, free of shortlist-recall confounds.
- Debug build; timings are not a performance result.
- The run is not sealed into a report root, and the historical 9,984-case regression
  replay was not re-executed in this change.

## Consequences for the plan

- Retire the `vsa` score term and the coarse lattice tier from the critical path
  (Stage 1 gate), pending confirmation on a wider position count.
- Re-examine the lane tables as a net-negative term rather than a foundation.
- Reorder Stage 3: the learned-codebook work is justified by finding 1, but the
  **immediate** win is artifact shrinkage (finding 2) — remove 64 % of the bytes for a
  small quality gain before adding capacity.
