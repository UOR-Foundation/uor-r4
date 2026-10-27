# Prep sheet: a content-keyed read inside the in-space trunk, against no read (in-space-trunk-to-text, claude-145, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-145](../../loop/tasks/in-space-trunk-to-text/claude-145/ORDER.md). Code: branch `claude/content-read` at `CODE_SHA`. The run's first step re-runs the CUDA tests on its own build.

TEST FITNESS: pending (one fresh non-author review covers the FIT and the D67 pre-run review: light gates, per the owner's 27 Oct direction on #2028)

## The question

claude-144's Flock read keyed on the hidden point and added the next token's point after the last layer. It landed in cell D at both seeds: the weight moved, but neither openings nor reach cleared their bars.

The premise count (`train-in-space --opening-bound 1`, in the ORDER) shows where the openings' information is:
- **The tokens just before an opening name `161` of 1,226.** They are always the same marker, the same as naming the commonest opening.
- **The prompt's first 8 tokens name `607`.**

**Does a read that finds earlier tokens by their own points, inside the trunk, lift openings and reach together?**

## Steps: the arms (identical except as stated)

Common to every arm, unchanged from claude-144:
- `train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0`;
- the FBS sheet trunk, `build_fbs_model(OneAtomActive)`;
- one RTX 5090 per pod, with two arms side by side.

| Arm | Read | Seed | Pod |
|---|---|---|---|
| `content-s1` | `--content-read 7` | 1 | A |
| `none-s1` | none | 1 | A |
| `content-s2` | `--content-read 7` | 2 | B |
| `none-s2` | none | 2 | B |

**The read** (`flock_read_content.rs`) runs after layer 3 of 6:
- **Key:** each earlier position's own token point. **Query:** the residual point. Each is rotated per 4-lane block by a learned unit quaternion and put on S³.
- **Choice:** the 7 nearest earlier positions in mean L1, hard and detached.
- **Payload:** their residual points, added through a weight that starts at 0.

All four arms run one binary built at the code commit. A read arm's twin starts from the identical model. Its step-0 readings must match exactly (I-2).

## Bar (pre-registered): the decision table, read arm against its twin, both at step 16,384

claude-144's table, unchanged, so the two reads compare.

Definitions, per arm:
- **`O`** = `nearest.opening_hits`.
- **`R16`** = `shift(16) / shift(0)`, from `--reach`.
- **`T`** = `nearest.hits`.

The two tests:
- **Openings pass** when `O_read ≥ max(197, O_none + 36)`. 36 is three binomial standard deviations at the commonest-token rate.
- **Reach pass** when `R16_read ≥ max(0.05, 2 · R16_none)`.

| | Reach pass | Reach fails |
|---|---|---|
| **Openings pass** | **A.** The read lets openings use the prompt. If `T_read ≥ 7,002` too, the item's done-when (1) and (2) are met. Unlocks long runs and the reply text on the read. | **B.** Openings improve from nearer context. Unlocks a unit on which tokens the read keeps. |
| **Openings fail** | **C.** The read reaches far, but its payload does not help openings. Unlocks a unit on the payload or the layer. | **D.** The read does not help. If I-3's weight stayed near 0, its start is the cause; otherwise the key or the layer is. |

- **Flag F**, reported beside the cell: `T_read < T_none − 500`, meaning the read costs fit.
- Both seeds must land in the same cell for the reading to count. If they split, a third seed (3) runs on both arms, and the cell two of the three share is the reading.
- A missing number gives no cell: that step is rerun, never read as a fail.

## Instrument checks

- **I-1:** the crate's CUDA library tests pass on the run's build, including the four content-read tests:
  - keys come from token points and the payload is the kept residual;
  - a zero weight leaves the residual unchanged;
  - the credit's sign and size, computed by hand, with its negative gate (the sign flipped must fail);
  - the trunk with a zero-weight read is unchanged, and a moved read changes it.
- **I-2:** each read arm's step-0 DEV reading equals its twin's exactly.
- **I-3:** the read's weight moves off 0. The mean `|w|` at step 16,384 is reported (`--inspect`, `content_weight_mean_abs`).
- **I-4:** no non-finite loss; the runner stops on one.
- **I-5:** the premise count's no-key row equals DEV's commonest-opening count, `161` (done: it does).

## Probe ladder

PROBE_LADDER

## Data split (pinned in `hamilton_sheet.rs`, checked at load)

- **Training** reads only TRAIN, `mw-balp/train`: 68,218 usable windows.
- **Every reading** is on DEV, `ft/dev`: 14,287 targets, 428 windows.

The files are claude-144's, under `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`:

| File | sha256 |
|---|---|
| `tokenizer.json` | `d36d3e87…` |
| `mw-balp/train/tokens.u16` | `3dee858f…` |
| `mw-balp/train/response_mask.u8` | `b5e7352d…` |
| `ft/dev/tokens.u16` | `0919e90f…` |
| `ft/dev/response_mask.u8` | `a838bb1a…` |

## Base artifact

No trained checkpoint. Every arm starts from `build_fbs_model(OneAtomActive, seed)`. The read starts with identity rotations and a zero weight. The binary's sha256 is recorded in the job log.

## Minimum test power

Two seeds per arm, and they must agree; otherwise a third. claude-144's no-read twins read `200` and `271` openings at the same settings, so one seed's opening count is weak alone.

## Cost

COST

## Stop rule

- Stop on a non-finite loss or a crash. A healthy run is never cut short.
- Results go to `caseyallard/uor-r4-store:results/claude/content-read-2026-10-27/` with a SHA256SUMS. They are read back with `uor-store ls` before any pod comes down.
