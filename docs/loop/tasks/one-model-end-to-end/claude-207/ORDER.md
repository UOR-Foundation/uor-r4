# ORDER: one-model-end-to-end, claude-207

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `6baa1c835`.

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

The protocol's two training instruments (`docs/harness/prep/one-model-end-to-end-protocol-2026-10-29.md`, "The instruments the run needs"):
- **I-B, memory dropout.**
- **I-C, the transport arm as a flag,** for training and for the scoring the protocol uses.

The other instruments (I-A, I-D to I-H) are later units. The item stays `ready`.

## Cancel probe (run first)

Run at this ORDER's base, before any work. **Class: instrument (both are missing). Go.**
- **Memory dropout does not exist:** `git grep -n -i dropout` in `crates/uor-r4-float-train/src` finds nothing.
- **The arm is hard-set.** `run_long` sets `FbsArm::OneAtomActive` and builds with it. So do the scorers (`decode_speed`, `inspect_run`, `score_rows`). The trainer has no flag for it.
- **The frozen arm needs an optimizer filter.** The registered sheet's runner keeps the routing queries (`.spiralcore_fbs`) out of the optimizer under `FrozenQueries` (`spiralcore_fbs_sheet.rs:497`, `:717`). `run_long`'s main optimizer filters only the decay rates.

## Files in scope

The item's scope. This unit edits `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs` and `docs/loop/tasks/one-model-end-to-end/claude-207/**`.

## Done when

1. **`--turn-memory-dropout P`** (thousandths, 0–1000, needs `--turn-memory`):
   - each training window is dropped by a seeded draw from its own stream (`memory_dropped`), and names no opening or reply row;
   - at P = 0 the batch order, the rows named and `config.json` are unchanged;
   - scoring keeps the memory at every row.
2. **`--fbs-arm one-atom|identity|frozen`:**
   - `run_long` sets the hook, builds the model and filters the main optimizer by the arm. Under `frozen`, no routing query is updated (`main_optimizer_updates`);
   - `--score-rows`, `--inspect` and `--decode-speed` take the arm, and refuse one other than the run's own `config.json` names (`check_run_arm`);
   - the default writes the same `config.json` as before.
3. **Tests:**
   - a unit test for each rule, every one passing, and a negative gate for the frozen filter;
   - the crate's full test suite;
   - a stub-nvcc `cargo check --features cuda`.
4. **The GPU halves of the protocol's checks are named as the run's probe-ladder steps.** These are I-B's "byte-identical at P = 0", I-B's "the reply weight stays 0.0 at P = 1", and I-C's "frozen queries equal their seed after 256 steps". They need CUDA and data, so they run on the pod before the arms.
5. **A REPORT,** negatives first.

## Stop and decide if

- A change would alter the default path's training or scoring. The default must stay byte-identical.
