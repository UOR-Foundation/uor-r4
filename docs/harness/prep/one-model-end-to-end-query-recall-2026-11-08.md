# Prep sheet: recall where the query is not the key (one-model-end-to-end, claude-327, 8 Nov 2026)

TEST FITNESS: pending (a fresh non-author FIT, then a D67 pre-run review, before any pod).

## The question

[claude-326's assessment](../../loop/tasks/one-model-end-to-end/claude-326/ASSESSMENT.md) (Q4, Part B) asks whether the trained Hamming read can **choose a value by a query that is not the key**, the way `m2-fresh256`'s "What is my job?" chooses the value after "I am a … by trade". The 27 Oct probe ([sheet](in-space-trunk-to-text-hamming-recall-2026-10-27.md), notebook row 109) recalled 0.984–0.994, but its query writes the key again (`in_space_long.rs:11803`). There the copy head's exact match on the current token does all the work, and the type heads' learned query is never needed.

claude-327's Part A shows why the panel cannot answer this directly, on claude-322's and claude-323's served artifacts:
- the reply's first token is one of 2–3 frame words (" Your", " It", " G") across all 256 rows;
- the asked relation (job against home) does not change which one.

So the first-target rank there reads the reply's format, not the choice. This probe takes format out.

## Steps: the arms

**Common to every arm**, as row 109's sheet:
- the FBS sheet trunk, `--objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 4096 --eval-every 512 --lr 0.001 --lr-end 0`;
- context 192;
- seeds 1 and 2;
- held-out windows scored at every checkpoint, decoded as the whole table's nearest token.

| Arm | Task | Read |
|---|---|---|
| **B0-h** | `--recall 4`, unchanged: the query writes the key again | `--hamming-read 4` |
| **B0-n** | the same | none |
| **B1-h** | two-class (below) | `--hamming-read 4` |
| **B2-h** | two-class with a 4-token header (below) | `--hamming-read 4` |
| **B2-n** | the same as B2-h | none |

**The two-class task** (new, `--recall-query 1`):
- **Relations:** a window draws 2 relation tokens `K_a`, `K_b` from a 16-token relation range.
- **Facts:** it writes facts `K v` at distances in the far buckets, 33–96 and 97–180, with fresh values. Each relation is written 1 to 3 times; its latest value counts, as the panel's update rows.
- **The query:** a query writes `Q(K_j)`, where `Q` is a **fixed bijection** from the 16 relation tokens to a disjoint 16-token query range, the same in every window and seed. The query token is never a relation token.
- **Targets:** the scored target is `K_j`'s latest value, and it is the window's only scored target. Values, relations, queries and filler come from disjoint ranges.
- **What it rules out:** a copy head keyed on the current token can match nothing at the query, so recall needs a learned query.
- **Recording:** each held-out query records whether nearest names the asked value (`R_ask`), the other relation's latest value (`R_oth`), or neither.

**The header** (`--recall-header 4`): four fixed tokens `H1 … H4`, from their own range, follow `Q(K_j)`, and the target is scored after `H4`. This is the panel's distance from the question's word to the answer position (`?` … `Assistant:`).

**The code this needs** (the next unit's build, in `crates/uor-r4-float-train`, added to the item's scope in its ORDER):
- `recall_window` gains the two flags, default off, so `--recall 4` alone writes byte-identical windows (I-1);
- `--score-rows`-style held-out recording of `R_ask` and `R_oth`.

## Bar (pre-registered), at step 4,096, both far buckets, both seeds

`R_ask(arm)` and `R_oth(arm)` are counted over 512 held-out queries per bucket.

| Arm | Passes when |
|---|---|
| **B0 (instrument)** | `R(B0-h) ≥ 0.98` and `R(B0-n) ≤ 0.02`, as row 109. Otherwise the run is an instrument fault and nothing else is read |
| **B1 (choice)** | `R_ask(B1-h) ≥ 0.50` and `R_ask(B1-h) ≥ R_oth(B1-h) + 0.25` |
| **B2 (carry)** | the B1 rule for B2-h, and `R_ask(B2-h) ≥ R_ask(B2-n) + 0.10` |

Copying the latest value of either relation scores about 0.5 on `R_ask` and on `R_oth`. That is why the bar asks for selectivity, not recall alone.

## Outcome map (read in order)

1. **B0 fails:** an instrument fault. Repair it and rerun; nothing else is read.
2. **B1 fails:** this read's configuration (type-head rotation query, read 4, width 256) does not learn a query-conditioned choice on this task. The result is mechanism-side for the configuration (§4.1, §4.2). Next: lever (b) of the assessment, a question-keyed read, under its own sheet. It must pass B1 and B2 here before any language run.
3. **B1 passes, B2 fails:** the read can choose, but the trunk does not carry the query four tokens forward. Next: the carry, either a read at the query token whose result is written forward, or the answer scored at the query. Tested on B2 first.
4. **B1 and B2 pass:** a learned query can make the panel's kind of choice in this read. The blocker is then data or the language setting. Next: claude-325's bare-answer recipe, larger or with a curriculum from B2's form to the panel's.

**Partial results:**
- one seed passing is read as not passed;
- the margins and every cell are reported either way.

## Instrument checks

- **I-1:** with neither new flag, `recall_window` writes windows byte-identical to `main`'s for seeds 1 and 2 at indices 0–127. This is a unit test that fails if the default path moves.
- **I-2 (two-class windows):**
  - the query token is never a relation token;
  - `Q` is the same in every window;
  - the target is the asked relation's latest value;
  - both relations appear;
  - exactly one target is scored;
  - the ranges are disjoint.

  Each is a unit test with a mutation that makes it fail.
- **I-3 (header):** the target sits 4 tokens after `Q(K_j)`, and the header tokens are fixed. Tested the same way.
- **I-4 (finite CUDA):** a 32-step smoke of every arm on the pod, at finite losses, before the timed run.

## Probe ladder

B0 first on the pod. If it misses its bar, stop and repair; B1 and B2 do not run on a failed instrument.

## Data split (generated, none from the store)

Every window is generated from `(seed, index)`. Held-out windows use the 27 Oct sheet's seed offset, so they never equal a training window at these run lengths.

## Base artifact

None. Every arm trains from step 0 at the pin the next unit names.

## Minimum test power

512 held-out queries per bucket per arm. At 0.5, one arm's standard deviation is about 0.022, and a difference of two arms about 0.031:
- the 0.25 selectivity margin is about 8 standard deviations;
- the 0.10 carry margin is about 3.2.

Two seeds must agree.

## Cost

Row 109 ran three arms on one RTX 5090 at about 12,187 context tokens a second with the read. Each arm is 4,096 × 16 × 192 = 12.6M positions, about 17 minutes with the read. Five arms at two seeds is about 2–3 hours of one card, about $1–2 at Vast's RTX 5090 rate, inside the caps.

## Stop rule

- Stop at B0's failure, or at any non-finite loss.
- A pod that does not answer SSH is retried once on another offer.
- A watcher is armed on every pod (L-004).
