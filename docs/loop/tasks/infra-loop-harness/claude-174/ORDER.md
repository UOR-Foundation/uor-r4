# ORDER: infra-loop-harness, claude-174 (item maintenance: put D80 and D81 into the serving engine)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `e9310cd1f`.

Setup row: 2 (item-file maintenance: a new gap item and PLAN's direction row).

## Goal

D80 and D81 (owner, 28 Oct) let served integer code use the hardware multiply and divide; floats stay banned. The serving engine still enforces the old bans, and docs still state them:
- `scripts/audit_zero_matmul_serving.py --stack` refuses them.
- `crates/uor-r4-integer/src/report_output.rs`' disassembly verifiers refuse `mul`, `sdiv`, `udiv` and the `__udivti3` builtin. The `uor-r4-stack` cost check and `tests/m5_full_path_cost_controls.rs` use those verifiers.
- The rule summaries and model pages listed in claude-172's REPORT still describe the multiplier-free rule as current.

No `ready` item's scope covers these docs and audits. This unit adds the item `integer-mul-div-serving`, whose scope covers them and the FBS kernels. Its units:
1. align the docs and the audits;
2. use the instructions in the hot products and the norm;
3. measure against claude-169's 3,042–3,044 tok/s.

## Cancel probe (run first)

Run at `e9310cd1f`:
- **No item** named `integer-mul-div-serving` exists.
- **No open PR** touches `PLAN.md` or `docs/roadmap/items/`.
- **All 17 scope files exist.**

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `docs/roadmap/items/integer-mul-div-serving.md` (new)
- `PLAN.md` (the D81 direction's row, naming the item)
- `docs/roadmap/README.md` (rendered)
- `docs/loop/tasks/infra-loop-harness/claude-174/**`

## Done when

1. The new item has:
   - the owner's words and the measured gap;
   - what is missing, with sources;
   - a done-when;
   - `How to start`, `Rules that apply` and `Sources`;
   - a checked scope and `status: ready`.
2. PLAN's D81 row names the item.
3. `roadmap render --check` passes. The PR is merged and verified.

No step waits on Casey.
