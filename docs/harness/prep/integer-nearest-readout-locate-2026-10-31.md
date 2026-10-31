# Prep sheet: locate claude-232's parity gap, trunk and Hamming read on the GPU (integer-nearest-readout, claude-233), 2026-10-31

- Item: [integer-nearest-readout](../../roadmap/items/integer-nearest-readout.md) (#3462). ORDER: [claude-233](../../loop/tasks/integer-nearest-readout/claude-233/ORDER.md).
- **The run:** one GPU job on one leased RTX 5090, `local/claude/claude-233/job.sh`. It is not in git, and its copy is stored with the outputs. It runs against the pushed head.

## 1. Question

claude-232's end-to-end parity missed. The integer model disagreed most where the turn memory's exact key misses, and collapsed on p-w at s3. claude-227 had checked the trunk alone, at s1, on fixed sequences.

**Do the integer trunk alone, and the trunk with its Hamming read, compute what the float trunk computes, at s1 and at s3?** Which layer does the gap begin in?

## 2. Arms

Four copies of (r-drop) step 16,384, each without the turn memory: `fbs-snapshot` refuses readouts beside it, and its readouts do not apply it. Each copy keeps the run's `config.json` two levels up (retention on).

| Variant | Seed | Files |
|---|---|---|
| t1 | 1 | `config.json`, `model.safetensors` (repeats claude-227) |
| t3 | 3 | the same |
| h1 | 1 | plus `hamming_read.safetensors` |
| h3 | 3 | plus `hamming_read.safetensors` |

**The steps for each:**
1. `fbs-snapshot` writes the snapshot and its plain, export and D11 readouts. The read is applied to the readouts (claude-200) and carried in the snapshot.
2. **The writer copy.** The run's `objective` is `cell`, so the snapshot carries `"readout_nearest":true` (claude-231), and its artifact would serve the nearest token. The D11 readout's token is always the native writer's (`hidden_and_next`). So the export and the compare read a copy of the snapshot less exactly that last field (23 bytes), which is how claude-227's snapshots were written.
3. `fbs-export` of the writer copy (writer head, snapped).
4. `fbs-compare --parity-only` of the writer copy against the D11 readout.

**The readings,** per section (`single_tokens`: 4,096; `sequences`: 8 × 192 = 1,536):
- the writer token agreement, d = positions − agreements;
- the largest query gap.

The writer's token is on both sides, so this compares queries and the writer's pick on them. The nearest readout is not in this comparison, and its table is checked only by reading the code.

## 3. Bar and decision table

**Bar per variant:** d ≤ 3% of the positions in each section (claude-227's bar, with the same basis).

| Row | Reading | Decides | Next |
|---|---|---|---|
| 0 | **An instrument fault** (§4), including t1 not reproducing claude-227 | Nothing is located. | Repair, then rerun. |
| R | t1 and t3 pass; h1 or h3 misses | The Hamming read's integer form diverges from the float read. | integer-hamming-read's step 5 (#3385): trace the read's hooks against the float. |
| T | t3 misses (with t1 passing) | The trunk at s3 diverges, with no read involved. | A per-layer residual trace at s3, as claude-127 traced the D11 exporter. |
| B | t3 misses, and h1 or h3 misses beyond t3 | Both: reported as T, and R named. | The trunk first, then the read. |
| N | all four pass | Trunk and read agree on fixed sequences; the gap is on real prompts, or in the memory or readout. | Real-prompt readouts (`fbs-snapshot` over DEV and p-w windows, a code change), and the readout table checked on the disagreeing prompts. |

The rows are read in order; the first that holds applies.

## 4. Instrument checks (any failure is row 0)

1. **`I-Data`:** each seed's six files pass the run's own sums.
2. **`I-Export-Tests`:** the `fbs_export::` GPU tests pass at the head, with at least one run.
3. **`I-Reading-Carried`:** every snapshot carries `"retain_rates":true` once. The h snapshots carry a read, and the t snapshots none.
   - Every snapshot carries `"readout_nearest":true` once.
   - Its writer copy is exactly 23 bytes shorter and carries no flag.
   - Its artifact holds no `native_vector_nearest`.
4. **`I-Artifact-Identity`:** each artifact's sha256 equals its comparison's `artifact_sha256`.
5. **`I-Repro`:** t1 reproduces claude-227's counts exactly: 1,531 of 1,536 sequence positions and 4,023 of 4,096 single tokens. That ties this job's instrument to the earlier one.
6. **`I-Finite`:** `fbs-snapshot`, `fbs-export` and `fbs-compare` exit 0 for each variant. A variant that fails is reported as such and is not counted.

## 5. Base artifact and data split

- **Base:** (r-drop) step 16,384 at s1 and s3, from `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/`, with its run configs (`retain_rates: true`, `hamming_read: 4`).
- **Data split:** none, and no corpus is read. The readouts' inputs are `fbs_float_readout`'s 4,096 single tokens and 8 fixed sequences, token (37s + 11t + 1) mod 4,096. No TRAIN, DEV or panel split is opened.

## 6. Steps

1. The data, every file checked (`I-Data`).
2. The source pin at the pushed head.
3. The `fbs_export::` GPU tests (`I-Export-Tests`).
4. The builds.
5. The four variants, each with snapshot, export and compare (`I-Reading-Carried`, `I-Artifact-Identity`, `I-Finite`).
6. `SHA256SUMS`; then the output is stored to `results/claude/integer-nearest-readout-locate-2026-10-31/` and listed back; then pod down.
7. Read `I-Repro` and the rows from the stored comparisons.

## 7. Minimum test power

- 1,536 sequence positions and 4,096 single tokens per variant. At the 3% bar that is 46 and 122 positions.
- claude-232's p-w s3 disagreement was 95% of first tokens, and a fault of that size would show far past the bar.
- A fault that appears only on real prompts would not show here. That is row N, which names its own next step.

## 8. Cost and stop rule

- **Cost:** one RTX 5090 at about $0.56–0.95/h. claude-227's job took under 2 minutes after a warm bootstrap; four variants take a few more. That is under $1, with a $2 ceiling (`--hours 1`).
- **Stop rule:**
  - any §4 check that fails stops the job (exit 3, 4 or 6), never counted;
  - every exit copies the logs and `EXIT` into the output and seals it;
  - a watcher runs from launch to down, and a healthy run is never cancelled for a self-set budget.

## 9. Non-author TEST FITNESS review

TEST FITNESS: PENDING

## 10. Pre-run review

PENDING
