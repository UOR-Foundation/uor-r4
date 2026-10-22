# ORDER: infra-loop-pr10-supervisor, deepseek-18

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md). Setup row: 10. Base: `f1c78da8e`.

## Goal

The item's done-when reads:

> `loop drive` starts a fresh headless session per unit with the lab's pinned model and stops on the loop page's stops, including the kill switch. A watcher outlives the session for a leased GPU run and never cancels a healthy run. The 2-hour check-in is soft (D53): real progress continues with a note on the issue, a spiraled or stalled session closes, and the PM prompts any lab that stopped. The supervisor stops only its own pids and does not count Claude Code's after-stop summary session as a lab session. The planted fault turns the check red: **a fake stalled client past 2 hours is closed and carded, while a fake progressing one continues with a note.**

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-harness/src/loopdrive*.rs`
- `crates/uor-r4-harness/src/githubcards.rs`
- `crates/uor-r4-harness/src/heartbeatcard.rs`
- `crates/uor-r4-harness/tests/loop_drive*.rs`

## Done when

To fill in.

## Stop and ask if

To fill in.
