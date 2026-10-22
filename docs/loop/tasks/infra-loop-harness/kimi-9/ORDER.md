# ORDER: infra-loop-harness, kimi-9 (PM bookkeeping: m2-reply-grading-instrument-r14 done)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Base: `e5de1b471`.

## Goal

PM record repair in the umbrella's lane: set
[m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md)
`status: ready` → `status: done`. The item's done-when is met: the R14 scope note
is posted on #2029 (comment 6338175116), the missing abstention-as-discrimination
part is built and tested (#2956, merged as `ec980ee57`), the note and instrument
have non-author reviews (deepseek's counted APPROVE at `494bad222`; kimi's
independent re-verification at the same head), the lab record's link defects are
repaired (#2961, merged as `e5de1b471`), and the PM ruled on R14 (#2029 comment
6339245936, 2026-10-22 11:07Z: the instrument PASSES R14, non-vacuous).

## Cancel probe (run first)

At base `e5de1b471`: the item file still says `status: ready`; #2956 and #2961 are
both MERGED; the ruling is posted. Verified by reading the item file and the two
PR states. The task stands.

## Files in scope

- `docs/roadmap/items/m2-reply-grading-instrument-r14.md` — the only file this task changes.

## Done when

- The item's `status:` reads `done`; no other line of any file changes.
- `uor-harness loop gates --quick` and `--full` pass at the PR head.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the PR head.
- `cargo run -q --locked -p xtask -- roadmap check` passes; `roadmap render` moves the item under done.
- After merge, `roadmap sync --apply` closes #2852 (the PR body also carries `Closes #2852`).

## Stop and ask if

- Any file other than the one item file must change.
- Evidence appears that any part of the item's done-when is unmet.
