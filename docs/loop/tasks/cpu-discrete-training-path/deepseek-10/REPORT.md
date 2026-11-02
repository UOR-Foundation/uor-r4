# REPORT: repair apply/chooser state coherence, deepseek-10

Lab: deepseek. Item: [cpu-discrete-training-path](../../../../roadmap/items/cpu-discrete-training-path.md) (#3501). Base: `4fdec6487`. Repair of [deepseek-9](../deepseek-9/ORDER.md).

## What did not work

- **The first apply review found stale chooser state.** Route/Table mutation updated the scoring model but left chooser maps/codebooks at their pre-apply values.
- **The first mutation tests were too weak.** They did not pin the exact Table root transform or verify chooser synchronization.

## What changed

- Successful Route apply now flattens the updated route maps into chooser state.
- Successful Table apply now copies the updated codebook into chooser state.
- Tests pin exact Route composition and Table root movement, chooser synchronization, refusal immutability and epoch-`u64::MAX` refusal.
- Sequential Route and Table controls now choose the next equal-loss tie from the updated chooser maps/codebooks.

## Measurements

- Focused apply/no-op and exact mutation tests pass after state synchronization.
- Exact mutation tests pin sequential chooser state, Table root transform, Route composition, refusal immutability and epoch-max refusal.

## Checks run at the head

- Focused release apply/mutation tests pass after known duplicate-artifact collision retries. Full repaired-head verification remains pending.

## Skills used

No repository skill was invoked. The repair follows the reviewer's source-cited findings.

## Still open

- BVG-K joint application.
- Deterministic cursor, checkpoint and restart/resume.

## Plain summary

This repair keeps apply and chooser state coherent without widening into joint commits or persistence.
