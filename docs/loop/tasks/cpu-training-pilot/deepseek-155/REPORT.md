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
- four attempt-3 analyses **0.52-0.54 s each**, peak **159,512 KiB**;
- analyzer binary **1,346,288 bytes**, SHA-256
  `29a9aee4cd8bbdbb7c2ee4f981f00b9434d1087289d3f01e0b7e9857a7782fdf`;
- fresh target storage **4,663,321,830 bytes**;
- published evidence **109,581 bytes**;
- admitted cumulative projection: **78,227.677487699 s**.

Attempt-3 raw manifest SHA-256:
`ac7ddd5bb6bb5bb54cddbca5e1b5b01edbcdf97b78f2dfe7aa5b7d86929af82a`.
Published evidence SHA-256:
`5bf4fbf494fd21b45b61d5e410888cb30d3cb1926495f973ae548de5499aaf80`.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores; **74** binary tests
  and **3** focused oracle tests passed.
- Scoped Clippy, rustfmt, evidence manifests and `git diff --check` passed.
- All four sealed reports reconstruct to the expected row counts and trace
  lengths.
- Both artifact seeds and both panels agree on the termination-only outcome.
- No serving, integer, CUDA, float-training or Claude-owned source changed.

## Size

Against base `835cfeb66`, this unit changes **2** code/manifest files
(**+415 / -0**), **8** task, notebook, roadmap, current-state and lab-record
files (**+343 / -1**), and **28** published evidence files
(**+3,579 / -0**): **38 files, +4,337 / -1**. No generated corpus or new model
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
