# REPORT: one-model-end-to-end, claude-340

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Code head `d3a5f2e1f467b28d1c6af4ee2022318122a63f8d`, on base `59494646cdae3e2f8874939680324ed8cd5affd5`.

**In one line:** `served-reply-panel`'s two rank modes now parse the artifact once per run instead of up to twice per row, and take about 2.5 to 3 times less wall time on a 2 MB artifact, with every scored field unchanged.

## What did not work, first

- **34 of the instrument crate's 115 locked `.crate` tarballs were missing from the shared `~/.cargo/registry/cache`.**
  - Their unpacked sources were still present, but `cargo --offline` needs the tarball, so no offline build of this crate could run. The cause is not known.
  - The session that started this unit relaxed its `--offline` rule for one command: `~/.cargo/bin/cargo fetch --locked` on this crate's manifest. It restored exactly those 34 tarballs (509 to 543 in the cache).
  - `Cargo.lock` is unchanged. Every build and test after it was `--offline --locked`.
- **The gap this unit was opened for was already closed on the integer path.**
  - claude-311 named a per-row parse in the greedy loop. [claude-316](../claude-316/REPORT.md) added the decode-only timer, and [claude-338](../claude-338/REPORT.md) built one forward and one model before the loop.
  - **So `--engine integer` decode tokens per second does not move here.** That is the figure the project leads with. This unit does not make it faster or more honest; it already was a decode figure.
- **No artifact near claude-311's 51 MB was timed.** That file is not on this machine. The 6.4 MB artifacts that are here ask or carry, so the CPU port refuses them, and the rank modes run only on the CPU port. The times below are on 2 MB artifacts, where a parse is cheap; the saving on a larger artifact is not measured.
- **The first integer timing runs were not interleaved.** All "before" ran first, and load then slowed some "after" runs (median 591 to 516 at s1). Those medians reflect the machine, not the code. They were rerun interleaved, below.
- **The parse count is a static check, not a runtime count.** The test counts call sites in the source; it does not count calls during a run.

## What changed

All in `docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`:

- `teacher_forced_row` and `paired_first_row` take the once-built `&CpuFbsForward` instead of the artifact's bytes, and clone it per row, as the greedy loop already does.
- `the_artifact_is_parsed_once_before_every_timer`: in non-test code, `CpuFbsForward::from_artifact` and `IntegerStackModel::parse` each have one call site, and it sits before the first timer.

## Measurements

On the laptop, no GPU. Both binaries built from the same tree, before with the fix set aside. Panel `m2-fresh256`, 256 rows, claude-322's two sealed artifacts (2,156,416 bytes each).

**Unchanged output:**

| Check | s1 | s2 |
|---|---|---|
| I-c, `--engine integer`, timing fields removed, before and after | `5a27775c…` both, equal to the sheet | `7eb8dbf5…` both, equal to the sheet |
| Rank modes, CPU port, `--teacher-forced 1 --paired-first 1`, timing fields removed | `116d1fdf…` both, byte-equal | `f7f677f5…` both, byte-equal |

**Rank-mode wall time, milliseconds, median of 3, before and after interleaved:**

| Field | s1 before | s1 after | s2 before | s2 after |
|---|---|---|---|---|
| `teacher_forced.wall_milliseconds` | 17,522 | 7,048 | 16,043 | 5,379 |
| `paired_first.wall_milliseconds` | 14,732 | 5,760 | 13,489 | 4,376 |

**Parses:** a 256-row rank run went from 512 forward parses inside those timers (one per row in each mode) to none. The run builds one forward before any timer.

**`--engine integer` decode tokens per second, the unchanged reference, 4 runs each, interleaved:**

| | s1 before | s1 after | s2 before | s2 after |
|---|---|---|---|---|
| Best run | 592 | 594 | 635 | 633 |
| All runs | 546–592 | 558–594 | 421–635 | 514–633 |

The laptop was shared with other sessions, so the slow runs are load on both binaries. The best runs agree within 0.4%. On this path `tokens_per_second` (wall) and `decode_tokens_per_second` differ by at most 1.

## What this does and does not show

- **Shows:** the rank modes hold no artifact parse in their timers, their scored output is byte-identical, and the integer path's output and speed are unchanged.
- **Does not show:** any change to a model, or to the lead tokens-per-second metric. The rank wall times include the ranking itself and a clone per row, so they are not per-token decode figures.

## Checks run at the head

- **Host, at `d3a5f2e1f`:** `served-reply-panel` tests with `--features cpu-parity`, `--offline --locked` and the pinned tokenizer: 24 passed, 0 failed. Clippy on the binary and its tests: no warnings in this file. `rustfmt --check` clean.
- **Negative gate:** with a parse put back in `teacher_forced_row`, the new test fails with `CpuFbsForward::from_artifact( call sites at [9724, 25147]`. Restored byte-identical, it passes.
- **The in-unit fresh review** is the counted record on the PR.

## Skills used

- None beyond the loop itself: the ORDER, the host tests with the negative gate, the real parity and timing runs and the fresh exact-head review.

## Still open

- **The line for `local/claude/FRICTION.md`** could not be written from this worktree, because the edit guard refuses the main checkout. It is in the shared session notes for the starting session.
- **The older REPORTs that say "the timer re-parses"** (claude-312 to claude-320) stay as written, as records of their time. They do not describe the code now.
- **DeepSeek's draft #3895** edits this file in other hunks; whichever lands second merges the other.

Closes nothing: #3407 stays open.

## Plain summary

The grading program has two modes that rank the right answer against every other word. Each mode re-read the whole model file for every test conversation, so a run read it up to 512 times. It now reads the file once and copies the loaded model for each conversation.

Those two modes run about 2.5 to 3 times faster on a small model, and their grades are identical, byte for byte. The measured speed of the served model does not change: that path was already fixed by two earlier units, which this work confirmed.

## Records

- Local: `local/c340/` in this unit's worktree held `measure.log`, `interleave.log` and the reports. Those files go with the worktree; the numbers that matter are in the tables above.
- Notebook: one row in `docs/research/notebook/float-training.md`.
