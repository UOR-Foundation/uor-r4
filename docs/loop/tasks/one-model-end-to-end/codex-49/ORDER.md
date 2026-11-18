# ORDER: one-model-end-to-end, codex-49 (the decision rule, registered before the readings)

Lab: codex. Written **before** the 0.85-epoch checkpoint and the instruction panel are scored, so the
next move is not shaped by the result. The reading is two numbers the run already prints plus one
panel count; each outcome has a fix and a gate.

## The readings

1. **R** = `nearest / top-1` on the final checkpoint's donor DEV (capped 101,175 rows). Both are
   printed every evaluation. The narrow model gave 17×; the donor run at 1.42 epochs gave ~10×.
2. **P** = the panel's arithmetic stratum (the 196 frozen items, `boxed` + `number`) against its
   **copy-the-prompt control**, on the paired band `2·sqrt(b+c)`.
3. **N** = `nearest` against the split's unigram floor, so "both low" can be told from "both absent".

## The rule

| reading | diagnosis | next unit |
|---|---|---|
| **R ≥ 4 and P fails** | the representation carries the token; the **writer's admission** discards it | **W1** below |
| **R < 4 and P fails** | the table/trunk does not separate tokens yet | **W2** below |
| **N below the unigram floor** | the trunk has not learned this split | **W3** below |
| **P passes** | instruction following exists at this shape | scale the corpus, not the mechanism: the 1 B path, with **U2** first |

## W1 — the writer's admission (cheapest, do first)

1. **`kept_cells` 8 → 16** (the writer allows 1..16). It admits 8 of 256 cells = **3.1 %** of the
   8,192-token table; doubling that doubles the admitted set for almost no compute.
   *Gate:* admitted rises, `top-1` rises with the same table, the dev loss does not rise.
2. **The admission margin** (`margin_exp` and the score gap): at 3.3 % admitted rows the threshold is
   conservative. *Gate:* admitted rises, `top-1` follows, the loss holds.
3. **The identity partition** — `token-id mod 256` gives 256 cells of 32 slots at 8,192 tokens.
   Finer cells raise address resolution; it changes the served geometry, so it is last and needs its
   own artifact and parity. *Gate:* a new artifact, tokens/s, and the panel again.

Each of 1 and 2 is a config change and a **~1-hour run** at the current 0.30 s/step, not 20 hours.

## W2 — the representation (epochs, not data)

The token term was still falling at the last checkpoint of the short run (1.00197 → 0.91393 by 1.42
epochs). If `R < 4`, run the **continuation epochs** from the final checkpoint (`--init-from`), the
same shape, and read the same three numbers. No new data, no new mechanism.

## W3 — the trunk

If `nearest` itself is below the unigram floor on a held-out split, the shape is not learning. Then
the order is: the objective A/B (`nearest` vs `cell` at equal steps), then width/depth, then the
optimiser — measured one at a time, each against the same DEV.

## What is not allowed

- Choosing the threshold after seeing the result.
- Calling the panel "instruction following" if it only beats the empty control.
- Reaching for more data when the failure is admission (the narrow model's 17× gap is the standing
  evidence that admission can waste a working representation).
- Any dense or transformer comparison (D28).

## Where the numbers come from

`train-in-space` on `tokens-8k-clean` (223 shards, 452,523 conversations, 385,284,895 tokens),
`--width 512 --layers 12 --vocab-size 8192 --context 512 --objective cell --fast-rivals 1
--dev-tokens 200000`; the panel is `instr-panel-2026-11-17.json`, graded by `uor-chat` on the
exported integer artifact.
