# ORDER: one-model-end-to-end, claude-213 (Flock rank inside the nearest token's cell)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).

## Goal

Casey, 30 Oct: "do not rule flock out, improve it instead". claude-211 found two things about the Flock rank rule:
- **Keeping a nearest end of turn (`FLOCK_END`) fixed completion and loops.** (r-drop) completes 508–510 of 512 at a repeat rate of 0.002, against greedy's 0.50.
- **Its content is often word salad.** The 16-wide harmonic draw lands below rank 0 about 70% of the time, on tokens the model has separated from the nearest.

This unit builds and measures `FLOCK_CELL`, a geometric narrowing:
- **The candidate set:** the top-16 nearest tokens whose score is within the cell objective's margin of the nearest. That margin is 1.0 (`rival_hinge`), the margin training pushes a target past its nearest rival.
- **Inside the margin,** the model has not separated the candidates, and the harmonic draw chooses among them.
- **Outside it,** the model has, and those tokens are never drawn.
- **The end-of-turn rule stays.**

It is measured against `FLOCK_END` and greedy on (r-drop)'s sealed step-16,384 checkpoints, at all three seeds, on a new held-out-phrasing panel. P-dev has been opened once already, and p-f is bar (f)'s.

## Cancel probe (run first)

To fill in: confirm that no other unit changes `decode_reply` or the Flock rules, and that the sealed (r-drop) checkpoints are in the store.

## Files in scope

As the claim recorded: `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`, `docs/harness/prep/one-model-end-to-end*`, `docs/labs/one-model-end-to-end/**`, `docs/loop/tasks/one-model-end-to-end/**` and `docs/research/notebook/float-training.md`.

## Done when

1. **`DecodeRule::FLOCK_CELL`** exists, with `--flock-cell 1` for `--score-rows` and `--grade`. A unit test pins it, with a negative gate.
2. **A new held-out-phrasing panel** is drawn by `heldout-panel` at a new seed, with its own census.
3. **A prep sheet with a fresh FIT and pre-run review,** registering the comparison before any decode is read: complete count, repeat rate, verbatim share against the references, and a fixed reading of 20 replies per rule.
4. **The GPU decode of (r-drop)** at three seeds, under greedy, `FLOCK_END` and `FLOCK_CELL`. It is sealed, graded, and reported, negatives first and tokens per second first.

## Stop and ask if

Stop and decide (never wait) if any of these holds:
- **The margin admits only the nearest at almost every step.** Then `FLOCK_CELL` is greedy with the end-of-turn rule. Record that, and try the margin the trained model actually shows.
- **A fault is in the instrument, not the rule.** Classify it before calling a result (CHARTER §4.1).
