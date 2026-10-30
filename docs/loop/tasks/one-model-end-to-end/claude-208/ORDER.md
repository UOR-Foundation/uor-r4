# ORDER: one-model-end-to-end, claude-208

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `4916350fe`.

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

The protocol's three scoring instruments (`docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`, "The instruments the run needs"):
- **I-A:** `--score-rows` on an unpinned panel (P-dev);
- **I-E:** the Flock rank decode;
- **I-F:** the table hit rate.

I-B and I-C are done (claude-207, `4916350fe`). I-D, I-G and I-H follow. The item stays `ready`.

## Cancel probe (run first)

Run at this ORDER's base. **Class: instrument (all three are missing). Go.**
- **`score_rows` cannot score P-dev.** It reads DEV and the pinned Fresh V2 only, and samples DEV only. `--panel NAME=DIR` exists for the census (`load_unpinned_uort`, claude-204).
- **No Flock rank decode exists.** `greedy_reply` takes the argmax. The harmonic table is `RAW_RANK_WEIGHTS_Q16` (`crates/uor-r4-integer/src/stack/flock.rs:382`). The integer crate's native root does not export `stack`, so the float side cannot import it.
- **No hit rate under generation exists.** The decoder forms each later step's key with `greedy_reply_key`. Nothing counts whether the table names it.

## Files in scope

The item's scope. This unit edits `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `docs/loop/tasks/one-model-end-to-end/claude-208/**`, and the protocol sheet's I-F row (an instrument-check correction, found in review).

## Done when

1. **I-A:** `--score-rows ... --panel NAME=DIR` scores and samples each unpinned panel like DEV, its sha recorded in the output. Its rows are `rows-{name}-{memory|no-memory}.json`.
2. **I-E:** a Flock rank decode (`DecodeRule::FlockRank`, `decode_reply`), exactly as the protocol specifies:
   - the 16 nearest table tokens, ranked by score with ties to the smaller id;
   - a rank drawn by `flock_rank_draw` with the harmonic weights `1/(i+1)` in Q16;
   - xorshift64*, seeded by the run seed and `sample · 64 + step`.

   The weights are a copy of `RAW_RANK_WEIGHTS_Q16`'s first 16, pinned to that file by a test. Greedy decoding is unchanged (`greedy_reply` calls `decode_reply` with `Greedy`).
3. **I-F:** each sample records its reply-table keys and hits, under generation and under teacher forcing on the true reply. The key is `greedy_reply_key`, the decoder's own, and a test pins it equal to the census's `reply_key` on a fixture's later rows. Every DEV later row is checked in I-G.
4. **Samples:** greedy with the memory on and off (DEV's file names unchanged), and Flock rank with the memory on, for DEV and each panel.
5. **Tests:**
   - a unit test per rule, with negative gates;
   - the full crate suite;
   - a stub-nvcc `--features cuda` check.

   The GPU halves run on the run's pod:
   - I-A's 16,417 P-dev rows;
   - I-E's "K = 1 equals greedy" on the model, which the unit test checks on the draw;
   - I-F's full count runs on the laptop, in I-G, over every DEV later row.
6. **A REPORT,** negatives first.

## Stop and decide if

- Greedy decoding or the existing files change. Both must stay as they are.
