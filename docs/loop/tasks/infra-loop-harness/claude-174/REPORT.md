# REPORT: infra-loop-harness, claude-174 (item maintenance: put D80 and D81 into the serving engine)

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: item-file maintenance, no run.

## What did not work, first

- **The bans are enforced in more places than one script.**
  - `report_output.rs`' disassembly verifiers refuse `mul`, `sdiv`, `udiv` and the 128-bit division builtin `__udivti3`. The `uor-r4-stack` cost check and `tests/m5_full_path_cost_controls.rs` use them.
  - Other crates hold zero-multiply checks too: `uor-r4-core`, `uor-r4-graph-certify`, `uor-r4-wasm-router` and `uor-r4-api`. Those guard other engines, which need no multiply, and can stay strict.
  - The new item covers only the checks on the FBS stack's serving step.
- **CI runs neither audit script** (no match in `.github/workflows`). The audits bind only when a unit runs them. The item's done-when keeps them in each unit.
- **The D11 entry's "amended by" pointer** sits in an owner path. The item says it goes by patch under the owner's permission, as D80 and D81 did.
- **The first two drafts' scopes were incomplete**, caught by the fresh review:
  - **Round 1:** missing `scripts/audit_zero_matmul_wasm.py` (it counts `i64.mul` and `i64.div_u` in the browser build of the same step and expects zero), `docs/mechanisms/README.md` and `docs/model/gpu-and-cuda.md`. The draft's norm share (11%) also had no source; claude-169's profile gives 43 of 940 samples.
  - **Round 2:** missing `docs/contributing/HARD-RULES.md` and `docs/geometry/README.md`, which still state the rule, plus several summaries. Three `ready` items also kept "multiplier-free" as a rule.
  - **My own sweep then found 226 matching lines in about 100 files.** Most describe other engines, describe code that still has no multiply, or are dated history. The item now lists the 16 pages that state the rule or report a zero-multiply audit as current, and it names the groups left out on purpose. Both rounds' findings are fixed at this head.

## What changed

- **`docs/roadmap/items/integer-mul-div-serving.md` (new, `ready`, area serving, `done_class: host`):** the owner's words; what the old rule cost; the audits and docs still stating it; and a four-step done-when.
  1. Docs and audits (native and WASM) agree with §3.6. A float instruction still fails each verifier, and integer `mul` and `udiv` now pass, shown by the Rust tests and a `--self-test` in each audit script.
  2. The kernels use the instructions, each exact against the old one, with byte-identical replies.
  3. Speed against claude-169's sealed binary.
  4. A notebook row and a REPORT per unit.

  Its scope lists 36 existing files, every one present on `origin/main`, plus the item file and its task folder.
- **Three `ready` items' rule lines** (`fbs-serving-vector-kernel`, `integer-hamming-read`, `turn-memory-integer`) now cite §3.6 as amended. The hardware integer multiply and divide are allowed once this item's audits allow them, and the multiplier-free forms apply until then.
- **`PLAN.md`:** the D81 direction's row names the item.
- **`docs/roadmap/README.md`:** re-rendered.

## Measurements

None: no run.

## Checks run at the head

- `roadmap check`, `render --check`, `loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The item carries out D80 and D81 as worded. Floats and dense matmul stay banned, and the stricter checks of other engines stay.

## Still open

- **The item's first unit:** the docs and the audits.

## Plain summary

Casey allowed the laptop engine to use the chip's built-in multiply and divide. The safety checks and several documents still forbid them, so this adds a work item to update those first, then switch the engine over and measure the speed.
