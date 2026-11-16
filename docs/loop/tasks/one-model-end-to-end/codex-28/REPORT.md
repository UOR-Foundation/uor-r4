# REPORT: one-model-end-to-end, codex-28 (the relative-offset key is built and untried)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-27](../codex-27/REPORT.md).

## What was found

The mechanism the literature recommends — a question key over a **relative offset**, not absolute
positions — is already implemented end to end **and has never been trained**:

| Piece | Where | State |
|---|---|---|
| the carry in the trainer | `HammingReads::with_carry` (`flock_read_hamming.rs:1072`), `--hamming-carry-question H` | built (claude-335), off in the p recipe |
| the export | `format.rs:2815` writes `carry_rows`; `hamming.d{depth}.carry` gains | built |
| the served read | `AskImage` with carry, Q16 gains, `ASK_CARRY_MAX` (`hamming.rs:99-110`) | built |
| the recipe | `--hamming-place KIT --hamming-place-pull 1` | uses the **place** read instead |

The two are exclusive by construction: `--hamming-carry-question` refuses `--hamming-place`, because
both change the readout read's ask heads. So the project has been running the place arm and has never
run the relative-offset arm.

## Why it matters

codex-27's trace puts the failure at the readout depth's ask key: the depth-2 ask head keeps the
assert position, the readout's does not, and the first token is the format bigram. The carry is
exactly the unbuilt lever at that depth. It is the difference between "the model has no question
key" and "the model has a question key it was never given".

## What is needed to run it

- The next02 corpus (`tokens.u16`), which is a store object, not in this worktree.
- A CUDA build of `train-in-space` on a 4090.
- The 64-step smoke, then the export, then codex-27's trace as the gate, then the full arm
  (16,384 steps, ~1.4 h a seed).

## Checks run

- **The 64-step smoke trained, exported and parsed.** Corpus `next02/input/uor-next02-inputs-92d7c0b36`
  (4 MB, 28 MB extracted), the trainer built with `--features cuda` on the 4090, the arm at
  `--hamming-ask 1 --hamming-carry-question 2`, 64 steps in 10 s. `fbs-snapshot` then
  `fbs-export --legacy-artifact` produced a `/7` artifact whose record reads
  **`asks [True, True]`, `carry_rows 2`, depths [2, 6], heads 4, kept 16**, with the table
  **`hamming.d6.carry`** present. To our knowledge this is the first artifact in the project to carry
  the relative-offset question key.
- **The `turn-memory-tail 9` variant refuses the export at 64 steps**:
  `memory.reply.keys: entry 0 is not a key this engine reads`. The base recipe (no tail) exports. The
  tail-nine chain is therefore not trainable-then-exportable at this budget; it is a separate defect
  to inspect before arm rt's configuration is rerun.
- **The 64-step trace is uninformative, as expected**: the carry gain starts at zero, and every head
  keeps the same 16 positions at row 135 at this budget. The gate needs the trained arm.
- The full arm (seed 2, 16,384 steps, ~82 min on the 4090) is running from the same recipe; its
  artifact is the one the trace and the bare-value panel are read on.
- The `--decision-trace` path refuses an unsnapped artifact ("the snap is not a decision it
  records"), so the gate uses `--hamming-trace` alone, which steps the artifact and emits each head's
  kept positions per row.

## The full arm and its gate (seed 2, 16,384 steps, 45 min)

The arm trained on the 4090 in **2,687 s** (0.164 s/step; dev top-1 peaked at 527/14,287 at step
12,288 and read 398 at 16,384; openings 587/1,226). It exported to an 18,661,248 B `/7` artifact,
sha `a890d112…`, whose record reads `asks [True, True]`, **`carry_rows 2`**, depths [2, 6], heads 4,
kept 16, with `hamming.d6.carry` present.

**The gate is negative on the episode, and the carry did train.** At the first reply read (row 135)
no head at either depth keeps any of the value's three occurrences — pos 9 (assert), 22 (ack) or 126
(the memory line). The carry table itself is not idle: **1,359 of 4,096 gains are non-zero**, the
largest +83,762 and -71,127 at Q16. But the **value token 2194 (`' ser') has a gain of −158 at Q16**
(about −0.0024), i.e. the carry learned other tokens and not the value.

**Why: the reach is two rows and the gap is six to eight.** At the reply start the rows behind are
the memory line's tail and the role header, not the question; the question's last token (and the
line's value) sit ~6–8 rows back. `ASK_CARRY_MAX` is 4,096, so this is not a structural ceiling — it
is a vocabulary of reach. The arm tested H = 2 and the protocol's injected line makes the gap longer
than 2.

**The next unit is therefore one flag**: the same arm at `--hamming-carry-question 8`, which spans
the gap to the question and to the line's value, with the same gate and then a bare-value read (no
line, held-out value).

**Also found:** `--turn-memory-tail 9` refuses the export at 64 steps (`memory.reply.keys: entry 0 is
not a key this engine reads`), and `--decision-trace` refuses an unsnapped artifact ("the snap is not
a decision it records"), so the gate runs on `--hamming-trace` alone.

## The H = 8 arm: the reach finds the positions; the margin zeroes their weight

The restarted arm trained to 16,384 in 2,717 s (dev top-1 532, openings 608/1,226 — the best of the
run) and exported an 18,661,248 B `/7` artifact, sha `84a8042f…`, with `asks [True, True]`,
**`carry_rows 8`** and `hamming.d6.carry` (1,410 non-zero gains, max +106,402; the value token's gain
**+871**, against −158 at H = 2).

The trace at the first reply read (row 135) now shows the change the reach was for:

| Read | Assert (pos 9) | Ack (22) | Line (126) |
|---|---|---|---|
| H = 2, any head | absent | absent at d6 | absent |
| H = 8, d6 ask h2 | **kept, 23 bits** | absent | **kept, 23 bits** |
| H = 8, d2 ask h2 | **kept, 24 bits** | absent | absent |

**But the weight is zero.** The learned margin at d6 h2 is **1,325,781 at Q16 ≈ 20.23 bits**, and the
distance to positions 9 and 126 is **23 bits**, so `max(0, m − d)/m = 0`. The mask keeps them; the
readout gives them nothing. The `/10` artifact's ask margin was ~35 bits, which is why its keeps at
24–26 bits carried 0.26–0.32.

So the failure has moved: it is no longer *where the read looks* (reach 8 fixed that) but **the
margin that turns a keep into a weight** — the same "write scale" defect as decision 12, now on the
Hamming ask head. The margin is a learned per-head parameter (initialised at 16 bits, grown to 20.2
here against ~35 in `/10`), and it is a table in the artifact, so a margin sweep is a **counterfactual
that needs no training**: raise h2's margin to 30 bits and the same keeps would weigh 0.23; 35 bits,
0.34.

## The run's history, for the record

The first H = 8 launch died silently at step 6,144 with no error and no OOM record; the restart
reproduced steps 2,048 and 4,096 exactly (1,378; 604) and passed 6,144, so the death was
environmental. Two restarts at the same step would have been a defect; one was not.

## The bare-value episode: the ask head keeps the assert, the mixture still loses

The encoder is built (`crates/uor-r4-integer/src/bin/encode-dialogue.rs`, `--turn role=text`,
`encode_assistant_prefix`, prints the prompt ids and the reply's starts), so the episode is exact and
reusable. The bare episode (45 ids, first reply row 44) states the fact once
(`The new puppy at our house is named Pickle.`), acknowledges it neutrally (`Got it.` — so the value
appears **only** at position 12), and asks the question. No memory line, no format preamble in the
prompt.

**First reply token, by ask margin** (the reference is `971`, ` Pickle`):

| d6 h2/h3 margin | first pick |
|---|---|
| 20.2 / 19.8 bits (as trained) | 1399 |
| 30 / 30 | 617 |
| 35 / 35 | 617 |
| 60 / 60 | 1856 |

The margin **does** change the argmax (1399 → 617 → 1856), so the ask weight reaches the logits, but
**no margin produces the value**. The trace says why the lever is not enough:

- **the ask head keeps the assert**: d2 h2 at 28 bits, d2 h3 at 27, **d6 h3 at 29** (d6 h2 misses it
  at 38);
- **the copy heads keep exact matches at 0 bits** (positions 4, 22, 31 — the role and punctuation
  markers), two heads of the four, each at a 16-bit margin;
- at a 60-bit margin the assert's weight is `(60 − 29)/60 = 0.52` on **one head of four**, so its
  move is about an eighth of the readout's total against two exact copy moves.

**So the next lever is the readout's aggregation, not the reach and not the margin.** ADR-0007
already names this as the gap: five selection rules and four aggregation rules for one operation.
The unit is a learned per-head weight (or copy precedence at the reply's first row), measured on this
exact bare episode and on held-out values — with the value-swap and the oracle-key falsifiers.
