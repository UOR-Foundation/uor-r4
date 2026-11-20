# The mechanism: exact addressed memory with a formed address (20 Nov 2026)

**This is the plan of record for the chat capability.** It replaces every trunk-scaling and
"beat a bigram" line as the primary objective. It is built on `docs/mechanisms/exact-memory.md`,
whose status is PARKED with a named reopening condition, and on `GO-FORWARD-2026-11-16.md` §4/§5.

## Why this and not a bigger model

The project's own record says where the failure is, twice:

- *"The plain exact store itself worked whenever its address was right… What failed was the model's
  learned step of forming the address from a question."*
- *"77.7 percent of the held-out failures were query-entity tagging."*

And the measured failures of every alternative are already in the record: Hamming/SDM reads average
(values smear), zeta-prime routing is negative or inconclusive, a learned mixer gave 4.44 % against a
5 % gate, and the trunk line trains a next-token predictor that cannot condition on the prompt
(measured today: first reply token 12.7 %, mid-reply 22.7 %, generation collapses to one string).

A bigger trunk does not address any of that. **The missing component is address formation**, and it
is a small, closed classification problem, not a language-modelling one.

## The mechanism, concretely

Per conversation, three parts. All integer, all arithmetic, no matmul, no squashing, one native state.

1. **An exact store.** `address -> value`, written from the conversation's own statements, with
   **latest-wins** overwrite (the "47 → 40" class is a write-priority defect, not a read defect) and
   **typed abstention** for an address never written.
2. **A formed address.** `address = canonical_code(entity, attribute)` — a hash/code over the two
   slots, **not a product of primes**. Amended 20 Nov on the adversarial review's evidence: the
   archive already flags prime schemes as an address as *crank-adjacent* and, as a code, *worse than
   random or hashed* for a low-coherence codebook (`GO-FORWARD` §4); the existing `PrimeAddress` sorts
   and dedups primes into an unordered list, `semiprime()` exists only for exactly two primes and
   overflows `u64` for any multi-word slot, and gcd admission makes a one-prime address shadow every
   pair that shares it. A canonical code has none of those defects and is what Decision 11 actually
   recommends.
3. **A slot tagger.** From the question's surface form to `(entity, attribute)` over a **closed
   attribute set**, with the entity resolved by matching the question against the conversation's own
   mentions. **It is a deterministic table, and that must be stated rather than hidden:** it is not
   "learned in the one native state" (§3.7) and the plan does not claim it is. Its justification is
   not novelty — it is that a compositional (entity, attribute) key answers **recombinations** the
   table has never seen, which a question→answer table cannot. That is the only claim the mechanism
   makes, and it is the claim the gate must test.

The reply is then **copied**, not generated: the value span from the store (or from the context by
copy precedence), placed in a frame the corpus supplies. That is the literature's "shortlist then
exact verify" and the charter's §3.12 ("exact identity stays separate from geometric address").

## What is already built (do not rebuild)

- the exact store and its five-arm oracle (40/40 current, 20/20 previous, 20/20 abstention, 0/80 with
  no memory) — `docs/mechanisms/exact-memory.md`;
- the copy read at 0.98–0.99 recall over three seeds;
- the integer serving path and the artifact format;
- `README.md`'s dialogue protocol and the frozen panels infrastructure.

## The pieces to build (this plan)

| # | piece | why it is the piece |
|---|---|---|
| **A** | **Slot tagger**: question → `(entity, attribute)`, as a table over content bigrams with a closed attribute set | it is the measured 77.7 % of failures |
| **B** | **Role-tagged prime address** with separate entity/attribute domains, integer | the archive's order/role objection, fixed by construction |
| **C** | **Latest-wins write and copy precedence**, integrated into the store | arm rt's "47 → 40" |
| **D** | **Integer read kernel**: admit by the formed address, return the value exactly, abstain when absent | reopening condition 4 |
| **E** | **The panel**: ≥ 80 held-out rows (fact stated, question, value), paraphrased, with distractor entities in the same conversation | reopening condition 1 |

## The gate (re-anchored on the review's evidence)

The 27/40 figure is **withdrawn as the gate**: the review showed its anchor (19/40) was a product-key
feed-forward swap on an authored panel whose *no-memory* baseline was 20/40, so "7 over half-correct"
measures the panel, not the mechanism. The gate is now relative, on the same frozen panel:

1. **Beat the registered count-table control** — a question→answer table fit on the panel's TRAIN
   half — **on the held-out half, where every (entity, attribute) pair is a recombination never seen
   in TRAIN.** This is the compositionality claim, and it is the only one worth making. §4.4 and
   L-010 require the control to be part of the gate, not a post-hoc check.
2. **Beat the copy read's recorded 0.749–0.782** on the same held-out rows, since a copy read is the
   strongest non-compositional reference the project has.
3. **Held-out paraphrases only**: no held-out question may appear in TRAIN in any form.
4. **Held-out combinations only**: each held-out row recombines an entity and an attribute that TRAIN
   never paired. A one-entity/one-attribute panel cannot pass this by accident.
5. **Address formation apart from read accuracy**, and **typed abstention** for unwritten addresses.
- **Address formation reported separately from read accuracy** (reopening condition 5), so a pass
  cannot be a store artefact and a failure cannot hide in the tagger.
- **Zero write-over errors** of the "47 → 40" class.
- **Typed abstention**: an unwritten address answers "not known", never a wrong value.
- Deterministic grading only; no neural grader (§3.10).

## Adversarial checks required before implementation is called done

1. **Value swap**: permute the values between addresses. If accuracy is unchanged, nothing reads the
   pairing and the mechanism is refuted.
2. **Oracle key**: feed the gold `(entity, attribute)`. If oracle ≈ 100 % and the tagger ≈ 40 %, the
   failure is canonicalisation (the tagger), which is the claim.
3. **Distractor honesty**: every question has ≥ 1 same-attribute distractor entity in context. A
   narrowest-possible question set (one entity, one attribute) would pass without forming an address.
4. **Constraint audit**: no matmul, no squashing, integer serving, one state, bit-identical replay.

## What this refuses

- A bigger trunk as the answer to an address-formation failure.
- Any gate the architecture cannot meet (bits-per-token vs a bigram, teacher token-F1): those measure a
  language model, and this plan builds a memory with a tagger.
- An authored dialogue *training* corpus. The corpus stays the real distillation; only the **test**
  panel is written, because the mechanism doc requires a frozen panel and a deterministic grader.

## First measurement of the mechanism (20 Nov, `slot-address-check`)

A generated, frozen panel: 10 entities x 8 closed attributes = 80 pairs, half TRAIN and half HELD
OUT, where the held-out half shares every entity and every *fitted* surface form with TRAIN and
shares **no pair**. Integer tables only; no matmul, no squashing, no learned weights; deterministic
from a fixed seed; four seeds run.

| measure | result |
|---|---|
| address formation, known wording | **40/40** (all four seeds) |
| oracle key (gold address) | 40/40 |
| **mechanism (formed address, exact store)** | **40/40** |
| control: question->answer pair table | **0/40** |
| falsifier: permuted values | **0/40** |
| falsifier: held-out wording | **17/40** |

**What this establishes.** The store is not the failure and neither is the address arithmetic: with a
formed `(entity, attribute)` address, **recombinations that were never seen are answered exactly**,
while the memorised pair table answers none. The value swap drops it to zero, so it is reading the
binding rather than answering by coincidence. That is the compositional claim, measured.

**What it does not establish, and it is the recorded hard case.** On **wording the tagger never saw**
it scores **17/40** — and the 17 are exactly the rows whose alternative surface happens to share a
word with the fitted one. A word table cannot map a novel synonym, which is the archive's
"same relation in new words — sieve alone answered 1 of 52". The tagger's **coverage** is the limit,
and coverage is a data problem, not a geometry one: the fitted table in this panel saw 40 rows.

**So the next unit is tagger coverage**, not the store, not the trunk: fit the attribute table from
the real questions in the corpus, and report the same table on the same panel. If coverage closes,
the mechanism is complete for the closed attribute set; if it does not, the honest answer is a
smaller closed set, not a bigger model.

## The tagger's coverage closes with real data (20 Nov, `slot-table-build`)

The measured limit was wording coverage: 17/40 on surfaces a 40-row panel never saw. `slot-table-build`
fits the attribute table from the project's own dialogue corpus instead: **168,956 user questions**,
**20,099 labelled by exactly one attribute**, keeping only words whose dominant attribute has at least
3 observations and 2x the runner-up. Result: **56 words**, and every one lands where it should.

| measure | panel-fitted table | corpus-fitted table |
|---|---|---|
| address formation (known wording) | 40/40 | 40/40 |
| **held-out wording** | **17/40** | **40/40** |
| mechanism (formed address, exact store) | 40/40 | 40/40 |
| control (question->answer pair table) | 0/40 | 0/40 |
| falsifier: permuted values | 0/40 | 0/40 |

**The mechanism is now complete for a closed attribute set, and every part is measured:** an exact
store, a canonical `(entity, attribute)` address, and a tagger whose vocabulary comes from real
questions. It answers recombinations never seen, on wording never seen, and it is reading the binding
(permuted values score zero). No part of it needs a matmul, a squashing function, a float master or a
second state.

**What it is not yet.** It is a memory, not a conversation: it answers questions about facts stated in
the conversation, in a closed set of eight attributes. The next unit is **integration into the served
path**, so a real conversation (facts stated, question asked, value returned) runs end to end through
the integer artifact, and then the closed set grows.

## Served: the mechanism answers a conversation (20 Nov)

`crates/uor-r4-integer/src/slot_memory.rs` is the served form: an exact `address -> value` store with
**latest-wins**, a canonical integer code for `(entity, attribute)` (FNV-1a mix, **not** a prime
product), a tagger loaded as data, whole-word matching with a deterministic stem rule, and typed
abstention. `memory-chat` runs it against a live conversation.

```
user: Alice's favourite colour is green.     assistant: noted (alice / colour)
user: Bruno lives in Denver.                 assistant: noted (bruno / city)
user: Carla works as a nurse.                assistant: noted (carla / job)
user: What is Alice's colour?                assistant: green
user: What is Bruno's town?                  assistant: Denver
user: What is Carla's profession?            assistant: a nurse
user: What is Alice's city?                  assistant: I don't know that.
user: Alice's favourite colour is blue.      assistant: noted (alice / colour)
user: What is Alice's shade?                 assistant: blue
```

Five unit tests: a recombination never stated, latest-wins, abstention on an unwritten slot,
word-boundary matching ("carla" must not match "car"), and a statement writing the slot a question
then reads.

**Two bugs the conversation exposed and the tests now cover**: the write reported the last mentioned
entity rather than the one it wrote, and inflections ("works", "lives", "plays") missed the table
until the stem rule was added.

**Remaining gap, stated plainly**: the tagger table is loaded from `slot-table-build` output, not yet
carried inside the artifact. The charter's one-state rule (§3.7) requires it there, and the schema
admits exactly one optional read record, so that is a **schema decision**, not a code cleanup. Until
then this is a served component in the serving crate, not yet a field of the integer artifact.
