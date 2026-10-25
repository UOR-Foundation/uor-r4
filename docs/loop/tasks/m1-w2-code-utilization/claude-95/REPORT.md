# REPORT: m1-w2-code-utilization, claude-95

Lab: claude. Order: [ORDER.md](ORDER.md).
Closes #3179

## What did not work, first

- **The notebook row is not written.** The loop guard refuses the edit to `docs/research/notebook/m1-discrete-attention.md`. `loopguard.rs` (around line 2820) allows only:
  - the ORDER folder;
  - the item file;
  - `docs/harness/mistakes/`;
  - FILEMAP;
  - the shared files.

  It does not allow the item's `notebook:` page, though `docs/loop/README.md` ("Scope, without a line cap") lists "new rows on its notebook page" as always allowed. I did not work around the guard. The item's full record is in [the lab README](../../../../labs/m1-w2-code-utilization/README.md) and in the sealed record. The row to add, verbatim, once the guard allows it:

  > | 2026-10-25 | W2 code-utilization diagnostic on the stored W2 capture: host post-processing only, no GPU. Precedence 0 reproduced every W2 number exactly, three ways. Arm R was not run: the Q1–Q3 checkpoints are missing. | Code-count admission 9,064 against the writer's 1,920 and a fixed most-frequent-cells list's 5,935; code-table gain over the last data doubling 69 (projection 3,552); code × query table 7,262 against the bigram's 6,300 | The sheet's §5 paired floor, (c − b)² > max(576, 4(b + c)), per question; QB's line is W2's H_law line (about 6,435) | positive (QA A1a, QB B2, QC C1: admission is recoverable from the code, more data will not reach the bar, and the code adds information beyond the last token; G1 declined) | [m1-w2-code-utilization](../../labs/m1-w2-code-utilization/README.md) |

  Item `status: done` is set anyway: everything else in the done-when is met, and leaving the item `ready` would re-offer a finished run.
- **The first builder subagent wrote nothing.** The guard makes subagents read-only without a suborder in `local/claude/<unit>/orders/`. The builder also reported starting at about 150k tokens of context, at its hand-back mark, so this session built the instrument itself.
- **This unit was first numbered claude-94.** `loop claim` gave the same number to `m3-nonrelation-prose-panel` (#3187), so this unit is claude-95: `0f9ae0647` renames the folder, and `efe524657` retitles the ORDER.
- **Arm R is NOT_RUN**, as registered. The Q1–Q3 checkpoints are in neither the HF store's `local-archive/2026-10-20/uor-r4-local.files.txt` (no `checkpoint-4101`, `8202` or `12303`, padded or not) nor `archive/storage/`.
- **QA's admission sets are not load-matched,** as the FIT reviewer foresaw. A_T admits a median of 988 candidate tokens against the writer's 255, so part of its gain is a larger candidate set. The README reports the load quartiles beside the band.

## What changed

Size: code 2 files (+1,645: `lib.rs` 1,544 and `main.rs` 101), tests 1 file (+39), generated 1 (`Cargo.lock` +344), docs 4 files (the lab README; the item status line; the rendered roadmap page, +3/−3; this task folder). Nothing under `crates/`.

- **The instrument,** `docs/labs/m1-w2-code-utilization/instrument/`. It is a detached workspace with a read-only path dependency on `uor-r4-discrete` (no `cuda`), and exact integers only.
  - **Load and authenticate:** the pins and byte sizes of the stream, sidecar, capture and W2 record, plus range and conservation checks.
  - **Precedence 0:** 11 W2 numbers, each from W2's public functions, from the sealed record and from the instrument's own tables. Six arms are compared row by row with W2's own scoring.
  - **QA, QB, QC and the descriptive arms** exactly as the sheet defines them; arm R as NOT_RUN.
  - **Determinism:** the CLI runs twice and refuses a difference.
  - **Unit tests:** the floor boundary, the precedence-1 rule, every band (including A1u and the decisive-pair scope), the table tie rule and leave-one-out, the cell-ranking ties, the spread rule and the projections.
  - **An env-gated real-capture test.**
- **`docs/labs/m1-w2-code-utilization/README.md`:** the numbers, what they unlock, the instrument, the artifacts, how to reproduce, and the limits.
- **The item:** `status: done`, and the roadmap page re-rendered.

## Measurements

The tool ran on the authenticated `w2-out-b/` from `run-b.tar` `6aa0a766…` and W2's `record.json` `69717971…`, on this laptop, host only, in about 6 s per run.

- **Precedence 0:** all 11 checks equal the sheet's values, W2's functions and the record. All 6 row comparisons differ on 0 rows. Σk_r = 114,296, F² = 18,288, M = 6,300.
- **QA = A1a.**
  - A_T 9,064 against A_W 1,920, with (b, c) = (307, 7,451); against A_U 5,935, with (882, 4,011).
  - A_G 12,214, and A_Ω 13,726.
  - **A_U beats A_W:** a fixed list of the 8 most frequent cells admits more target cells than the trained address key.
  - **T's hits by admission:** T_adm 798, T_non 2,616.
- **QB = B2.**
  - T on the stride subsamples: 2,785, 2,981, 3,176, 3,345, 3,414.
  - (b, c) = (235, 304), so d_T = 69 and T_proj = 3,552. **G1 is declined.**
- **QC = C1.**
  - J 7,262 against G 6,300, with (571, 1,533).
  - 11,727 DEV rows have a seen joint key, and J_m2 is 7,251.
- **Record:** `results/claude/w2-utilization-2026-10-25/record.json`, sha256 `a0af69ae4fef30dcc24c05dcdb79b35b41dd6ec92cc7140ca3cbb623ff2f45e2`. A fresh download matches it byte for byte.

## Checks run at the head

Recorded in the PR's review record.
- **`cargo test --release`:** 9 unit tests pass.
- **The real-capture test:** passes, including byte-identical runs.
- **Hygiene:** `cargo fmt --check` is clean, clippy finds nothing in the instrument, and `grep -rnwE "f32|f64" src tests` finds nothing.
- **Loop checks:** `loop report --check` and `loop pr-check --local`.

## Skills used

- holding-the-goal: the M1 bar (T2 = 6,300) is what QB and QC measure against.
- verify-before-done: precedence 0 first, then the bands, then the record round-trip.

## Still open

- **The notebook row** above. It waits on the guard allowing the item's `notebook:` page, as `docs/loop/README.md` documents.
- **Decision reviews** these bands feed, none taken here:
  - **#2834 (full-vector address key) next.** Its references: A_T 9,064, the ceiling A_Ω 13,726 and the fixed list A_U 5,935, which the trained key loses to.
  - **A read-change card,** with J = 7,262 as its reference.
  - **Stage T's "only if H_law" gate.**
- **G1 is declined.**
- **The scratch data** (`local/claude/w2-utilization/`, about 90 MB with the build) is removed after the merge.

## Plain summary

The model reads its context into a 16-bit code and predicts the next word from that code alone. This run asked, using only data saved by the earlier W2 run, where that code loses information.
- **Admission.** The step that picks which groups of words to consider admits the right group on 1,920 of 14,287 rows. A rule keyed on the code would admit it on 9,064, and even a fixed list of the most common groups gets 5,935, though those rules pass on more candidates.
- **More data.** More training data would barely help a code-keyed table: its gain had shrunk to 69 rows per doubling, far short of the 6,300 needed. So the extra GPU run was not worth doing.
- **The previous word.** The code combined with the previous word predicts 7,262 rows, more than the previous word alone (6,300). So the code holds useful information, but it does not keep the previous word itself.
