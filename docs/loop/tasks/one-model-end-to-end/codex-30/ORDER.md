# ORDER: one-model-end-to-end, codex-30 (a k-term Cl(0,6) projection: the capacity lever)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parents: [codex-29](../codex-29/REPORT.md), [the zeta re-evaluation](../../../../labs/one-model-end-to-end/ZETA-RING-REEVALUATION-2026-11-16.md).
Owner: "can we build that thing the way we need to take advantage of Cayley–Dickson … it's rather
worth testing."

## What is already true

Every projection in the served trunk is **already** a Cl(0,6) signed permutation:
`stack_fbs_gather` (`session.rs:4147`) — "output block `o`, channel `j` is the input value its code
names in atom `atoms[o]`, negated when the code says so … a gather and a sign: no weights." The
planted-symmetry probe (`tests/sh_spin_probe.rs`) learns a hidden five-fold relation at **1.000**
held-out against popcount **0.133** and L1 **0.200**, so the structure can carry such a relation.
There is no matmul to replace.

## The ceiling, and where it is enforced

Each output block reads **one atom** through **one** Cl(0,6) state. The packed table is one `u32`
per output block, `atom | (state << 16)` (`pack_fbs_routes`, `format.rs:2158`). The exporter refuses
a block that reads two atoms:

> `block {o} reads two atoms: the projection is not a signed gather` (`fbs_export.rs:157`)

That refusal is the capacity ceiling, and it is explicit, not incidental.

## Design (k terms, still multiply-free)

Let `k` be the number of terms per output block (1..=4; k = 1 is today).

1. **Integer reader** (`format.rs`, `session.rs`): the packed map table is `out_blocks * k` words;
   the record carries `terms` (absent means 1). `FbsMap` holds `atoms[o][t]`, `codes[o][t]`;
   `stack_fbs_gather` sums `k` gathers and `k − 1` adds into the block, saturating at the residual
   exponent. k = 1 must produce **byte-identical** tables and the same served rows.
2. **Exporter** (`fbs_export.rs`): `coded_projection` allows up to `k` distinct atoms per block;
   a block with more than `k` is still refused by name. The snapshot's `FbsRoutes` carries `terms`.
3. **Trainer** (`geometric_stack_native.rs`): `FbsRoutes { atoms, states }` becomes
   `atoms: Vec<[u16; k]>`, `states: Vec<[u8; k]>`; the secant surrogate credits each of the 2k
   tables; a block's terms start at the k=1 solution (term 0) and zero elsewhere, so a resumed
   checkpoint is a valid starting point.
4. **Rate**: the gather is `k` reads and `k − 1` adds per block against a trunk that is **98.6 %** of
   a served token (`claude-158`), so even k = 4 stays far below a dense projection.

## Done when

1. A unit test serves a hand-built k = 2 map as the exact sum of its two gathers, and a k = 1
   artifact's bytes and served rows are unchanged.
2. The exporter writes and refuses k correctly (a 3-atom block under k = 2 is refused by name).
3. The trainer trains k = 2 and k = 4 at the p recipe and exports them; the k = 1 arm is unchanged
   byte for byte.
4. The reading: k = 2 and 4 against k = 1 on the recall panel, held-out values, with tok/s.

## Cancel

Stop if a k = 1 artifact changes a byte or a served row, or if a k-term forward fails its
dense-reference fixture. Do not run the arm before that fixture passes: a capacity change on a
broken mixer measures nothing.
