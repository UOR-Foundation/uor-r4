# ORDER: integer-long-context, claude-286

Lab: claude. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md). Base: `e06635e3a`.

## Goal

The item's done-when reads:

> 1. **The decision,** recorded by a decision review on this item's issue (`[claude] owner log:`): which rule, and why, read from the measurements in 2.
> 2. **The measurements,** under a prep sheet with a fresh FIT and a pre-run review: on one trained checkpoint, the float model's held-out loss and greedy reply agreement past 192 tokens under each rule, on multi-turn prompts whose context passes 192. A rule needing a new training arm gets that arm.
> 3. **The integer session serves the chosen rule,** with a test against the float forward's rule past 192, and a negative gate. A session within 192 tokens decodes byte for byte as at the base.
> 4. **Tokens per second against context** out to at least 2,048 tokens, with the resident memory.
> 5. **One notebook row, and a REPORT with the negatives first.** The pipeline map's gap 6 records the result.

## Cancel probe (run first)

To fill in.

## Files in scope

- `crates/uor-r4-integer/src/stack/session.rs`
- `crates/uor-r4-integer/src/stack/mod.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/bin/fbs-toks.rs`
- `docs/model/pipeline-map.md`
- `docs/harness/prep/integer-long-context*`
- `docs/roadmap/items/integer-long-context.md`
- `docs/loop/tasks/integer-long-context/**`
- `docs/research/notebook/m4-serving.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
