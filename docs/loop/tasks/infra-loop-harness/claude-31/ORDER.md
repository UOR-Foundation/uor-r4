# ORDER: infra-loop-harness, claude-31 (no idle stops: WAIT, a polling `loop next --wait`, and a drive that re-polls)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `4bb1feede`.

## Goal

Labs park with nothing to wake them. Since the joins on 21 Oct, the longest silences on #2028 were Kimi 5.5 h, Claude 4.3 h, Antigravity 2.8 h and DeepSeek 2.3 h. Three code paths cause it:

1. `loop drive` exits on any STOP (`loopdrive.rs:676-678`). The loop page promises "it then polls GitHub every 10 minutes", but that poll was never built.
2. `loop next` has no WAIT state. A lab whose open PRs all await review hits `open_pr_limit` (`loopclaim.rs:715-728`). Every claim is refused, and the lab prints `STOP held work` or `STOP empty queue` (`loopnext.rs:635-640`), with the real reason only in a detail line and no re-check time.
3. The docs tell a lab to stop on a pending owner decision ("A pending owner question means wait", `AGENTS.md:70`; loop page "Stops"), with only "The PM prompts any lab that stopped" to resume it.

The planted fault: **a lab whose open PRs all await review is told to STOP, with no time to look again.**

## Cancel probe (run first)

At `4bb1feede`, does `loop next` print anything other than `GO …` or `STOP …`? Does `loop drive` sleep and re-check after a STOP? If either is true, the task changes. Class: object.

## Calls

- **`loop next` outcomes.**
  - `STOP` is kept only for the kill switch: the file or the `loop:pause` label.
  - Every other no-work case prints `WAIT <reason> — check again in <n>s`, exits 3, and names what is pending:
    - the lab's own PRs awaiting review, with their numbers, when `open_pr_limit` refused claims;
    - held items;
    - items waiting on dependencies;
    - an empty queue.
  - The default re-check is 600 s.
- **`loop next --wait [--poll SECS] [--max SECS]`.** It runs `loop next` repeatedly.
  - It ends on `GO` (exit 0) or `STOP` (exit 1).
  - On `WAIT` it sleeps `--poll` (default 600) and asks again, until `--max` elapses (exit 3, with the last WAIT line).
  - A transient read failure (exit 2) is printed and retried at the next poll. Three in a row end with exit 2.
  - The point is that any client can hold one command open, within its own command limit, instead of ending its session.
- **`loop drive`.** On `WAIT` it sleeps `--poll` (default 600) and asks `loop next` again instead of exiting. `STOP` still ends it. The default prompt's last line tells the session to run `loop next` again after requesting review, and to end only on WAIT or STOP.
- **Docs.**
  - The loop page's "Stops" becomes "Stops and waits": only the kill switch stops. A lab never ends a session on WAIT without a timer; it holds `uor-harness loop next --wait --max <its command limit>` or arms its client's own timer. A pending decision card or owner question parks only its own item.
  - `AGENTS.md:70` becomes: "A pending owner question parks only its own item: take other claimable work meanwhile, and do not act on your own reading of it."
- **Gate.** A `docs/harness/gates.json` entry, following the `loop-drive` gate's pattern, whose planted fault is a lab at `open_pr_limit` with all its PRs awaiting review. The gate is green when `loop next` prints WAIT with a re-check time, and red when it prints STOP.

## Owner approval

`AGENTS.md` is an owner path, so this ORDER carries the owner's approval. Casey's direction in Claude's chat at 2026-10-22T13:50:03Z, recorded verbatim on #2028 (comment 6342668364):

> "can you fix the loop harness errors because they are causing the labs to stop their loops and wait without timers to resume so they become stale... I don't want everything to get piled up behind a single review"

The one `AGENTS.md` line this ORDER changes is the "pending owner question means wait" rule that parks a whole lab. It keeps that rule's other half: never act on your own reading of the question.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`, `crates/uor-r4-harness/tests/loop_next*.rs`
- `crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, and `AGENTS.md` (in the umbrella's scope once claude-29, #2971, merges)
- Shared: `crates/uor-r4-harness/src/{main.rs,doccheck.rs,loopgates.rs}`, `docs/loop/README.md`, `docs/harness/gates.json`
- `docs/loop/tasks/infra-loop-harness/claude-31/**`

## Done when

- A unit or integration test shows a lab at `open_pr_limit`, with PRs awaiting review, gets `WAIT` naming those PRs, plus a re-check time and exit 3. **With the fix set aside, the same test fails** (it sees STOP).
- `loop next --wait` returns GO as soon as work appears, and exits 3 after `--max`. The tests use an injected clock or sleeper with no real sleeping.
- `loop drive` re-asks after WAIT and still exits on the kill switch, tested with the existing fake-client harness.
- `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings` passes, and so do the whole `cargo test -p uor-r4-harness`, `loop gates --full` (with `--break all` CAUGHT), `loop report --check` and `loop pr-check --local`.

## Stop and ask if

- Exit code 3 is already used by `loop next` or by any caller of it.
- A client's command limit makes `--wait` unusable for some lab (say which).
