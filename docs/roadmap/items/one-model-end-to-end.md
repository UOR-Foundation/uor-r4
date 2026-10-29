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
scope: [docs/labs/one-model-end-to-end/**, docs/harness/prep/one-model-end-to-end*, docs/loop/tasks/one-model-end-to-end/**, docs/research/notebook/float-training.md, docs/roadmap/items/one-model-end-to-end.md, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs]
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
2. **The overturn check (GPU, under a prep sheet with a fresh FIT and a pre-run review).** Score turn-memory-reply claude-194's (r), (r0), and (r) with its memory off. The checkpoints are pinned by store path, `SHA256SUMS` and step.
   - **Rows:** DEV's 3,406 later rows the reply table does not name, and its 32 unkeyed openings, judged pooled (3,438).
   - **Band, fixed in advance:** `max(2·√(b+c), the two-seed spread)`.
   - **The cells:**
     - (r) ahead at both seeds: the memory recipe stays the training default;
     - (r) behind at both seeds: a memory-dropout arm joins the training run;
     - mixed: both arms train.
   - **Descriptive only:** Fresh V2, and 64 greedy samples per arm scored against the census index.
3. **The protocol.** A prep sheet, with a fresh FIT, for the training run of the model to ship. It holds:
   - **arms:** (r), memory-off, (r0), tables only, and a longest-suffix table over TRAIN;
   - **the transport controls:** frozen and identity SpiralCore routes, under the fixed backward;
   - **the read-by-meaning arms:** the Flock read, and the type spin;
   - **decode rules:** greedy, and a Flock rank rule with the harmonic weights of `stack/flock.rs`;
   - **the verdict rule,** written in advance.

   A decision stratum is a panel no decision has touched. Fresh V2 is never one.
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
