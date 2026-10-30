# ORDER: infra-loop-harness, claude-209

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `ef25fccc0`.

## Goal

The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

Setup row: 2 (an owner rule change recorded: charter text, decision index, change log, PLAN's owner directions and M1 bar, and the roadmap item that measures it).

## The owner's words (verbatim)

Casey, in Claude's chat, 2026-10-29 (recorded on #2028, comment 6511159403), answering:

> "Should M1 also require a held-out-phrasing panel? M1's fresh-transfer bar (e) passed on Fresh V2, but Fresh V2 comes from the same generator as TRAIN, so it tests new values inside known templates, not new phrasings. A held-out-phrasing panel now exists (P-dev, claude-204: 0.000 of its prompt templates occur in TRAIN). P-dev itself is reserved for the coming training run's decisions, so an M1 bar would use a fresh panel drawn the same way at a new seed."

Casey's answer: **"Yes, add the bar (Recommended)"**. That option read: "M1 also requires beating the T2 bigram and unigram on a fresh held-out-phrasing panel (drawn by heldout-panel at a new seed, with its own census), with the same paired band as bar (e). It proves the model reads new phrasings, not just recalls templates." The other options offered were "Not yet, after the run" (decide once the training run's verdict is in) and "No, keep M1 as is" (close on the current bars).

## Cancel probe (run first)

Run at this ORDER's base. **Class: none needed (a record). Go.**
- **No open PR touches the files below:** `gh pr list --state open` shows #3415 (this one) and antigravity's #3323 (`crates/uor-r4-object-store` and its prep sheet `docs/harness/prep/object-store-consumer-m3-2026-10-26.md`).
- **The last decision is D81,** so this one is D82.
- **CHARTER §4 item 8 reads:** "Bars (a), (b), (c) and (e) are unchanged. [D39, owner 19 Oct]".
- **M1's five bars passed.** `antigravity-110`, `m1-close-saved-model`, merged, reports `PASS_M1_ALL_FIVE_BARS` on the saved two-seed discrete artifact across two builds. M1 (#2029) is still open.

  So this bar keeps M1 open until that saved model passes it too. That is the decision's direct consequence, and it is written into each record.

## Files in scope

- `CHARTER.md` (§4 item 8)
- `docs/integration/DECISIONS.md` (the D82 row)
- `docs/rules/changes.md` (the dated entry)
- `PLAN.md`: the owner direction's row, and "The path to M1"'s bar paragraph
- `docs/roadmap/items/m1-heldout-phrasing-transfer.md` (new), and its rendered views by `cargo xtask roadmap render`
- `docs/loop/tasks/infra-loop-harness/claude-209/**`

## Done when

1. **§4 item 8** carries D82's added bar.
2. **`DECISIONS.md` has the D82 row,** with the owner's words verbatim.
3. **`docs/rules/changes.md` has a dated entry,** with the owner's words.
4. **`PLAN.md`:**
   - the owner direction's row, status `done` once this merges (the bar is recorded; it is measured by the item below);
   - the M1 bar paragraph names the added bar.
5. **A roadmap item, `m1-heldout-phrasing-transfer`,** for measuring the bar on M1's saved model:
   - the panel: `heldout-panel` at a new seed, from M-world v2's held-out phrasings;
   - the panel's census, counted against that model's own TRAIN split;
   - the unigram and T2 tables, and bar (e)'s paired band.

   The item passes `cargo xtask roadmap check`, and the views are rendered.
6. **A REPORT,** and a fresh review of the diff.

## Stop and decide if

- The charter or decision text would say more than Casey's words. Then quote his words and keep the rule to them.

## Owner approval

Owner paths changed: `CHARTER.md`, `docs/rules/changes.md` and `docs/integration/DECISIONS.md`.

**The decision.** Casey, in Claude's chat, 2026-10-29, answered "Yes, add the bar (Recommended)" (words above).

**The owner paths.** The loop guard refuses lab edits to owner paths. Asked at 2026-10-30T00:06:57Z:

> "May I write D82's text into the three owner-only files, as you allowed for D80? The loop guard refuses lab edits to CHARTER.md, docs/integration/DECISIONS.md and docs/rules/changes.md. For D80 you gave permission, and the edits went in as one patch, applied with git apply and checked line by line by a fresh reviewer against your words. D82's text adds only bar (f) to CHARTER §4 item 8, the D82 row and its dated log entry, quoting your answer. Note: antigravity-110 already passed all five current M1 bars, so bar (f) keeps M1 open until that saved model also passes on a held-out-phrasing panel."

Casey chose **"Yes, write them (Recommended)"**. That option read: "Same route as D80: an edited copy under local/claude/, a patch kept in the task folder, git apply, and a fresh line-by-line review against your words before merge."

**How the owner paths are written.** Each edited copy is made under `local/claude/d82/`. The difference is kept as `d82-owner-paths.patch` in this task folder, and applied with `git apply` under that permission. The patch touches only:
- `CHARTER.md`: §4 item 8, bar (f);
- `docs/integration/DECISIONS.md`: the D82 row;
- `docs/rules/changes.md`: the entry.

The fresh review checks it line by line against the owner's words.
