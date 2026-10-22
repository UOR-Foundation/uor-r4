# ORDER: infra-loop-pr10-supervisor, deepseek-18 (loop drive, watchers, kill switch, the soft check-in)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `f1c78da8e`.

## Goal

Build the supervisor the item's done-when describes: `loop drive` runs a fresh headless session per unit with the lab's pinned client and model and stops on the loop page's stops (kill switch first, then the `loop next` decision); a watcher outlives a session for a leased GPU run, renews the lease, notes it on the issue and never cancels a healthy run; the 2-hour check-in is soft per D53 (real progress continues with a note on the issue, a spiraled or stalled session closes with WIP committed and pushed, a REPORT stub and a PICKUP from git state, and a card to the PM); the supervisor signals only its own session's pid/process group, waits out the tracked tree, and never counts Claude Code's after-stop summary session as a lab session. The gate's planted fault must turn red when the stalled-session close is removed: a fake stalled client past two hours is closed and carded, while a fake progressing one continues with a note.

## Cancel probe (run first)

Class **instrument** (loop tooling; no model or research result). At base `f1c78da8e`: `loopgates::cli` answers `loop drive` with "not built yet (see the build plan)"; `loopnext::run` already returns `Decision::{Go,Stop}` and checks the kill switch file `local/loop/STOP` and the `loop:pause` label on #2028 before anything else; `sampling`·`budget`·`heartbeatcard`·`githubcards` and the PR 9 client adapters (`docs/harness/onboarding/<client>.json`) exist to build on. The done-when conditions are absent, and the loop page's build-plan row names this PR, so the task stands. Class: instrument, not mechanism.

## Files in scope

`crates/uor-r4-harness/src/loopdrive*.rs`, `crates/uor-r4-harness/src/githubcards.rs`, `crates/uor-r4-harness/src/heartbeatcard.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, plus the shared registration files any loop PR may touch (`src/lib.rs`, `src/main.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- `loop drive --client <c> --model <m> --prompt <text> -- <launch command with {prompt}>` runs one fresh session per unit: the kill switch is checked first (via `loopnext::run`, not re-implemented), the launch happens in its own process group, the pid/pgid, client, model, prompt hash and start time are recorded under `local/loop/drive/<lab>/`, and only that recorded pid/group is ever signalled.
- The soft 2-hour check-in: at 7200 s cumulative the supervisor classifies progress (new commits, gates turning green, in-scope changes, repeated failure); progressing sessions continue with a note on the owning issue; stalled or spiraled sessions close with WIP committed/pushed, a REPORT stub, a PICKUP from git state, and a PM card.
- A watcher (`loop drive --watch ...`) outlives the session for a leased GPU run: it renews the lease, notes it on the issue, and never cancels a healthy run or another session's resources.
- The after-stop summary session of Claude Code is not counted: the supervisor waits only on its tracked pid/tree and never treats a later unrelated process as the lab session.
- The `loop-drive` gate in `docs/harness/gates.json` runs the tests and its planted fault (removing the stalled-session close) turns it red with the item's own sentence: "a fake stalled client past 2 hours is closed and carded", while the progressing fake continues with a note.
- The docs (`docs/harness/README.md` `loop drive` section, `docs/loop/README.md` if it needs the command, the new flags in `src/doccheck.rs`) are current; `loop gates --full` GREEN and `--break all` CAUGHT; `report --check` and `pr-check --local` pass.

## Stop and ask if

- A client cannot be launched headless without a new adapter or credential path owned by another lab (use the PR 9 adapter data, and stop rather than invent one).
- The 2-hour classification needs a signal the repository does not record (tool-call telemetry): the fallback is commits/gates/working-tree only, named in the REPORT.
- The kill switch cannot be honoured without touching the owner's `local/loop/STOP` or the #2028 label.