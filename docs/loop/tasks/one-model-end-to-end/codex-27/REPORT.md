# REPORT: one-model-end-to-end, codex-27 (the first reply token, traced)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-26](../codex-26/REPORT.md). Owner direction: the $0 measurement before any run.

## What the measurement is

`fbs-toks --forced` (with `--decision-trace`) then `--hamming-trace`, on a reachable
`uor-r4.lut-stack/10` artifact (arm Q, q-s2, 56,397,696 B) and one real memory episode drawn from
the next02 corpus (conv 132 @ 17916, tokenizer `d36d3e87…`):

```
User: My code word is sermor.  Assistant: Okay, sermor is the code word.
…  User: What is the code word?
System: Memory: The code word is sermor.
Assistant:
```

The value's first token `' ser'` is at pos 9 (assert), pos 22 (the ack echo) and pos 126 (the memory
line). The artifact has four heads at depths 2 and 6, kept 16, copy-latest; **no ask, carry or place
sections**, so heads h0/h1 are copy heads and h2/h3 are type heads asking from the current token.

## The trace at the first reply read (row 135, the token `':'`)

| Read | Head | What it keeps |
|---|---|---|
| d2 | copy h0/h1 | **121 `' The'` at 1.000** — the format bigram |
| d2 | ask h2 | 122:0.318, 73/76/92/95:0.289, **9:0.261** (the assert, 6th of 16) |
| d2 | ask h3 | 54:0.364, 115:0.335, …, 125:0.219 |
| d6 (readout) | copy h0/h1 | 121 `' The'` at 1.000 |
| d6 (readout) | ask h2 | 59/76/95:0.357, …, **22:0.217** (the ack echo, weakest of 16) |
| d6 (readout) | ask h3 | 3/35/67/103:0.414, …, 125:0.247 |

- **No head at the readout keeps the assert position 9.** The only readout copy of the value's first
  token is pos 22, at the weakest weight of its 16.
- **Pos 126 — the memory line, the occurrence the model actually uses — is kept by no head at this
  row.**
- The pick is `287 ' The'`; the reply opens "The code word is sermor."
- Four rows later (`' is'`), both copy heads keep **126** at full weight and pick `' ser'`. The value
  arrives there, from the memory line, through the copy chain.
- The memory lookup never ran (`memory: null` at every step): the answer is the Hamming copy chain.

## What it means

**The first reply token is a format bigram, not a choice.** `Assistant:` continues as `' The'` in
TRAIN, and the copy heads stamp it at full weight; the value is deferred to the next rows, where the
copy chain finds it — in this episode, from the memory line.

That explains the item's anchors without a new mechanism: where the reply is a bare value ("Pickle"),
there is no "The job is" preamble to hide behind, the format prior takes the first token, and without
a restated line the copy chain has nothing near the answer. Where a line exists (arm r), recall is
0.75–0.78. It also explains arm rt's "47 → 40": the later write overwrote the copied value.

## The falsifier, and whether it passed

The order of work said: if no head keeps the asked value, arms (a) and (b) are dead. **The ask head at
depth 2 does keep the assert position (0.261), but the readout depth does not.** So the query path
exists and is reachable — it is the **readout-depth key and its weight** that fail, not the presence
of a query. That is a narrower, cheaper target than a new mechanism:

1. make the ask head reach the assert position **at the readout depth** — the strongest form is the
   relative-offset bigram of the question (unit 18);
2. make that keep **beat the format copy** at the first token — copy precedence is the wrong lever
   here (the format copy is *correct* once the value has been placed); the lever is a slot/place key
   that speaks the value at the reply's first row (unit 19), which is what arm r's line did by hand.
3. keep latest-wins so a later write cannot clobber it (the "47 → 40" fix).

## Caveats, stated

- The artifact is **arm Q, not arm r** (arm r's `/7` is unreachable without a GPU export).
- The D11 readout file carries tokens only (`fbs-snapshot` needs a GPU); the integer trace is the
  artifact's own, and its row-135 kept sets match the decision trace exactly.
- `--hamming-trace` does not call `mark_reply_start`, so rows past 135 are not the served path; row
  135 is unaffected.
- One episode. It diagnoses; it does not measure a rate.

## Checks run

- `cargo build --release -p uor-r4-integer --bin fbs-toks --features serving-wasm`.
- The two commands above, both exit 0; outputs under `/tmp/opencode/first-token-trace/`.
