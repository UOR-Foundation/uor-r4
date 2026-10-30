# ORDER: infra-loop-harness, claude-212

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `f22ff26aa`.

Setup row: 2 (item-file maintenance and PLAN: a milestone close recorded, M1).

## Goal

Record M1's close where the lab may write. A saved model passes all six closing bars of CHARTER §4 item 8. Bar (f) is claude-211's: PR #3419, merged as `2ba9f27a3`.

The decision review (D67–D69, #2029's owner log) approved closing M1 on conditions:
- conditions 1 and 2 are met: the close grade on fresh main, and #2029's criteria;
- this unit meets 3 and 4: the roadmap items, and `PLAN.md`'s M1 rows and headline;
- after merge, #2029 is closed;
- condition 5 is the owner's, sent to #2028: `AGENTS.md`, `DECISIONS.md` and `CHARTER.md`.

The item's own done-when (unchanged, not this unit's) reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

## Cancel probe (run first)

Run at 07:20Z on 30 Oct.
- **M1 is still open,** and nothing records its close: #2029 is open, `PLAN.md` says "0 of 8 milestones", and `m1-heldout-phrasing-transfer` is `ready`.
- **No other PR touches these lines.**

Result: **go**. The probe checks the setting, so it is an instrument check.

## Files in scope

- `PLAN.md`
- `docs/roadmap/items/*.md`
- The rest of the item's scope is listed by the claim, and this unit uses none of it.

## Done when

- **`PLAN.md`:**
  - the goal's milestone count is 1 of 8, with the date;
  - the M1 premise row is closed, with its answer;
  - the M1 path paragraph records the close and its limit (0.625 verbatim replies);
  - steps 8 and 9 are done;
  - #2846 is marked not needed for M1's close.
- **`m1-heldout-phrasing-transfer`** is `done`, and the roadmap is re-rendered.
- **After merge,** #2029 is closed with a results comment.

## Stop and ask if

Stop and decide (never wait) if any of these holds:
- **A record that needs changing is an owner path.** It goes to the owner's queue on #2028, which is already posted.
- **Another unit's PR conflicts** with these lines.
