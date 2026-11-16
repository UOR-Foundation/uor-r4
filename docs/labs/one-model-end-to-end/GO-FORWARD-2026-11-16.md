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
- **The copy read carries recall when it is allowed to.** Arm r (copy read latest-exact, turn memory
  leaving copied keys alone, a memory line): exact matched pairs **0.749 / 0.759 / 0.782** at three
  seeds (claude-345). Arm rt (three-key turn memory): E 0.71, script general 10/10, but **55 panel
  replies wrong, 33 of them after a copied value or a changed number** — "47" served as "40".
- **The published discrete artifacts failed both fixed multi-turn memory anchors** (item, 2 Nov),
  and the discrete M1 model is the one that closed all six M1 bars (DEV 7,002/7,001, Fresh V2
  7,215/7,228, p-f 7,850/7,872 at two seeds; 10,876/10,566 tok/s on an M1; artifact 86.5 MB).

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

## 3. Decision 10 — which path, measured on one corpus

| Axis | FBS trunk + read (this line) | Discrete D11 (M1 line) |
|---|---|---|
| Memory capability | **arm r 0.749/0.759/0.782** matched-pair recall, 3 seeds | anchors failed (item, 2 Nov) |
| M1 bars | not closed | **all six, two seeds** |
| Decode rate (M1) | 1,567–1,636 tok/s (p recipe) | **10,876 / 10,566 tok/s** |
| Prefill (M1) | 4,309–4,499 tok/s with the read | not recorded here |
| Artifact | 19.3 MB (116 MB with pair rows) | 86.5 MB |
| Training cost | **~0.20 s/step at 16,384 steps ≈ 55 min on a 4090** | far larger (19 tables, K16/A8 doses) |

**Decision (recommendation).** Capability decides: the FBS trunk with the copy read and the turn
memory leaving copied keys alone is the only configuration that has shown held-out memory recall,
and it is also the cheapest to train and the smaller artifact. The discrete path's advantage is
decode speed (about 7×) on a task it has already passed. ADR-0007 already puts both under D11
integer serving, so this is not a fork in the runtime — it is a choice of trunk and read.

**What would change it:** a measurement that the discrete trunk, given the same read, reaches the
same recall. That is one run at M1's shape and it is cheap; it has never been tried.

## 4. Decision 11 — what "memory" is

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

**Decision (recommendation).** Memory should be **an exact table keyed by the question's slot
(entity, attribute), read at the readout, with copy precedence over the table**. The zeta/angular
machinery is a *code* for the key (a fixed angular code per token costs nothing per token) but the
measured record says it is not a router and not a value carrier. The two things that must be built
are the two the archive never built: the **slot key** and the **write priority**.

**Falsifier.** On arm r's matched-pair panel, a slot-keyed table with copy precedence must beat the
0.75–0.78 copy-only recall on *reworded* questions, with the "47 → 40" write-over count falling to
zero. If it does not, the key is not the mechanism and the next candidate is a learned content key
(the question's content word), not the geometry.

## 5. Decision 12 — the read's input and its write

Two different defects have been measured and must not be confused:

- **Input separation** (codex-20/21): the integer stream at depth 6 agrees with the float replay in
  sign 0.887; a read retrained on the integer stream recovers to 0.86/0.89 held. So the loss is
  real, bounded, and already recoverable by training on the served stream.
- **Write scale** (claude-386/387): the read's payload is **2.6% of the stream's norm**; a 97%
  correct choice moves the logits to at most 19/1,024. Scaling the payload ×16 reached R_ask 0.367.

**The owner's guess** (that the transport's Hamming distances, spins and spherical-harmonic angles
are not stored) is half right: the read consumes the residual row only. The transport's rotations
exist per layer and are **not** offered to the read as a key; the Hamming codes are a separate read
(head 4), not part of the Q31 read's key. Whether adding them separates the fact positions is
**untested**.

**Decision (recommendation).** Fix the write first, because it is cheap and measured: give the read
a **learned write weight** (or a declared scale) so the picked row enters at a strength the readout
can see, and measure the payload share beside the picks. Only then test a **spin-augmented key** —
the per-layer transport quaternion, or the Hamming codes, appended to the read's address — as the
alternative the owner suggests. The cheap version of the key test is a host/device experiment on
the existing dumps: score with the residual alone, then with the residual plus the spin, and compare
the rank of the asked fact. **Size: 1 unit for the write, 1–2 for the key.**

**Falsifier.** If a write weight that raises |p|/|x| to ≥ 0.2 does not raise R_ask, the read's
problem is the choice, not the write, and the key experiment is the only remaining lever.

## 6. The training plan (under one day on a 4090)

**The arithmetic, from measurement.** The p recipe runs at **0.2033 s/step** on an RTX 4090
(claude-381, "wick"), at 16,384 steps: **≈ 55 minutes per model**, about 70 minutes with a vector
read (1.2–1.4×). Three seeds fit in a working day. The recipe sees 16,384 × 16 × 192–256 ≈ **50–67M
tokens**, so the 1B donor corpus is **15–20×** more than one run consumes; one epoch over 1B tokens
would be 10–14 h on a 5090 and 20–30 h on a 4090 — over the day, and not obviously better than more
epochs over a memory curriculum.

**What to train.** Not general chat first. The failing capability is memory recall, and the cheapest
configuration with a measured prior is **arm r's recipe on a memory-line corpus** (recall=reply) with
the p trunk, plus whichever fix from §5 the evidence supports:

1. retrain the read on the *served* stream (minutes; codex-21's path);
2. add the payload write weight (§5);
3. add the slot key (§4) as the next arm;
4. 16,384 steps, batch 16, context 256, seeds 1–3, on wick.

**Gates.** Every arm is scored on the protocol panel, not on TRAIN: exact matched pairs, the
write-over count, and the first-divergence ledger. A run that raises exact matches by copying TRAIN
replies is not a capability (the census catches it).

**What training cannot fix** (do not spend a run on these): the read's input fidelity (a read
retrain, not a trunk run), the artifact size (compression), the seam and the service (code), and the
C2′ choice (the gate family's ceiling — a mechanism change, not more steps).

## 7. Order of work

| # | Unit | Owner | Blocks |
|---|---|---|---|
| 1 | Service slide (§2.8) | codex | anyone using the API |
| 2 | Kept-set + tok/s curve (§2.6–7) | codex | the protocol's measures |
| 3 | The read's write weight (§5) | codex | whether a run can succeed |
| 4 | The slot key (§4) | codex | the memory capability |
| 5 | The protocol panel + census (§2.4) | codex | the claim |
| 6 | The run + contract + export (§2.1–2, §6) | claude (run) / codex (export) | everything after |
| 7 | Report mapping (§2.5) | codex | the verdict |
| 8 | Compression (§2.3) | codex | the product budget |
| 9 | Hamming adapter (§2.9) | codex | the seam paying for itself |

Items 1–5 are code and can run beside claude's pipeline. Item 6 is the run. Item 7 closes the
verdict. Item 8 is the only research item that could change what ships.

## 8. What would falsify this plan

- If the slot key does not beat copy-only recall on reworded questions, the memory thesis is wrong
  and the next candidate is a learned content key.
- If the write weight does not raise R_ask, the read's choice is the limit, not the write.
- If a training run on the memory curriculum cannot beat arm r at three seeds inside a day, the
  bottleneck is data or the corpus, not the optimizer — stop and re-read claude-326's assessment.
- If the discrete trunk with the same read matches arm r's recall, switch paths on cost.
