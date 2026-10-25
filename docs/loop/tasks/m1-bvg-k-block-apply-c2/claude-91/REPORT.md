# REPORT: m1-bvg-k-block-apply-c2, claude-91

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2835
References #2836

## What did not work, first

- **The cancel probe stopped the sheet.** Writing the Stage T pre-registration now would register a learning-law test that the project's own registered result says not to run yet.
  - The Stage T item (#2836) is "only warranted if W2 blames the learning law (H_law)".
  - W2 came back **H_mixed** (#2845, #3177). Its sheet's row 3 says: "Code-utilization/capacity diagnostic; do not change the learning law yet".
  - Separately, the Stage T draft's precondition 2 is still unmet: no base lineage reaches its own admission gate. On 19 Oct the injective pre-flight admitted 127 of 3,847 P_succ rows, below 500.
- **`loop next` offered this item anyway** (GO, "unblocks 13"), because neither item declares the two prerequisites. It will keep offering the sheet to every lab until they do. The routing fix is the separate unit claude-92.

## What changed

- This ORDER and REPORT only. No sheet was written.

## Measurements

None.

## The decision (owner log, D69; decision review below)

- **The C2 item (#2835) and Stage T (#2836)** now wait on two items:
  - `m1-w2-code-utilization` (#3179), the diagnostic W2's row 3 calls for;
  - `m1-a8-finite-cuda-qualification` (#2830), the admission-preserving base.
- **Stage T stays on the M1 path (D79).** Its order now follows the W2 result.
- **If the diagnostic shows the learning law is the limiter,** the Stage T sheet comes next, with its bar on the then-current admission-matched next-word table. W2's Q4 numbers are: the bigram restricted to admitted cells gets 1,482 of 1,920, and the unigram gets 1,226 on the full panel.
- **If the diagnostic does not show that,** the Stage T line is re-planned by a decision review, and M1's trained model comes from the A8 full dose (#2831).
- **To reverse:** drop the two dependencies.

## Checks run at the head

`uor-harness loop report --check` and `loop pr-check --local` were run before the PR went ready (see the PR).

## Skills used

None.

## Still open

- **claude-92:** add the two `depends_on` entries to both items and re-render.
- **The C2 item's done-when** (the sheet) stays open until both prerequisites are done.

## Plain summary

The next M1 step looked like "write the Stage T sheet", but the W2 result says the learning rule is not the first thing to change. And the base model for Stage T still loses its shortlist during training. So the sheet waits for the code-utilization diagnostic and for Antigravity's A8 qualification. This unit records that, and claude-92 makes the roadmap say it.
