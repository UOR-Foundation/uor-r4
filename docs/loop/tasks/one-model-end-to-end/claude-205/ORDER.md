# ORDER: one-model-end-to-end, claude-205

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Base: `d82294994`.

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
>    - **The pod** also runs `integer-hamming-read`'s step 5 parity. It is pinned to the read alone, with kept sets and first divergence, and the memory off on both sides.
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

Done-when 2, the overturn check. Done-when 1 (the census) is done (claude-202, claude-204). Done-when 3 (the protocol) and 4 follow in later units, so the item stays `ready`.

## Cancel probe (run first)

Run at `43818576a`, before any work. **Class: instrument (the scorer is missing). Go.**
- **No per-row scorer exists.** `git grep` for `score-rows`, `score_rows` and `fn overturn` in `crates/uor-r4-float-train/src` finds nothing. `nearest_reading` (`in_space_long.rs:728`) returns totals only, so paired discordant rows cannot be counted from any existing mode.
- **The checkpoints exist.** `uor-store ls` lists `step-016384/` for (m), (r) and (r0) at both seeds, under `results/claude/turn-memory-reply-2026-10-29/s{1,2}-fc66ed74/`, each with `model.safetensors`, `hamming_read.safetensors` and `turn_memory.safetensors`, and (r)'s 46 MB `turn_memory_reply.safetensors`.
- **No other unit holds it.** Open PRs: this one (#3411) and antigravity's #3323 (`m3-durable-memory-close`, `crates/uor-r4-discrete` and its docs), which shares no file. No prep sheet `one-model-end-to-end*` exists.
- **The memory moves the readout only, and only at rows its tables name** (`flock_read_memory.rs:1-25`). So on the decision rows (r) and (r) memory-off must pick the same token at every row. That becomes one of the sheet's instrument checks, not a finding.

## Files in scope

- `docs/labs/one-model-end-to-end/**`
- `docs/harness/prep/one-model-end-to-end*`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-float-train/src/flock_read_memory.rs`
- `crates/uor-r4-training/examples/m-world.rs`

This unit edits only the first five and the two `uor-r4-float-train` files.

## Done when

1. **The scorer** (GPU), `train-in-space --score-rows CHECKPOINT --out DIR`:
   - writes every scored row of DEV, and of Fresh V2 under `--fresh-v2`, as its corpus position, target, nearest pick and writer pick. One file is written with the checkpoint's memory as loaded, and one with it taken out;
   - writes greedy samples, nearest-decoded, of the reply after each of the first `--samples` sample prompts, with the memory on and off. Each holds prompt, reply and reference ids, and their text;
   - its per-row picks come from the same loop as `nearest_reading`, so their sums are its readings by construction.
2. **The grader** (laptop, counts only, over stored GPU outputs), `train-in-space --overturn 1 --scored NAME=DIR ...`:
   - recomputes the census's decision rows from TRAIN and DEV;
   - reports each arm's hits on them, on all of DEV and on Fresh V2;
   - reports the paired `b`, `c`, the band and the cell at each seed, and the verdict;
   - scores the samples' free-generation measures against TRAIN's replies: verbatim share, new n-grams for n = 4–8, distinct-1 and distinct-2, and the repeated 4-gram rate. DEV's own reference replies are scored the same way.

   Each rule has a unit test, and every new test passes.
3. **A prep sheet,** `docs/harness/prep/one-model-end-to-end-overturn-2026-10-29.md`, holds the arms, the rows, the band, the cells and the instrument checks. Its fresh FIT and pre-run review are posted on #3407 before any pod is rented.
4. **The run:** one RTX 5090 pod, the six step-16,384 checkpoints scored.
   - The output is sealed to `results/claude/one-model-end-to-end-2026-10-29/` with a `SHA256SUMS`, and graded on the laptop.
   - The pod is released and down.
5. **The REPORT,** negatives first, leading with tok/s where it has one. One notebook row.
6. **The owner log on #3407 for the parity split** (below), with its decision review.

## The scope call, recorded for decision review

Done-when 2 says the pod "also runs `integer-hamming-read`'s step 5 parity ... with kept sets and first divergence".
- **What this pod seals.** This unit's pod seals the float side of that parity with the memory off: (r)'s per-row writer and nearest picks on DEV, and its greedy samples.
- **What it does not seal: kept sets.** Capturing kept sets needs a hook in `flock_read_hamming.rs` (the float side) and in `crates/uor-r4-integer/src/stack/hamming.rs` (the integer side). Neither file is in this item's scope, and the first is in no item's.
- **Where kept sets go.** Kept sets go to `integer-hamming-read`'s own step 5 unit, which the float picks and samples sealed here make cheaper. That unit adds the hook, and a pod if its float kept sets need one.
- **Review.** This is a change to a done-when line. It goes ahead on this unit's decision review, logged on #3407 as `[claude] owner log:`.

## Stop and decide if

- **The readings do not reproduce.** The scored totals must equal claude-194's sealed last-checkpoint readings: (r) 11,513 / 11,361; (r) memory out 4,101 / 3,807; (r0) 10,079 / 11,171. If they do not, the instrument is repaired before any cell is read.
- **The picks differ on the decision rows.** If (r) and (r) memory-off differ on any decision row, classify it before reading the cell. Either the census's rows are not the memory's, or the memory reaches rows it does not name.
- **The pod costs more than $3,** or projects past 1 hour.
