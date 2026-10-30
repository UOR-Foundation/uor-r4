# ORDER: infra-loop-harness, claude-210

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `2f3b0b761`.

## Goal

The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

Setup row: 2 (a loop-guard repair the owner asked for, and the roadmap's issue map).

## The owner's words (verbatim)

Casey, in Claude's chat, 2026-10-29, after the guard refused a write for the third time that hour: "can you please change the fucking harness to allow this shit so we can move forward and stop messing around with tooling".

## This unit

**Two things.**

**1. The loop guard's false refusals are fixed** (`crates/uor-r4-harness/src/loopguard.rs`). On 29 Oct the guard refused, among others:
- **A heredoc's body.** The body (`cat > file <<'EOF' … EOF`) was parsed as commands. A file's text that mentioned `> "$out"`, or `git checkout -- docs/roadmap/issues.json`, was refused as a write.
- **A variable the command set itself.** `F=path && … > "${F}"` was refused, because the guard could not resolve `$F`.
- **An owner path the owner had approved.** CHARTER.md was refused for D82, after Casey approved the edit in chat. That needed a `git apply` workaround (claude-209). `loop pr-check` already accepts an ORDER's `## Owner approval`, and the guard did not.

**The fix:**
- **Heredoc bodies are data.** A heredoc's body is taken out of the command before parsing (`split_heredocs`, quote-aware, with `<<-` and quoted delimiters). It is checked as commands only when the line feeds it to a shell (`bash`, `sh`, `zsh`, `eval`, `source` and the like), or when it is unterminated.
- **Variables the command sets are expanded.** `NAME=value` assignments that stand alone (or follow `export`) are recorded, and `$NAME` / `${NAME}` in later targets expand to them. `$HOME` is known. Every other variable stays as it was: refused if it names a target.
- **Owner paths open with a recorded approval.** An owner path is writable by the lab's main session when its active ORDER's `## Owner approval` names it, with an ISO UTC time: the same `owner_approval_missing` check `loop pr-check` runs.
  - `docs/harness/owner-paths.json` itself never opens.
  - A subagent never writes an owner path.
  - The path must still be in the item's scope.

**What stays.** R10 (the shared primary checkout) is unchanged: it protects the owner's own uncommitted work. Scope, `local/` partitioning, archive immutability and every command rule are unchanged.

**2. The roadmap's issue map.** After claude-209 merged, `cargo xtask roadmap sync --apply` created #3416 for `m1-heldout-phrasing-transfer`. It ran from the shared main checkout by mistake, which left `docs/roadmap/issues.json` (`"m1-heldout-phrasing-transfer": 3416`) and `docs/roadmap/README.md` changed there. This unit lands the same two lines from a worktree. The main checkout's copies equal what lands, and Casey can restore them with `git -C ~/uor-r4 checkout -- docs/roadmap` (R10 refuses it to a lab).

## Cancel probe (run first)

- `origin/main`'s `issues.json` has no `m1-heldout-phrasing-transfer` key, and #3416 is open.
- `loopguard.rs` on `origin/main` has no heredoc handling: `tokenize_shell_segments` reads `<<` as a word, and every body line as a command. No variable expansion exists for targets.
- No open PR touches `loopguard.rs`, `tests/loop_guard.rs`, `issues.json` or the roadmap README.

**Class: instrument (the guard's parser). Go.**

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-210/**`

## Done when

1. **Each change has an integration test in `tests/loop_guard.rs`, and the existing refusals all still hold** (`command_and_cross_lab_guards_refuse_violations`). The tests cover:
   - a heredoc body as data; a body fed to bash; an unterminated heredoc; a `<<` in quotes; a command after a body;
   - a variable set in the command, exported, owner-pathed, unknown, and in a command's own environment;
   - an owner path with an approval, one without, one with no time, another owner path, and `owner-paths.json`.

   A negative gate: with the guard change set aside, the three new tests FAIL.
2. **The harness's full test suite and `loop gates --full` pass.**
3. **The two roadmap files** equal what `roadmap sync --apply` wrote, and `roadmap check` passes.
4. **A REPORT,** and a fresh review.

## Stop and decide if

- A change would let a lab write a path the old guard refused for a real reason, beyond the three named above.
