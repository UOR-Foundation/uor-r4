# REPORT: one-model-end-to-end, codex-26 (the read's margins)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-25](../codex-25/REPORT.md).

## What did not work, first

- **The owner's "averaging" is not in this read.** The Q31 first-pick read returns one pick; the
  equal mean of four picks that claude-387 measured is the float vector read's kept-cell payload.
  The Q31 read's write is `weights ⊙ table row` with a declared `gain_shift`; the write's problem is
  its **magnitude**, not a mean.
- **The margin does not change a pick.** It is a reading only; the picks are `apply`'s.
- No capability claim: this unit measures entropy, not recall.

## What changed

- `crates/uor-r4-native-vector/src/read.rs`: `PickMargin` and `Q31FirstPickRead::margins`, with a
  test (picks equal `apply`; a tie is zero margin and goes later; one candidate has no second).
- `crates/uor-r4-integer/src/bin/read-panel.rs`: the margins per scored row, per bucket and overall.

## Measurements

The same read (the ext-s1 artifact's), the same 1,024 scored rows, two streams:

| Stream | Mean margin | Zero-margin rows | No-second rows | Asked picks |
|---|---|---|---|---|
| float replay (`dump-ext-s1-s24`) | **479,134,037** | 0 | 0 | 440 |
| the served FBS stream (`fbs-ext-s1.dump`) | **714,606** | 0 | 0 | 440 |

**The served stream's address margin is 670× smaller than the float replay's** (714,606 against
479,134,037, a ratio of 0.00149 ≈ 2^-9.4). No row in either stream ties, so the picks agree; what
differs is the **entropy** each address carries — exactly the leading indicator the literature names.
This is the same defect codex-20 measured as a sign agreement of 0.887 and claude-390 as a residual
at 2^18.5 against gates at 2^31: the integer address is computed at the bottom of the domain, so its
nearest/second gap collapses.

## Checks run at the head

- `cargo test --locked -p uor-r4-native-vector` — 49 passed.
- `read-panel` on both dumps — the table above.

## Still open

- Whether raising the address's scale (or the write's) raises recall: the write sweep needs the
  decode instrument (claude-387), not the panel.
- The transport quantities as key components (the owner's guess): the inventory confirms they are
  computed and discarded; harvesting them is its own unit.
