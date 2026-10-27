# REPORT: infra-loop-harness, kimi-52

Lab: kimi. Order: [ORDER.md](ORDER.md). Decision: [DECISION.md](DECISION.md).

References #2904 and #2029. No item, milestone or issue closes.

## What did not work, first

The proposed `PLAN.md` correction from 0 to 1 of 8 failed D68. The accepted 19-table model's DEV/export/fresh results are real, but they do not cover canonical zero-update G-Pre. Its lineage reaches 4,244/14,287 before training, below 4,894. The closing instrument's `bar_a` checks D11 self-containment instead, and its 5,009 is post-training G-Adm, not G-Pre. The accepted FrozenZeroV1 arm also has no actual R>0 row shown. `PLAN.md` stays at 0 of 8 and #2029 stays open.

## What changed

Only this ORDER, DECISION and REPORT. No `PLAN.md`, bar, threshold, premise, roadmap item, public status, code or issue state changed. The D68 rejection is logged on #2029 at comment `6433873393`. Size: docs 3 files, about 5.7 KB; the ORDER gave no numeric estimate.

## Measurements

No new experiment or GPU work. The review checked artifact identity, parity and both seeds: DEV top-1 7,002/7,001 and fresh top-1 7,215/7,228 are accepted, while zero-update G-Pre is 4,244. Bars (b), (d) and (e) are supported; the milestone still lacks the canonical (a)/(c) evidence.

## Checks run at the head

- D68: REJECT, agent-59 (`DeepSeek/deepseek-flash`), with the full artifact and bar chain read.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check`: 95 items OK; no status changed.
- Full gates, local policy, report check, fresh exact-head review, required checks, queue and fresh-main verification are required.

## Skills used

- **uor-research-architect:** distinguished the accepted model from the 4-table controls and tested the whole canonical bar, not only the strongest results.
- **solution-integrity:** accepted the D68 rejection rather than publishing a milestone close on substituted labels.

## Still open

- A later count update needs a lineage/initializer whose zero-update evaluation clears G-Pre on both seeds while preserving (b)–(e), or Casey's charter amendment to a specified non-vacuous zero-update capacity gate.
- Scoped record repairs remain for the bar-label/evidence mapping, fresh baselines, artifact bytes and stale 4-table DEV pins. Preserve the fresh/DEV evidence; do not rerun it.
- The other public records and #2029 remain untouched.

## Plain summary

The strongest model passes the DEV and fresh tests, but one required pre-training gate is not met and another required property is not shown. The project therefore stays at 0 of 8. This PR records that decision instead of hiding it with a label change.
