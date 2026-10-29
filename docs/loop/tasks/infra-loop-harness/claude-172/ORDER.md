# ORDER: infra-loop-harness, claude-172 (D80: integer serving may use the multiply instruction)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `ee369b4ca`.

Setup row: 2 (an owner rule change recorded: charter text, decision index, change log, PLAN's owner directions).

## Goal

The owner decided, in Claude's chat, that served integer code may use the hardware integer multiply instruction (words below).

The context: the serving engine is integer-only and multiplier-free under D11 (CHARTER §3.6). Six exact units made it 11.3× faster (claude-163 to claude-169). Their profiles show the remaining cost is almost all exact products done without a multiplier: a table of sixteen multiples per operand, then a read per radix-16 digit. A hardware integer multiply computes the same exact product in one instruction.

**What changes (D80):**
- **§3.6:** the exported artifact still has no float, transcendental or integer-divide instruction. The ban on multiplier instructions is lifted, so a product of two integers may use the hardware multiply.
- **What stays:** serving stays integer-only and deterministic, since integer products and sums are exact. The dense-matmul ban (§3.3) is unchanged.
- **§3.9:** the efficiency report names "integer kernels" instead of "multiplier-free kernels".

**What does not change here:** the zero-multiply audit (`scripts/audit_zero_matmul_serving.py`) and the integer crate's zero-multiply checks still refuse multiply instructions. Relaxing them for integer multiply, while they keep refusing float and divide, is a follow-up serving unit. So is using the multiply in the kernels, measured against claude-169's 3,042–3,044 tok/s.

## Cancel probe (run first)

Run at `ee369b4ca`:
- **No open PR** touches `CHARTER.md`, `docs/integration/DECISIONS.md`, `docs/rules/changes.md` or `PLAN.md`.
- **The last decision** is D79, so this one is D80.
- **§3.6 reads** "no float, transcendental, multiplier or integer-divide instruction (D11 R1–R5)".

All held. **Not cancelled** (instrument: none needed).

## Files in scope

- `CHARTER.md` (§3.6, and §3.9's one word)
- `docs/integration/DECISIONS.md` (the D80 row)
- `docs/rules/changes.md` (the dated entry)
- `PLAN.md` (the owner direction's row)
- `docs/loop/tasks/infra-loop-harness/claude-172/**`

## Done when

1. §3.6 and §3.9 carry D80's text.
2. `DECISIONS.md` has the D80 row, with the owner's words verbatim.
3. `docs/rules/changes.md` has its dated entry.
4. PLAN's owner directions hold the direction, `open` until the audit and kernels follow.
5. A fresh review confirms that the edits say what the owner chose and change no other rule.
6. The PR is merged and verified.

## Stop and decide if

- **The edit would change any rule beyond the multiply instruction.** For example, it would touch integer divide, floats, or dense matmul. Then narrow it.

No step waits on Casey.

## Owner approval

Owner paths changed: `CHARTER.md`, `docs/rules/changes.md` and `docs/integration/DECISIONS.md`.

Casey, in Claude's chat by 2026-10-29T03:09:43Z:
- "ok I think the integer serving for determinism that gets us out of matmul is a worthy goal, unless the other mechanism is just ridiculously faster..."
- "sounds like reverse d11 is the answer".
- Asked "Which D11 change do you want: keep serving integer-only and deterministic but allow the hardware multiply instruction, or something wider?", he chose "Allow integer multiply (Recommended)".
- The loop guard refuses lab edits to owner paths. Told of that, he wrote: "i give you permissing to fix the stuff you need to to make this work".

**How the owner paths are written.** Each edited copy is made under `local/claude/d80/`. The difference is kept as `d80-owner-paths.patch` in this task folder and applied with `git apply`, under that permission. The patch touches only `CHARTER.md` (§3.6 and §3.9), `docs/integration/DECISIONS.md` (the D80 row) and `docs/rules/changes.md` (the entry). The fresh review checks it line by line against the owner's choice.
