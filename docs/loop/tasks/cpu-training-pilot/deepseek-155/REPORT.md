# REPORT: cpu-training-pilot, deepseek-155

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `835cfeb66`.

## Status

Decision: **termination-only blocker**. With an oracle BOS inserted at the
reference length, longest-first produced **11 / 453** exact train replies and
**13 / 449** exact dev replies on both artifact seeds. `k2>k1` produced 8/8,
random depth 0/2, and fixed depth 1 and writer produced zero. The longest copy
trajectories contain a small complete-reply signal that the actual decoder
cannot expose without a stop mechanism.

This is a ceiling analysis over sealed traces, not a deployable decoder,
language result or serving claim.

## What did not work

- Attempts 1 and 2 are instrument faults. They computed oracle body
  eligibility and trailing-mismatch histograms but copied the original
  no-stop `exact_reference` field into the oracle reading, leaving the result
  at zero. Both attempts are preserved.
- Attempt 3 had the correct oracle count but omitted report/corpus hash
  binding, per-row full-trace hashes, step-count equality and mean-prefix
  output. Attempt 4 enforced those fields but compared each row against the
  arm-level aggregate hash. Attempt 5 is the first complete analysis.
- Most copy bodies remain incomplete even with perfect stopping: 442/453 train
  and 436/449 dev longest-first replies are not exact.
- No new model generation ran. This unit cannot determine whether a trainable
  termination signal can recover the 11/13 ceiling.

## What changed

Added `cpu-copy-key-reply-oracle`, a read-only analyzer for the sealed
deepseek-153 reports. For each selected-token trace it:

1. reconstructs every row in generated-step order;
2. rejects missing, duplicate or non-monotonic decisions;
3. truncates at the normalized reference length;
4. inserts the BOS that an oracle terminator would provide;
5. recomputes exact, opening and body-position readings from the truncated
   trace;
6. compares longest-first against `k2>k1`, depth 1, random depth and writer
   using the existing paired helper.

It also reports maximum/common-prefix statistics and the number of replies
with a zero-, one-, two- or three-token tail mismatch.

## Measurements

Both artifact seeds produced identical copy-arm readings because every
deepseek-153 copy decision overrode the writer.

| Panel | Longest exact | `k2>k1` | Depth 1 | Random | Writer |
|---|---:|---:|---:|---:|---:|
| train,train 2711 | **11 / 453** | 8 | 0 | 0 | 0 |
| dev,dev 2712 | **13 / 449** | 8 | 0 | 2 | 0 |

Longest versus `k2>k1` was 3:0 train and 5:0 dev for exact oracle replies.
Against random it was 11:0 train and 11:0 dev; depth 1 and writer were also
0/0 controls. All exact and body-position paired comparisons passed.

Longest-first common-prefix totals were **325 / 4,661** and **451 / 4,614**.
Its maximum common prefix was 41 and 48 tokens. It also led `k2>k1` by 34 and
101 correct body positions.

Resources:

- release build **18.92 s** plus **0.40 s** incremental rebuild;
- four attempt-5 analyses **0.52-0.53 s each**, peak **159,424 KiB**;
- analyzer binary **1,349,440 bytes**, SHA-256
  `32b6e72c5eafb4c33f92dcf46b04c69f26679cb90d83cb85fb0c9625c2a8f2e0`;
- fresh target storage **4,663,321,830 bytes**;
- published evidence **155,159 bytes**;
- admitted cumulative projection: **78,227.677487699 s**.

Attempt-5 raw manifest SHA-256:
`f82b075e59e751b7eea1bc676dc361ce00f0552d845292ce0c3c9bfb4b5172de`.
Published evidence SHA-256:
`f3c116d2506944b084cf2f82830212bea04ebd981905c2cab90d613d3d7a54fe`.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores; **74** binary tests
  and **3** focused oracle tests passed.
- Scoped Clippy, rustfmt, evidence manifests and `git diff --check` passed.
- Report token/mask identities, each row's full-trace hash, and source
  generated-token counts match before oracle truncation.
- All four sealed reports reconstruct to the expected row counts and trace
  lengths.
- Both artifact seeds and both panels agree on the termination-only outcome.
- No serving, integer, CUDA, float-training or Claude-owned source changed.

## Size

Against base `835cfeb66`, this unit changes **2** code/manifest files
(**+491 / -0**), **8** task, notebook, roadmap, current-state and lab-record
files (**+360 / -1**), and **40** published evidence files
(**+5,037 / -0**): **50 files, +5,888 / -1**. No generated corpus or new model
binary is tracked.

## Skills used

No system-provided Codex skill applies. Project CPU research and evidence
review procedures were used.

## Still open

- Add an actual termination mechanism to the longest-first copy path.
- Measure whether a trainable terminator can recover the 11/13 exact ceiling
  without premature stops.
- Improve the final-token body reconstruction where no exact reply is yet
  reachable.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

If stopping were perfect, longest-first copying would complete 11 train and 13
dev replies, beating its controls. Perfect stopping is therefore the immediate
blocker. The copy body itself still misses most references, so termination and
last-token accuracy are the two next targets.
