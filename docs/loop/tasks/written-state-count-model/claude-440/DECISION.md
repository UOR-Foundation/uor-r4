# DECISION: P1's S5 copy path is A1, Pick over the D11 trunk codes (`CHARTER.md` §5.1, D68)

- **Lab:** `claude` (top session), 19 Nov 2026.
- **Order:** [ORDER.md](ORDER.md). **Decision review:** [DECISION-REVIEW.md](DECISION-REVIEW.md).
- **Evidence:** claude-439's [REPORT](../claude-439/REPORT.md) and its [`run2/`](../claude-439/run2/)
  packet (100% of C, wick's RTX 4090, code head `932234a4b`). No new number is measured here.
- **Authority:** §5.1 and D68. A scope choice goes ahead when the lab's own whole-project decision
  review passes. It does not wait on the owner, and it is logged for Casey, who can reverse it.
  Nothing in §3 or the caps changes. [claude]

## The decision

P1's S5 session-copy expert is **A1**: the Pick read over the D11 trunk codes with Identity
transport, latest position on a tie and then the lowest original id, as fitted on reserved TRAIN in
claude-439. P1(b)'s deployable count model carries A1's S5 column. So does the reserved-TRAIN
mixture that codex-28 binds and the next prediction unit packages into an active D11. [claude]

**C2** (exact token pairs) and **C3** (the same read over a permuted token → code map) stay as
required controls in every unit that measures or changes S5. **C1** (read disabled) stays as the
no-copy control. These are the controls codex-14's DECISION asks to keep: exact-token-pair, rotated
read and geometric read disabled. [claude]

## Evidence (claude-439, DEV-tune at 100% of C, micro-BPB)

| Arm | DEV-tune all | DEV-tune novel | Read cost at 100% |
|---|---|---|---|
| A0 pure count | 1,432,620 | 1,747,432 | none |
| A1 Pick, D11 codes | 1,360,806 | 1,651,188 | about 9 min device read per pass |
| C2 exact pair | 1,365,580 | 1,657,701 | about 1,300 ms on the host |
| C3 rotated codes | 1,360,932 | 1,651,364 | about 9 min device read per pass |

Paired bands (2,670 records, 1,000 resamples): A1 − C2 [−5,232, −4,318] overall and
[−7,141, −5,889] on novel targets; A1 − C3 [−232, −24]; A1 − A0 [−77,388, −66,248]. [claude-439]

## Why A1

- **It is the better measured path, by a band that clears 0 widely.** A1 beats C2 by 4,318 to 5,232
  micro-BPB out of sample, about 3½‰ of A1's bits, and by more on novel text. Of the measured arms
  only C3 comes close, and C3 costs the same device read. So C3 is never the cheaper choice. [claude]
- **The cost is small where it lands.** A1's extra cost over C2 is offline: about 9 min of device
  reads per pass on wick, inside an hour-scale run. P1's budget is ≤ 1 GPU-day. At serving time the
  read is per token over one session's context. Its cost on the M1 has not been measured, so the
  reversal rule below covers it. [claude, est]
- **It is the path the track already planned.** claude-409's card names S5 as "the Pick read" over
  the trunk's codes. P2 already carries "the output Pick expert, claude-408's CPU Pick applier and test
  gaps". Choosing C2 would leave A1's read in the tree and add a second copy operator to keep
  bit-identical across train and serve (D85). [claude]
- **It adds no state.** Both paths read the session's own tokens. A1 reads codes the D11 artifact
  already holds, so the deployed footprint codex-14 asks to record grows by S5's fitted weights in
  either case, and by nothing else. [claude]

## What this card does not decide

- **The geometry label.** claude-439's reading stands: `geometric_gain`, small. The D11 codes beat a
  permuted code map by only 24 to 232 micro-BPB, and that edge narrowed at every size of C. C3
  suggests that most of A1's lead over C2 comes from firing at distance > 0 under any code map, not
  from the codes. This card chooses A1 for its measured bits. It does not credit the D11 codes with
  that lead. P2's bar (≥ 1% fewer DEV-test bits than the best of three equal-byte controls) still
  decides geometry. [claude]
- **A cheaper nearest-pair path.** A read that fires on the nearest earlier pair under a cheap
  injective map might hold most of A1's lead at C2's cost. That is an untested hypothesis
  (claude-439, "Still open"). A unit that tests it keeps A1 as the incumbent and C2, C3 as controls.
- **Per-distance weights.** EM gives the distance > 0 class 10‰ of Pick's weight, against 190‰ at
  distance 0. A split by distance (1, 2, 3+) may change that. That is a future unit, not a choice made
  here.

## Reversal rule

A new card switches S5 to C2 if either of these holds:

1. **Bits.** At P1(b)'s deployable size, the paired A1 − C2 band on DEV-tune, or on the sealed
   DEV-test in P1(b)'s one sealed run, includes 0.
2. **Speed.** On one M1 performance core, warm at 2k context, the served model with A1 misses the
   1,000 tok/s bar while the same model with C2 meets it.

A failed D85 replay (the CPU Pick read differing from the CUDA read on any of ≥ 64 prompts) is a bug
to fix, not a reason to switch. [claude]

## Alternatives turned down

- **C2 now.** It is cheaper offline, but 4,318 to 5,232 micro-BPB worse. Its saving is minutes of
  wick time per pass. Revisit only under the reversal rule. [claude]
- **C3.** Same read cost as A1, slightly worse, and its code map is a control by design. [claude]
- **Both A1 and C2 as separate experts.** C2 fires on 4,108,760 DEV-tune targets and A1 fires at
  distance 0 on 4,111,759, so a C2 column would presumably repeat most of A1's distance-0 class (not
  checked). No arm measured the pair. A unit may test it under its own sheet. [claude]

## Cost to undo

Low. The choice is one column in the S5 fit and the packer. C2's column can be recomputed on the host
in about 1,300 ms from the same split, and claude-439's code keeps every arm. [claude]

## Index row: not written

`docs/integration/DECISIONS.md` is an owner path (`docs/harness/owner-paths.json`), and this unit has
no `## Owner approval` section, so the card lives here, as claude-409's did. [claude]
