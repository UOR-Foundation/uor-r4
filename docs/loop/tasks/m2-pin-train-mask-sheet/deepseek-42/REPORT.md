# REPORT: m2-pin-train-mask-sheet, deepseek-42 (is the Amendment 6 sheet live under D37?)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2851 (`References`; the item stays `ready` until the PM answer and a non-author FIT). Base: `8720c5d6d`.

## What did not work, first

- **The conflict is real and is not a lab call.** Amendment 7 froze the sheet under D32 (float ban on the learning path), but **D37** later revised that ban: floating point is legal in **offline training only**, with a reported export gap. Whether that revives the Amendment 6 sheet or still wants a new discrete-law sheet is exactly the item's done-when question, and it is the PM's or owner's to record - I did not answer it for them.
- **The four pins had to be re-verified, not re-quoted.** I checked the source commits exist (`2fd6358bb`, `9e904a834`), the CLI still carries `train_mask`/`expected_train_targets` (`geometric-stack-native.rs`), the mask digest `b5e7352d…` is the current code pin, and the 2,425,925 count is carried by the count-reference archive. The DEV pins are restated from the Amendment 4 record; the DEV tar itself is not in this checkout.
- **A refresh must not quietly become a re-registration.** The bars, reference, guards, dev policy and noise floor are restated by pointer to the old sheet's authoritative wording rather than rephrased; changing a bar would need a decision card. `TEST FITNESS: PENDING` is honest: the old sheet's FIT stays in its own record, and this page still needs a non-author FIT plus the PM answer.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` | the refreshed one-page sheet: D37 legal basis, four re-verified pins, unchanged bars/reference/guards/dev policy (by pointer), steps with the exact CLI, stop rule, cost, power, `TEST FITNESS: PENDING` |
| `docs/labs/m2-pin-train-mask-sheet/README.md` | what was verified, the decision card, and the open prerequisites |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 150 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- Pins verified by command: `git cat-file -t 2fd6358bb/9e904a834` -> commit; `expected_train_targets` and `train_mask` present in `crates/uor-r4-training/examples/geometric-stack-native.rs`; `b5e7352d…` in `crates/uor-r4-discrete/src/attention_learning.rs`; `2,425,925` in `archive/labs/count-reference-2026-10-13/README.md`.
- `prep-check` on the new sheet: completeness recorded below after the run; fitness `PENDING` by design.

## Checks run at the head

`prep-check`, `loop report --check`, `loop pr-check --local`, `loop gates --full`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `solution-integrity`: re-verified each pin instead of trusting the old sheet's wording; kept the bar wording by pointer so nothing changed by paraphrase.
- `scope-guard`: docs-only inside the item's scope; no bar, hold, spend or owner value was touched.

## Still open

- The PM's recorded live/superseded answer (decision card on #2851/#2028).
- A non-author FIT of the refreshed sheet (Claude/Kimi/antigravity; deepseek authored it).
- M2 training and rental remain HELD; a FIT releases nothing.

## Plain summary

- Verified the four pins on current `main`, wrote the one-page D37-basis refresh, and asked the PM the one question only the PM/owner can answer.
- Nothing was run; no bar, hold or spend changed; the sheet waits on the recorded answer plus a non-author FIT.