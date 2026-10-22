# ORDER: infra-loop-pr10-supervisor, deepseek-28 (close-out: verify the delivered done-when and flip the item to done)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `fb6df367b`. Item PR (close-out of the setup row already built); other-vendor exact-head review at the head.

## Goal

`loop next` keeps offering this item as ready work although its done-when was delivered by #2946 (`56cd6288`, merged and verified) plus the #2949 follow-up (`caacd0aac`), and its three dependencies are done. Close the bookkeeping honestly: re-run the done-when evidence on fresh `main`, and flip the item to `done` only if every clause holds. No code change is expected.

## Cancel probe (run first)

Class **instrument** (status close-out; no code, no GPU). At `fb6df367b`: the item is `status: ready` while #2946/#2949 are merged and verified and `infra-loop-pr07b-session`, `infra-loop-pr07c-queue-board` and `infra-loop-pr09-client-adapters` are all `done`; `loop next` reports the item as ready and claimable. Re-running each done-when check below is the work.

## Files in scope

`docs/roadmap/items/infra-loop-pr10-supervisor.md` (`status:` line only), `docs/roadmap/README.md` (rendered), and this task folder.

## Done when

- Every done-when clause is re-verified on this base and recorded in the REPORT: `loop drive` starts a fresh session per unit with one `{prompt}` and records pid/pgid; stops only its own recorded pid/group; the soft D53 check-in continues real progress with a note and closes a spiralled/stalled session; both watchers outlive the session (and the lease watcher stops when the lease ends) and never cancel a healthy run; the after-stop summary is excluded by construction (the supervisor waits only on its own child); the `loop-drive` gate's planted fault is CAUGHT with "a fake stalled client past 2 hours is closed and carded"; the PM card names the lab, reason, head and needed prompt.
- The item's `status:` flips to `done` and the roadmap render is current; `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head.

## Stop and ask if

- Any done-when clause fails on this base: then the item is not done, and a repair ORDER replaces this close-out.