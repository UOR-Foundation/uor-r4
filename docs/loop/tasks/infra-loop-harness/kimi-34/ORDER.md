# ORDER: infra-loop-harness, kimi-34 (record the position premise probe)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `742bf59bc`.

## Goal

Record the answered position finite-control probe in PLAN so `loop next` stops offering it again. Close only that premise row, with a decision row preserving the pending registered order test and the exact finite-control scope. This is PM recordkeeping, not a mechanism or milestone completion claim.

## Cancel probe (run first)

At `742bf59bc`, PLAN's position premise is `open`, although `cargo test -p uor-r4-discrete --lib position_mechanism` has run at `b03d08e85` and passed 10/10. The result is recorded on #2843 (`6380719988`). Repeating the same probe adds no information, so the status record still stands.

## Files in scope

- `PLAN.md`
- `docs/integration/DECISIONS.md`
- `docs/loop/tasks/infra-loop-harness/kimi-34/**`

## Done when

- The position premise row is `closed`, dated 2026-10-24, and its probe text states the finite-control answer and the still-pending registered order test.
- A new append-only decision row gives the evidence, exact non-claims and reopening cost.
- No other premise, bar, hold, milestone or roadmap-item status changes.
- `loop next` no longer offers the position probe; report/local PR checks and diff-check pass.

## Stop and ask if

The probe result would be interpreted as language/order-effect evidence, a bar or hold must change, or any file outside scope is needed.
