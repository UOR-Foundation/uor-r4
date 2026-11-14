# Native CUDA kernel roadmap

The discrete path is the default native path because it already carries one integer
representation from training through serving. Work proceeds in dependency order; it does not
start a new mechanism while an existing transition lacks direct parity evidence.

## 0. Completed

- Inventory the native state and identify the F32 export gap.
- Compare CPU training to integer serving over two fixtures.
- Compare CUDA forward, evaluation, counterfactual losses, chooser, apply and post-apply state
  to the CPU and serving words on the RTX 4090.

## 1. Direct route and read trace

Add a read-only serving trace for:

- the route port selected by each block and stage;
- the Flock/Relation kept-node list.

Acceptance: every single port or kept-list fault changes a compared trace word; the existing
144 route faults and five gate faults that escape output-only checks become visible.

## 2. Loader symmetry

Make serving refuse a Route artifact with non-zero gates, matching the CPU training loader.

Acceptance: one malformed artifact is refused by training and serving at load, before any row
executes.

## 3. Shared typed operators

Give CUDA, CPU and serving one source definition for the operators each calls:

- signed-axis action;
- composition;
- `root_cost` with pinned arithmetic bounds;
- `snap_n`;
- `checked_scaled_add`.

Acceptance: a planted change to one operator definition turns the same transition red on every
backend; no backend keeps a hand-copied rule.

## 4. Exact update kernel parity

Extend the current proposal/apply leg to Table, Route and Gate moves plus status refusals:

- stale epoch 3;
- bad index 4;
- non-improvement 5;
- mode mismatch 9.

Acceptance: CPU and CUDA return the same status and leave identical table, route, gate and
epoch words.

## 5. Generation and stop state

Add the missing decoder loop and exact stop/carry state in the same integer representation.

Acceptance: a finite prompt replay produces identical prompt carry, route decisions, emitted
tokens, stop reason and final state through training simulation and serving.

## 6. Memory and conversational carry

Bring exact copy and turn-memory into the discrete native path.

Acceptance: memory key, address, read choice, copied value and resulting token are compared as
raw words, with empty-table and matched-shuffle controls.

## Current smallest next dependency

Step 1: expose and compare the route-port and kept-node words. No generation, training or
performance claim is required to finish it.
