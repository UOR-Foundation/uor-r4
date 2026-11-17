# Go-forward plan: one model end to end, 16 Nov 2026

Owner: Casey. Lab: codex. Items: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md),
[end-to-end-integration](../../roadmap/items/end-to-end-integration.md).

This is the durable plan for the remaining work. It exists so that a later session does not have to
rediscover the state, the evidence or the owner's three decisions. Numbers are measurements with
their source; anything marked **hypothesis** is untested. Nothing here overrides CHARTER §3.

## 1. Where the work stands (measurement)

- **Mechanism question, answered.** The C2′ choice ("return the *other* fact") is not expressible by
  the fixed gate family: 0.125/0.152 at the family's ceiling (codex-18), 0.12 train under the pick
  target (claude-395), 0.006/0.000 held under the hinge trainer (codex-22). The relation rule is
  exact on the identifiable rows in the served artifact (47/47, 50/50) but its record is written in
  the probe corpus's token ids, so it is a probe artifact, not a transferable mechanism.
- **The asked-fact read works once trained on the served stream.** 224/512 → 440/512 held (ext-s1),
  329/512 → 455/512 (ext-s2) with the read retrained on the FBS artifact's own stream (codex-21).
  The read's payload enters at depth 6 and its written row is **2.6% of the stream's norm** at
  16,384 steps (claude-387: |p|/|x| = 0.026, cosine 0.07–0.08); a 97%-correct choice scores at most
  19 of 1,024 at that scale (claude-386).
- **The trunk's read input is the residual stream**, whose integer signs agree with the float replay
  0.887 of the time at e_x = 16 against 24 (codex-20). The read retrained on that stream recovers
  0.86/0.89, so the loss is the input's separation, not the read family.
- **The served address has 670× less margin than the float replay.** The same read, the same 1,024
  scored rows: mean margin **479,134,037** on the float dump against **714,606** on the served FBS
  stream (codex-26). No row ties, so the picks agree — what collapses is the entropy, exactly the
  literature's leading indicator. With claude-390's residual at 2^18.5 against gates at 2^31, this
  is the input-separation defect measured directly.
- **The copy read carries recall when it is allowed to.** Arm r (copy read latest-exact, turn memory
  leaving copied keys alone, a memory line): exact matched pairs **0.749 / 0.759 / 0.782** at three
  seeds (claude-345). Arm rt (three-key turn memory): E 0.71, script general 10/10, but **55 panel
  replies wrong, 33 of them after a copied value or a changed number** — "47" served as "40".
- **The 2 Nov anchor failure was an FBS artifact, not a discrete one.** The item's baseline ran the
  `dual-c131` artifacts, which are `uor-r4.lut-stack/5` FBS transports with **no read record**. The
  discrete 19-table model has **never been scored on a memory panel**, because no in-tree binary
  serves text from a `.d11`. Correcting this changes decision 10 (below). The discrete M1 model did
  close all six M1 bars (DEV 7,002/7,001, Fresh V2 7,215/7,228, p-f 7,850/7,872 at two seeds;
  10,876/10,566 tok/s on an M1; artifact 86.5 MB).

## 2. The plan (13 items)

Each item is a unit with its own ORDER/REPORT and its own gate. Sizes are honest estimates.

### To make the run's output exist as a served artifact

1. **The `/17` contract.** `conformance/stack/*.toml` for the run's composition, schema
   `uor-r4.lut-stack/17`, with the `q31-read` section and mechanism row; validated by
   `cargo xtask stack-contract-hash`. Needs the run's pins. **Size: 1 unit.**
2. **The export pipeline on a real checkpoint.** `fbs-snapshot` → `OUT.q31read` → `fbs-export
   --stack-contract … --q31-read …` at the run's shape, then read/serve parity. Exercised on
   `ext-s1` only. **Size: 1 unit (GPU).**
3. **Compression.** The FBS artifact is 19,345,344 B without pair rows and 116,050,880 B with them;
   the discrete M1 artifact is 86,536,132 B. The product's served baseline used 11,592 KiB RSS. The
   read and the rows are the size. **Size: 1–2 units, research.**

### To make it claimable

4. **The protocol panel.** Held-out rows labelled `keyed` / `unkeyed` / `new_template`, censused for
   template share and TRAIN-reply overlap *before* the run; Fresh V2 is banned by the item. The
   panel writer (`heldout-panel`, chained mode) and the census exist; the protocol panel does not.
   **Size: 1–2 units.**
5. **The report→strata mapping.** `VerdictInput` from an `m2-evidence` report. **Size: 1 small unit**
   (needs one real report).
6. **Kept-set agreement.** The ledger compares picks; a set-valued comparison does not exist.
   **Size: 1 small unit.**
7. **tok/s against context.** One point measured (1,473 tok/s at 200 tokens, codex-25); the item
   asks for the curve. **Size: 1 small unit.**

### To make it usable

8. **The service's slide.** `OpenAiServing` builds a `StackChat` per request and never enables the
   policy, so the loopback API refuses a conversation past the context. **Size: 1 small unit.**
9. **The Hamming/pair-rows adapter to the `Read` seam.** ADR-0007 names those as the second
   consumer; the Q31 and relation reads are adapted. **Size: 1–2 units.**

### Decisions (owner)

10. **Which path ships.** See §3.
11. **What "memory" is.** See §4.
12. **The read's input and write.** See §5.

### The run

13. **I6: the training run** — claude's pipeline. See §6.

## 2b. The owner's directives of 16 Nov (binding)

1. **Decision 10**: try the discrete trunk with the same read before deciding; it should be faster and
   cheaper without losing capability.
2. **Decision 11**: go with the exact-table recommendation; **re-explore the zeta and Hopf routers
   regardless** — the zeta ring is the complex discrete C2 ring carrying the routed zeros; the UOR
   rings are `Z/256Z` and may carry context. The archive's negatives must be audited, not trusted.
3. **Decision 12**: fix the read as recommended (write + margins + spiky pick), and **build the
   transport quantities into the read's key** — the transport is the SpiralCore FBS.
4. **Build what the literature says for both SDM/Hamming and absolute-position keys**, and **build
   the exact slot memory**.
5. **Every falsifier must be inspected against the data and the mechanism** before it is used — past
   falsifiers were invalid for the capability under test.
6. Agents are dedicated to the **first reply token**; every change must keep the downstream
   (serving, artifacts, compatibility) working.
7. 20.3 h for the full 1B is acceptable; work in the order below.

## 2c. Units added by the directives

14. **The planted phase-ring probe** (zeta re-evaluation §3): ring address, the bijection control,
    cosine, gcd; bar Δ ≥ 0.10 at δ = 1. Host, sub-second.
15. **`PhaseQ29` modular closure**: make `wrapping_add` an exponent add (today it round-trips through
    f64), with a closure test.
16. **The C2 ring as a carrier**: the phase accumulator as `[u8; 8]` (four complex pairs); the UOR
    ring's `neg`/`bnot` as a signed ring buffer's involutions.
17. **The first-reply-token trace — RUN (codex-27).** On a reachable `/10`, the first reply token is
    a **format bigram** (`':' → ' The'`) stamped by the copy heads at full weight; the depth-2 ask
    head does keep the assert position (0.261, 6th of 16) but **no readout-depth head does**; the
    value arrives four rows later from the memory line via the copy chain. So the query path exists
    and is reachable, and the failure is the **readout-depth key and its weight**, not the absence of
    a query. The narrow targets are units 18 (relative-offset ask key) and 19 (slot/place key that
    speaks the value at the reply's first row, latest-wins).
18. **The ask-head arm — the mechanism is already built (codex-28).** `with_carry`
    (`--hamming-carry-question H`) is the relative-offset question key, exported as `carry_rows` +
    `hamming.d*.carry` and applied by the served `AskImage`; it has **never been in the recipe**,
    which uses the place read instead, and the two are exclusive. The arm is the recipe with
    `--hamming-ask 1 --hamming-carry-question 2` and no place, gated by codex-27's trace and scored on
    bare-value anchors with no line and held-out values.
19. **The slot arm**: the place read as the slot key with **latest-wins** (DeltaNet's read-then-
    overwrite), which also explains and must fix arm rt's "47 → 40".
20. **The SDM/Hamming + absolute-position key build** (literature): a Hamming shortlist keyed on
    **relative offsets** with an exact verify, beside an **absolute-position** key as the control
    that the records say fails for 128 < t < 256.
21. **The transport-key harvest**: the per-token transport quaternion (and optionally the decay
    bucket and Hamilton gate) appended to the read's address, measured on the existing dumps.

## 2d. Decisions of the late-16-Nov sitting (binding)

**Decision 13 — score the checkpoint dev picked, not the last one.** The served arm r artifact was
step 16,384, the *worst* point of its run: dev loss bottoms at **0.54 epochs** (2.247 at step 2,048)
and rises to 2.371 by 4.34 epochs while train loss keeps falling. Exported at the same recipe and
scored on the same fresh panel:

| checkpoint | epoch | dev loss | value-present | right attribute | grammatical frame |
|---|---|---|---|---|---|
| **step 2,048** | 0.54 | **2.247** | **57 / 256** | 57 / 57 | 34 / 57 |
| step 4,096 | 1.09 | 2.335 | 56 / 256 | — | — |
| step 16,384 (what we had) | 4.34 | 2.371 | 25 / 256 | — | — |

The panel tracks dev. **2.3× was the cost of having scored the last checkpoint.** The served artifact
is `/tmp/opencode/armr-step002048.uorlut` (sha `5bb0f95d…`). A checkpoint is chosen by dev loss
*before* the panel is spent, and the panel then checks that rule.

**Decision 14 — the k-term family is met exactly, so post-hoc k is dead.** `route-terms-probe` read
every projection's effective dense map through the model's own CUDA projection on the unit basis:
**1,584 of 1,584 blocks are exactly one signed Cl(0,6) gather** (residual 0.000000000 at k = 1).
Exporting the same master at `terms = 2` or `4` is a provable no-op. The earlier "rank-1 covers 1.7 %
of a random dense projection" figure describes a *random* map; trained maps adapt to the family. A
k-term gain can only come from **training with the terms in the forward** (codex-30 slice 3), and
it is now a capacity question rather than an export-fidelity one.

**Decision 15 — the reply is the owner sentence; bare values only where justified.** The owner's
rule of 16 Nov: a general model answers in sentences unless a single word is justified. The panel's
`membership_only` labels accept the bare value, so `complete` stays 0 for a correct sentence; the
item's measure is **value-present**, with **right attribute** and **grammatical frame** beside it.
At the dev-best checkpoint all 57 carried the right attribute and 34 of 57 a well-formed frame; the
23 that blend the corpus's reply phrasings (`I current job is`) are a **corpus-shape defect**, not a
memory or read failure. Codex-32's "the sentence is not composed" was read off the worst checkpoint
and is corrected here.

**Decision 16 — "training scale" means data volume at the same mixture, not more steps.** The recipe
overfits from 0.54 epochs at 8.13 M tokens for a 1.11 M-value trunk. claude-152's "13× more text
hurt" was *13× more mixed text* (long general chat), so mixture and volume are separated: codex-34
generates 8× the **same** mixture with the same seed and shares and trains the same 16,384 steps to
the same pass count. If dev loss does not improve at the same pass count, the trunk is not
data-limited and capacity (the k-term arm) is the fork.

**The compatibility gate, proven.** `fbs-export --legacy-artifact` reproduces arm r's artifact **byte
for byte** (`f21046823640ea40…`), before and after the `terms`/`counts` change in codex-30 slice 2.

## 3. Decision 10 — which path, measured on one corpus

| Axis | FBS trunk + read | Discrete D11 (M1 line) |
|---|---|---|
| Memory capability | **arm r 0.749/0.759/0.782** on a matched-pair panel, 3 seeds; **no anchor pass** | **never scored on a memory panel** (no binary serves text from `.d11`) |
| The 2 Nov anchors | failed on `/5`, an FBS artifact **with no read** | not run |
| M1 bars | not closed | **all six, two seeds** |
| Decode (same host, x86 9950X, 64 steps) | **1,607 tok/s** with Hamming + turn memory (`/10`); 6,082–6,283 readless (`/5`) | **14,043–14,231 tok/s** (8.8× the `/10`, 2.3× the `/5`) |
| Artifact | 19.3 MB (116 MB with pair rows) | 86.5 MB |
| Training cost | 0.300 s/step, 16,384 steps ≈ 1.4 h on a 4090 | far larger (19 tables, K16/A8 doses) |

**Decision (recommendation, corrected).** The discrete path is **8.8× faster** and much cheaper at
serve time, and the item's anchor failure does **not** count against it — that was an FBS artifact
without a read. But the discrete trunk **cannot be compared on the same read today**, structurally:
its schema admits no read record, its read is a 16-bit orbit-root code over a 32-int trunk vector
(not a width-256 residual), and the Q31/Hamming adders both refuse a non-stack container at
`Container::parse`. The FBS+read line is the only one with memory-panel evidence (arm r 0.749–0.782),
and it has never passed the anchor either. So the choice is not settled by capability; it is settled
by **reachability of capability**.

**The cheapest decisive step** (no GPU, ~1 unit): score the discrete 19-table model's own read
(`query_read_table`) on a chained panel with `queries.json`, using the panel's pick scoring ported to
`IntegerDiscreteModel`. That answers "does the discrete read choose the asked value at all?" — the
missing capability number — and it does not need the schema. If it passes, the discrete is the path
on cost alone; if it does not, the FBS+read stays. A `q31_read` over the trunk vector (schema
`/2`) is the ~3-unit follow-up if the discrete wins.

**What would change the verdict:** the discrete read's panel number, or one `q31_read` record on the
discrete schema. Both are specified above.

## 4. Decision 11 — what "memory" is

**The memory already exists; the choice does not.** The p recipe carries four Hamming reads
(copy-latest, ask, place, copy-longest) over 64-bit sign codes, and the copy read's recall is
**0.98–0.99 at three seeds** (served as `/6`). What fails is the **first reply token**: choosing
*which* prompted value the question asks for (claude-326). So the remaining build is the **query
key and the write priority**, not a new store.

**Two constraints that shape any fix.**

- **One read per artifact.** The schema admits exactly one optional read record: a stack with a
  Q31 read cannot also carry Hamming reads, an address table, a point key or a relation read
  (`format.rs:1285-1311`; `session.rs:4343-4345`). So "copy + choose" must live inside **one** read:
  either the Hamming **ask head** (already in the recipe, `--hamming-ask 1`) learns the query, or
  the Q31 read gains a copy path. A second read is a schema decision, not an addition.
- **The read sees one row.** The native Q31 read consumes only the residual `b.x` at its depth; the
  per-token transport output, the decay bucket, the Hamilton gate quaternion and the FBS atom/code
  identity are **computed and discarded**, and its 16-lane L1 distances are reduced to an argmin
  with the margins thrown away. The zeta phases on the served path are telemetry-only.

The archived prime/angular router was read back from the files (pipeline map, "The archived prime
and angular router"). What is measured:

- **Zeta phases** θ_j(p) = wrap(γ_j·log p), the semiprime transition experts, the n-let key (the
  product of the last k primes), gcd admission, the S³/Hopf observation and the ℤ[φ] golden radial
  bridge are all built as algebra. The bridge passes its source-free fixtures.
- **As a router they are negative or inconclusive:** Hopf sector routing MRR 0.0045 against 0.0743
  (#430), 7 and then 43 of 512 sectors occupied (#303, #306); the learned 4-coordinate mixer gave a
  4.44% advantage against a 5% gate (#951).
- **The prime pointer raised the hit rate 0.377 → 0.545 and the memory answers stayed at 19, 21 and
  23 of 40 against a bar of 25.** Its own reading: the found token did not reach the prediction.
- **The exact-memory sieve with the true address is 80/80, 0/80 with no memory.** The mechanism is
  not the problem; the key and the write are.
- **Never built: the slot-address key, entity prime × attribute prime.** The pipeline map says so
  explicitly ("The slot-address key … remains untested").

What the FBS line adds (measurement): arm r's recall is the **copy read** with a memory line, and
the turn memory adds nothing to it; arm rt's failures are **write-overs after a copied value**.

**Outside evidence (research of 16 Nov, cited in §9).** The literature agrees with the archive and
narrows the design:

- **An explicit key→value table is the only exact, cheap, capacity-safe family at this size.** VSA /
  HRR binds at ~0.04–0.06·d items (40–70 pairs at d = 1,024) and every VSA family ends in a
  cleanup table anyway; our 16–32 facts do not need it.
- **Hamming/SDM addressing fails on the two things we need.** Its read is a majority centroid, not
  an exact value, and its counters are commutative — **SDM has no recency, so "latest exact" cannot
  fall out of it**. That is exactly our symptom. Attention matches SDM only after cosine/phase
  normalisation, not raw Hamming.
- **Zeta zeros as an address scheme appear nowhere** outside self-published preprints; treat as
  crank-adjacent. Prime phases are *structured* (multiplicatively dependent), which is worse than
  random or hashed for a low-coherence codebook. Prime products are also **unordered** — gcd cannot
  tell (a,b) from (b,a) without squaring one factor — and single-token rows shadow pair rows that
  share a factor. That is a precise criticism of our pair rows and prime route.
- **The published answer to generalisation is shortlist-then-copy:** a coarse fuzzy code selects m
  candidates, then an **exact equality verify** returns the value (FAISS-style; RETRO and kNN-LM
  both). Memorizing Transformers state it plainly: retrieval does **not** average or summarise; it
  retrieves exact values. Quantised 64-byte keys cost ~3% for 64× compression.
- **Relative offset, not absolute position.** With relative position the previous-token head stores
  >90% of associations at any position; with absolute encoding it **fails for 128 < t < 256**. Our
  turn keys are absolute token ids, which predicts exactly the context-length cliff we see.

**Decision (recommendation).** Memory is **an exact table keyed on a canonical code of the
question's slot, shortlisted coarsely and verified exactly, with latest-wins overwrite and copy
precedence over the table**. Concretely: key = (entity id, attribute id) or, cheaper and more
general, a canonical **bigram of the question's content words with a relative offset**; the code
selects ≤ m rows by popcount/L1 over a 64-bit code; an exact compare returns the value; a written
value overwrites by recency ("read-then-overwrite", DeltaNet's fix). The zeta/angular machinery is
at most a *code* for the key, and the measured record says it is neither a router nor a value
carrier. The two things still to build are the two the archive never built: the **slot key** and the
**write priority**.

**Falsifiers (any one retires the mechanism).**
1. **Value swap:** same keys, values permuted. If accuracy is unchanged, nothing reads the pairing.
2. **Oracle-key headroom:** feed the gold key. Oracle ≈ 100% with the derived key ≈ 40% and flat as
   paraphrases are added means the failure is canonicalisation, not the read.
3. **Capacity curve:** exact recall against N at fixed d. Collapse near N ≈ d (or 0.05·d) is a
   capacity bug, not an addressing bug.
4. **Address entropy:** if the top pick's weight over candidates is below half, exact copy is
   impossible — a cheap leading indicator.
5. **Prime ambiguity:** the fraction of queries whose gcd match set has size > 1. If it grows with
   context, drop the prime scheme.
6. On arm r's panel: the slot-keyed table must beat 0.75–0.78 copy-only recall on *reworded*
   questions with the "47 → 40" write-over count at zero.

## 5. Decision 12 — the read's input and its write

Two different defects have been measured and must not be confused:

- **Input separation** (codex-20/21): the integer stream at depth 6 agrees with the float replay in
  sign 0.887; a read retrained on the integer stream recovers to 0.86/0.89 held. So the loss is
  real, bounded, and already recoverable by training on the served stream. Its **scale** is the
  second half: the read's residual reaches ~2^18.5 against the gates' 2^31 (claude-390), so the
  address is computed at the bottom of the domain.
- **Write scale** (claude-386/387): the read's payload is **2.6% of the stream's norm**; a 97%
  correct choice moves the logits to at most 19/1,024. Scaling the payload ×16 reached R_ask 0.367.

**The owner's guess is confirmed, with a correction.** The read consumes the residual row only. The
per-token transport quaternion, the decay bucket (λ, κ), the Hamilton gate quaternion and the FBS
atom/code identity are **computed every token and discarded**; the Hamming distances belong to a
*different* read that cannot share the artifact. There is no spherical-harmonic read anywhere on the
served path (the 120 icosians are used only for snapping). What the read does keep is its argmin —
the **distance margins are thrown away**, so today we cannot see the address entropy the literature
says must be spiky. Whether the transport quantities would separate the fact positions is
**untested**; they are free to harvest in the float trainer and available as `NativeQ31Carry` in the
integer session.

**Outside evidence (16 Nov).** The read's two defects have known names. The payload's dilution is
**averaging**: a read that mixes k picks by an equal mean cannot return an exact value, and the
retrieval literature is explicit that exact memory must not average (Memorizing Transformers); a
spiky, monotonic address is what makes copy possible (Hedgehog). The layer choice also matters: the
key works best from the **pre-FFN (LN2)** hidden state, not the block output (kNN-LM's layer study).

**Decision (recommendation).** Fix the write first, because it is cheap and measured: give the read
a **learned write weight** (or a declared scale) so the picked row enters at a strength the readout
can see, and **stop averaging** — make the pick spiky (one row, or a weighted mix with a declared
distribution) rather than the equal mean of the kept cells. Then take the key from the **pre-FFN
row** rather than the block output, and only after that test a **spin-augmented key** — the
per-layer transport quaternion, or the Hamming codes, appended to the read's address — as the
alternative the owner suggests. The cheap version of the key test is a host/device experiment on the
existing dumps: score with the residual alone, then with the residual plus the spin, and compare the
rank of the asked fact. **Size: 1 unit for the write, 1 for the spiky pick, 1–2 for the key.**

**Falsifier.** If a write weight that raises |p|/|x| to ≥ 0.2 and a single-row pick do not raise
R_ask, the read's problem is the choice, not the write, and the key experiment is the only remaining
lever. The value-swap test of §4 applies here too: if permuting the values leaves R_ask unchanged,
the read is not reading the binding at all.

**Falsifier.** If a write weight that raises |p|/|x| to ≥ 0.2 does not raise R_ask, the read's
problem is the choice, not the write, and the key experiment is the only remaining lever.

## 6. The training plan (under one day on a 4090)

**The model.** The p-recipe FBS model is **1,112,576 learnable values**: the token table is
1,048,576 (94%), the six recurrence layers 63,744, the final norm 256. There is no dense matmul;
the projections are FBS table-gathers and quaternion rotations, and the exported artifact is
19,345,344 B (turn memory is most of the table bytes).

**Measured step times (not estimates).**

| Hardware | Steps | Rate | Note |
|---|---|---|---|
| RTX 5090 (claude-358) | 16,384 | 0.120–0.127 s/step | 33k context tok/s |
| **RTX 4090, wick (claude-370)** | 16,384 | **0.300 s/step** | 13,653 context tok/s; 3.33 steps/s |
| RTX 4090 (claude-380, W 192) | — | 0.308–0.355 s/step | corroborates |

So **the complete 16,384-step recipe is ≈ 1.4 h on wick**, and one epoch over the full 1B donor
corpus is **≈ 20.3 h** — both inside the owner's day. The 0.2033 s/step in an earlier note was a
256-step smoke, not a full run; 0.300 is the full-run figure.

**What that means.** Compute is not the bottleneck, and "train a complete model" is already cheap.
The bottleneck is that the current recipe **fails the goal**: claude-326 records 0 complete replies,
0 pairs, and the failure is the **first reply token** — choosing which prompted value the question
asks for. More steps on donor text do not target that choice. The donor corpus is 15–20× one run's
tokens; one epoch of it is general chat, not the fix.

**The order that avoids wasting the day.**

0. **The pre-flight (≈ $2, half a day).** The registered probe of claude-326 Q4: Part A (laptop) —
   the first-target rank and a Hamming trace on our own seals: does any head keep the asked value's
   position? Part B (GPU, ~$1) — a two-class recall probe with **query ≠ key** and a **3–4-token
   header gap**. B1 failing means this read cannot learn a query-conditioned choice at all; B2
   failing means the trunk does not carry the question forward. Either failure makes a language run
   pointless. *Note from this session:* on the synthetic recall family the asked-fact read already
   reaches **0.977** (float) and **0.99** (FBS-retrained) when trained on the served stream
   (codex-21), so the open part of the question is narrower — does the **text** trunk carry the
   question word? The same probe answers it.
1. **The write and the pick (§5).** Cheap, measured, and they decide whether a read can move the
   logits at all: the read's input is ~2^18.5 against the gates' 2^31 (claude-390), and its payload
   is 2.6% of the stream (claude-387). A run trained before this fix inherits both.
2. **The run.** Arm r's recipe on the memory-line curriculum, p trunk, 16,384 steps, batch 16,
   context 256, seeds 1–3, on wick: **≈ 1.4 h per seed**, ≈ 4.5 h for three. The read retrains on
   the served stream in minutes (codex-21's path).
3. **The measurement.** The protocol panel (§2.4), the ledger, and the verdict (§2.5). No arm is
   called a capability on TRAIN.

**What training cannot fix** (do not spend a run on these): the read's input fidelity (a read
retrain, not a trunk run), the artifact size (compression), the serving seam (float→integer
near-ties differ on 1.0–1.6% of rows, claude-354), and the C2′ choice (the gate family's ceiling).

## 7. Order of work

| # | Unit | Owner | Blocks |
|---|---|---|---|
| 0 | The pre-flight probe (§6.0) | codex | whether any run can succeed |
| 1 | Service slide (§2.8) | codex | anyone using the API |
| 2 | Kept-set + tok/s curve (§2.6–7) | codex | the protocol's measures |
| 3 | The read's write weight (§5) | codex | whether a run can succeed |
| 4 | The slot key (§4) | codex | the memory capability |
| 5 | The protocol panel + census (§2.4) | codex | the claim |
| 6 | The run + contract + export (§2.1–2, §6) | claude (run) / codex (export) | everything after |
| 7 | Report mapping (§2.5) | codex | the verdict |
| 8 | Compression (§2.3) | codex | the product budget |
| 9 | Hamming adapter (§2.9) | codex | the seam paying for itself |
| 10 | First-reply-token trace (§2c.17) | codex | **decides the arms; $0** |
| 10b | The discrete read's panel score (§3) | codex | decision 10, no GPU |
| 11 | Ask-head + slot arms (§2c.18–19) | codex | the capability |
| 12 | SDM/absolute-position keys (§2c.20) | codex | the key family |
| 13 | Transport-key harvest (§2c.21) | codex | the read's input |
| 14 | Phase-ring probe + `PhaseQ29` (§2c.14–15) | codex | the owner's claim, fairly |
| 15 | Discrete trial (§3) | codex | decision 10 |

Items 1–5 are code and can run beside claude's pipeline. Item 6 is the run. Item 7 closes the
verdict. Item 8 is the only research item that could change what ships.

## 9. Sources outside the project (16 Nov research)

kNN-LM (arXiv 1911.00172) · RETRO (2112.04426) · Memorizing Transformers (2203.08913) · Zoology /
MQAR (ICLR 2024) · Hedgehog (2402.04347) · DeltaNet (PMLR v139) · Plate HRR and FHRR · Kent
resonators (2020) · Kanerva SDM · SDM capacity (Chou 1987) · attention ≈ SDM (2111.05498) · modern
Hopfield (2008.02217) · Lisman & Idiart theta-gamma (1995) · RPE vs APE induction heads (2412.11459)
· L1 vs L2 contrast · quantised kNN-LM (2109.04212).

## 8. What would falsify this plan

- If the slot key does not beat copy-only recall on reworded questions, the memory thesis is wrong
  and the next candidate is a learned content key.
- If the write weight does not raise R_ask, the read's choice is the limit, not the write.
- If a training run on the memory curriculum cannot beat arm r at three seeds inside a day, the
  bottleneck is data or the corpus, not the optimizer — stop and re-read claude-326's assessment.
- If the discrete trunk with the same read matches arm r's recall, switch paths on cost.
- **No falsifier is used until it is inspected against the data and the mechanism.** For each one:
  does the training corpus contain the capability's structure, does the test's arm differ from its
  control in the named way, and can a control pass it by an unrelated route (occupancy, injectivity,
  copying TRAIN replies)? The five archive negatives in the zeta record are the template: three of
  them could not have tested what they were cited for.
