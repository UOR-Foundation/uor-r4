# ORDER: one-model-end-to-end, codex-18 (C2′ gate-family reachability)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Owner direction, 2026-11-16: answer claude-397's gate-family question on the real ext state; recreate the
state from the store if it is gone.

## Why

claude-397 left one question open: can one fixed set of Q31 gates with an L1-nearest pick express the
other-fact rule? The c395 C2′ trained on TRAIN windows reached 0.12. This unit asks whether the family can
fit the **same** rows at all, which separates capacity from generalization.

## Method

- Recover the ext-s1 state from the HF store (`ext-seed1-flockg-r0-try-5.tar.gz`); verify it against the
  c387 probe counts.
- Dump the 1,024 eval rows with `recall-payload-probe --first-only --native-replay --native-dump`.
- Fit the native gates on the same rows with the other-fact target, 512 and 2,048 steps, identity start,
  shift 8, margin 0, batch 64, and read `chose_target` per bucket.

## Done when

1. The recovered state reproduces the saved c387 probe counts exactly.
2. The same-row fit is reported at 512 and 2,048 steps.
3. The capacity reading is stated, not a generalization claim.

## Cancel

Stop if the state cannot be recovered or the probe does not reproduce.

STOP and report when done.
