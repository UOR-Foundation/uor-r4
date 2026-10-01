# Saved-model hard capture/hold bridge

This study continues the [variable-gap learned latch](geometric-attention-latch-2026-10-01.md) on #1512. It replaces the learned soft gate only, using the four saved capture-credit models. No fitting, new data draw, changed source identities, candidate mask, q/k map, radius normalization or metric is introduced.

## Mechanism and experiment

At each token, the learned raw gate logit determines CAPTURE if `z >= 0` (including zero), otherwise HOLD. The Held recurrence copies the complete current gained RMS representation on CAPTURE and retains the complete prior state on HOLD. The Local control copies on CAPTURE and clears on NO_CAPTURE. Both expose the previous identity to q/k before updating it. The previous-input register advances on every token. Values, current-role q/k contributions, age, NoRead and the full causal support remain unchanged.

The public APIs are per-call diagnostic overrides. Ordinary model forward and saved model format remain soft. Hard diagnostic logits, binding masses and actions propagate through the same hard read policy, including earlier read layers. Nonfinite gate logits are refused. This is a floating-point reference to a discrete mechanism; it is not a D11 served artifact or a straight-through training bridge.

The evaluator loads all four saved models from the sealed `run-3-capture` parent. Each receives exactly the original `evaluation.json` and `stress.json` bytes. It records full answer-position logits, predicted answers, exact correct-occurrence masses on both heads, gate logits and hard classifications. It compares ordinary soft outputs before and after each hard intervention, preserving the saved parents. Source labels observe read probabilities; event labels observe action correctness; neither enters model execution.

Each panel contains 128 rows in 32 correlated intervention groups. Existing longer gaps, changed queries, swapped values, reversed ordering and duplicate-value distinctions remain. Both panels are open development evidence. Context cap128, width32, observed original/stress maximum lengths50/76 are distinct quantities. Configuration `rra` puts the latch in the final read, so its pre-read trunk/logits are unchanged by this intervention; a future multi-read stack need not retain that equality.

## Results

| Model | Original soft / hard answers | Stress soft / hard answers | Hard head-zero source majorities, original / stress |
|---|---:|---:|---:|
| Held-s1 | 128 / 128 | 128 / 128 | 128 / 128 |
| Held-s2 | 128 / 128 | 128 / 128 | 128 / 128 |
| Local-s1 | 47 / 47 | 59 / 59 | 24 / 37 |
| Local-s2 | 54 / 54 | 49 / 49 | 29 / 36 |

Every cell is out of128. Hardening changes zero answer predictions in every model/panel and loses zero previously correct answers. Both Held models preserve every head-zero correct-source majority, all29 original and27 stress changing-answer query pairs, and the3/5 equal-answer query pairs with distinct source occurrences. Both are perfect in all disjoint write/query-gap stress cohorts; their group memberships overlap and must not be summed.

The hard Held source mass minima are0.990442/0.981927 for seed1 original/stress and0.997173/0.986671 for seed2. Maximum absolute head-zero mass changes are6.50e-6/1.12e-5 and3.87e-6/1.02e-5. The largest absolute change over all answer-position vocabulary logits in either Held model/panel is9.60231e-5. This is measured floating-point drift, not exact equality or a general tolerance guarantee. Head1 retains zero correct-source majorities in Held; no all-head routing claim follows.

All four models classify every unpadded capture event correctly:4,168/4,168 original positions and6,224/6,224 stress positions per model. Correct event classification alone is insufficient: hard Local controls remain at47/54 original and59/49 stress answers. Local-s1 gains one head-zero original source majority (23→24); its answers remain unchanged. Other source-majority totals remain unchanged.

All ordinary soft predictions and source masses reproduce the saved parent row by row (maximum accepted comparison tolerance1e-6 for source mass). Full soft answer-position logits and recorded diagnostics are unchanged before/after the hard call. The fixed rra actual hard-action getter equals raw-logit thresholding throughout. The evaluator runs125.779s internally (127.89s process wall), zero training updates,2Rayonthreads, maximum resident set232,390,656bytes. Debug/unoptimized execution is not optimized serving performance. The run is claimed, sealed and verified; all source/data/model/executable identities and row outputs are retained.


## Decision and limitations

**Decision: retain the hard diagnostic bridge and advance integer latch lowering.** No new fit is justified by these panels. The original work card defines the decision: preserve successful soft models regardless of the hard result. If hard actions preserve answer and occurrence decisions, retain the discrete capture/hold mechanism and advance faithful integer latch lowering. If they lose binding, inspect row-level action/state/score differences before a training change. Neither outcome retires geometric attention or qualifies language.

Integer continuation must align current and previous input exponents before the gate accumulator, keep the held mantissa with its capture-time exponent, and align current and retained q/k contributions before rounding. HOLD copies neither the current exponent nor a unit-normalized replacement into retained state. Hard action success does not establish low-bit gate/map fidelity, bounded candidate access, complete integer inference or energy savings.
