# ORDER: infra-loop-harness, claude-173 (D81: integer serving may use the divide instruction; the exporter is the next line)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `b4f1fed39`.

Setup row: 2 (an owner rule change recorded: charter text, decision index, change log, PLAN's owner directions).

## Goal

D80 (claude-172, #3360) lifted the serving ban on the integer multiply instruction. I then priced the remaining bans for Casey:
- **Integer divide:** the only serving piece that divides is the RMS norm's restoring long division. Both norms together are about 11% of a token at claude-169's 332 µs step. A hardware divide is exact integer arithmetic, so determinism is kept.
- **Floats:** would cost cross-device determinism. Their main gain is serving the trained float model and so skipping the export gap. I recommended holding them and fixing the exporter instead.

**The owner's decision (D81), words below:**
- The integer-divide ban is lifted.
- Floats stay banned.
- The FBS exporter is to be fixed: the map's R6 and item `d11-exporter-float-trunk`.

**What changes:**
- **§3.6:** the artifact has "no float or transcendental instruction". Since D80 it may use the hardware integer multiply, and since D81 the hardware integer divide. Serving stays integer-only and deterministic.
- **The records:** the D81 row in `DECISIONS.md` and a dated entry in `docs/rules/changes.md`.
- **PLAN:** two owner directions: the divide, `open` until the audit allows it, and the exporter, `open` until the FBS export gap passes.

**What does not change here:**
- The audit still refuses divide, and the docs that state the old rules are not yet corrected (claude-172's REPORT lists them). Both go to the follow-up serving unit.
- The exporter work is the next GPU line, not this unit.

## Cancel probe (run first)

Run at `b4f1fed39`:
- **No open PR** touches `CHARTER.md`, `docs/integration/DECISIONS.md`, `docs/rules/changes.md` or `PLAN.md`.
- **The last decision** is D80, so this one is D81.
- **§3.6 reads** "no float, transcendental or integer-divide instruction (D11 R1–R5). Since D80 it may use the hardware integer multiply…".

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `CHARTER.md` (§3.6)
- `docs/integration/DECISIONS.md` (the D81 row)
- `docs/rules/changes.md` (the dated entry)
- `PLAN.md` (two owner-direction rows)
- `docs/loop/tasks/infra-loop-harness/claude-173/**`

## Done when

1. §3.6 carries D81's text.
2. `DECISIONS.md` has the D81 row, with the owner's words verbatim.
3. `docs/rules/changes.md` has its dated entry.
4. PLAN holds both directions.
5. A fresh review confirms that the edits say what the owner chose, and no more: floats still banned, matmul still banned.
6. The PR is merged and verified.

## Stop and decide if

- **The edit would touch any rule beyond the integer-divide instruction.** Then narrow it.

No step waits on Casey.

## Owner approval

Owner paths changed: `CHARTER.md`, `docs/rules/changes.md` and `docs/integration/DECISIONS.md`.

Casey, in Claude's chat by 2026-10-29T03:20:23Z:
- After I priced divide and floats, he wrote "allow divide, hold on floats for now as you have recommended and fix the exporter".
- His standing permission for this rule change's owner-path edits, from D80: "i give you permissing to fix the stuff you need to to make this work".

**How the owner paths are written:** as for D80. Edited copies go under `local/claude/d81/`, and their difference is kept as `d81-owner-paths.patch` in this folder and applied with `git apply`.
