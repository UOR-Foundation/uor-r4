# DECISION (D68): Stage T runs now, under its FIT sheet

Lab: claude. Unit: claude-107. Item: `m1-bvg-k-stage-t-training` (#2836). Status: proposed; the fresh decision review is recorded below.

## The question

The item says Stage T "is only warranted if W2 blames the learning law (H_law)". W2 returned H_mixed (§4.7 row 3), and the code-utilization diagnostic, which was H_mixed's next step, is done. Its README says "Stage T's gate … is not changed here. These bands are inputs a decision review would weigh" (`docs/labs/m1-w2-code-utilization/README.md:100`), and it names `m1-address-key-full-vector` (#2834) as "the next M1 mechanism unit" (A1a, line 94). Does Stage T run now?

## The decision

Yes. Stage T runs now, as registered, and #2834 follows it.

## Why

1. **H_mixed blames the learning law in part, with a measured size.** On W2's 1,920 writer-admitted rows, the admission-matched code table scores 1,138 against the writer's 149, so the learned output map loses at least 989 hits after admission (`docs/labs/m1-w2-capacity-capture/README.md`). Admission loses more, at least 2,276. "Mixed" means both, so the law clause is met in part, not refuted. Stage T's comparators are exactly these admission-matched tables (sheet §3, `B_adm_table`), so it tests the law share and nothing else.
2. **A8's full dose gave Stage T a second, separate reason** (sheet §1, chain point 1). Both K = 1 arms stopped on `consecutive_rejections`, classed instrument. A K = 1 greedy step on 64 local rows, audited against a 3,847-row global anchor every 64 batches, throws away whole windows. K = 16 joint blocks with a verified commit test that coupling directly.
3. **PLAN.md's approved order holds Stage T as step 4 on the M1 close (D79).** W2 (step 1) unlocks both Stage T and #2834 (`PLAN.md:58`). The two are not alternatives. Each scope holds `crates/uor-r4-discrete/**`, so they run one after the other, and `loop next` ranks Stage T first because it unblocks 12 items.
4. **Every outcome changes a decision** (sheet §3):
   - a pass unlocks steps 6 and 7;
   - a unigram-only partial routes the next work to #2834 and #2846;
   - a diagnostic stop closes the K = 16 configuration and unlocks #2846 and #2834.

   None leaves the M1 path where it is.

## Rejected

- **#2834 first, and Stage T after it.** That gives up the measured law share and A8's controller finding for a unit that waits on the same crate lock anyway. And #2834's result does not change Stage T's design: Stage T's base codebook is pinned (A8's `address-structured-v1.json`).
- **Drop Stage T from the M1 close.** That amends D79, so it is the owner's call, not a lab's. Nothing here argues for it: the law share is real.

## To reverse

Close claude-107 before its run starts, and claim #2834. Once the run starts, it finishes to its stop rules.

## Review

To be recorded at the head that holds this card.
