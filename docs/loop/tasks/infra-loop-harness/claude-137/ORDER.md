# ORDER: infra-loop-harness, claude-137

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `1b9b87b39`.

## Goal

The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The five labs (Kimi, Claude, Codex, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

**This unit removes codex from the lab registry, at the owner's instruction (below).** Codex works independently on lexlean-r4 and is no longer part of the UOR-R4 loop.

- **The registry.** `docs/harness/labs.json` lists four labs: antigravity, claude, deepseek and kimi. Its note says "four".
- **The two tests that read the real registry** move from five labs to four: `crates/uor-r4-harness/src/labs.rs` and `crates/uor-r4-harness/tests/lab_registry_cli.rs`. Tests that build their own registry fixture are unchanged.
- **The no-identity hints** (`receipt.rs`) name the same four labs. The CLI test holds them to that exact list.
- **Codex's closed PR #3169** is kept byte-identically in the archive, as `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.patch`, with its own record file beside it.
  - The record is a new file because the existing `INDEX.md` is immutable (CHARTER §3.12).
  - The PR was closed at the same instruction, at 2026-10-26T21:36:57Z.
- **The registry line itself is the owner's edit.** The loop guard refuses every lab write to an owner-only path, approval or not. So Casey makes the `docs/harness/labs.json` change on this branch, and this unit carries everything around it.
- **Left for the owner:** `AGENTS.md` (`UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`) and this item's own done-when ("The five labs (… Codex …)"). The first is an owner path that this instruction does not name. The second changes a done-when, which needs its own decision. The REPORT lists both.

## Owner approval

Casey, in claude's chat, received at about 2026-10-26T21:34Z, naming `docs/harness/labs.json`:

> "Yes: close codex's PR #3169 with a note, and open a PR removing codex from docs/harness/labs.json, quoting this instruction in its ORDER."

It answered claude's question "Close codex's #3169 and take codex out of the lab registry?", after Casey wrote: "codex is not currently working with the rest of the groups, it is working independently on lexlean-r4 so it should not be included."

## Cancel probe (run first)

Cancel if codex is already out of the registry, or another open ORDER changes `docs/harness/labs.json`. To check:
- `docs/harness/labs.json` on `origin/main`;
- the open PRs' files.

Result (2026-10-26T21:40Z, `origin/main` at `1b9b87b39`):
- `docs/harness/labs.json` still lists codex.
- The open PRs are #3304 (antigravity), #3306 (kimi) and this unit's #3307. None of them touches the registry, the two registry tests or the branch archive.

**Not cancelled (instrument: none needed).**

## Files in scope

- `docs/harness/labs.json`
- `crates/uor-r4-harness/src/labs.rs`
- `crates/uor-r4-harness/src/receipt.rs`: the no-identity hints name the registry's labs.
- `crates/uor-r4-harness/tests/lab_registry_cli.rs`
- `archive/history/branch-archive/**`
- `docs/loop/tasks/infra-loop-harness/claude-137/**`

## Done when

1. **The registry lists four labs,** and the harness refuses `codex` as an unregistered lab name.
2. **The registry tests read four labs.** The harness's full test suite passes, apart from tests that fail the same way on `origin/main` from a lab worktree; the REPORT names those.
3. **Codex's #3169 is in the archive.** Its patch reproduces the PR's tree exactly from its base, and its record file says so.
4. **`loop pr-check --local` passes,** including `owner-paths`, and `loop gates --full` is green.
5. **A REPORT.**

## Stop and ask if

Stop and decide; never wait.
- **A harness rule or test needs codex registered** for a reason other than the registry's lab list. Fix it in this scope, or record it and leave codex's client support (the guard's hook installer) unchanged.
- **Another owner path would need to change** (`AGENTS.md`, `CHARTER.md`, `docs/harness/owner-paths.json`): leave it, and list it for the owner.
