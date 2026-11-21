# [codex] One cursor advance repairs one token and recovers zero exact replies

On Wick, the current-source intervention moved the first wrong token one place later on all 24 job questions where the retained pass repeated a prompt source. The supplied-original-source intervention did so on 14 of those 24. Neither recovered or lost an exact reply. Of 44 reached job questions, one was already exact through EOS under form promotion, and stayed exact under both interventions. The two panels have different exposure and reachability.

| Ordinary conversation | Questions reached / attempted | Exact through EOS, promoted | Current + 1 | Original start + 1 |
|---|---:|---:|---:|---:|
| Both facts in entirely new first turn, job asked | 0 / 64 | — | — | — |
| Job in new first turn, home in question turn, job asked | 32 / 32 | 0 | 0 | 0 |
| Same bridge, home control | 16 / 16 | 0 | 0 | 0 |
| Historical dual-fact first turn, new job question | 12 / 12 | 1 | 1 | 1 |
| Historical dual-fact first turn, new home control | 4 / 4 | 1 | 1 | 1 |

## Paired first errors

Offsets count from zero in the generated reply, including its leading space and terminal EOS as tokenized. All reached questions start after an ordinary assistant reply that naturally emitted EOS. An unreached question has no reply to score. The second-turn capture includes max-token replies.

| Job questions | Form promotion | Advance current occurrence | Advance original first-user value start |
|---|---|---|---|
| 16 fresh bridge, familiar question | 16 first wrong at 4 | 16 move 4 → 5 | 14 move 4 → 5, two stay at 4 |
| 16 fresh bridge, paraphrased question | Four first wrong at 5; 12 first wrong at the required EOS (9–13) | Same 16 replies | Same 16 replies |
| Eight supplemental, two prefaced familiar questions × four first turns | Eight first wrong at 3 | Eight move 3 → 4 | Eight stay at 3 |
| Four supplemental, new job wording | Two wrong at 4, one at 8, one exact | Same four replies | Same four replies |

All 20 home-control outputs were identical across arms by construction. Sixteen of 16 fresh home replies copied the whole answer through its period and then continued. Among all 64 reached questions, **each arm had 2 exact replies through EOS**, both in the enriched panel. Merely emitting EOS gives different counts. Among the 32 fresh bridge job replies, 12 promoted, zero current, and two original replies emitted EOS. Among the 12 supplemental job replies those numbers were 10, 10 and 12. Ending an incorrect answer earned zero exact credit.

## Where the writer loses the value

* **Repeated start in the first user fact.** For “cinnamon orchard,” the baseline copied source 11/token 280 at offset 3 and repeated it at offset 4. Both interventions read source 12/token 272 at offset 4, exactly as required. The next native step entered with cursor 12 and selected source 11/token 280. At offset 5 the reference required token 3750. All 16 fresh current-source improvements followed the same address sequence, 11 → 12 → 11. The substituted cursor survived into the next step; the next source selection sent it back.
* **A standalone space changes the starting address.** For “vermilion glacier” and “verdant telescope,” prompt position 11 held space token 223. Position 12 held token 322, the first letters of the value. At offset 4, current + 1 produced the required 322. The supplied span correctly began at 12, so original start + 1 selected position 13 and skipped 322. This explains both differences between the arms on the fresh panel.
* **Repetition before the value starts.** For “Just checking. What is my current job?” with original value “indigo harbor,” the promoted writer repeated token 435 (“ is”) from position 51 in genuine assistant history at offset 3. The expected token was 2874, the start of “indigo.” Current + 1 read position 52/token 2874, then returned to 51/token 435 at offset 4, which required 313. Original start + 1 read position 25/token 313 at offset 3, one token too early. Among the eight supplemental interventions, six repeats came from assistant history and two from the first user turn. None of the 24 retained interventions produced equal replacement IDs at different addresses between the two arms; that condition was covered by the finite instrument control only.

## Evidence and scope

The sealed D11 was unchanged throughout. Its SHA-256 is `2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302`. Promotion placed the supplied job form 1223 or home form 1553 first in the same lexical candidate list. Only job replies received a one-shot source/token substitution on the first repeated concrete prompt occurrence in a pass. Native generation then continued autoregressively with the normal retry and 48-token limit. Dispatch supplied the actual prompt, role and original value start. It contained no reference reply, span end or stop label. Its ordinary client reply served only as a replay assertion. Every reached baseline matched the separately restored stock observer in final reply, retry and every emitted 23-word step. Every lexical reply matched the original client. The applied intervention counts on retained passes were 16 and eight respectively. All model execution and builds were on Wick. These measurements describe the frozen ordinary integer serving path.

All 64 first turns with two new facts hit the 48-token limit. That failure prevents any claim about cursor behavior on their intended second turns. The supplemental design was chosen from this reachability result before grading the fresh interventions. Its manifest was sealed before its own client capture and interventions, about two seconds after the first panel's grade file was written. It reuses four previously published, deliberately reachable first turns, including a historical wrong-source case, and varies their second turns. The first assistant replies in this panel exactly reproduced their historical IDs. The capture lane did not use fresh intervention predictions to choose these prompts. Paired wordings and repeated fact families are descriptive cases, not independent estimates of open-ended chat quality.

A preflight fixture caught an initial trace hook that missed concrete reads selected via a pointer. The corrected hook passed the historical planted divergence and all finite tests before fresh interventions. During capture-tool setup the old stock trace executable was briefly overwritten. It was restored to its previously published byte checksum before independent replay. The source check also explicitly handles an independently encoded separating space.

[Ordinary capture and scripts](capture/README.md) · [Enriched capture](supplemental/README.md) · [Independent FIT](FIT.md) · [Source and build recipe](../../../../labs/question-causal-state/cursor-advance/README.md) · [Fresh per-reply grades](evidence/grades-primary.json) · [Supplemental per-reply grades](evidence/grades-supplement.json) · [Fresh native steps](evidence/steps-primary.jsonl) · [Supplemental native steps](evidence/steps-supplement.jsonl) · [Run pins](evidence/wick.sha256)

## Next discriminating test

Keep form promotion and D11 fixed. Test a persistent original-source cursor that distinguishes entering a value from continuing it. At entry use the appropriate first output token, including a standalone space where the protocol requires it. During the copy, advance from the last correctly emitted source token on successive steps. Trace whether source selection returns to the start and whether the writer stops after the period. Keep EOS logic fixed first: the 12 fresh paraphrased job replies which already match through punctuation give a separate stopping test. The 64 early first-turn stops are an independent ordinary-conversation limitation.
