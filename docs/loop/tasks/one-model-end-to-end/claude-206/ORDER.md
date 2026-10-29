# ORDER: one-model-end-to-end, claude-206

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `4e3299bb9`.

## Goal

The item's done-when reads:

> 1. **The census (laptop, counts only, CHARTER §4.5).** A standalone Rust instrument under `docs/labs/one-model-end-to-end/instrument/` must:
>    - resolve the TRAIN size conflict, and build the TRAIN reply index (exact replies and reply 8- and 13-grams);
>    - report for DEV, Fresh V2 and `m2-fresh256`: the share of reply n-grams in TRAIN replies, the share of prompt templates seen in TRAIN with values masked, and the 8-token key share;
>    - read only the manifest of the K16 fresh slice, and leave the slice unopened.
>
>    It also says which panel, if any, holds new templates.
> 2. **The overturn check (GPU, under a prep sheet with a fresh FIT and a pre-run review).** Score turn-memory-reply claude-194's (r), (r0), and (r) with its memory off.
>    - **Pinned in the sheet:** the checkpoints by store path, `SHA256SUMS` and step; the binary's sha; a lease and cost inside the caps.
>    - **The pod** also seals a float reference for the read's parity: claude-194's (r), with its memory off, at both seeds. That is every DEV row's writer and nearest pick, and its 64 greedy samples. The parity itself, kept sets and top-1, is `integer-hamming-read`'s step 5, on the checkpoint that item's prep sheet names; the first greedy divergence is done-when 3's integer-artifact measure (claude-205's decision review, on #3407).
>    - **Rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged pooled (3,438).
>    - **Band, fixed in advance:** `max(2·√(b+c), the two-seed spread)`, where b and c are the paired discordant rows.
>    - **The cells:**
>      - (r) ahead at both seeds, beyond the band: the memory recipe stays the training default;
>      - (r) behind at both seeds, beyond the band: a memory-dropout arm joins the training run;
>      - mixed (the seeds disagree, or either seed is inside the band): both arms train.
>    - **Descriptive only:** Fresh V2, and 64 greedy samples per arm scored against the census index.
> 3. **The protocol.** A prep sheet, with a fresh FIT, written before the training run of the model to ship. It holds:
>    - **Strata:**
>      - keyed rows;
>      - unkeyed rows;
>      - new-template rows, from a panel no decision has touched. Fresh V2 is never one.
>    - **Arms:** (r), memory-off, (r0), tables only, and a longest-suffix table over TRAIN.
>    - **Scores:**
>      - teacher-forced top-1, per stratum;
>      - free generation: the share of replies that are verbatim TRAIN replies, new n-grams for n = 4–8, distinct-1 and distinct-2, the repeated-n-gram rate, and the table hit rate under generation against teacher forcing;
>      - on the integer artifact: kept-set agreement, first divergence, and tok/s against context.
>    - **Decode rules:** greedy, and a Flock rank rule with the harmonic weights of `stack/flock.rs`.
>    - **The verdict rule, written in advance:**
>      - "memorized": the gains appear only on keyed rows, and generated replies are mostly TRAIN replies;
>      - "generalizes": the model beats (r0) and the longest-suffix table on unkeyed and new-template rows, with novelty near DEV's own replies.
>
>    The training run itself (ASSESSMENT §6, step 5) adds:
>    - the transport controls: frozen and identity SpiralCore routes, under the fixed backward;
>    - the read-by-meaning arms: the Flock read, and the type spin.
> 4. **Lead with tokens per second** (decode against context). One notebook row per result; each REPORT negatives first.

## This unit

Done-when 3, the protocol: a prep sheet, `docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`, with a fresh FIT, written before the training run of the model to ship. Done-when 1 (claude-202, claude-204) and done-when 2 (claude-205, `4e3299bb9`) are done.

## Cancel probe (run first)

Run at this ORDER's base, before any work. **Class: instrument (no protocol exists). Go.**
- **No protocol sheet exists.** `ls docs/harness/prep/ | grep one-model` lists only the overturn sheet.
- **Step 2's verdict sets the arms.** claude-205's REPORT is MIXED, so the training run carries (r) and a memory-dropout arm as equal candidates (the overturn sheet's rule).
- **These tools exist:**
  - `--score-rows` and `--overturn` (claude-205);
  - the census and its decision rows (claude-202);
  - the P-dev panel, in the store at `inputs/claude/heldout-panels-2026-10-29/p-dev` (claude-204);
  - the `FbsArm` transport arms `IdentityTransport` and `FrozenQueries` (`spiralcore_fbs_arms.rs:29-34`);
  - the trainer's `--flock-read K` and `--type-spin 1`;
  - the harmonic table `RAW_RANK_WEIGHTS_Q16` (`stack/flock.rs:379`).
- **These tools do not exist:**
  - memory dropout;
  - a flag choosing the `FbsArm` (`in_space_long` sets `OneAtomActive`);
  - `--score-rows` on an unpinned panel;
  - the tables-only and longest-suffix controls;
  - a Flock rank decode;
  - the table hit rate under generation.

  The sheet names each as an instrument the run needs, with its check. This unit writes the sheet only.

## Files in scope

The item's scope. This unit edits `docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`, `docs/loop/tasks/one-model-end-to-end/claude-206/**` and the item file (done-when 3's line only, if the FIT asks).

## Done when

1. The protocol sheet holds:
   - the strata, the arms, the scores, the decode rules and the verdict rule, as the item's done-when 3 lists them;
   - the training run's added arms (transport controls; the Flock read and the type spin);
   - each instrument the run needs, with the check that it works;
   - the seeds, the band, the cost and the stop rule.
2. A fresh non-author FIT ("TEST FITNESS: FIT") is posted on #3407, and recorded in the sheet.
3. A REPORT, negatives first. The item stays `ready`: the instruments and the training run follow.

## Stop and decide if

- The FIT finds that a stratum or verdict cannot be decided with the data that exists. Then fix the sheet, or name the data unit first.
- The run's cost projects past the caps ($8/h across labs, 4 pods). Then cut arms, and record which.
