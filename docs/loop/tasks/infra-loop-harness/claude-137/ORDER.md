# ORDER: infra-loop-harness, claude-137 (widen the item's scope to take codex out of the registry)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `1b9b87b39`.

## Goal

**This unit is the item PR for a D68 scope and bar decision, at the owner's instruction (below).** It edits one file, `docs/roadmap/items/infra-loop-harness.md`, so that the next unit can take codex out of the lab registry. Codex works independently on lexlean-r4 and is no longer part of the UOR-R4 loop.

**Why a separate PR.**
- The loop guard takes a unit's scope from the item as it is on `origin/main`, so a branch cannot widen its own scope (`loopguard.rs`, `check_repo_rel_write`).
- No item's `scope:` holds the registry, its two tests, the no-identity hints or the branch archive. The guard refused the first test edit: "REFUSED: crates/uor-r4-harness/src/labs.rs is outside active ORDER … scope [PLAN.md, …]".
- The loop page says "Widening scope takes a separate item PR" ([docs/loop/README.md](../../../README.md), "Checked at the base"). The charter lets it go ahead when the lab's own decision review passes (§5.1, D68).

**The two edits.**
1. **Scope:** `scope:` gains five globs, exactly the files the registry change touches. Nothing else is added.
   - `docs/harness/labs.json`: the registry. It stays owner-only, so the guard still refuses every lab write and Casey makes that one edit.
   - `crates/uor-r4-harness/src/labs.rs`: its tests read the real registry and list five labs.
   - `crates/uor-r4-harness/src/receipt.rs`: the no-identity hints name the labs.
   - `crates/uor-r4-harness/tests/lab_registry_cli.rs`: its tests read the real registry and the hints.
   - `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*`: codex's closed #3169, kept as a patch and its record.
2. **Bar:** the done-when's "The five labs (Kimi, Claude, Codex, DeepSeek, Antigravity) are onboarded" becomes "The four labs (Kimi, Claude, DeepSeek, Antigravity)". The bar keeps the same shape. It no longer asks for a receipt and a loop task from a lab that has left. Codex's onboarding receipt and its past units stay in the repository as history.

**Left as they are.**
- **The item's title and its "Why it matters" prose** ("five-lab", "Five labs from five vendors"). They describe the build and no gate reads them; the REPORT lists them.
- **`AGENTS.md`** (`UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`): an owner path this instruction does not name. The REPORT lists it for the owner.

**Next unit.** Once this merges, claude-138 claims the item for the registry change: the tests and hints move to four labs, the patch goes into the archive, and Casey edits `docs/harness/labs.json` on that branch.

## Owner approval

Casey, in claude's chat, received at about 2026-10-26T21:34Z, naming `docs/harness/labs.json`:

> "Yes: close codex's PR #3169 with a note, and open a PR removing codex from docs/harness/labs.json, quoting this instruction in its ORDER."

It answered claude's question "Close codex's #3169 and take codex out of the lab registry?", after Casey wrote: "codex is not currently working with the rest of the groups, it is working independently on lexlean-r4 so it should not be included."

## Cancel probe (run first)

Cancel if codex is already out of the registry, or another open PR changes the item file or `docs/harness/labs.json`. To check:
- `docs/harness/labs.json` and the item file on `origin/main`;
- the open PRs' files.

Result (2026-10-26T21:40Z, re-checked 22:05Z, `origin/main` at `1b9b87b39`):
- `docs/harness/labs.json` still lists codex, and the item's `scope:` holds none of the five globs.
- The open PRs are #3304 (antigravity), #3306 (kimi) and this unit's #3307. None of them touches the item file, the registry, the two registry tests or the branch archive.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line and the done-when sentence)
- this task folder

## Done when

1. The two edits are made, and `xtask roadmap check` passes.
2. A fresh decision review, reading the whole project, approves the scope and bar change. Its record goes to the owner's log on #2028.
3. A fresh exact-head review approves the PR, and `loop report --check`, `loop pr-check --local` and the required checks pass.
4. The REPORT says `References #2904`.

## Stop and ask if

Stop and decide; never wait.
- **The review finds a narrower way** to make the registry change, for example a glob already in another item's scope. Take it, and drop the globs it makes unneeded.
- **The bar edit needs more than the lab list**, for example a gate that counts onboarding receipts. Record it, and keep this unit to the two edits.
