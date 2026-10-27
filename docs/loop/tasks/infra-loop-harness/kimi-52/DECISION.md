# D68 decision record: public M1 status correction, kimi-52

**Verdict: REJECT.** Do not change `PLAN.md` from 0 of 8 and do not close #2029. No GPU work is needed for this decision.

The fresh non-author D68 reviewer was agent-59 (`DeepSeek/deepseek-flash`) at ORDER commit `9e5364d7e`.

## Evidence accepted

The accepted 19-table `trunk-joint-pointer-v1-d11` model is distinct from the failed 4-table controls. On both seeds it has exact export parity and B4, DEV top-1 7,002/7,001 above 6,324, fresh top-1 7,215/7,228 above the executed T2 floor, and fresh `PASS_BAR_E_FRESH_TRANSFER`. The fresh seal and artifact identities match. Bars (b), (d) and (e) are supported.

## Why the milestone count does not change

CHARTER §4.8 leaves bars (a)–(c) unchanged: G-Pre admission at least 4,894 before training, G-Adm on both seeds, and G-Occ with an actual R>0 row. The accepted lineage's zero-update reach is 4,244/14,287, below 4,894. The closing instrument's `bar_a` instead checks D11 self-containment, and the closing card's 5,009 is post-training G-Adm, not zero-update G-Pre. No G-Pre receipt clears the canonical bar, and no actual R>0 row is shown for the accepted FrozenZeroV1 arm. The ORDER's stop condition is therefore met: the evidence does not cover every closing bar.

The 4-table failures are not being confused with the 19-table result. The rejection is about a missing canonical gate, not a failed mechanism.

## Record issues kept separate

- Sheet/grade bar labels (a)–(c) substitute different checks for the canonical definitions.
- Fresh registration and execution carry different baseline numbers.
- The sheet's 19-table byte count differs from the actual 86,536,132/84,943,328-byte artifacts.
- The standalone 4-table refusal receipt has stale DEV pins; its negative result remains scoped, but those pins need repair.
- Other public files and #2029 remain unchanged. No coherent one-file count update is made.

## Next decision

Keep `PLAN.md` at 0 of 8 and #2029 open. A later unit must either:

1. produce a lineage/initializer whose own zero-update evaluation clears canonical G-Pre on both seeds while preserving (b)–(e); or
2. ask Casey for a charter amendment replacing G-Pre with a specified non-vacuous zero-update capacity gate. Only Casey can change the charter.

Before any later count update, repair the bar-label/evidence mapping, fresh baselines, artifact byte count and stale 4-table DEV pins in scoped units. Preserve the fresh/DEV evidence; do not rerun it.
