# Surface-form breadth in the synthetic memory corpus (phrase lock-in) — 3 October 2026

Status: measurement in progress (draft PR #1645; refs #1512, #820).

## Question

Training on 76 relations gave 14/120 hits on unseen relations while training on
10 gave 0/120, but a one-letter change in a relation phrase
(`color` -> `colour`) breaks binding, and the model fails to answer a question
about a fact it has just been given while it can echo a value verbatim. The
working hypothesis is **phrase lock-in**: binding is keyed to the exact trained
*shaped phrase*, not to the relation concept, and relation breadth helped only
because it multiplied surface phrases.

Prediction under test: training with **many surface forms per relation**
generalises to unseen *forms* the way 76 relations generalised to unseen
*relations*.

## Instrument

`crates/uor-r4-training/src/bin/synthetic-memory-corpus.rs` (extended on top of
#1641's 76-relation table, which is not yet on `main`; #1642 edits the same file
and will conflict textually):

* `forms=N` — surface forms per relation the generator may draw from. Form 0 is
  the relation's canonical phrase. Forms 1..7 are paraphrase question and
  statement templates over a hand-authored per-relation *topic*; the **reply
  keeps the single canonical answer form in every arm**, so the output shape is
  held fixed and only the input wording varies.
* Forms 8..12 are **held out**. `forms` is clamped to 1..=8, so the draw index
  never reaches the held-out tail and no training store can contain those forms
  (`forms=13` is refused: "forms must be 1..=8; a larger value would train on
  the held-out tail"). The panel draws them from the same table, which is what
  makes "unseen form" a property of construction rather than of inspection.
* `panel_out=DIR` writes the panel (`requests.json`, `expected.json`,
  `manifest.json` including the full value table).
* Startup validation: every relation has exactly one slot, every slot names a
  relation, and the value sets cover the table exactly.

**Byte identity of the fixed-form path.** `forms=1` must reproduce the previous
recipe exactly. With `rows=30000 seed=17` the new binary writes

```
tokens.u16        9bae7202ca4a94552b8894739e7f001fb8967d28da79ed04e99c3bc524c703fd
response_mask.u8  ed15cb21a948c7e308609ada9f352b3d7cf079609818ee9b80274174d5f2fa41
```

which are the hashes of `wide-memory-corpus2`, the store the existing
wide-relation arm trained on. The pre-change wide generator, built in another
lane, reproduces the same two hashes independently. Only the manifest's
`synthetic` block gains fields.

## Panel design

464 rows, 74 per condition over the 74 relations that have paraphrase forms
(`garden` and `mortgage` have `have` / `do not have` values and keep the
canonical single form in every arm, so they are excluded from the form axis):

| condition | statement | question |
|---|---|---|
| `seen` | canonical (trained) | canonical (trained) |
| `seen-comp` | canonical | canonical, with a competing statement between |
| `unseen-state` | held-out form | canonical |
| `unseen-ask` | canonical | held-out form |
| `unseen` | held-out form | held-out form |
| `unseen-comp` | held-out form | held-out form, with a competing held-out statement between |
| `perturb-ask` (n=20) | canonical | canonical with **one word substituted** (contraction or spelling variant) |

The competing conditions reuse their non-competing twin's exact statement and
question, so the extra turn is the only difference. The held-out form index
varies with the relation, so all five held-out templates are exercised.

Scoring (`scripts/copyfid-score.py`) is value presence in the last turn's reply:
**strict** = the value's words in order with no word character touching either
end (so `greenish` does not count for `green`); **lenient** = case-insensitive
substring. Degeneracy is reported separately (empty, truncated, missing EOS,
repeated word 4-grams, five-word echo of the question). Wilson 95% intervals
and an exact McNemar test over shared rows are in the summary JSON.

### Tool limits, not patched

* `lut-chat` refuses a panel above 128 requests ("a request panel needs 1 to 128
  requests with distinct ids, each with 1 to 8 nonblank user turns"), so the
  panel is run as four pre-split chunks and scored as a union
  (`scripts/copyfid-split-panel.py`).
* No surviving binary carries the corpus generator as a `geometric-stack` mode
  (it is a separate `bin`), and none of the surviving `synthetic-memory-corpus`
  binaries has the wide table, so the generator was built from this worktree
  (`cargo build --release -p uor-r4-training --bin synthetic-memory-corpus`).

## Arms

Matched on init, steps, learning rate, batch, data seed, dev seed, policy and
synthetic share:

| | F0 (phrase-locked) | F1 (form-varied) |
|---|---|---|
| synthetic store | `wide-memory-corpus2`, 30,000 rows, 1,717,822 tokens, seed 17 | `copyfid-f1-corpus`, 25,991 rows, 1,723,669 tokens, seed 17 |
| forms per relation | 1 (canonical) | 8 (canonical + 7 paraphrases) |
| synthetic share of the mixed store | 2.039% | 2.046% |
| mixed store | `mixed-wide-2`, 84,247,512 tokens | `copyfid-mixed-f1`, 84,253,359 tokens |
| init | `chat-dose-continue-1/dialogue/model` | same |
| steps / batch / lr / warmup / min_lr | 2500 / 16 / 0.0002 / 50 / 0.1 | same |
| data seed / dev seed / policy | 20260929 / 20260930 / `full_prefix` | same |

Rows were scaled, not the token count: F1 spreads the *same* synthetic token
budget over eight times as many surface forms, so each individual form is seen
about 43 times against F0's ~395 repetitions of its single form.

F0 is the **existing** `wide-bind-1` arm (same init, settings and store, built
from the surviving `geometric-stack-9a80e47a`), not a fresh run: its synthetic
store is byte-identical to what `forms=1` produces here. F1 is a new run at the
same settings. **Confound:** F0 was trained at 8 threads (before the 2-thread
cap applied to this measurement) and F1 at 2, so float reduction order differs
between the arms. A 2-thread F0 replication is the natural check and is listed
under what remains unresolved.

A third, untrained-for-this-purpose control is scored on the same panel:
`chat-dose-continue-1/integer/model.lut`, the init both arms start from.

## Results

FILLED IN AT THE END

## What could not be resolved

FILLED IN AT THE END
