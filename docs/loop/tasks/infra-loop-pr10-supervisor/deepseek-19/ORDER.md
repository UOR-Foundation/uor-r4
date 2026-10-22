# ORDER: infra-loop-pr10-supervisor, deepseek-19 (repair: live check-in, safe WIP, guarded signalling)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `13430063e06a3e05fbb09836c38e4df4ec9c0b41`.

## Goal

Answer the counted other-vendor CHANGES record at `13430063e` (Claude, D59, fresh-context process and stall lenses). Required, in order:

1. **HIGH — the check-in must run while the session is live.** Today `child.wait()` blocks and the 10 ms observation pair makes every long session look stalled. Fix: take the before-observation at launch, poll `try_wait` with an injectable clock, at 7200 s classify live progress and terminate **only** a stalled session (never a healthy run, never one covered by a lease); keep progressing sessions with a note; count consecutive stalls so spiraled is reachable. Test with a fake client that commits during a fake 7200 s (Continue) and a silent one (closed and carded).
2. **HIGH — the WIP commit must be scoped and legal.** Refuse unless the branch is `<lab>/*` and not `main`; stage only the active ORDER's task folder; add the `Order:` trailer; strip `GIT_*` from the git child.
3. **MEDIUM — signalling safety.** Refuse a pgid of 1 or less, or the drive process's own group; verify identity (recorded `ps -o lstart=` plus a nonce in the child's environment) before any signal.
4. **MEDIUM — watcher liveness** must be pid plus identity, not `kill -0 -pgid` on a record that outlives the session.
5. **MEDIUM — a spawned child with no record** is terminated on its own pid if the record write fails.
6. **MEDIUM — the flaky test** uses a sleeping child and no liveness race.
7. **MEDIUM — the gate's planted fault must exercise the card path**, not only a pure match arm: a CLI-level test with a fake clock, a fake client and a fake card sink pins "closed and carded".

LOW, all in scope: a per-lab O_EXCL `drive.lock`; `{prompt}` refused as argv[0]; `--interval` floor; `--prompt-file` parsed or removed; a fresh Go per session's prompt; fix the `:36` comment; card dedupe/`<time>`; trailing newline in `gates.json`; unix-only in the README; receipts at the head; correct REPORT measurements.

## History

- deepseek-18 `c04f5bbf8` built the supervisor; the review at `13430063e` accepted the launch, stop, group-signal, lease and card mechanics and required the live check-in, the scoped WIP commit, identity-checked signalling, the watcher liveness fix, child cleanup, the flaky test fix, and a CLI-level planted-fault test.

## Cancel probe (run first)

Class **instrument**. The CHANGES record is counted at the current head; reading `loopdrive.rs:744-756` reproduces that `child.wait()` precedes both observations, and the `loop_drive` test failure at `tests/loop_drive.rs:110` reproduces (the `exit 0` child is gone before `kill -0`). The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- The check-in runs while the child is live on an injectable clock: progressing past 7200 s continues with a note; stalled is terminated and carded; leased sessions are exempt; `stalled_repeats` is counted so a second stall is spiraled.
- WIP commit/push refuses `main` and non-`<lab>/*` branches, stages only the active ORDER's task folder with an `Order:` trailer, and strips `GIT_*`.
- Signalling refuses pgid 1/own group and mismatched identity; the watcher checks pid+identity; a spawn whose record write fails is terminated by its own pid.
- The fake-clock CLI test proves a committing fake continues and a silent fake is "closed and carded"; the flaky record test no longer races; the lows are fixed.
- `AGENTS.md` stays 71 lines; six gates GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- The scoped WIP commit cannot find the active ORDER (then refuse the close action and card instead of guessing a path).
- A real Claude Code after-stop run is needed to prove the exclusion (the structural wait-only-on-tracked-child fix stays).