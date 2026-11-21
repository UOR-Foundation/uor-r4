# [codex] Ordinary job and home replies with the requested form supplied

Supplying only the correct job or home form recovered no complete replies on either panel. It removed every lexical first-token error on the 70 reached questions, but lost three previously exact replies per panel. All 21 job replies whose first error repeated a source occurrence still repeated it. The comparison that retained the other lexical candidates recovered six replies on the supplemental panel, all home answers referring to a fact in the current user turn.

## Ordinary client results

The primary panel attempted 96 two-turn conversations with novel first user texts. On 58 of those the actual first assistant answer reached the 48-token cap without EOS. The client correctly stopped there. The comparison uses the same genuine prompt and history for every arm on each reached query. The bridge is a separate panel, added after early reachability observations and before corrected outcome scoring. All 32 adjacent matched asks reached turn two, with the job in the previous user turn and the home fact restated in the query turn. We did not assign writer errors to unasked questions.

| Panel | Questions reached / attempted | Lexical exact through EOS | Supplied singleton exact | Promoted form exact |
|---|---:|---:|---:|---:|
| Both facts in prior turn | 20 / 64 | 4 | 2 | 2 |
| One requested fact in prior turn | 18 / 32 | 2 | 1 | 1 |
| Primary total | 38 / 96 | 6 | 3 | 3 |
| Job in prior turn, home in query turn | 32 / 32 | 3 | 0 | 9 |

Across the primary reached rows, lexical ended with EOS on 22, singleton on 9, and promotion on 9. On the bridge those counts were 20, 0 and 18. Exactness requires original-ID equality throughout the reference including its terminal EOS. Canonical answers use the tokens for “ Your job is <value>.” or “ You live in <value>.”. Matching the surface value alone did not earn exact credit.

| Compared with lexical | Error later | Error earlier | Same nonexact offset | Newly exact | Lost exact | Both exact |
|---|---:|---:|---:|---:|---:|---:|
| Primary singleton or promoted | 13 | 3 | 16 | 0 | 3 | 3 |
| Bridge singleton | 16 | 5 | 8 | 0 | 3 | 0 |
| Bridge promoted | 10 | 0 | 13 | 6 | 0 | 3 |

On the primary panel all 13 later singleton errors moved from offset 0 to 1. All three earlier errors and all three lost exact replies also first failed at offset 1. On the bridge the lexical baseline opened all 16 home queries with the wrong token. The singleton repaired that opener in all 16, but no singleton bridge reply emitted EOS. Promotion retained all three exact job replies and recovered six of 16 home replies, covering three distinct fact assignments under two wordings. Each of the six terminated with a suffix-ladder EOS. On the familiar wordings the singleton continued past the period. On the three paraphrases the singleton first failed at offset 1, while the promoted writer used another available form (ID 9) at that step. All 38 primary singleton and promoted replies were identical. [Primary row grades](evidence/primary-v2.json) · [Bridge row grades](evidence/bridge-v2.json)

## First repeated source

A repeat means the actual read source equals the entering cursor, with both concrete. Every one of the 13 primary and eight bridge repeats occurred at zero-based output offset 4 on a job query, and was the first wrong token. All three arms produced the same count. In the singleton traces 17 repeats were at prompt position 11 and one each at positions 24, 50, 51 and 52. Eighteen repeated occurrences were in the first user turn and three were inside the prior generated assistant reply. Advancing by one original position would yield the expected next ID in 19 of 21 cases. Two cases had already bound to a different occurrence with the wrong successor.

- On the bridge job query for **bronze willow**, output offset 3 copied token 564 from prompt position 11. At offset 4 the cursor was still 11, and the writer copied 564 from 11 again. The reference required 292, present at position 12. The produced text began “ Your job is br bronze willow.”
- On the dual-fact job query for **indigo harbor**, offset 4 repeated position 52 inside the genuine assistant history. It emitted 2874 in place of 313. Position 53 held 1566. This case would also need the original fact occurrence.
- On a paraphrased bridge job query, the lexical answer for **bronze willow** was correct through the period and then emitted 201 in place of EOS at offset 9. The singleton instead failed at offset 1, copying 377 (“do”) from query position 63 where the reference required 1541 (“job”). The known role alone did not preserve the correct writer slot.

The two native trace files contain every final-pass 23-word writer step, its source occurrence and entering cursor. When the writer retried an empty read, its public API retained only the last pass. [Primary steps](evidence/primary-v2/steps.jsonl) · [Bridge steps](evidence/bridge-v2/steps.jsonl)

## Controls and custody

The exact D11 SHA-256 is `2e16a8ea27b316cce01dac5744c6b02da38e0ee6c0abc3fcb610e381b1c85302`. The Wick product client, tokenizer, reader, byte-identical artifact and writer stayed fixed. Each reached baseline reproduced the actual client IDs including any early exhaustion. Full first assistant replies and EOS appear in the actual second prompt. The corpus-overlap check found zero complete first utterance matches (bare or with leading space) among all 128 first-turn cases in the original reader TRAIN. Familiar question wordings were intentionally reused. This is a finite ordinary integer serving observation.

The supplied singletons are job 1223 and home 1553, verified against original query and reply text, 14-word form identities and active writer reads. Each completed the correct canonical answer through EOS on both of its existing ordinary-ID controls; swapping them failed on every control. The second intervention prepended the same known role form and retained all other lexical candidates in relative order. The initial role-to-ID labeling was reversed; 38 preliminary outputs were quarantined without grading. The corrected executable refuses the inverted mapping before reading test rows. [Release and fitness evidence](FIT.md) · [Decoded controls](evidence/dev-semantics-v2.json)

Wick run root: `/mnt/plexpool/uor-r4-corpus/runs/20261121-codex-form-ordinary`. The graded outputs are `primary-v2` and `bridge-v2`. The initial `primary/` output is quarantined. The two captures are published as [primary](capture/records.jsonl) and [bridge](capture-bridge/records.jsonl) JSON Lines, each carrying the exact script, transcript and stderr text keyed by ID. Unpacking those strings to `scripts/ID.json`, `transcripts/ID.json` and `stderr/ID.txt` reproduced the two original checksum manifests on Wick. The grade files contain all original gold/reply IDs and the first differing expected/actual IDs, with null for missing output. Two independent checks agreed on every first-error offset and cursor-repeat index across 210 arm trajectories. These repeated fact/wording combinations are descriptive and do not change a milestone bar.

The next discriminating writer test is a source-occurrence intervention on job replies with the correct form still supplied. Compare advancing an already-correct occurrence with supplying the occurrence from the user fact and then advancing. Keep the original writer's stopping decision visible: getting the value right through the period can still be followed by text from genuine assistant history.
