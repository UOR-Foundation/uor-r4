# REPORT: infra-loop-pr04-gates, claude-13 (loop PR 4: loop gates runner, gates.json, ledger and breaks)

Lab: claude. Base: `e61746628`. Order: [ORDER.md](ORDER.md), committed (`918a845a4`) before the work. Its cancel probe found no `gates.json` and no `loop` subcommand. A fresh-context Opus builder wrote the code. The main session re-ran the gates and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **No `sha2`.** Adding it to the harness crate would change `Cargo.lock`, which is not among the ORDER's files. Content hashes use `git hash-object --stdin` instead, so the PR adds no dependency.
- **`cargo xtask links` cannot be a single gate run.** CI runs it twice: once at the base to record a baseline, then again with `--baseline`. The tree already has 931 broken links, so a planted broken link stayed green, which is a gate defect. Links stays out of `gates.json` until the runner can carry a base baseline.
- **The `xtask` cargo alias does not resolve in a scratch worktree.** Gate commands use `cargo run -q --locked -p <package>` instead.
- **A pre-commit hook staged the branch claim in `docs/harness/claims.json`.** It was taken out of the commit. That file stays uncommitted, as for every claim.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopgates.rs` (new), `lib.rs`, `main.rs` | code | 649 | `uor-harness loop gates [--quick\|--full\|--keep-going\|--status\|--break G\|all]`. Other `loop` actions exit 2 with "not built yet". `--watch` exits 2 with a pointer to loop PR 11. A bad `UOR_LAB` exits 2 through `lab_or_refuse`. |
| `crates/uor-r4-harness/tests/loop_gates_run_rules.rs` (new) | tests | 252 | 11 tests, one or more per run rule. They use temporary git repositories, strip every `GIT_*` variable from child processes, and assert that each repository is inside its own temp directory. |
| `docs/harness/gates.json` (new) | data | 41 | Schema version 1. Each gate has `id`, `proves`, `command` (argv), `reads` (globs), `quick`, and `break {file, append, expect}`. The three gates are `fmt-claim-wording`, `fmt-harness` and `claim-wording`. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | ~20 | Usage, and the loop page's runner line now says what is built. |

The total is about 960 lines, about 1.07 times the 900-line estimate.

**Design calls:**
- **Verdicts.** A run gate ends GREEN, RED, REFUSED (it cannot run), UNKNOWN (an earlier gate was not green and `--keep-going` was not given) or VOID (its read files changed during the run). Any verdict other than GREEN stops the run, REFUSED and VOID included.
- **Break verdicts.** A break ends CAUGHT (red with its declared text), REFUSED (red for another reason) or DEFECT (still green).
- **Exit codes.** 0 when every gate is green or caught, 1 otherwise, and 2 for usage errors, a bad lab or an unreadable `gates.json`.
- **Logs and ledger.** Logs and the true exit code go under `local/<lab>/gates/<run-id>/`. The ledger is `local/<lab>/gates.jsonl`, with one row per run gate: gate, verdict, seconds, commit, working-tree hash, exit code and timestamp. Break runs are not written to it. `--status` shows the latest result per gate, and STALE when the working-tree hash differs.
- **Working-tree hash.** `HEAD`, plus `status --porcelain=v1 -z --untracked-files=all`, plus `diff HEAD --binary`, plus the path and bytes of each untracked, unignored file. `local/` is ignored, so logs and the ledger never change the hash.
- **Read-file hash.** The path and bytes of every tracked, or untracked and unignored, file that matches the gate's `reads` globs.
- **Breaks.** A break appends text to a file in a detached scratch worktree at `local/<lab>/gates/<run-id>/scratch/`. A Drop guard always removes it with `git worktree remove --force`. If that fails, it deletes that exact path and then runs `git worktree prune`.
- **Finding cargo.** A leading `cargo` in a command resolves to `$CARGO`, then `~/.cargo/bin/cargo` if it exists, then `PATH`.
- **Git environment.** Gates and the runner's own git calls run without `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran each of these after the builder.

- `cargo test -q --locked -p uor-r4-harness --test loop_gates_run_rules`: 11 passed.
- Negative gate, on the builder's run. With only the three lines of the UNKNOWN-after-red rule set aside, `later_gates_read_unknown_after_a_red` FAILED. Restored, all 11 passed.
- `UOR_LAB=claude uor-harness loop gates --quick`: the three gates were GREEN with exit 0, and ledger rows were written.
- `UOR_LAB=claude uor-harness loop gates --break all`: every gate was CAUGHT, each red with its own declared text:
  - "crates/uor-r4-claim-wording/src/lib.rs";
  - "crates/uor-r4-harness/src/labs.rs";
  - "Prohibited claim phrase without a linked proof artifact".

  No scratch worktree was left under `local/`.
- `UOR_LAB=gemini … loop gates --quick` exits 2. `loop gates --watch` is refused with a pointer to PR 11.
- `cargo fmt -p uor-r4-harness --check` is clean. Clippy shows no new warnings in the new code.
- Claim-wording passed.
- The PR body lists the full `uor-r4-harness` suite, `xtask links` on the changed files and the local copyright check.

## Skills used

None of the private kit's skills.

## Still open

- **Gates still to add to `gates.json`:**
  - clippy and library tests for changed crates;
  - `uor-harness check`, `lint check` and `shard-check`;
  - the d23, d25 and d32 ratchets;
  - `archive-verify --base`, `roadmap check` and `docs-catalog --check`;
  - links, which needs a base-baseline mode first.

  A "create file" break kind may be needed, for example for `roadmap check`. Today a break can only append to a file that already exists.
- **The old breaks of a changed gate do not yet run against the new gate.** That needs the merge base's `gates.json`.
- **`--watch`** is loop PR 11.
- **A docs hook says no docs page lists `loopgates.rs` in its `sources`.** No page in the ORDER's scope could take it.

## Plain summary

`uor-harness loop gates` runs the checks listed in `docs/harness/gates.json`. For each one it keeps the true exit code and log, records a ledger row, and later marks that row STALE if uncommitted edits arrive. It stops at the first gate that is not green unless told to keep going, and voids a gate whose input files changed during the run. `--break all` plants each gate's declared fault in a throwaway copy of the repository and confirms that every gate turns red for its own reason. Tests pin each of these rules.
