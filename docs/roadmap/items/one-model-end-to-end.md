---
id: one-model-end-to-end
title: Score one model end to end on prompts it has not seen, with tokens per second
milestone: research
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [turn-memory-reply]
notebook: docs/research/notebook/float-training.md
scope: [docs/labs/one-model-end-to-end/**, docs/harness/prep/one-model-end-to-end*, docs/loop/tasks/one-model-end-to-end/**, docs/research/notebook/float-training.md, docs/roadmap/items/one-model-end-to-end.md, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, crates/uor-r4-float-train/src/flock_read_memory.rs, crates/uor-r4-float-train/src/flock_read_hamming.rs, crates/uor-r4-training/examples/m-world.rs]
---

## Why it matters

This item is the last part of the open owner direction's order: "`turn-memory-reply` first, then the integer serving of the parts, R7, and one model end to end" (`PLAN.md:86`). Casey added on 29 Oct (about 20:05Z) that it be done "in a smart way that considers the whole project and outside research we can draw from to see what exactly we are probably dealing with instead of making quick (and wrong) decisions".

[The assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) is that reading. A fresh decision review passed it in its third round (owner log on #2028, `6507593846`). Its findings:
- **The replies are templated, and no panel holds new templates.**
  - A reply-only control, keyed on the reply's own last tokens, already gets 8,089 of DEV's later rows. The chosen key gets 8,822 / 9,655.
  - The sealed Fresh V2 panel comes from the same generator. The bigram table T2 scores 6,582 / 14,479 there, against 6,300 / 14,287 on DEV.
- **The memory gives a real gain inside the templates.** Openings are 598.7 / 598.3 with it, against 266.3 / 281.0 without, under a ceiling of 607.
- **The trunk now leans on it.** Without its tables, turn-memory-reply's (r) falls to 4,101 / 3,807, below the empty-table control (r0)'s 10,079 / 11,171.
- **Generated text has never been scored.**

So before the model the project ships is trained, the scoring must be able to tell recall of a template from reading the prompt.

## What is missing

- **A census of the data:**
  - the TRAIN size conflict: 7,512,797 tokens (`hamilton_sheet.rs:81-82`) against 26,214,400 for the same `tokens.u16` sha (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:62-65`);
  - a TRAIN reply index;
  - which panel, if any, holds templates TRAIN never shows.
- **The check that could overturn the diagnosis:** whether (r) falls below (r0) where no key matches.
- **A panel with held-out phrasings** (claude-202's census, `443a980e5`). Neither DEV nor Fresh V2 qualifies: 0.750 and 0.777 of their replies are verbatim TRAIN replies.
  - M-world v2 already splits phrasings: `dev_phrasing × dev_value` is its development cell (`crates/uor-r4-training/src/milestone_world_v2.rs`, "Cross cells").
    - **The split covers user-turn phrasings and fillers only.** The assistant's reply templates are shared across splits. They include `MQAR_ACKS`, `MQAR_REPLIES`, `UPDATE_ACKS` and v1's closers and absent replies.
    - So the panel tests held-out *prompts* with new values, not held-out reply wording. Its census's masked reply-template share is expected to stay high.
    - **Two risks the panel's census must count:** a TRAIN paraphrase near-equal to a dev template (the generator refuses exact equality only), and chat-v0 turns that coincide with dev phrasings. It also records the `paraphrases=` setting TRAIN was built with.
  - But `m-world corpus` writes TRAIN conversations only. The writer is a `cell=` option on `corpus world=v2` in `crates/uor-r4-training/examples/m-world.rs`, added to this item's scope by claude-203.
  - The panel it writes is counted by `train-in-space --census` before it is used.
- **A scoring protocol, fixed before any training run, with:**
  - strata: keyed, unkeyed, new-template;
  - free-generation measures;
  - decode rules: greedy against a Flock rank rule;
  - integer-artifact measures.

## Done when

1. **The census (laptop, counts only, CHARTER §4.5).** A standalone Rust instrument under `docs/labs/one-model-end-to-end/instrument/` must:
   - resolve the TRAIN size conflict, and build the TRAIN reply index (exact replies and reply 8- and 13-grams);
   - report for DEV, Fresh V2 and `m2-fresh256`: the share of reply n-grams in TRAIN replies, the share of prompt templates seen in TRAIN with values masked, and the 8-token key share;
   - read only the manifest of the K16 fresh slice, and leave the slice unopened.

   It also says which panel, if any, holds new templates.
2. **The overturn check (GPU, under a prep sheet with a fresh FIT and a pre-run review).** Score turn-memory-reply claude-194's (r), (r0), and (r) with its memory off.
   - **Pinned in the sheet:** the checkpoints by store path, `SHA256SUMS` and step; the binary's sha; a lease and cost inside the caps.
   - **The pod** also seals a float reference for the read's parity: claude-194's (r), with its memory off, at both seeds. That is every DEV row's writer and nearest pick, and its 64 greedy samples. The parity itself, kept sets and top-1, is `integer-hamming-read`'s step 5, on the checkpoint that item's prep sheet names; the first greedy divergence is done-when 3's integer-artifact measure (claude-205's decision review, on #3407).
   - **Rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged pooled (3,438).
   - **Band, fixed in advance:** `max(2·√(b+c), the two-seed spread)`, where b and c are the paired discordant rows.
   - **The cells:**
     - (r) ahead at both seeds, beyond the band: the memory recipe stays the training default;
     - (r) behind at both seeds, beyond the band: a memory-dropout arm joins the training run;
     - mixed (the seeds disagree, or either seed is inside the band): both arms train.
   - **Descriptive only:** Fresh V2, and 64 greedy samples per arm scored against the census index.
3. **The protocol.** A prep sheet, with a fresh FIT, written before the training run of the model to ship. It holds:
   - **Strata:**
     - keyed rows;
     - unkeyed rows;
     - new-template rows, from a panel no decision has touched. Fresh V2 is never one.
   - **Arms:** (r), memory-off, (r0), tables only, and a longest-suffix table over TRAIN.
   - **Scores:**
     - teacher-forced top-1, per stratum;
     - free generation: the share of replies that are verbatim TRAIN replies, new n-grams for n = 4–8, distinct-1 and distinct-2, the repeated-n-gram rate, and the table hit rate under generation against teacher forcing;
     - on the integer artifact: kept-set agreement, first divergence, and tok/s against context.
   - **Decode rules:** greedy, and a Flock rank rule with the harmonic weights of `stack/flock.rs`.
   - **The verdict rule, written in advance:**
     - "memorized": the gains appear only on keyed rows, and generated replies are mostly TRAIN replies;
     - "generalizes": the model beats (r0) and the longest-suffix table on unkeyed and new-template rows, with novelty near DEV's own replies.

   The training run itself (ASSESSMENT §6, step 5) adds:
   - the transport controls: frozen and identity SpiralCore routes, under the fixed backward;
   - the read-by-meaning arms: the Flock read, and the type spin.
4. **Lead with tokens per second** (decode against context). One notebook row per result; each REPORT negatives first.

## How to start

Read [the assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md) sections 1, 5 and 6. Then read:
- `docs/research/notebook/float-training.md` rows 14–15;
- `docs/loop/tasks/turn-memory-reply/claude-194/REPORT.md`;
- the fresh-transfer instrument in `docs/labs/m1-d11-artifact-scorer/instrument/` (`fresh_rows_from_corpus`, `build_train_count_tables`), to reuse rather than rebuild.

## Rules that apply

- **Deterministic scores only:** count tables are lawful controls, and there are no neural or transformer baselines (CHARTER §3.10, §4.4).
- **No bits per byte without a lawful likelihood** (§4.8). A rank code is a diagnostic only.
- **Model scoring on the GPU; counting the data on the laptop** (§3.5).
- **Changing the M1 bar is the owner's** (§4.8, D39). The question is on #2028 (`6507594237`).

## Sources

- [The assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md), with its outside literature: kNN-LM in generation arXiv:2305.14625; RETRO arXiv:2112.04426; Engram arXiv:2601.07372; RAVEN arXiv:2111.09509; degeneration arXiv:1904.09751.
- [float-training notebook](../../research/notebook/float-training.md), rows 14–15; [turn-memory-reply claude-194](../../loop/tasks/turn-memory-reply/claude-194/REPORT.md); [the Fresh V2 sheet](../../harness/prep/m1-fresh-panel-transfer-2026-10-26.md).
