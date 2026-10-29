# REPORT: infra-loop-harness, claude-173 (D81: integer serving may use the divide instruction; the exporter is the next line)

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: a rule change recorded, no run. The divide's share is an estimate: the RMS norm's long division sits inside the two norms, about 11% of a token at claude-169's 332 µs step. The gain is measured in the follow-up unit.

## What did not work, first

- **The owner-path edits passed outside the loop guard again,** as for D80. One patch (`d81-owner-paths.patch`, in this folder) was applied with `git apply`, under Casey's standing permission for this rule change ("i give you permissing to fix the stuff you need to to make this work"). The patch touches only the three owner files.
- **"Fix the exporter" is a direction, not a result.** The FBS export gap still fails (map stage 10). This unit records the direction. The work is the map's R6 under `d11-exporter-float-trunk`: a deterministic FBS backward, then the gap re-measured with repeats. It is GPU work, with its own sheet, FIT and pre-run review.

## What changed

- **`CHARTER.md` §3.6** reads: "The exported artifact has no float or transcendental instruction (D11 R1–R5). Since D80 it may use the hardware integer multiply, and since D81 the hardware integer divide, so a product or quotient of two integers is exact integer arithmetic in one instruction". Its sources read "D80, D81, owner 28 Oct". No other clause changes.
- **`docs/integration/DECISIONS.md`:** the D81 row, with Casey's words verbatim, the pricing that preceded them, the alternatives (floats too, keep the divide ban) and a low reversal cost.
- **`docs/rules/changes.md`:** the entry "28 October 2026: integer serving may use the divide instruction; floats stay banned (D81)".
- **`PLAN.md`:** the owner direction, `open` until the audit and the kernels allow integer multiply and divide, and until the FBS export gap passes.
- **This folder:** `d81-owner-paths.patch`.

## Measurements

None: no run.

## Checks run at the head

- `uor-harness loop report --check`, `loop pr-check --local` (with owner-paths) and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The change is what the owner chose. Floats, transcendentals and dense matmul stay banned.

## Still open

- **The follow-up serving unit**, which now covers both D80 and D81:
  - correct the docs claude-172's REPORT lists, adding divide;
  - relax `scripts/audit_zero_matmul_serving.py --stack` and the integer crate's checks for integer multiply and divide, still refusing float;
  - use the hardware multiply in the hot products and the hardware divide in the norm;
  - measure exactly against claude-169's 3,042–3,044 tok/s.
- **The exporter:** `d11-exporter-float-trunk`'s next unit, the deterministic FBS backward (R6), then the export gap re-measured.

## Plain summary

Casey also allowed the laptop engine to use the chip's built-in division, while keeping decimals out so every device gives the same answer. He asked for the step that converts the trained model into the laptop model to be fixed next, so the laptop model computes exactly what was trained.
