# ORDER: infra-loop-harness, kimi-4 (scope deltas for the shared-nine decision)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `940fc1730`.

## Done in kimi-1..3 (history, merged as `940fc1730`)

- kimi-1: scopes on all 51 ready items, seven `Closes` steps; commits `add1b4ea7`, `952a6e6ba`, `fa28f9339`.
- kimi-2: un-serialized scopes, owner-path notes, real issue numbers; commits `92e0160ae`, `f1ae1f70b`, `4f8314027`, `a52e7e62c`.
- kimi-3: round-2 deltas and the one-ORDER fold; commits `b5573967f`, `108b21179`. Counted D59 APPROVE at `108b21179`; merged `940fc17303c429503ab50983218382e4a9c70644`; verified on fresh main; sync applied.

## Goal

Apply claude's kimi-4 list from #2028 (the shared list is now nine after Casey's 00:05Z decision; claude-22 builds the exemption into `pr-check` and the validator):

1. pr11: add `crates/uor-r4-harness/src/looparrive.rs` and `crates/uor-r4-harness/tests/loop_arrive*.rs` (raising arrive's warnings to errors changes them).
2. pr12: add `rules/*-hook.sh` and `crates/uor-r4-harness/tests/git_hooks*.rs`.
3. infra-loop-harness (umbrella): add the finished loop items' modules — `crates/uor-r4-harness/src/loopreview.rs`, `src/loopnext.rs`, `src/loopreport.rs`, `src/loopclose.rs`, `src/loopclaim.rs` and `crates/uor-r4-harness/tests/loop_review*.rs`, `tests/loop_next*.rs`, `tests/loop_report*.rs`, `tests/loop_close*.rs`, `tests/loop_claim*.rs`, `tests/loop_repair_orders*.rs` — and drop `docs/roadmap/README.md` (now shared; no item scope lists it).
4. Verify no scope lists any of the nine shared files.

## Cancel probe (run first)

Does any scope still miss claude's kimi-4 additions or list one of the nine shared files? Result at `940fc1730`: yes — pr11 lacks the arrive modules, pr12 lacks the hook scripts and git_hooks tests, the umbrella lacks the finished modules and still lists `docs/roadmap/README.md`. The task stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-pr11-progress-window.md`, `docs/roadmap/items/infra-loop-pr12-git-hooks.md`, `docs/roadmap/items/infra-loop-harness.md`
- `docs/loop/tasks/infra-loop-harness/kimi-4/**`

## Done when

- The three items carry the additions, `docs/roadmap/README.md` and the other eight shared files are in no scope (`grep` clean), `xtask roadmap check` and `render --check` pass, `loop pr-check --local` fails no rule except any still-expected scope-at-base note, the REPORT is written, and a review request naming the head is posted on #2028.

## Stop and ask if

- The shared list changes again mid-task, or any file outside the list above needs to change.
