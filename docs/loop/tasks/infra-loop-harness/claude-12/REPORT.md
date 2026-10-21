# REPORT: infra-loop-pr02-item-keys, claude-12 (loop PR 2: item keys, Cancel if, done-when class, validator)

Lab: claude. Base: `e61746628`. Order: [ORDER.md](ORDER.md), committed (`c4ea41236`) before the work. The cancel probe found no handling of the new keys, so the task stood. A fresh-context Opus builder wrote the code. The main session then hardened the root-wide check and wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **The first head refused only five spellings of a root-wide scope.** The list was `**`, `**/*`, `/**`, `*` and `./**`. Equivalent spellings such as `**/**`, `./*` and `*/**` still passed. Since the planted fault is "an item scoped `**` is refused", that was a gap in the fault itself. The main session replaced the list with a rule: after leading `./` and `/` are stripped, a glob is root-wide when every path segment is made only of `*`. This also catches `/` and `.`. The refusal test now covers all eleven spellings.
- **The negative gate could not diff the whole file against the base.** `git diff e61746628 -- xtask/src/roadmap.rs` would also have removed every other new rule. So the builder set aside only the five refusal lines, as a patch against the committed head (`local/root-wide-refusal.patch`, not committed). After the rule replaced the list, the gate was run again by forcing `is_root_wide` to return `false`. The refusal test FAILED, and it passed again once the function was restored.
- **The notebook existence check cannot live in `parse_item`.** `parse_item` and `validate` see only file names and text, not the repository. The form check stays there: the path is relative, has no `..` and ends in `.md`. The existence check runs in `load_checked`, which finds the repository root three directories above `docs/roadmap/items`.

## What changed

- `xtask/src/roadmap.rs`:
  - `scope`, `size`, `lane`, `rank`, `notebook` and `done_class` join `OPTIONAL_KEYS`. `REQUIRED_KEYS` is unchanged, and an unknown key is still reported as before.
  - `check_loop_keys` validates each key only when it is present:
    - `scope` must be a non-empty `[a, b]` list of repository-relative globs. A root-wide glob is refused with an error that names the item file (`is_root_wide`).
    - `size` and `rank` must be positive integers in plain digits.
    - `lane` must be a slug, `[a-z0-9-]+`.
    - `notebook` must be a repository-relative `.md` path, and `load_checked` checks that the file exists.
    - `done_class` must be `host` or `gpu`.
  - A `## Cancel if` section is optional. When present it must not be empty. Its position among the sections is not checked.
  - Ten new tests, one per rule.
- Size by class, counted against base `e61746628`: code plus tests come to 274 changed lines in `roadmap.rs` (268 added, 6 removed), and docs are this ORDER and REPORT. That is a ratio of about 1.1 to the 250-line estimate.

## Measurements

None. This is a host-only validator change.

## Checks run at the head

All commands ran with `CARGO_TARGET_DIR` set to a per-task directory.

- `cargo run -q --locked -p xtask -- roadmap check`: 87 items OK.
- `cargo test -q --locked -p xtask roadmap`: 42 passed. The full `cargo test -q --locked -p xtask`: 190 passed, 0 failed.
- Negative gate: with `is_root_wide` forced to `false`, `roadmap_loop_scope_refuses_every_root_wide_glob` FAILED. Restored, it passed.
- `roadmap render --check` reports current, so the renderer's output is unchanged.
- `cargo fmt -p xtask --check` is clean. `cargo clippy` shows no warning in `roadmap.rs`. The four warnings it reports were already there, in other files.
- Claim-wording, `xtask links` on the changed files and the local copyright check against the Loop Kit: listed in the PR body.

## Skills used

None.

## Still open

- **The keys become required later.** That happens after the PM backfills the ready items, as a later step. No item file is changed here.
- **Issue bodies don't show the new keys yet.** `roadmap_sync.rs` does not render the new keys or the Cancel if section. That is left to loop PR 3 or a follow-up.
- **"Notebook line" is read as a file.** The loop page says "a notebook line". This PR reads that as a path to an existing notebook `.md` file, so a line anchor is not accepted. If a line anchor is meant, the PM or the owner should say so.

## Plain summary

`roadmap check` now understands the six loop keys and a "Cancel if" section, and checks each one when it is present. All 87 current items still pass, because every new key is optional. An item scoped to the whole repository is refused, however the glob is spelled, and a test pins that.
