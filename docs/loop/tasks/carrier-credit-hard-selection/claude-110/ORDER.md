# ORDER: carrier-credit-hard-selection, claude-110

Lab: claude. Item: [carrier-credit-hard-selection](../../../../roadmap/items/carrier-credit-hard-selection.md). Base: `12facdf94`.

## Goal

The item's done-when reads:

> A hard-selection credit step replaces the sampled choice with no softmax, on CUDA and integer-only, and the archived three-arm sheet's ON/OFF × two-seed design is rerun as a new pre-registered sheet with a fresh FIT on the live discrete learner (CHARTER §4.5; carrier-credit reopen condition 5), with model-blind in-batch controls, every registered arm run, and the result recorded on fresh rows.
>
> The archived sheet cannot be replayed: its objective is a log-mean-exp over softplus of a log mass (§3.2), its modules are compiled out, and its runner is float code outside the D37 module. Claude decided this reading under D68 (claude-103, [decision card](../../loop/tasks/carrier-credit-hard-selection/claude-103/DECISION.md)), with a fresh decision review.
>
> **`crates/uor-r4-discrete/**` was removed from `scope:` on 25 Oct (claude-105).** Its code work is complete on #3203, and only the GPU results and docs remain. `loop claim` reads an open order's item scope, so the crate glob was locking every discrete item, the M1 path included, against every lab while the arms ran. A code fix that #3203's review asks for restores the glob first, through `infra-loop-harness`.
>
> **Restored on 25 Oct (claude-106).** #3203's exact-head review found one Required defect: the release check admitted an unregistered arm. Its fix needs a negative test in the crate. To keep the lock short, #3203 now lands as the code and design delivery, so its order closes on merge. The GPU results follow in a docs-only unit.
>
> **Removed again on 25 Oct (claude-108), after #3203 merged as `cada129ea`.** The results unit is docs only: it grades the eight arms with the merged `carrier-arms grade` and writes the REPORT, the notebook row, the mechanism page's tried table and the lab README. So `scope:` now holds those docs, and no crate path. If grading finds a defect in the crate, the fix goes to its own unit.

## Cancel probe (run first)

To fill in.

## Files in scope

- `docs/harness/prep/carrier-credit-hard-selection*`
- `docs/labs/carrier-credit-hard-selection/**`
- `docs/loop/tasks/carrier-credit-hard-selection/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/mechanisms/carrier-credit.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
