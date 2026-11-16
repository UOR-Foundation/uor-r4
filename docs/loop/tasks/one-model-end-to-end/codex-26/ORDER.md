# ORDER: one-model-end-to-end, codex-26 (the read's margins, and the address entropy they expose)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-25](../codex-25/REPORT.md). Owner direction: fix the read, keep the margins.

## Why

Decision 12 names two defects: the input's separation and the write's scale. The write's first half is
measurable only if the read's **margins** are kept — today `first_pick` reduces the 16-lane distances
to an argmin and throws the second away, so no one can say whether an address had any entropy. The
retrieval literature's cheapest leading indicator is exactly that margin.

## Design

- `Q31FirstPickRead::margins(stream, ids, table, batch, time) -> Vec<PickMargin>`: the same
  addressing as `apply`, no payload written, nothing trained. `PickMargin { pick, best, second }`
  with `margin()` and `zero_margin()`.
- A test pins that the margins' picks are `apply`'s picks row for row, that a tie has zero margin
  and goes to the later row, and that a row with one candidate has no second distance.
- `read-panel` reports the margins for every scored row: count, sum, zero-margin count, no-second
  count, per bucket and overall.

## Done when

1. The margin test passes and the picks agree with `apply`.
2. The panel reports margins on the float dump and on the FBS dump.
3. The two readings are stated, with their ratio.

## Cancel

Stop if the margins' picks differ from `apply`'s on any row.
