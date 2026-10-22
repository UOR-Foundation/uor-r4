# REPORT: infra-loop-harness, claude-28 (one scope entry so #2949 can fix main's stale harness test)

Lab: claude. Base: `55311a46b`. Order: [ORDER.md](ORDER.md), committed first (`a9fa4deed`).

The cancel probe found no item whose `scope:` matches `crates/uor-r4-harness/tests/loop_gates_run_rules.rs`. The nearest, `infra-loop-pr11-progress-window`, covers only `tests/loop_gates_watch*.rs`.

## What did not work, first

- **`main` has been red on this test since #2946 merged, and my review let that through.** I posted a counted APPROVE on #2946 without reading its CI rollup. `harness rules and tests` was already red there (clippy, and this test). Main's post-merge CI skips that job, so neither failure showed. My review records now state the CI result at the head.
- **The fix cannot land where it belongs without a PM edit.** The PR-4 item that added this test has no `scope:` line, and no other item covers the file. So DeepSeek's #2949 stopped and asked (card 6336226999). Casey chose option (a) in Claude's chat: "Extend PR 10's scope (Recommended)". Kimi, the PM, had been silent since 06:51Z. This edit is the PM's, made at the owner's direction, and Kimi should check it.
- **My first cancel-probe grep was too loose.** It flagged `pr11` (`tests/loop_gates_watch*.rs`) and `m3-policy-precedence-and-sieve-tests` (`crates/uor-r4-training/**`). A shell glob match shows that neither covers the file.

## What changed

| Commit | What |
|---|---|
| `a9fa4deed` | This ORDER, alone. |
| `d1f82675b` | `infra-loop-pr10-supervisor`'s `scope:` gains `crates/uor-r4-harness/tests/loop_gates_run_rules.rs`; nothing else in the item changes. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is a roadmap record, with no model run.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK (ready 43, held 26, decision 4, done 15). The render leaves `docs/roadmap/README.md` unchanged, so the README is not in this PR.
- Claim-wording finds nothing in the changed files. `xtask links` finds 930 broken links repo-wide, as on `main`, and none in the changed files.
- The copyright check finds 0 of this diff's 8-word runs in LoopKit, and 0 in CorvusWing.
- `loop report --check` and `loop pr-check --local` pass at the head.

## Skills used

scope-guard: the PR changes one item line and its own task folder. The test fix stays in #2949, which owns it.

## Still open

- **#2949 (DeepSeek):** after this merges, merge `main`, re-run `pr-check`, and push the fix that changes the unbuilt-action example at `loop_gates_run_rules.rs:330`. Then `main`'s harness tests are green again.
- **Kimi** checks this edit on return.

## Plain summary

A harness test went stale when loop PR 10 built the command it used as its "not built yet" example. This one-line roadmap edit lets the follow-up PR touch that test file, at Casey's direction, so `main` can go green again.
