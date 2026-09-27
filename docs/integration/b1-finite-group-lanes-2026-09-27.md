# B1: finite-group tracking lanes — Stage A on the A5 word problem

2026-09-27 · Lab 1 (Claude) · [ROADMAP](../../ROADMAP.md) track T1(b) · References #820, #973

**Status.** This is the implementation gate pre-registered in ROADMAP §4.1(b), and a synthetic
word-problem result, **not a language result**. Stage B (inside the stack, on text) is in §6.

Labels:
- **Measured:** Rust runs on the owner's M1 at 2 threads.
- **Derived:** arithmetic from measured records.
- **Literature.**

## 0. Findings

1. **The gate passes.** Token-conditioned lanes trained only on words of length ≤32 track the A5 word problem exactly to length 4,096 (128×) once snapped to a group automaton. Non-commutative lanes succeed in 17 of 18 seed×rate runs; commutative and frozen lanes stay at chance in 18 of 18 (*Measured*, §3).
2. **The capability belongs to the finite group, not to quaternions.**
   - The ordinary control is two learned reflections per token (DeltaProduct-style). It learns A5 in 9 of 9 runs; quaternion lanes do so in 8 of 9.
   - Both minimise to the same 60-state automaton, which is A5 itself.
   - By the pre-registered kill rule, the **quaternion-specific serving claim is retired**: 2I lanes do not serve more cheaply than the ordinary control.
3. **Training lands on exact icosahedral geometry either way** (*Measured*/*Derived*, §4).
   - Quaternion lanes converge to 2I, the 600-cell: every transport's real part is within 0.0001–0.016 of an exact 2I value.
   - Reflection pairs converge to the icosahedral rotation group acting on R³ (plus a fixed axis). Their trace signature (1+φ)/4 = 0.6545 is that of a 72° icosahedral rotation.
4. **Snapping, not float precision, makes long lengths exact.**
   - A float reflection-pair model fell to 0.039 accuracy at length 4,096 (lr 0.03, seed 1).
   - Its snapped automaton stayed at 1.000.
   - The served automaton is one byte of state and two table reads per token: no multiplier, no float, no weight map.

What survives for the programme is the finite-group claim: learned non-abelian lanes compile to an exact
automaton of the icosahedral group. Diagonal (commutative) recurrences provably cannot represent this
state, and constant-depth log-precision transformers are conjectured unable to track it at arbitrary
length (TC⁰ ≠ NC¹; *Literature*, arXiv 2404.08819). Whether such lanes earn a place in a real language
model is Stage B's question.

## 1. Question and pre-registration

From ROADMAP §4.1(b), fixed before any run:
- **Stage A gate:** 2I lanes are exact at 4,096 when snapped in ≥2 of 3 seeds, with commutative lanes at chance.
- **B1 kill rule** ([review §9.3](first-principles-review-2026-09-25.md)): if 2I lanes fail to match the strongest non-diagonal control's tracking *at lower serving cost*, retire the geometric-state claim from the serving path.

## 2. Setup (*Measured* configuration)

**Task.**
- The A5 word problem over three generators of orders 2, 3 and 5 in A5: canonical 2I roots with real parts 0, ½ and φ/2, the first generating triple in canonical order.
- Target at every position: the A5 class (±-pair) of the running product. There are 60 classes, so chance is 0.0167.
- Labels come from the repository's exact 2I Cayley table (`uor_r4_core::…::group_table`), in `compose_context_roots` order.

**Lanes.**
- K = 8 per model, token-conditioned, with no additive input and no decay: `h_t = M[token] h_{t−1}`, starting from `h_{−1} = 1`.
- Free parameterisation: standard-normal raw parameters, not near-identity.
- Arms:
  - **quaternion:** `x ↦ q x`;
  - **reflection_pair:** `x ↦ H(v₂)H(v₁)x = p x r`;
  - **phase:** a unit complex number, which is commutative;
  - **frozen:** the identity.
- Read-out: MLP 32→128→60.

**Training.**
- AdamW, 4,000 updates, batch 64.
- Length curriculum from 4 to 32 over the first 60% of updates; cosine decay to 10%.
- Learning rates 0.003, 0.01 and 0.03; seeds 1–3.
- Every arm with the same seed sees identical words.

**Evaluation.** 256 fresh words at each length from 32 to 4,096, the same words for every arm. The score is accuracy at the final position.

**Serving** (`LaneAutomaton`):
- Close the lane's per-token 4×4 transports into a group by breadth-first search, merging products within Frobenius distance 0.5. Distinct 2I elements are ≥1.236 apart.
- Fit a read-out table by majority over 512 words of length 32.
- Serving is one Cayley-table read per token plus one read-out read.
- Moore partition refinement reports the minimal equivalent automaton.

**Source.** `crates/uor-r4-training/src/stack_tracking.rs` and `examples/tracking-lanes.rs` (`a5` mode). Five focused tests pass:
- the tensor Hamilton product;
- float states equal the served transport products for all four kinds;
- 60 paired A5 classes with a generating triple;
- exact generators close to 120 elements, minimise to 60 and serve exactly at length 1,000;
- the stack split equals the stack forward.

## 3. Results (*Measured*)

The table shows final-position accuracy at length 4,096 for the float model and for the snapped automaton. Runs are seeds 1, 2 and 3 at each rate.

| Arm | lr | Float @4096 | Snapped @4096 | Automaton order |
|---|---|---|---|---|
| quaternion | 0.003 | 0.012, **1.000, 1.000** | 0.023, **1.000, 1.000** | 1 (collapsed to identity), 120, 120 |
| quaternion | 0.01 | **1.000 ×3** | **1.000 ×3** | 120 ×3 |
| quaternion | 0.03 | **1.000 ×3** | **1.000 ×3** | 120 ×3 |
| reflection_pair | 0.003 | **1.000 ×3** | **1.000 ×3** | 60 ×3 |
| reflection_pair | 0.01 | **1.000 ×3** | **1.000 ×3** | 60 ×3 |
| reflection_pair | 0.03 | 0.039, **1.000**, 0.996 | **1.000 ×3** | 60 ×3 |
| phase | all | 0.004–0.023 | 0.016–0.027 | 8–16 (tolerance-quantised, meaningless) |
| frozen | all | 0.016 | — | — |

**Training time**, 4,000 updates at 2 threads:
- quaternion 14.6–17.8 s;
- reflection pair 67.2–71.0 s;
- phase about 15 s;
- frozen about 12.5 s.

The reflection pair is slower here because of its two scans and extra products, which is an implementation cost, not an intrinsic one.

**Minimal automata.**
- The exact-generator test shows that 2I read out by A5 class minimises to 60 states: `s` and `−s` share outputs.
- The grid records predate the minimisation field; the §5 rerun records it for every run.

## 4. Geometry the training found (*Measured*/*Derived*)

**Quaternion lanes.** `max_trace_deviation` is the largest distance from any closure element's real part to the nearest 2I value in {0, ±½, ±φ/2, ±1/(2φ), ±1}. It is:
- 0.0029–0.0158 at lr 0.003;
- 0.0001–0.0020 at lr 0.01;
- 0.0016–0.0049 at lr 0.03.

The learned group is 2I up to conjugation. This is forced: every exact A5 representation in SU(2) is conjugate to 2I.

**Reflection pairs.** Their transports are SO(4) rotations `x ↦ p x r`. The constant deviation of 0.1543–0.1545 is |(1+φ)/4 − φ/2|:
- (1+φ)/4 is trace/4 of a 72° rotation of R³ ⊕ R, namely (1 + 2cos 72° + 1)/4;
- the other classes give 0 (180°), ¼ (120°) and (2−φ)/4 (144°).

The ordinary control therefore learned the icosahedral rotation group of R³, with the fourth axis fixed.

## 5. Reading against the pre-registration

- **Gate:** PASS at every rate (2 of 3, 3 of 3, 3 of 3), with phase and frozen lanes at chance.
- **Kill rule:** the reflection-pair control matches tracking (9 of 9 against 8 of 9). Its automaton minimises to the same 60 states. **2I lanes have no serving-cost advantage, so the quaternion-specific claim is retired.**
- **Survives:** exact finite-group state, learned by any non-commutative lane and served as a table automaton. Quaternion lanes remain the cheaper parameterisation to train here, and the more robust in float at long length: all 8 learned quaternion runs score 1.000 in float at length 4,096, against 7 of 9 reflection-pair runs. That is an engineering preference, not a claim.

## 6. Stage B: inside the stack on text

Pre-registered in ROADMAP §4.1(b): LM gate within 0.05 nats of the lane-free stack at equal tokens over 3 seeds, and A5 accuracy inside the context ≥0.99 with lanes.

**Design** (`tracking-lanes mixed`, `TrackedStack`):
- The geometric stack is recurrence-primary: `rrar`, width 128, 4 heads, MLP 384, context 128, Dot reads, the stack's own rotation. Its vocabulary is 4,096 text tokens plus 3 A5 tokens.
- An optional lane side channel, zero-initialised like any new residual branch, is added to the embeddings. The lanes see every token.
- An auxiliary A5 read-out sits on the final states.
- Each update is one text batch plus one A5 batch, with A5 lengths ramping from 8 to 128.
- Arms: none, quaternion, reflection_pair and phase, 3 seeds each.
- Data: the #1017 reference's token store (`issue-1017/tokens/train.u16`, 120M tokens, ids < 4,096) and its `dev.u16`.

*Results pending; recorded in the next revision of this note.*

## 7. Resources

- **Build:** about 3 minutes on a cloned warm cache (`/Volumes/UOR-Workspace/BuildCaches/claude-b1-20260927`, an APFS copy-on-write clone). Incremental rebuilds take 30–40 s.
- **Tests:** 31 s.
- **Stage A grid:** 36 runs, about 12 minutes of wall time at 2 threads, peak RSS 1.08 GB.
- **Storage:** reports are 584 KB in `/Volumes/UOR-Workspace/uor-r4-lab/claude-b1-20260927`.
- No model slot was needed; these were light jobs under ROADMAP §6.
