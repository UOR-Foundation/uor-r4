# ORDER: one-model-end-to-end, codex-45 (the general-text run at the writer's ceiling)

Lab: codex. CHARTER §4.5 sheet for the first geometric run on real instruction data.

## What this run decides

Whether the geometric trunk — routed gather transport, Hamiltonian gate, native vector writer, no
softmax and no matmul — **learns real chat text at 30 M parameters**, and whether the address term
moves with it. It does not test the read (reads are off), so a flat address term here is the trunk's,
not the read's. It does not claim chat capability; it is the precondition for one.

## Configuration (all pinned)

| | |
|---|---|
| model | width 512, **12 layers**, vocabulary **8,192**, context 512, heads 4, MLP 384, Hamming 0, rotation quaternion, rec gates geometric, objective cell |
| tokenizer | `tokenizer-8192` `blake3:226a2016832c27ae7d3982911eb7063906f6cd61f4f0b9b9bef1cfc2905777ca` |
| TRAIN | `donor-8k` (shards 0–2): 5,738 conversations, 4,894,652 tokens, tokens `256edb61…`, mask `beefe4b2…` |
| DEV | `donor-dev` (shards 3–4, held out): 4,096 conversations, 3,120,930 tokens |
| steps | 16,384 ≈ 11.6 epochs of the 4.89 M-token train slice, or a larger train slice at 1–3 epochs |
| measured rate | ≈ 4,500 tokens/s → **≈ 20 h per 332 M-token epoch** at batch 8 × 512 |

**Before the long run**, the 2,000-step run (`gen-run`) must show the token term falling and the DEV
top-1 rising above the chance floor on the donor DEV. If it does not, the long run is not authorized
by this sheet.

## Bars

1. **Token term** on the donor DEV falls monotonically across the four evaluations of the monitored
   run and below its step-0 value by a margin larger than the run-to-run spread.
2. **DEV top-1** on the donor DEV rises above the unigram floor for that split, reported as a count
   against the split's own scored rows.
3. **Address term**: reported, not barred. It is the transport signal; a rise here with a falling
   token term is the known trade and must be named as such.
4. **Tokens/s** reported with hardware, as CHARTER §3.9 requires.

Bars are counts and paired differences, never a single composite. No dense/transformer control
(D28); the controls are the count tables the project already uses.

## What each outcome decides

| outcome | decision |
|---|---|
| token term falls, top-1 rises | proceed to the three-epoch run, then a held-out chat panel read as text |
| token term falls, address rises sharply | run with the read on (`--hamming-read 4`) and compare the address term, read vs no read, on the same DEV |
| token term flat | the trunk or the writer is the limit at this shape; measure the writer's admission against a count table before any more data |
| both flat and tokens/s far below 4,500 | the shape is not trainable on this device; report and stop |

## Cancel

Stop and report if the monitored run's DEV does not move (no long run), or if any measurement is
taken on the pinned narrow `ft/dev` by mistake (it is a different tokenizer and world).

## Out of scope

The read's write scale and the other-fact ceiling (claude's line, claude-395/397); a 32k vocabulary
(the writer's identity partition caps at 8,192 — codex-43); any dense or transformer comparison.
