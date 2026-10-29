# ORDER: one-model-end-to-end, claude-202 (step 1: the data census)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `f0e4a7a7a`.

## Goal

The item's done-when 1, a count with no model and no GPU (CHARTER §4.5), run on the laptop as claude-192's `--reply-bound` probe was. It settles four questions for the [assessment](../../infra-loop-harness/claude-201/ASSESSMENT.md)'s order of work:
- how big TRAIN is;
- how much of DEV and Fresh V2 TRAIN already holds;
- which rows step 2 judges;
- whether any panel holds new templates.

**One new mode, `train-in-space --census 1`** (`in_space_long.rs`, `bin/train-in-space.rs`). It reuses the sheet's loaders and the probe's key code, so its counts are the same counts training and scoring use. It writes one JSON report of counts and row positions, never text.

**The inputs.** Each is pinned by sha, and the census refuses on a mismatch:
- TRAIN `mw-balp/train` and DEV `ft/dev` come from the HF store (`w2-inputs/2026-10-25/driver-data/`), fetched with `scripts/pod/uor-store get` into the worktree's `local/claude/data/` (gitignored).
- **Fresh V2** comes from its preservation archive, `codex-attention-dose1-fresh-v2-preservation.tar` in iCloud.
  - Its `tokens.u16` (`0b7e754c…`) and `response_mask.u8` (`a1cda529…`) are copied read-only into `local/claude/data/fresh-v2/`.
  - It was opened once, for bar (e), and here it is only counted (descriptive).
- **`m2-fresh256` is not opened.** It is M2's sealed acceptance panel ("NOT_RUN"). The K16 fresh slice is not opened either.

**The measures.** Every threshold is fixed here, before any count runs.

A document runs from one `<|bos|>` (id 0) to the next. A prompt is a maximal run of mask-0 tokens after the BOS or after a reply. A reply is a maximal run of mask-1 tokens.

| # | What | For |
|---|---|---|
| M0 | Header count, payload tokens, eligible targets (`p > 0`, `t ≠ BOS`, mask 1), documents, prompts, replies. For TRAIN, it explains 7,512,797 against the sheet's 26,214,400 | TRAIN, DEV, Fresh V2 |
| M1 | **Reproduction (the instrument check):** DEV's 1,226 openings and 1,194 keyed; the reply table's 9,655 named rows and 8,822 right; the reply-only control's 8,089 (`float-training.md` rows 14–15). All must match exactly | DEV |
| M2 | **Step 2's rows:** the positions of DEV's later reply rows the table does not name, and of its unkeyed openings, expected to total 3,406 + 32 = 3,438 | DEV |
| M3 | The share of replies whose whole token sequence equals a TRAIN reply; the same for prompts | DEV, Fresh V2 |
| M4 | The share of reply n-grams, and of prompt n-grams, that occur among TRAIN's, for n = 8 and n = 13, counting windows inside one reply or one prompt | DEV, Fresh V2 |
| M5 | **Value-masked templates.** Mask every token outside TRAIN's K most frequent tokens, and every digit token, then collapse runs of masks. Report the share of prompts and of replies whose masked sequence equals one of TRAIN's, at K = 256 and K = 1,024 | DEV, Fresh V2 |
| M6 | The 8-token opening-key share, with the probe's anchor | Fresh V2 |

**What decides "new templates".** A panel holds new templates if its M5 prompt-template share at K = 1,024 is at most 0.5. This threshold is fixed here. M5 is reported at both K, so the reading can be checked at the other.

## Cancel probe (run first)

- **Does a census already exist?** No.
  - The only overlap record is Fresh V2's "0 exact full TRAIN/DEV document overlap" (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:69`).
  - No n-gram, template or row census exists for the FBS line's TRAIN and DEV (`git grep -n -i "n-gram"` over `docs/research` and `docs/labs` at `f0e4a7a7a`).
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `docs/labs/one-model-end-to-end/**`
- `docs/loop/tasks/one-model-end-to-end/**`
- `docs/research/notebook/float-training.md`

## Done when

1. **`--census 1`** writes `docs/labs/one-model-end-to-end/census-2026-10-29.json` (counts and positions only) and a README table.
2. **M1 reproduces every pinned number exactly.** If one does not match, the census is an instrument fault, and the unit stops to find why before reporting anything else.
3. **Host tests** cover each measure on a tiny synthetic corpus whose answers are known by hand. That includes the masks of M5 and the row selection of M2.
4. **The REPORT** gives M0–M6, the TRAIN size answer, which panels hold new templates by the rule above, and one notebook row.
5. **Checks:** the float crate's suite passes, and `loop gates --full` is green. No GPU.

## Stop and ask if

Stop and decide (never wait) if:
- M1 does not reproduce: then classify it (instrument or data) before going on;
- the Fresh V2 archive is not readable without extracting more than the two named files: then Fresh V2 is reported as NOT REACHED;
- the census needs a third file outside this scope.
