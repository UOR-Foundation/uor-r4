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
