# Prep sheet: one read of the sealed `m2-fresh256` panel by the saved integer reader (m2-train-grounded-replies, claude-443, 21 Nov 2026)

Lab: claude. Machine: Casey's laptop, host CPU only. No GPU, no rental, no training on the panel.

## The question

The saved integer reader answers 512 of 512 on the M2 development panel `m2-reply512`. The last candidate that did well there (`saved437`, 437 of 512) scored 0 of 128 on a fresh draw. Does this reader carry over to values and histories it has never seen, on the sealed 256-row panel `m2-fresh256`?

Milestone: M2 (#2030). This is the registered `fresh-transfer-256` bar of [`m2-fresh256.json`](../panels/m2-fresh256.json), 0 of 128 today and never run.

What this sheet does not claim, whatever the result:

- It does not move the M2 headline by itself. The headline counts replies of "one accepted saved model"; accepting this reader as that model is a decision for the top session's decision review, with this result as its evidence.
- The reader is a host-side integer file. It is not yet the one persistent native state served through the shared train/serve operators with a bit-identical replay check (D85), and nothing here is exported to D11.
- TRAIN is 20,000 synthetic cases in the panel's published wording. A pass shows the reader reads a told value from the context under that wording. It shows nothing about open conversation.

## Base artifact

Two readers, one per TRAIN seed, both built by `conjunction-probe` at commit `3857560277eb48ce6d9a72cb91d31ad08c76edf1` (the reader code is unchanged since `31f3579aad73a20b1264f5521e50de3340f63afb`) with `reader-save TOKENS MASK OUT 4096 192 48 0 1 8 - 1 - pair open precise,across`. No model, trunk, donor or count table is loaded.

| reader | TRAIN seed | file bytes | reader sha256 | TRAIN tokens sha256 | TRAIN mask sha256 | `m2-reply512` |
|---|---|---|---|---|---|---|
| A | 1 | 35,574,650 | `44574adb84fd0c4cd30e8c182cf0f835b5d0348618b6b248b23c2c5742972c3c` | `bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24` | `89e0732b5858363e31a9729627e55e775107ababd4fda19ecc55e2aa9cb47e0f` | 512 of 512 |
| B | 2 | 36,412,674 | `078dcfdc05cfa8ab13d14120e46d9c253816cc84b1f4223f3d6cc9fe38810c2b` | `c5ecf61ce3f4a1c5b63768648bb361295d6d29b02bba28c3eb036d21bf47b9b6` | `225bd338e019981fcfa52d2194790f94eb2f95a062ef55e00e9b5387e4a291dd` | 512 of 512 |

On `m2-reply512` each reply is token-for-token the reference `m2-format` builds, and all 512 references decode to an answer the development labels accept (checked at commit f8198c2b5). Reader A built on wick and built on the laptop are the same bytes. The two readers give the same 512 development replies. `--grade` takes `--reader SHA256` and refuses a report whose `reader_sha256` is not that value; `reader-replies` writes the sha256 of the file it loaded into its report.

Grammar: [`reader-grammar.json`](../../labs/m2-fresh-heldout-panel/reader-grammar.json), sha256 `90429301dccbf2e1b9c1594fb9869c7b285e61f137beffe6008392e5a4ba32c4`. Tokenizer: the pinned `ByteBpeTokenizer` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

## Data split

- TRAIN (each seed): 20,000 cases drawn by `conjunction-probe m2-format reader-grammar.json <dev inputs> OUT 20000 SEED`. Each case is two statements from the four published statement openings, one of the twelve published questions, and a reply in the asked role's opening. A value is 1 to 12 token ids drawn from a pool of 2,534 whole-word tokens that no development case uses.
- The panel's 34 literals were not read to build TRAIN, the grammar, the reader or this sheet. Single pool tokens may also be words of a fresh literal: the pool is every lower-case whole-word token outside the development cases. A whole fresh literal of two or more tokens appearing as a TRAIN value has a chance near 40,000 values × (1/12) × (1/2,534)² per literal, about 5 in 10,000 for one seed and twice that for the two; it is not measured.
- The reader's code and its three read choices (`pair`, `open`, `precise,across`) were developed on `m2-reply512` and the M-world development panels p14 and p23 only. The opened fresh128 draw was not read by this reader: a run on it was refused on 20 Nov and not retried.
- `m2-fresh256` is read once, by this sheet's steps, and by nothing before them except the instrument's own pass-or-fail tests. Its `labels.json` is opened by no command here: the grade is computed inside the instrument's process from the panel it generates, whose four sha256 seals it checks first.
- Known mismatches between TRAIN and the panel, stated before the read: TRAIN values are at most 12 tokens and the panel's targets run to 19 tokens with the period; TRAIN values are whole-word tokens and a fresh literal may hold word pieces. On development, 160 of 512 references are over 14 tokens and all are answered.
- The panel's inputs carry the two current records of a history only: an overwritten or repeated earlier turn is absent for every candidate. A pass on the `update` and `reassert` rows is therefore not evidence that the reader handles an overwrite.

## Steps

All in a new folder `local/claude/m2-fresh256-read/` on the laptop, with the binaries built at the commit above.

1. Check both reader files' sha256 against the table. Then `cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked`: 8 tests pass. One requires every one of the 256 cases to be in the forms `m2-format` accepts (two statements in the grammar's published openings, a question the grammar holds under the asked role) and reports only pass or fail; it passed when this sheet was written, and nothing was changed in answer to it. Another decodes the grammar's reply openings before each of the 2,534 pool tokens and requires the text to start with the string `--grade` removes.
2. `cargo run --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked -- --emit-dir local/claude/m2-fresh256-read/sealed`; check `inputs.json` is 524,850 bytes with sha256 `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`.
3. `conjunction-probe m2-format docs/labs/m2-fresh-heldout-panel/reader-grammar.json local/claude/m2-fresh256-read/sealed/inputs.json local/claude/m2-fresh256-read/fmt 1 1`: writes the 256 prompts (`fmt/panel.json`). Its one TRAIN example is not used.
4. For reader A, then reader B: `conjunction-probe reader-replies READER fmt/panel.json replies-{A,B}.json 48`.
5. For each report: `cargo run --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked -- --grade replies-{A,B}.json --reader <that reader's sha256 from the table>`.
6. Record both grade outputs whole, the sha256 of both reports, and both readers' tokens per second from the reports, in the M2 notebook row and on #2030. The reports and grades are kept. `sealed/labels.json` is read by no step; the instrument can write it again at any time, so the seal is this sheet's rule, not the file's absence.

How a reply is graded. The panel's labels accept the bare value (`VALUE.`). The reader replies ` Your job is VALUE.` or ` You live in VALUE.`, the form the development labels accept. `--grade` removes the asked role's opening (` Your job is ` for a job question, ` You live in ` for a home question, the role read from the case id) from a reply that starts with it, encodes the rest again, and calls the registered grader `grade_generated_replies_with_evaluator_oracle` unchanged: a row is complete when the reply ends on EOS and the text is exactly the accepted answer. A reply without its role's opening is graded as it stands. The openings are fixed strings that name no value. The instrument's test shows this mapping scoring 0 of 256 for a reply with no EOS, with the other role's opening, or with the pair's other value.

## Bar (pre-registered)

- PASS: reader A and reader B each reach `pairs_both_complete_distinct` at or above 52 of 128 and complete rows at or above 104 of 256, the bar the owner registered on #2030 (comment 6350135678). The reported numbers are the lower pair count and the lower row count of the two readers.
- Otherwise the result is both readers' counts, and it is a miss for this reader configuration (these TRAIN draws, these read choices), not for held question forms or pointer reads in general.
- Controls, already measured through the same grader by the panel's instrument: copy the first stated value, copy the last stated value, the other role's value, and the best constant reply each score 0 of 128 pairs.
- Reported beside the bar and not part of it: complete rows, replies with no EOS, replies with no opening, `count.exact` from `reader-replies` (against a reference built from the prompt's own statement), and tokens per second.
- Before step 2 nothing of the panel exists outside the instrument's own tests. A failure in step 1 (a test fails, or a reader file's sha256 differs from the table) is repaired without changing any source file (a code change needs a new sheet and a new fitness read) and the run starts again under this sheet, with the grammar, TRAIN, both readers, the grading mapping and the bar as written here.
- From the start of step 2 the panel is spent for this reader line, whatever happens next. A step that stops, crashes or is refused from step 2 on gives no result for the reader concerned (both readers at the shared steps 2 and 3), which counts as 0 pairs and 0 rows, so the bar is missed; it is recorded as an instrument stop at that step, beside any count that was produced. No sheet reads this panel again with this reader line, with or without a repair. The result is fixed by the step 5 outputs; step 6 only records them and may be retried.
- Replies with no opening, no EOS or a wrong value are model results.

## Minimum test power

128 pairs in 16 blocks. The panel's own measurement puts the cluster-adjusted standard error at the bar at 4.32 pairs (8.64 rows), and every control at 0 pairs, so 52 pairs stands 12 standard errors above the controls. Two TRAIN seeds are read, as the panel entry asks for a decisive run. Both readers see the same 256 prompts, so the two counts are not independent samples of the panel; they bound the spread from the TRAIN draw only.

## Cost

About a minute of laptop CPU: two instrument builds, 256 prompts, two reply passes of well under a second each. No money, no GPU, no lease.

## Stop rule

One pass of steps 1 to 6. No second read of the panel by this reader line under any sheet: a change to the reader after this read is developed and judged on a newly drawn panel. Step 1 may be repeated after a repair. From the start of step 2 there is no repeat: a stop there or later is the result. Results are written to the new folder only; nothing is written into an earlier run's folder or into the instrument's files.

What each outcome unlocks. PASS: a decision card for the top session to accept this reader as M2's saved model for the development and fresh counts, and the next unit is the same read inside the persistent native state with a replay check (D85). MISS: the replies are read by cause (opening, EOS, value), the fault is fixed on development panels, and the next fresh read waits for a new panel.

Fitness reads. Read 1 (fresh-context reader, not the author, same lab): NOT_FIT, one blocker: nothing tied the reader's opening token ids to the strings the grader removes. Fixed by the grammar test in step 1 and the `--reader` check; the other notes of that read are written in above. The read did not reach CHARTER §4.5.

Read 2 (a second fresh-context reader): NOT_FIT, one blocker: after a stop at step 3 the author would have seen the panel's inputs and could refit the grammar or TRAIN before a second read. Fixed in the instrument-failure and stop rules above. Read 2 confirmed the read 1 fix and ran the instrument tests. CHARTER §4 item 5 asks for bar, noise, controls, seeds, cost and what each outcome decides; each has its section above.

Read 3 (a third fresh-context reader): NOT_FIT, two blockers: a tooling repair after a stop could still refit `m2-format` or the grading to the panel, and three failure cases had no class. Both answered by one rule: step 1 may be repaired and repeated, and from the start of step 2 the panel is spent and any stop is the result. The form check that could have stopped step 3 now runs as an instrument test in step 1.

Read 4 (a fourth fresh-context reader): FIT. It confirmed read 3's blockers closed and step 1 no loophole. Its four notes were written in after the read, each one narrowing a rule: a step 1 repair changes no source file; the split says the instrument's tests have already read the inputs; the result is fixed at step 5; "both readers" at the shared steps.

TEST FITNESS: FIT - read 4, a fresh-context reader that is not the author (same lab, claude); reads 1 to 3 were NOT_FIT and each blocker is fixed above. No other lab has read this sheet.
