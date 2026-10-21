# REPORT: infra-loop-pr04-gates, claude-13 (loop PR 4: loop gates runner, gates.json, ledger and breaks)

Lab: claude. Base: `e61746628`. Order: [ORDER.md](ORDER.md), committed (`918a845a4`) before the work. Its cancel probe found no `gates.json` and no `loop` subcommand.

Fresh-context Opus builders wrote the code: one for the first head, then one builder per commit for the review fixes. The main session re-ran every gate and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **No `sha2`.** Adding it would change `Cargo.lock`, which is outside the ORDER's files. Content hashes use `git hash-object --stdin` instead, so the PR adds no dependency.
- **`cargo xtask links` cannot be a single gate run.** CI runs it at the base to record a baseline, then again with `--baseline`. The tree already has 931 broken links, so a planted broken link stayed green: a gate defect. Links stays out of `gates.json` until the runner can carry a base baseline.
- **The `xtask` cargo alias does not resolve in a scratch worktree.** Gate commands use `cargo run -q --locked -p <package>` instead.
- **The first head failed the full harness suite.** 678 tests passed and 1 failed: `the_real_repo_passes_the_ported_rules`. The `harness.documented_commands` rule found the new usage lines naming `uor-harness loop gates`, a command its tables did not know. The main session registered `loop` and its `gates` action, and the suite passed.
- **Two fresh-context reviews of `e7986fcef` asked for changes.** Fixed in three commits, one builder each:
  - **Containment, `0f3cbccb2`.** A break's `file` was joined to the scratch path unchecked, so an absolute path, `..` or a symlink could write outside the scratch worktree. Gate ids were used in file paths unchecked. Both are now refused at load, with exit 2. A break target that is a symlink, or that resolves outside the scratch worktree, is refused before any write.
    - The Drop guard no longer runs a repo-wide `git worktree prune`.
    - At the start of `--break`, a sweep removes stale scratch worktrees whose paths match `local/<lab>/gates/<run>/scratch/wt-<id>` exactly. These are left behind when Ctrl-C kills a run.
    - Each ledger row is one `write_all`.
    - An empty selection exits 2.
  - **Verdict precision, `7495beef6`.** A break was CAUGHT whenever the gate went red with text that a parse error or an already-red tree would also print. Now a break runs the gate once before planting the fault, and the result is REFUSED if the gate is already red. Each `expect` is text that only the planted fault produces: `+fn planted_break() {}` for the two fmt gates, and for claim-wording the planted sentence itself (see `docs/harness/gates.json`).
    - Hashes ignore user diff configuration (`--no-ext-diff --no-textconv --no-color`) and are length-prefixed, so a missing file and an empty file hash differently.
    - A gate whose `reads` match no file is REFUSED, since VOID would otherwise be blind.
    - A signal death records `signal N`.
    - `--break` refuses an uncommitted `gates.json`.
  - **Tests and tidy, `4f8e5b404`.** The `--break all` test now uses distinct markers, so a shared worktree would fail it. The `--watch` test asserts the PR 11 message, not just exit 2. New tests cover a missing break, the real `gates.json`, `--quick` selection, ledger fields and the `--keep-going` exit code. `--watch` left `FLAGS`, and `cmd_loop` no longer separates `lab_or_refuse` from its doc comment.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopgates.rs` (new), `lib.rs`, `main.rs`, `doccheck.rs` | code | 861 | `uor-harness loop gates [--quick\|--full\|--keep-going\|--status\|--break G\|all]`. Other `loop` actions and `--watch` exit 2, and `--watch` points to loop PR 11. A bad `UOR_LAB` exits 2 through `lab_or_refuse`. `doccheck.rs` registers `loop`, `gates` and five flags. |
| `crates/uor-r4-harness/tests/loop_gates_run_rules.rs` (new) | tests | 467 | 24 tests over temporary git repositories, with every `GIT_*` variable stripped and each repository asserted inside its own temp directory. |
| `docs/harness/gates.json` (new) | data | 41 | Schema version 1: `id`, `proves`, `command` (argv), `reads` (globs), `quick`, `break {file, append, expect}`. Gates: `fmt-claim-wording`, `fmt-harness`, `claim-wording`. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 12 | Usage, verdicts, and the loop page's runner line. |

The total is about 1,380 lines, excluding this ORDER and REPORT. That is about 1.5 times the 900-line estimate, after one review round of fixes.

**Design calls:**
- **Run verdicts.** A run gate is GREEN, RED, REFUSED (it cannot run, or its reads match no file), UNKNOWN (an earlier gate was not green, without `--keep-going`) or VOID (its read files changed during the run). Any result other than GREEN stops the run.
- **Break verdicts.** A break is CAUGHT (green at HEAD, then red with its declared text once the fault is planted), REFUSED (already red before the break, or red for another reason) or DEFECT (still green).
- **Exit codes.** 0 when all gates are green or caught. 1 otherwise. 2 for a usage error, a bad lab, an unreadable or invalid `gates.json`, an empty selection, or `--break` with an uncommitted `gates.json`.
- **Logs and ledger.**
  - Logs and the true exit code (or signal) go under `local/<lab>/gates/<run-id>/`.
  - The ledger `local/<lab>/gates.jsonl` gets one row per run gate: gate, verdict, seconds, commit, working-tree hash, exit code, signal and timestamp. Break runs are not recorded there.
  - `--status` shows the latest row per gate, and STALE when the working-tree hash differs.
- **Hashes.** Every section is length-prefixed, and a missing file is marked distinctly.
  - The working-tree hash covers `HEAD`, `status --porcelain=v1 -z --untracked-files=all`, a config-free `diff HEAD --binary`, and the untracked, unignored files.
  - The read-file hash covers every tracked, or untracked and unignored, file that matches the gate's `reads` globs.
- **Finding cargo.** A leading `cargo` resolves to `$CARGO`, then `~/.cargo/bin/cargo`, then `PATH`.
- **Environment.** Gates and the runner's git calls run without `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran each of these at `4f8e5b404`, after the builders.

- `cargo test -q --locked -p uor-r4-harness --test loop_gates_run_rules`: 24 passed.
- **Negative gates**, run by the builders. Each set aside only its own fix's lines, rebuilt, and restored a byte-identical copy afterwards. For each, the named test failed with the lines set aside and passed after restoring:
  - the UNKNOWN-after-red rule: `later_gates_read_unknown_after_a_red`;
  - the containment check: `a_break_file_outside_the_tree_exits_2`;
  - the green-at-HEAD pre-check: `a_break_on_a_gate_already_red_at_head_is_refused`;
  - the `--watch` refusal arm: `watch_and_other_actions_are_refused`.
- **Worktree isolation.** The builder ran a real check. Planting gate 1's marker into gate 2 simulates a shared worktree, and that turned gate 2 REFUSED and failed `break_all_turns_each_gate_red_for_its_own_reason`.
- `UOR_LAB=claude uor-harness loop gates --quick`: all three gates GREEN.
- `UOR_LAB=claude uor-harness loop gates --break all`: all three CAUGHT, each with its own text. No scratch worktree was left under `local/`.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 694 passed, 0 failed.
- `cargo fmt -p uor-r4-harness --check` is clean. Clippy shows no new warnings in the touched files.
- Claim-wording passed. `xtask links` found 0 broken links in the changed files. The local copyright check against the Loop Kit found 0 matching 8-word runs.

## Skills used

None of the private kit's skills.

## Still open

- **Gates still to add to `gates.json`:**
  - clippy and library tests for changed crates;
  - `uor-harness check`, `lint check` and `shard-check`;
  - the d23, d25 and d32 ratchets;
  - `archive-verify --base`, `roadmap check` and `docs-catalog --check`;
  - links, which first needs a base-baseline mode.

  A "create file" break kind may be needed, for example for `roadmap check`.
- **The old breaks of a changed gate do not yet run against the new gate.** That needs the merge base's `gates.json`.
- **VOID compares a snapshot from the start of the run with one from the end.** A file that changes and changes back during the run is not seen.
- **The run summary prints `exit -` for a signal death.** The signal appears in the reason text and in the ledger.
- **`--watch`** is loop PR 11.

## Plain summary

`uor-harness loop gates` runs the checks listed in `docs/harness/gates.json`. For each one it keeps the true exit code and the log, and records a ledger row that is marked STALE once uncommitted edits arrive. It stops at the first result that is not green, unless told to keep going. It voids a gate whose input files changed during the run, and refuses one whose inputs match nothing.

`--break all` copies the repository into a throwaway worktree that cannot be escaped. There it checks that each gate is green, plants that gate's declared fault, and confirms the gate then turns red with that fault's own message. Tests pin each of these rules.
