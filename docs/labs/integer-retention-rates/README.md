# Lab record: integer-retention-rates

Item: [integer-retention-rates](../../roadmap/items/integer-retention-rates.md) (#3456). It serves the recurrence's retention reading, λ = 1 − 2^−(k+1), in the integer artifact.

## Parity of the retained trunk on the GPU (claude-227, 31 Oct 2026)

- **Sheet:** [integer-retention-rates-parity-2026-10-31](../../harness/prep/integer-retention-rates-parity-2026-10-31.md). It got FIT in two non-author rounds and passed its pre-run review.
- **Run:**
  - one RTX 5090 (`vast-58504603`), from bootstrap at 07:07:38Z; the job ran 07:09:16–07:11:01Z;
  - about $0.25 by the session's estimate (no stored file records the deletion time);
  - the job is `local/claude/claude-227/job.sh`, which is not in git. Its copy is stored beside the outputs (6,898 B, sha256 `de6adae7…`, outside `SHA256SUMS`), and it ran against the pushed head `9e6836242`;
  - EXIT 0.
- **Store:** `caseyallard/uor-r4-store:results/claude/integer-retention-rates-parity-2026-10-31/`.
  - `out/` holds 46 files: 45 under `SHA256SUMS`, and that file itself.
  - The session checked the sums on the pod (`sha256sum -c`, before the upload; it is not recorded in a stored file), and listed the store back before the pod went down.
  - `pod-workspace-uor-r4/` holds the pod's job and bootstrap records.
- **Input:** (r-drop) s1, step 16,384, trunk alone (`model.safetensors` sha256 `c5fcbb90…`), with its run config (`retain_rates: true`). The six files of each seed passed the run's own sums.
- **Instruments, all green:**
  - the `fbs_export::` GPU tests: 12 passed;
  - the snapshot carries `"retain_rates":true` once, and the control differs from it only in that flag (12,481,011 against 12,481,012 bytes);
  - each artifact's sha256 equals its comparison's `artifact_sha256`: retained `003c65fd…`, decay `7f2da7cb…`;
  - every tool exited 0.

### The comparisons (`fbs-compare --parity-only`, against the D11 readout)

| Arm | Sequences agree | d | Single tokens agree | d | Largest query gap (seq / single) |
|---|---|---|---|---|---|
| A_ret, the retained artifact | 1,531 of 1,536 | 5 (0.33%) | 4,023 of 4,096 | 73 (1.78%) | 3.89 / 5.50 |
| A_dec, the decay control | 17 of 1,536 | 1,519 (98.9%) | 84 of 4,096 | 4,012 (97.9%) | 16.00 / 16.00 |

### `G-Ret-Parity`: PASS

- **(a):** d_ret is 0.33% on the sequences and 1.78% on the single tokens, both within 3%.
- **(b):** on the sequences, d_dec − d_ret = 1,514, against 2√1,524 ≈ 78.1.
- **`I-Readout-Retained`:** green. The readout agrees with the retained artifact, not the decay one.

So the retained artifact computes what the trained trunk computes, within the exporter's known residual. An engine before claude-226 would have served a different model: it agrees with the trained trunk on 17 of 1,536 sequence positions.

### The by-product (no reading taken)

`fbs-snapshot` on each seed's full checkpoint (the turn memory and the Hamming read carried, no readouts) wrote:
- `out/full/snapshot-s1.json` (60,792,526 B, `dcbfb69b…`);
- `snapshot-s2.json` (60,760,866 B, `d0522d02…`);
- `snapshot-s3.json` (60,872,197 B, `9744d385…`).

These are the snapshots point-key-integer's run (#3450) exports on the laptop. Their `retain_rates` was read from each run's config as in the trunk arm, but not checked in the files here; that run checks it.
