# ORDER: infra-loop-pr12-git-hooks, deepseek-4 (loop PR 12 repair: answer the CHANGES record and Casey's three decisions)

Lab: deepseek. Item: [infra-loop-pr12-git-hooks](../../../../roadmap/items/infra-loop-pr12-git-hooks.md) (#2917). Setup row: 12. Base: `2faa7aa8`.

## Done in deepseek-3 (history)

- deepseek-3 ORDER `b37bde663`: three managed hooks, `check-message`, `push-check`, the `git-hooks` gate; code `a5f541b66`; REPORT, item done and roadmap; receipts `45971e203`; merges of PR 11's main and `kimi-4`.
- Claude's fresh-context review returned CHANGES at `77fd684db`, with three owner decisions recorded verbatim in its record.
- This repair folds deepseek-3 into deepseek-4 so the PR adds one ORDER; the deepseek-3 REPORT stays as history and points here.

## Goal

Answer the CHANGES record at `77fd684db` and build in Casey's three decisions of 22 Oct 2026, recorded verbatim by Claude:
1. **Trailers only on lab branches.** The commit-msg check applies only on a `<registered-lab>/…` branch while `UOR_LAB` is set; merge, Revert, fixup! and squash! messages pass, and the owner's commits and non-loop work are never checked.
2. **Pre-push chains with LFS.** The hook runs Git LFS's own step first on the same stdin, then the push check. The installer installs all three hooks or none, and backs up every hook it replaces to `local/loop/install-backups/`.
3. **The installer is owner-only.** `git install-hooks` prints its plan by default and writes only with an explicit flag; a lab session is refused.

## Cancel probe (run first)

The CHANGES record is the last counted record at `77fd684db`, so the repair is needed. Both HIGH findings reproduce on this branch: a lab-branch message with no trailers is refused today, and `push-check` reads the remote sha instead of the local sha (a new branch is skipped as a delete; a delete runs pr-check on zeros). Class: instrument, fixed here.

## Files in scope

- `crates/uor-r4-harness/src/githelp.rs`, `main.rs`, `doccheck.rs`, `version.rs` (readers only).
- `rules/*-hook.sh` (the two new hook scripts, at the root the item scopes).
- `crates/uor-r4-harness/tests/git_hooks*.rs`.
- `docs/harness/README.md`, `docs/harness/gates.json`.
- The item's `status:` line, `docs/roadmap/README.md`, and `docs/loop/tasks/infra-loop-harness/deepseek-4/**`.

## Done when

- The three owner decisions hold, each pinned by a test: lab-branch-only trailers with `UOR_LAB`; LFS first on the same stdin; dry-run by default, `--install` writes owner-only, all-or-none, with a backup of every replaced hook.
- `push-check` reads the local sha: a new branch is checked, a delete is skipped, and a `pr-check` exit of 2 does not block the push; a policy failure (exit 1) does.
- The `git-hooks` gate's planted fault is a real one: replacing the trailer branch in `githelp.rs` turns the gate red with the test's own refusal text.
- The full check set passes: gates --full green on five gates, --break all CAUGHT on five, the full harness suite, fmt and clippy.
- The hook scripts live at `rules/` and the README documents the owner-only install, the backups, the LFS chain and the lab-branch trailer rule.

## Stop and ask if

- A decision would change a hold, a cap or an owner path; a file outside scope must change; a new dependency is needed; the LFS chain cannot work without one.

Size estimate: about 350 lines.
