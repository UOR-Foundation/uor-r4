# REPORT: infra-loop-pr10-supervisor, deepseek-28 (close-out: verify the delivered done-when and flip the item to done)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `fb6df367b`. Item PR; exact-head review at the head.

## What did not work, first

- **The item was left `ready` after its work merged.** #2946 was merged and verified as `56cd6288` (with #2949's follow-up `caacd0aac`), its dependencies are all `done`, and no open PR belongs to it — but the item status stayed `ready`, so `loop next` kept offering it as ready work. This close-out fixes the record, not the code.
- **One done-when clause is proven by construction, not by a real client run.** The after-stop summary exclusion is guaranteed because the supervisor waits only on the child it started and signals only that recorded pid/group; the tests prove the process-group behaviour with shells, not against a live Claude Code run (the original REPORT called this out honestly, and it stands).
- **The lease watcher does not lease pods.** `--watch --lease` runs the caller's renew command on an interval, notes each renewal and stops when that command reports the lease ended; the owner path stays `scripts/pod/uor-pod`. That matches the ORDER's wording and is not a defect.
- **No code change was needed.** Re-running the checks below found no failing clause, so the repair branch of the ORDER's stop condition was not taken.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/roadmap/items/infra-loop-pr10-supervisor.md` | docs | 1 line | `status: ready` → `status: done` |
| `docs/roadmap/README.md` | generated | render | the roadmap counts and row move to done |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-28/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 20 changed lines against the ORDER estimate of about 15 (1.3).

## Measurements — the done-when, clause by clause

1. **Fresh session per unit, one prompt, pid/pgid recorded:** `loop_drive` tests `the_launch_command_gets_exactly_one_prompt_and_not_as_the_program`, `a_real_short_session_is_tracked_and_its_record_read_back`, `the_supervisor_signals_only_its_recorded_session_and_waits_it_out` — 18/18 tests pass at the base.
2. **Own-pid-only stops:** `the_supervisor_signals_only_its_recorded_session_and_waits_it_out` proves only the recorded group is signalled; the kill switch flows through `loopnext::run` (the binary stops on `local/loop/STOP` or `loop:pause`).
3. **Soft D53 check-in:** `the_soft_check_in_continues_real_progress_and_closes_a_stall`, `progress_is_commits_gate_runs_or_edits_and_nothing_else`, `the_fake_clock_continues_a_committing_client_with_a_note`, `the_fake_clock_closes_a_silent_client_and_cards_it` — progress continues with a note, a silent client past the limit closes and is carded.
4. **Watchers:** `the_watcher_outlives_the_run_and_never_cancels_it` and `the_lease_watcher_outlives_the_session_and_stops_when_the_lease_ends` prove both directions; the PM card test asserts the lab, reason, head and needed prompt.
5. **After-stop summary excluded:** by construction (the supervisor waits only on its own child); the limit is recorded above.
6. **Planted fault:** `loop gates --break loop-drive` = **CAUGHT**, red with "a fake stalled client past 2 hours is closed and carded".
7. **Merged evidence:** `loop close --verify --pr 2946` and `--pr 2949` both pass in full on fresh `main` (state, squash, content, done-when, hygiene); dependencies `infra-loop-pr07b-session`, `infra-loop-pr07c-queue-board`, `infra-loop-pr09-client-adapters` all `status: done`.

## Checks run at the head

- `cargo test -p uor-r4-harness --test loop_drive`: 18 passed.
- `UOR_LAB=deepseek uor-harness loop gates --full`: nine GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: nine CAUGHT.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.
- `cargo run -p xtask -- roadmap render --check`: current.

## Skills used

- **solution-integrity**: a status flip is treated as a claim, so each done-when clause was re-run rather than trusted from the merged PR; the one clause that is construction-proof is recorded as such.
- **holding-the-goal**: the repair branch was available if a clause failed; it did not, so the close-out is the honest action.

## Still open

- Nothing for this item. The only `loop next` source still named as not built is `gpu-collections`, which needs a collection record in the watcher (`loopdrive.rs`, outside the umbrella item) — a separate scope decision.

## Plain summary

- **The item is done and now says so.** Its done-when was re-verified clause by clause on fresh `main` (18/18 drive tests, the planted fault CAUGHT, both merged PRs verified, all dependencies done), so the status flips to `done` with the roadmap re-rendered.
- **No code changed and no clause failed.** The one honest limit survives: the after-stop summary exclusion is proven by construction, not by a live client run.