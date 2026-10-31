# Decision card: the probe's time projection (one-model-end-to-end, claude-220, 31 Oct 2026)

Lab: claude. ORDER: [claude-220](ORDER.md). Sheet: [one-model-end-to-end-served-key-2026-10-31](../../../../harness/prep/one-model-end-to-end-served-key-2026-10-31.md), Steps 4 and the Stop rule.

## What happened

The pod job (`vast-58466776`, built at `eb762e602`) stopped at its probe with exit 7, "the probe projects the main run past 5 hours". The probe itself passed:
- the crate's CUDA tests, including the served key's;
- **all 7 checks against claude-219:** six DEV files byte for byte, and `keys-dev.json` without its wall-clock fields;
- **the served arm's parity on DEV:** with `served_distance` stripped, it equals claude-219's keyed-point arm byte for byte.

No deciding panel has been read: the probe reads DEV only.

## The question

**May the projection be corrected, keeping the 5-hour stop, and the main run continue on the same pod from the same probe, binary and inputs?**

## The projection was wrong in two ways

The sheet's rule took the probe's whole wall time, 1,053 s, times 18: about 6 times the replies, times 3 for three seeds sharing the GPU. That gives 315 minutes. The probe's own output (`probe.out`, one line a file) shows two errors in it:
- **About a quarter of the probe does not scale with the replies.** The five DEV decodes took 800 s in all (about 106 tok/s, one process alone). The other 253 s are the TRAIN table's build, the rows and the model's load, once a checkpoint.
- **Sharing costs about 2 times, not 3.** claude-219's main run, three seeds sharing the same GPU type, decoded at 55–57 tok/s a process. That is half the probe's 106 alone.

## The corrected projection

The fixed overhead (the table build is the same TRAIN pass; the rows grow with p-w) plus 6.1 × 800 s × 2 is about 10,000 s, **about 2.8 hours**. Scaling every second of the probe by 6.1 × 2 bounds it at 1,053 × 12.2 ≈ 12,850 s, **about 3.6 hours**. Both are under the same 5-hour stop, and in line with the sheet's own 3–3.5 hour budget. The first watcher ended with job 1. A new one is armed before job 2 starts, with its time stop at 6 hours from the pod's start (`MAXSEC=21600`, about 4.5 hours past job 2's launch) and its $5 budget. Measured alongside, condition (c): the served search took 0.11 s for 19 single-query searches on DEV, about 5.8 ms a search.

## Recommendation

Correct the projection as above. Keep the 5-hour stop. Continue on the same pod with a continuation job (`job2.sh`):
- it refuses unless job 1 ended 7 after its 7 of 7, with the same binary and an empty `out`;
- it runs the main run, then job 1's p-v and DEV checks at every seed, and the expected files.

This changes no bar, row, panel or measure, and no deciding data has been read. Reversal costs a rerun.

## The decision review: APPROVE (a fresh non-author reviewer, 31 Oct, D68)

It checked:
- the numbers: 799.8 s for 85,012 tokens, 106.3 tok/s alone, against claude-219's 56.0–56.7 with three sharing;
- that this repairs a cost instrument and moves no goalpost: no bar, row, panel or measure changes, the 5-hour stop is kept, and p-w is unread;
- the continuation's refusals.

Its conditions, met:
- the fully scaled 3.6-hour bound is recorded;
- the watcher sentence is corrected, and a new watcher is armed before job 2;
- `job2.sh` also refuses on a dirty source, and re-checks the panels' and checkpoints' sums.
