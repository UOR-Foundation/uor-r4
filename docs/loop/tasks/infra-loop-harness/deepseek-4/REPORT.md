# REPORT: infra-loop-pr12-git-hooks, deepseek-4 (loop PR 12 repair: the CHANGES record and Casey's three decisions)

Lab: deepseek. Order: [ORDER.md](ORDER.md), the only ORDER this PR adds; deepseek-3 is history inside it.
Base: `2faa7aa8`. Closes #2917. References #2904.

## What did not work, first

Answering the CHANGES record at `77fd684db`, point by point, and building in the owner decisions it records:

- **HIGH 1, the commit-msg hook blocked the owner and every other lab.** The check ran on every branch. Owner decision 1 is now the rule: trailers are checked only on a `<registered-lab>/...` branch with `UOR_LAB` set; `Merge `, `Revert `, `fixup! ` and `squash! ` messages pass, and the owner's commits and non-loop work are never checked. Tests pin each case, including a real `git commit` through the installed hook.
- **HIGH 2, `push-check` read the wrong column.** A new branch (`local sha set`, `remote sha zeros`) was treated as a delete and skipped, and a real delete (`local sha zeros`) ran `pr-check` on zeros. The delete test is now `local sha` all zeros, a new branch is checked at its local sha, and tests use git's `<local ref> <local sha> <remote ref> <remote sha>` format.
- **Owner decision 2, the LFS chain.** The pre-push hook now copies its stdin to a temp file, runs `git lfs pre-push "$@"` on that same input first, then `uor-harness git push-check` on it. `install-hooks` is all-or-none: every refusal is checked before anything is written. Every hook it replaces is copied to `local/loop/install-backups/<unix-seconds>/<hook>`, which also backs up the pre-v2 `pre-commit`.
- **Owner decision 3, owner-only install.** `git install-hooks` now prints `would install/update/unchanged` and the command to write, and writes only with `--install`; `main.rs` refuses the command outright when `UOR_LAB` is set, and loop PR 9's guard will repeat that refusal.
- **MEDIUM, a `pr-check` exit of 2 blocked the push.** Only exit 1 blocks now; 2 prints `cannot run pr-check ... the push is allowed`. A test removes `origin` so `pr-check` cannot run and asserts the push passes.
- **MEDIUM, the planted break was generic.** `gates.json` no longer appends a panicking test: it replaces the trailer branch in `githelp.rs` with an early return, so the gate goes red on the assertion text of the trailer-refusal test (`a message without trailers must be refused`). `--break all` is CAUGHT on it.
- **MEDIUM, the LFS step would be refused with a partial install.** Resolved by all-or-none: a foreign or symlinked hook refuses before any of the three is written.
- **LOW, stale binary blocked every commit.** `check-message` now prints a warning for a provably older binary and allows the commit; the hook is installed by the owner, and a stale PATH copy must not block every commit. The warning path itself is not directly tested: it needs a build inside the checkout, which an external `CARGO_TARGET_DIR` does not leave.
- **LOW, the fixture could write hooks outside its repository.** Every test repository sets `core.hooksPath` locally and the helper asserts the resolved hooks directory is inside the temp root before anything writes.
- **LOW, out-of-scope files and churn.** `looparrive.rs` and `tests/loop_arrive.rs` are dropped from the PR (they belong to PR 11's scope), as is `docs/harness/claims.json`; the hook scripts moved from `crates/uor-r4-harness/rules/` to the repository-root `rules/` the item scopes; the README change is now only the hook section and command table, not the `checkout` bullet; `gates.json` keeps its base formatting and adds only the `git-hooks` gate.
- **LOW, the REPORT counts.** `tests/git_hooks.rs` now has 16 tests; the previous REPORT said 20 and the wrong line count.
- **LOW, trailer parsing.** Trailers are read only from the message's final paragraph, keys are matched case-insensitively, and a test puts trailers above a later body paragraph to prove they do not count.
- **LOW, no lab may bypass; the hooks must not suggest it.** Both hook scripts now say to fix the message or the policy failure and never print `--no-verify`.
- **Still structural:** on a repaired branch the local `commit-order` fails (the active ORDER is added after code already pushed; the history cannot be rebuilt without a force push, which the rules forbid). The queue gate today is the `uor-review v1` record plus CI, as on the PM's own repairs; the REPORT names it.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/githelp.rs` | code | +190/−95 | owner-only all-or-none install with backups; lab-branch-only, final-paragraph, case-insensitive trailer check with a binary warning; push-check reads the local sha, checks new branches, skips deletes, blocks only on exit 1 |
| `crates/uor-r4-harness/rules/commit-msg-hook.sh`, `rules/prepush-hook.sh` | data | +66 | the two new hooks at the scoped root: no bypass advice; LFS first on the same stdin, then push-check |
| `crates/uor-r4-harness/src/main.rs` | code | +30/−2 | `--install`, the lab-session refusal, and the two hook actions in the command table |
| `crates/uor-r4-harness/src/doccheck.rs` | code | +2 | registers `--install` |
| `crates/uor-r4-harness/tests/git_hooks.rs` | tests | +476 | dry-run, install, all-or-none, backup, lab-session refusal; branch, merge/fixup, final-paragraph and case tests; a real `git commit`; new-branch/delete/cannot-run push checks; LFS-first end to end |
| `docs/harness/README.md` | docs | +38/−17 | the three hooks, owner-only install, backups, LFS chain, lab-branch rules |
| `docs/harness/gates.json` | data | +12 | the `git-hooks` gate with its real planted fault |
| the item's status line, the roadmap, and the task folder | generated/docs | small | item done; roadmap re-rendered; deepseek-3 folded, deepseek-4 ORDER and this REPORT |

**Size:** about 900 changed lines, against the ORDER's 350-line estimate - the tests are the surplus (the extra cases the review asked for).

## Measurements

- **`loop gates --full`:** GREEN on all five gates.
- **`loop gates --break all`:** CAUGHT on all five; `git-hooks` red with `a message without trailers must be refused`.
- **`cargo test -p uor-r4-harness --test git_hooks`:** 16 passed.
- **The full harness suite, run alone:** green (the exact per-target line is in the PR checks).
- **`loop arrive`:** PASS checkout and binary, WARN hooks (the owner has not installed them), arrive: OK.
- **The hooks end to end:** a real `git commit` without trailers is refused on a lab branch and allowed with them; the pre-push hook runs a fake `git-lfs` first on the same stdin and then blocks on the policy failure; deleting a branch is skipped; a new branch is checked.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: every target green (run alone; two suites sharing one target dir can run each other's binaries).
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on five gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on five gates.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: `order`, `report`, `owner-paths`, `rule-sources` and `read-path` PASS; `scope` (shared files, see below) and `commit-order` (repaired branch) are the two named structural failures.

## Skills used

- **gates**, read from its private local install: the planted-fault pattern and the `replace` break the gate now uses.

## Still open

- **`looparrive.rs` is dropped from this PR.** PR 11's scope owns it and it is not on the shared list, so the three installed hooks are checked by `loop arrive` only for `pre-commit`. The ORDER done-when is recorded here as changed: extending it needs a scope delta for pr12 (asked on #2028) and would be a follow-up.
- **`scope` stays FAIL until claude-22's shared-file exemption lands**: `main.rs`, `doccheck.rs`, `docs/harness/README.md` and `docs/harness/gates.json` are the shared-nine, and the tool side is not on `main` yet.
- **The binary warning path has no direct test**, because it needs a build inside the checkout.
- **The LFS chain is tested with a fake `git-lfs`**, not the real one: the ordering and stdin are proven; real LFS transfer is not exercised.
- **A repaired branch cannot satisfy `commit-order`** without a force push, which the rules forbid; the counted review record and CI are the queue gate.

## Plain summary

- **The trailers are checked only where the loop applies.** On a `<lab>/<topic>` branch with `UOR_LAB` set, the commit message must end with `Lab:` and `Order:`; merge, Revert, fixup! and squash! messages pass, and the owner's own work is never checked.
- **The push hook runs the project's own checks, after LFS.** It gives Git LFS the original push input first, then runs `pr-check --local` on each pushed `<lab>/...` branch: a new branch is checked, a deleted one is skipped, a policy failure blocks, and a check that cannot run does not.
- **The owner installs the hooks, not a lab.** `git install-hooks` shows the plan and writes nothing until `--install`; a lab session is refused. All three install together, each replaced hook is backed up under `local/loop/install-backups/`, and a foreign hook is never overwritten.
- **The gate proves the trailer rule can fail.** Its planted fault disables the trailer branch in `githelp.rs`, and the gate turns red with the test's own refusal text - not with an appended panic.