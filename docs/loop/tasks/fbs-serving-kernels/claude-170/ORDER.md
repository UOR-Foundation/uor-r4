# ORDER: fbs-serving-kernels, claude-170 (decision: the exact kernels meet the item; set it done)

Lab: claude. Item: [fbs-serving-kernels](../../../../roadmap/items/fbs-serving-kernels.md). Base: `c910dd7e5`.

## Goal

The owner, 28 Oct 2026, in claude's chat: "I don't care what we serve at long as it is cheap". This is the map's R3.

claude-163 to claude-169 took the served FBS token on the M1 from 270 to 3,042–3,044 tok/s (11.3×), with every reply byte-identical. Done-when 1 (the split) is met. Done-when 2 and 3 name **a vector kernel** behind a runtime switch, written before the split showed where the cost was. Five exact scalar changes, each found by a profile, delivered the speed instead:
- the sign-compatible snap (claude-164);
- the bounded `ℤ[φ]` comparison (claude-165);
- shared Hamilton tables (claude-166);
- four-guard multiples tables (claude-167);
- the snap's scores by additions (claude-168);
- the Hamilton product by shifts (claude-169).

**This unit is a decision.**
- **The amendment:** done-when 2 and 3 are amended to the intent they serve, "a faster exact kernel for the largest part", vector or structured, measured with and without.
- **The status:** the item is set `done`.
- **The vector kernel** is recorded as an open follow-up, not a bar.

**Why.** The item's purpose is a cheaper served token, measured exactly. The structured kernels give that at 11.3× and pass every exactness and audit check the done-when asks of a vector kernel. Meanwhile the serving path's larger gaps are correctness:
- the FBS export gap;
- the integer read and memory (`integer-hamming-read`, `turn-memory-integer`);
- the writer (`writer-admission`).

Keeping this item open for a vector kernel would hold the serving files' lock against those.

A change to a done-when and a status is a decision. It goes ahead on a fresh decision review (CHARTER §5.1, D67) and is logged for Casey on #2028, open to reversal. The item has no issue of its own.

## Cancel probe (run first)

Run at `c910dd7e5`:
- **No open PR** touches the item file.
- **The six results are merged:** #3351, #3352, #3353, #3354, #3355, #3356 and #3357, for claude-163 to claude-169.
- **The item reads** `status: ready` with done-when 2 naming a vector kernel.

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `docs/roadmap/items/fbs-serving-kernels.md`
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/fbs-serving-kernels/**`

## Done when

1. The item's done-when 2 and 3 read the amended bar. Its status is `done`, with a note naming the units that met each done-when and the open vector-kernel follow-up. The roadmap is re-rendered, and `roadmap check` passes.
2. A fresh decision review reads the item, this ORDER, claude-163 to claude-169's REPORT headlines and the map's R3, and approves or rejects.
3. An owner log on #2028 records the decision and its review.

## Stop and decide if

- **The decision review rejects.** Then the item stays `ready` with its bar, and the next serving unit is the vector kernel.

No step waits on Casey.
