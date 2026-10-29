# REPORT: infra-loop-harness, claude-172 (D80: integer serving may use the multiply instruction)

Lab: claude. Order: [ORDER.md](ORDER.md). The umbrella item stays open; its done-when does not move.

**Speed.** Not applicable: a rule change recorded, no run. The last measured decode on the M1 is 3,042–3,044 tok/s (claude-169), under the multiplier-free rule this change lifts.

## What did not work, first

- **The loop guard refuses lab edits to owner paths, with no exemption** (`loopguard.rs`, "owner-only path"). My Edit of `CHARTER.md` was refused.
  - I told Casey. He wrote "i give you permissing to fix the stuff you need to to make this work".
  - The three owner files were then changed by one patch (`d80-owner-paths.patch`, in this folder), applied with `git apply`. It was built from edited copies under `local/claude/d80/`.
  - That route passes outside the guard, and only the owner's permission in this chat makes it right. The patch is kept so the change can be read and reversed as one piece.
- **Nothing measures the gain yet.** The zero-multiply audit and the integer crate's zero-multiply checks still refuse multiply instructions, so no kernel can use one until they change. That is the follow-up unit.

## What changed

- **`CHARTER.md`:**
  - **§3.6** reads "The exported artifact has no float, transcendental or integer-divide instruction (D11 R1–R5). Since D80 it may use the hardware integer multiply, so a product of two integers is exact integer arithmetic in one instruction". Its sources gain "D80, owner 28 Oct".
  - **§3.9:** "multiplier-free kernels" becomes "integer kernels".
  - **No other clause changes:** the dense-matmul ban (§3.3), the float ban and the integer-divide ban stand.
- **`docs/integration/DECISIONS.md`:** the D80 row, with the decision, Casey's words verbatim, the alternatives offered and a low reversal cost.
- **`docs/rules/changes.md`:** the dated entry, "28 October 2026: integer serving may use the multiply instruction (D80)", with his words and the clauses it changes.
- **`PLAN.md`:** the owner direction, `open` until the audit and the kernels follow.
- **This folder:** `d80-owner-paths.patch`.

## Measurements

None: no run.

## Checks run at the head

- `uor-harness loop report --check`, `loop pr-check --local` (its owner-paths rule reads the owner's words in the ORDER) and `loop gates --full`: in the review record.

## Skills used

- **holding-the-goal.** The change is what the owner chose, no wider. Integer-only, deterministic serving and the matmul ban are kept.

## Still open

- **Docs that still state the multiplier-free rule as current**, found by the fresh fidelity review. A lab reading them could refuse a kernel the charter now allows. Most are outside this ORDER's scope, so the follow-up unit corrects them, and the rule summaries come first:
  - `docs/rules/README.md:41`: the plain-language §3.6 row, "no float, transcendental, multiplier or integer-divide instruction";
  - `docs/rules/decision-map.md:40` (the D11 row, "Kept, 3.6") and `:28` (I1–I5, "multiplier-free kernels");
  - `docs/integration/DECISIONS.md:72–75`: the D11 entry, with no "amended by D80" pointer. It is an owner path, so it goes through the same patch route under the owner's permission;
  - `docs/mechanisms/lut-serving.md:44` ("R2: no multiply or divide instruction"), `:95` and `:102`;
  - `docs/model/serving-and-export.md:15` and `:49`, `docs/model/pipeline-map.md:264`, `:321`, `:338` and `:361`, and `docs/model/architecture.md:84`;
  - `scripts/audit_zero_matmul_serving.py`'s description, lines 2, 6 and 1510.
- **The follow-up serving unit,** under `fbs-serving-vector-kernel` or a new item:
  - `scripts/audit_zero_matmul_serving.py --stack` and the integer crate's zero-multiply checks allow integer multiply, still refusing float and divide;
  - the hot products (`stack_hamilton`, the snap's comparison, `stack_mul_u64` and `stack_mul_u128`) use the hardware multiply;
  - the result is measured exactly against claude-169's binary.

## Plain summary

Casey decided that the laptop engine may now use the chip's built-in multiply instruction, while keeping every number a whole number so answers stay identical on every device. The project's rule book now says so. The next step is to switch the engine's hand-built multiplications to the built-in one and measure the gain.
