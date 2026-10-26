# ORDER: m1-address-as-read, kimi-41

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `c947dc76d`.

## Goal

Turn the Stage 0 `COPY_GATE_FAIL` into an implementation-ready integer-read successor: name the served integration, exact-address oracle boundary, versioned emission controls and falsifying tests, and widen the item's declared scope so a later unit can implement it. This unit changes design and item metadata only; it runs no model evaluation and says `References #2833`.

The item's done-when remains:

> A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## Cancel probe (run first)

Stage 0 at #3273 reproduced 14,287/14,287 D11 parity and 0/40 exact copy in every arm. Repeating that configuration cannot add evidence. Source inspection finds sorted-merge GCD storage in `uor-r4-integer::m6_ablation`, but no served model read that can emit a stored value. The missing next dependency is an integer integration, not another panel. `classification: instrument`.

Claude-131's open ORDER reserves `crates/uor-r4-integer/**`, so this unit does not edit that crate. It produces the implementation design and item-scope repair only; implementation proceeds under a later ORDER after the scope is free. **Not cancelled.**

## Files in scope

- `docs/labs/m1-address-as-read/**`
- `docs/mechanisms/exact-memory.md`
- `docs/loop/tasks/m1-address-as-read/kimi-41/**`
- `docs/roadmap/items/m1-address-as-read.md`
- `docs/roadmap/README.md` (rendered only)

No `crates/uor-r4-integer/**` edits in this unit. The proposed item-scope addition is metadata for a later unit and requires the D68 decision review.

## Done when

- A design names the concrete integer integration points, artifact/version identity, exact-address and no-memory paths, value-emission boundary, bounded-resource accounting and failure behavior.
- The oracle protocol keeps no-hint and random-hint controls, separates address formation from read accuracy, and has finite host controls plus a GPU non-vacuity gate before any hidden-fact panel.
- The item scope admits the later integer implementation, with a whole-project D68 review recorded on #2833; `status: ready` and the item's done-when stay unchanged.
- Full gates, REPORT validation, local PR policy and a fresh exact-head review pass. No GPU run, code capability claim or `Closes #2833`.

## Stop and decide if

- The design would merely bypass the model and call a store lookup a model gain; redefine the oracle boundary instead.
- It requires dense matmul, softmax/squashing, host model evaluation, an unversioned artifact change, or an unbounded scan.
- It needs code in Claude's reserved scope now; leave that code to the later unit rather than overlapping.
- The Stage 0 evidence cannot distinguish address-oracle assistance from unaided generation; keep the controls and non-vacuity gate explicit.
