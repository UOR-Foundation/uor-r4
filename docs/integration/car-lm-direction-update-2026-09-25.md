# CAR-LM direction update — capability first, routing deferred

**Status:** branch-local decision of the autonomous research lab, 2026-09-25. Lives on
`codex/canonical-address-routing-20260925`; not proposed for promotion to `main`. It reorders
the CAR-LM plan's emphasis; it does not modify the plan's gates, D0-b, D5 or the energy goal.

**Basis:** the CAR-LM plan (`canonical-address-routing-plan-2026-09-25.md`), the executed G0a
cache-feasibility result (`canonical-address-cache-result-2026-09-25.md`,
`docs/evidence/canonical-address-cache-2026-09-25.json`), and council round 1 (CS path-to-chat,
experiment-designer chat instrument, red-team falsification, literature tiny-chat recipes,
history inventory).

## 1. What G0a established (the premise failed)

Evaluation-only, retained quaternion continuous checkpoint, evaluator-v2 comparison tail
(233,472 targets). Against the project's n-gram of record (evaluator-v2 order-5 count,
NLL 2.4056 on the same tail), **every** hand-built canonical-address cache arm fails quality
(best voronoi120 4.1866); the model's own readout is 2.0905 with 51.23% top-1 at the full dense
cost. Address stability is ≤7% (group2i120 5.0%, voronoi64 7.0%), so event-driven "compute on
change" amortisation yields nothing; the reported 100% hit rate is tune-bucket saturation, not
fresh predictability. The best cheap arm is an exact last-2-token cache (29.76% top-1, 4 ops),
and per-op it dominates every state-address arm. The predeclared G0 kill criterion is met.
Scope of the negative: one arm, one seed, exposed development text, the address-producing
recurrence not charged into the sparse numerator, matched ordinary arm not run.

**Reading:** at this scale and design the continuous state carries real but non-competitive
next-token information beyond local context. Caching a poorly-predicting state cannot create
quality.

## 2. What the council concluded

- **Capability before routing.** Routing reallocates compute; it creates no capacity. A routed
  model's ceiling is its teacher, and the only local teacher (#1017, 7.15M, raw continuation)
  was never shown to hold dialogue. Chat needs capacity, dialogue data and a response objective
  first.
- **The degeneracy is an objective/data problem until proved otherwise.** The current text is
  weak because training is raw continuation, not because the mechanism is exhausted. The cheapest
  test of this assumption needs no training.
- **Literature thresholds.** Single-turn instruction following is published at 28–33M params
  (TinyStories-Instruct; ≥2 layers, hidden ≥128); multi-turn coherence is unproven below ~100M
  (practical floor around SmolLM2-135M class). Retrieval (kNN-LM) improves memory-heavy recall
  but not open-ended generation. Delta-rule fast weights and early exit are the cheap local
  efficiency candidates; sparse upcycling and distillation are unproven below ~200M.
- **No chat artifact exists in-repo.** Closest are `r4-native-chat` over the prose `.rgm`
  (64-token context, no dialogue template), the authored dependent-language cases, and the
  integer continuation session. No accepted native model has run in Studio.

## 3. Decision (branch-local)

Adopt **capability-first, routing-later**:

1. The immediate objective is **chat-v0** — a frozen-panel-passing conversational artifact
   (single-turn answered prompts plus limited multi-turn fact recall).
2. Train with a **response-masked dialogue objective** on constructed dialogue (loss on
   assistant turns only), dense in float with matmul offline as D0-b permits.
3. **Quantize to the integer session** (D0-b) and re-evaluate there before any chat claim.
4. **Routing/CAR returns only at the export stage** as an optimization on a model that can
   already respond — measured in ops/token and J/token.

The energy thesis is retained as the end-state, not abandoned. Geometry keeps its roles:
exact addressed memory, canonical addressing, and bounded integer serving. The hand-built
continuous-state cache route is closed as a negative.

## 4. Reordered ladder to chat

| Rung | Deliverable | Acceptance | Kill |
|---|---|---|---|
| R0 | Evaluate retained artifacts (#1017, quaternion/ordinary) on a frozen response-masked panel against the count control | Does any arm beat the count reference on the same tokens? | No arm beats count → objective/data is the gap, not the mechanism |
| R1 | Response-masked dense dialogue model (5–15M first; escalate toward 30–60M within the M1 budget) on constructed dialogue | Held-out response NLL beats count by a predeclared margin; greedy output non-degenerate | No margin at budget → data or representation |
| R2 | Turn-boundary state + fact probe | ≥4/5 turn-3 fact recall | State never carries a prior-turn fact |
| R3 | Quantize into the integer session | Outputs within the predeclared drift bound; measured step cost | Quantization destroys the R1/R2 margins |
| R4 | chat-v0 panel (instrument below) | See instrument thresholds | — |
| R5 | CAR routing/caching on the capable model | ops/token and measured J/token at matched quality | Savings vanish under a dense residual |

## 5. Two cheap decisive tests (this cycle, evaluation-only)

- **T1 (R0):** run the retained artifacts and the count control on the frozen response-masked
  panel. This decides whether the next spend is a training run or a mechanism change.
- **T2 (G0-closer, red-team bar):** a **product address** (state cell × recent-token code) must
  beat the retained order-5 count (NLL 2.4056) at ≤10% of dense ops on a multi-turn slice, with
  the address-producing recurrence charged into the sparse numerator. If it cannot, state-
  conditioned addressing has no near-term future and routing stays a late export optimization.

## 6. Resources (prospective; record before execution)

Instrument authoring and T1/T2 are evaluation-only. R1 training is projected from the measured
graph (~3–4k targets/s at 256 context): 60–120M tokens per epoch, 7–13 h/epoch, peak RAM
≤12 GiB on the 16 GiB machine, ≤1.5 GiB retained checkpoints, 128 MiB storage stop margin,
wall boundary and checkpoint cadence fixed before launch, charged to the shared ledger. No
external compute. Runs share the machine with the D8 cycle; one cargo process at a time.

## 7. Owner decisions (2026-09-25)

1. **Target chat-v0 small first, then scale.** Build a 5–30M parameter response-masked
   dialogue model (the proven single-turn instruction regime, TinyStories-Instruct class) to a
   passing chat-v0, then scale toward a larger multi-turn model (chat-v1) as a separate rung.
2. **Permissive dataset fetching is allowed.** Dialogue data may be fetched from clearly
   permissive public sources; any source with an unverified or non-permissive licence must not
   have its content fetched — report it for an owner decision instead. Local construction from
   the existing TinyStories corpus remains available and preferred where sufficient.
