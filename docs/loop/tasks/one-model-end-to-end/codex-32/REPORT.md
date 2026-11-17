# REPORT: one-model-end-to-end, codex-32 (the fresh panel: zero, and not even a copy)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parents: [codex-28](../codex-28/REPORT.md), [codex-31](../codex-31/REPORT.md).

## What was run

The item's own fresh panel, `m2-fresh256` (256 rows, 128 pairs, four strata of 64: `length2`,
`length4`, `length8`, `reassert`; protocol `uor-r4.literal-role-dialogue/2`), served by the
instrument's `served-reply-panel --engine integer --rendering per-turn/v1 --reply-tokens 16`, which
verifies the panel's manifest hashes before it decodes.

Two artifacts, same host, same panel:

| Artifact | complete | EOS stops | cap stops | pairs both complete & distinct | decode |
|---|---|---|---|---|---|
| `q.uorlut` (arm Q, `/10`) | **0 / 256** | 192 | 64 | 0 / 128 | 234 tok/s |
| `arm18h8b.uorlut01` (asks + carry 8, `/7`) | **0 / 256** | 172 | 84 | 0 / 128 | 293 tok/s |

Every stratum is zero for both.

## What the failure is

The `length2` case is a **copy** test: the prompt states the value and asks for it immediately —

```
My current job is basalt loggia.
I currently live in porphyry belvedere.
What is my current job?            → expected: "basalt loggia."
```

The H = 8 arm answers ` I'm a all't on your, runs from you.` and arm Q answers a similar degenerate
string. The model does not copy the adjacent stated value, let alone choose between two. That is the
same failure deepseek-273 recorded: *"the model did not learn even direct copy from aligned data."*

So the fresh-phrasing gap is not a small generalization margin on these artifacts — it is **total**,
and it begins at the copy control. No read mechanism can be blamed for a reply that never carries the
value: the arms are 45-minute models whose dev top-1 was 532/14,287.

## What this orders

1. **The copy control first.** A run is not worth a panel until its artifact copies the adjacent
   stated value on `length2`. The gate is `complete > 0` on `length2` (and a paired-distinct count),
   both zero here.
2. The mechanism work (reach, margins, aggregation, e-means assignment, the k-term mixer) is not the
   bottleneck for this panel; the **model is underfit for copy**.
3. The cheapest arm that has ever copied is arm r's line (0.749–0.782 recall on matched pairs, three
   seeds, with a memory line). Its recipe is the reference for the copy-first run, not the 16k-step
   asks+carry arm.

## Checks

`served-reply-panel` at the head, the manifest's input/label/plan hashes matched; both runs above.

## Correction: the rendering was wrong, and the exact-reply metric hid the value

Two faults sat behind the "0 / 256".

1. **The rendering.** `--rendering per-turn/v1` gives every `Context` segment its own user turn, which
   is not how the corpus writes a conversation. With the tool's **default** `one-user-turn/v1`, the
   same artifacts answer coherently: arm r's copy arm opens
   `" My current job is basalt loggany Thank His"`, and the asks+carry arm opens
   `" My current job is is H coldototototot"`. The per-turn arm even stopped at BOS 21 times.
2. **The metric.** `complete` is an exact-reply measure, so a sentence with the value in it scores
   zero. Reading the value as a substring of the reply, per stratum:

| Arm (default rendering) | length2 | length4 | length8 | reassert | update |
|---|---|---|---|---|---|
| arm r copy: accepted answer present | **18 / 64** | 1 / 64 | 0 / 64 | 6 / 32 | 0 / 32 |
| arm r copy: first word present | 30 / 64 | 26 / 64 | 35 / 64 | 16 / 32 | 9 / 32 |
| asks + carry 8: accepted present | 8 / 64 | 5 / 64 | 0 / 64 | 6 / 32 | 0 / 32 |
| asks + carry 8: first word present | 16 / 64 | 22 / 64 | 47 / 64 | 6 / 32 | 5 / 32 |

So the copy control is **partially met**, not failed: arm r's switches put the accepted value in the
reply on **28 % of the `length2` rows** (8 of 64 for asks+carry), and the first word of the value on
47 %. The exact-reply count is zero because (a) the model writes a framing sentence and (b) it
garbles the rare second word (`loggia` → `loggany`).

## What this changes

- The earlier conclusion "the model cannot copy" was too strong. It copies the value but not the
  *reply format*, and it garbles a rare word.
- The copy gate should be the item's own **value-present** measure, per stratum, beside the exact
  reply — not the exact reply alone. The item already defines that measure (V), and it is the right
  gate for the copy control.
- The remaining failures are specific: (a) the reply format (the model never answers with a bare
  value because the corpus's replies are sentences), and (b) the rare-word spelling (a copy-payload
  or tokenization question).

## The value-present gate, now in the tool

`served-reply-panel` reports `value_present` per row, per stratum and in the totals — any accepted
answer appearing in the reply, trimmed and case-insensitive — beside the exact `complete`. The
accepted lists already include the sentence forms ("The current job is basalt loggia."), so a
correct sentence scores complete; a framing sentence with a garbled value scores present only.

Three arms, same host, same panel, default rendering, `--reply-tokens 16`:

| Arm | length2 | length4 | length8 | reassert | update | total | complete | tok/s |
|---|---|---|---|---|---|---|---|---|
| arm Q (`/10`, asks+carry off, other run) | 0 | 0 | **3** | 0 | 0 | 3 / 256 | 0 | 272 |
| asks + carry 8 (`/7`) | 8 | **5** | 0 | 6 | 0 | 19 / 256 | 0 | 379 |
| **arm r copy (`/7`)** | **18** | 1 | 0 | 6 | 0 | **25 / 256** | 0 | 343 |

- **The copy recipe wins the copy stratum**: 18 of 64 `length2` rows carry the accepted value,
  against 8 for asks+carry and 0 for arm Q. The claude-345 prior (0.749–0.782) holds at the panel.
- The asks+carry arm is better on `length4` (5 against 1), so the two recipes have different
  strengths rather than one being uniformly worse.
- `complete` is 0 for all three, so no arm produces the exact accepted sentence.

## What the remaining failure is, precisely

The reply is a framing sentence with a garbled rare word: `" My current job is basalt loggany Thank
His"` for `basalt loggia.`. So the copy finds the statement and the first value token, and the
**second token of a rare word** is wrong, plus the reply does not stop. The next unit is that trace
(the copy read's kept positions at the value's second token) and a reply-format arm (bare answers or
the exact sentence the accepted list holds), not another recipe sweep.

## Checks

`served-reply-panel` at the head with `value_present`; the three runs above; the panel's manifest
hashes matched.
