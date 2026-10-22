# ORDER: infra-loop-harness, claude-28 (one scope entry so #2949 can fix main's stale harness test)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `55311a46b`.

## Goal

`main` fails `watch_needs_a_full_run_and_other_actions_are_refused` (`crates/uor-r4-harness/tests/loop_gates_run_rules.rs:330`). The test used `loop drive` as its example of an unbuilt action, and loop PR 10 (#2946, `56cd628`) built `loop drive`. DeepSeek's follow-up, #2949, has the fix, but no item's `scope:` covers that test file, so DeepSeek stopped and asked (card 6336226999 on #2028).

Casey's answer, in Claude's chat on 22 Oct: "Extend PR 10's scope (Recommended)". This is option (a): add the one file to `infra-loop-pr10-supervisor`'s `scope:`. It is the PM's edit, made at the owner's direction.

The planted fault: **a test that names a built action as unbuilt sits outside every item's scope, so no PR may fix it.**

## Cancel probe (run first)

At `55311a46b`, does any item's `scope:` already match `crates/uor-r4-harness/tests/loop_gates_run_rules.rs`? If one does, the task is void, and #2949 proceeds under that item. Class: instrument.

## Calls

- `infra-loop-pr10-supervisor`'s `scope:` gains exactly `crates/uor-r4-harness/tests/loop_gates_run_rules.rs`. Nothing else in that item changes.
- The test fix itself stays in #2949. `pr-check` reads scope at the base, so this entry must be on `main` first.

## Files in scope

- `docs/roadmap/items/infra-loop-pr10-supervisor.md` (`docs/roadmap/items/*.md` is in the umbrella's scope)
- `docs/roadmap/README.md` (shared), only if the render changes it
- `docs/loop/tasks/infra-loop-harness/claude-28/**`

## Done when

- `cargo xtask roadmap check` passes, and the render reproduces the README byte for byte.
- The item's `scope:` equals the old list plus the one path.
- `loop report --check`, `loop pr-check --local`, claim-wording and the copyright check all pass.

## Stop and ask if

- `roadmap check` refuses the entry.
- Any other item's text would have to change.
