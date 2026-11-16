# ORDER: one-model-end-to-end, codex-28 (the relative-offset question key, trained)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-27](../codex-27/REPORT.md). Owner direction: the first reply token; the literature's
relative-offset key.

## Why

codex-27 traced the first reply token: the copy heads stamp the format bigram (`':' → ' The'`) at
full weight; the depth-2 ask head keeps the assert position (0.261) but **no readout-depth head
keeps it**; the value arrives four rows later from the memory line. The literature's answer is a
relative-offset question key, and the code already has one:

- `HammingReads::with_carry(rows, vocab)` (claude-335) adds `Σ_{s=1..rows} c[x_{t−s}] ·
  (point(x_{t−s}) + a[x_{t−s}])` to the readout read's asked point, with a per-token gain `c`
  starting at zero — a learned relative-offset question key.
- The trainer exposes it as `--hamming-carry-question H` (needs `--hamming-ask 1`).
- The export writes `hamming.d{depth}.carry` and the record's `carry_rows` (`format.rs:2815`), and
  the served `AskImage` applies it (`hamming.rs:99-110`, Q16, reach ≤ `ASK_CARRY_MAX`).

**It has never been in the p recipe.** The recipe carries the place read instead
(`--hamming-place KIT --hamming-place-pull 1`), and the trainer refuses the two together.

## Design

One arm, the p recipe with the ask heads and the carry, and **no place read**:

```
--hamming-ask 1 --hamming-carry-question 2        (in place of --hamming-place …)
```

The arm is the literature's relative-offset key at the readout depth; the existing arm r (the place
read plus a memory line) is its comparator. The falsifier must be applicable: score a **bare-value**
anchor — a question whose reply is the value alone, with **no memory line** — on **held-out values**
(the pure-retrieval cell), so copying a TRAIN reply cannot pass.

## Done when

1. A 64-step smoke trains and exports with `carry_rows` in the record and `hamming.d*.carry` in the
   tables, and the artifact parses and serves.
2. The trace (codex-27's instrument) on the smoke artifact shows whether the readout's ask head
   keeps the assert position, and at what weight.
3. Only then the full arm: 16,384 steps, batch 16, context 256, seeds 1–2 on wick (~1.4 h a seed).
4. The reading: bare-value anchors, no line, held-out values, against arm r and against the
   no-carry control.

## Cancel

Stop if the smoke's artifact does not carry `carry_rows`, or if the served read ignores it. Do not
score a panel with a memory line: that would measure the line, not the key.
