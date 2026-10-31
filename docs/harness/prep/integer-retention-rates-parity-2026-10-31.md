# Prep sheet: parity of the retained trunk on the GPU (integer-retention-rates done-when 3, claude-227), 2026-10-31

- Item: [integer-retention-rates](../../roadmap/items/integer-retention-rates.md) (#3456). ORDER: [claude-227](../../loop/tasks/integer-retention-rates/claude-227/ORDER.md).
- Run: one GPU job on one leased RTX 5090. Steps: `fbs-snapshot` with its three readouts on (r-drop) s1's trunk, then `fbs-export` and `fbs-compare --parity-only` on two artifacts. The job is `local/claude/claude-227/job.sh` at the pushed head; this sheet quotes its steps.

## 1. Question

Every one-model-end-to-end checkpoint since 27 Oct trained with `retain_rates: true`. In those, the recurrence keeps λ = 1 − 2^−(k+1) of its state, with keep = √(1 − λ²). claude-226 (#3457, `bd9660fa7`) taught the integer engine that reading, as `rec_gates = "geometric-retention"`. It is checked on a synthetic stack against an f64 reference.

**Does the retained artifact of a trained checkpoint compute what the trained float trunk computes?** And would an engine without claude-226 have served something measurably different? The trained checkpoint is (r-drop) s1 step 16,384, the model point-key-integer's run serves.

## 2. Arms

All arms read one snapshot of one trunk and one set of float readouts.

- **The trunk alone.** A copy of the checkpoint with only `config.json` and `model.safetensors`, beside the run's `config.json` two levels up. It leaves out the Hamming read and the two turn-memory files.
  - `fbs-snapshot` refuses readouts beside `turn_memory.safetensors` (`check_readout_reads`), because the float readouts do not apply the memory.
  - Leaving out the Hamming read keeps the comparison to the trunk's recurrence alone, where the reading lives. The read's own parity is integer-hamming-read's step 5 (#3385).
- **F, the float reference:** the D11 readout from `fbs-snapshot` on the GPU. It computes the trunk with its tables rounded as exported and the transition snapped to the 120 icosians, with retention on from the run's config (claude-226). It covers every token of the 4,096 alone, plus `READOUT_SEQUENCES = 8` fixed full-context sequences: 8 × 192 = 1,536 positions.
- **A_ret, the retained artifact:** `fbs-export` of the snapshot, which carries `"retain_rates":true`.
- **A_dec, the decay control:** `fbs-export` of the same snapshot with the flag set false. It is the only change, checked byte for byte with the flag stripped. This is the trunk an engine before claude-226 served.

`fbs-compare --parity-only SNAPSHOT readout.json readout-export.json readout-d11.json` gives, per section (`single_tokens`, `sequences`): the `positions`, the positions where F's token equals the integer session's (`d11_gpu_token_equals_integer_token`), and the largest query gap. For arm X in section s, d_X,s = positions − agreements.

## 3. Bar and decision table

| Gate | Bar | PASS unlocks | FAIL decides (instruments green) |
|---|---|---|---|
| `G-Ret-Parity` | **(a)** d_ret,s ≤ 3% of the positions in **each** section, **and** **(b)** on `sequences`, d_dec − d_ret > 2√(d_dec + d_ret) | Done-when 3 is met and the item is done. Point-key-integer's run serves the retained artifacts. | See below. |

- **Why 3% for (a):** a correct exporter's residual on this trunk family. claude-126's clean seed differed on 1.4% of single tokens and 1.6% of DEV-window positions at 6 layers. After claude-127's repair, about 1% of single tokens differed at 6 layers. Twice the larger reading leaves room for a new trunk's rounding, and still catches an export fault like claude-126 seed 1's (31% and 63%).
- **Why (b) is safe without per-position tokens:** let b count the positions where A_ret agrees and A_dec does not, and c the reverse. Then d_dec − d_ret = b − c exactly, and b + c ≤ d_dec + d_ret. So (b) implies b − c > 2√(b + c), the paired test, and is stricter than it. Without (b), a pass of (a) could not tell the retained reading from the decay one.
- **(a) fails, any (b): an instrument fault.** The retained export does not compute the trained trunk.
  - The next unit traces the sequences' largest query gaps, layer by layer, against claude-127's residual.
  - The point-key run is held until the fault is repaired. It is never a mechanism verdict.
- **(a) passes, (b) fails: inconclusive on the reading.**
  - The retained artifact agrees with the trained trunk, so the point-key run may serve it. But on these sequences the trunk's tokens do not tell the readings apart.
  - Done-when 3 is recorded as met for (a) only, with (b) reported as a power limit. The next unit reads DEV windows (`fbs_float_readout_on`) instead of the fixed sequences.
- **Expected:** a PASS. Between the readings, λ swings from 2^−(k+1) to 1 − 2^−(k+1) at every index past 0, so A_dec should disagree on most sequence positions.
- **No bar (readings):** the single tokens' d_dec, the largest query gap per section and arm, and the by-product snapshots (step 7).

## 4. Instrument checks (each must be green before the gate counts)

1. **`I-Export-Tests`:** the `fbs_export::` GPU tests pass at the head, with at least one test run (`--include-ignored`).
2. **`I-Data`:** each seed's six files (the run's `config.json` and five checkpoint files) pass `sha256sum -c` against the run's own `SHA256SUMS`.
3. **`I-Reading-Carried`:** the snapshot carries `"retain_rates":true` exactly once. The control differs from it only in that flag.
4. **`I-Artifact-Identity`:** each artifact's sha256 equals the `artifact_sha256` its comparison reports. `fbs-compare` re-exports from the snapshot, so the file and the comparison are the same artifact.
5. **`I-Finite`:** `fbs-snapshot` and both `fbs-compare` runs exit 0. `fbs-compare` exits non-zero only when the snapshot does not export, parse or serve, or a readout does not fit.
6. **`I-Readout-Retained`:** F was taken under retention. Its only test is the run itself: if d_ret exceeds d_dec on `sequences` by more than 2√(d_dec + d_ret), F is the decay reading's. That is an instrument fault in `fbs-snapshot`'s config reading, not a gate result.

## 5. Base artifact and data

- **Base:** (r-drop) s1, step 16,384, from `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-2026-10-29/training-460fe46e/s1/r-drop-s1/`.
  - **Its run config:** `retain_rates: true`, `fold_scores: false`, `reflect_floor: false`, `hamming_read: 4`, `turn_memory: 8`, `turn_memory_reply: true`.
  - **Its model config:** width 256, context 192, pattern `rrrrrr`, `rec_gates: "geometric"`, Hamilton gates, rotation on, vocabulary 4,096.
  - **The trunk:** `model.safetensors`, 4,457,344 bytes. Seeds 2 and 3 are fetched only for the by-product.
- **Data:** none beyond the checkpoint. The readouts' sequences are fixed by `fbs_float_readout`: token (37s + 11t + 1) mod 4,096. They are synthetic, not text, which parity does not need.
- **Pins:**
  - the pushed head;
  - the three binaries' sha256 (`binaries.sha256`);
  - the trunk copy's sha256 (`s1/trunk.sha256`);
  - every output under `SHA256SUMS`.

## 6. Steps and minimum test power

1. Pin the source at the pushed head; the job refuses a dirty tree or a reused output folder.
2. **Probe a:** the `fbs_export::` GPU tests (`I-Export-Tests`).
3. **The builds:** `fbs-snapshot` (`--features cuda`), then `fbs-export` and `fbs-compare` (`-p uor-r4-integer --features serving-wasm`).
4. **The data:** fetch and check (`I-Data`).
5. **The snapshot:** the trunk copy, then `fbs-snapshot` with the three readouts (`I-Finite`). Then the control (`I-Reading-Carried`).
6. **Each arm:** `fbs-export`, then `fbs-compare --parity-only` (`I-Artifact-Identity`, `I-Finite`). The job logs the counts, never the bar.
7. **The by-product, with no reading taken:** `fbs-snapshot` on each seed's full checkpoint (memory and read carried, no readouts). These are the snapshots point-key-integer's run exports on the laptop. A refusal is reported and leaves the bar unaffected.
8. **Archive:** the `SHA256SUMS`, then `uor-pod down --to-store` into `results/claude/integer-retention-rates-parity-2026-10-31/`. The session reads the bar off the stored comparisons, with `jq`.

**Minimum test power:** one seed, as the item names. In each arm, 4,096 single tokens and 1,536 sequence positions. At d_ret ≈ 1%, (b) needs only d_dec ≳ 15 + 2√(15 + d_dec); any d_dec past about 30 of 1,536 passes it.

## 7. Cost and stop rule

- **Cost:** one RTX 5090 at about $0.95–1.90/h, inside the caps of 4 pods and $8/h.
  - The builds take about 10–15 minutes and the tests a few. Each `fbs-snapshot` and `fbs-compare` takes seconds to a minute.
  - The estimate is under $1, with a ceiling of $2 (`--hours 1`, renewed while healthy).
- **Stop rule:**
  - Any instrument check that fails stops the job, with its exit code (3 data, 4 tests or build, 5 a tool, 6 an identity check), and never counts as a gate result.
  - Never cancel a healthy run for a self-set time estimate; extend the lease.
  - A watcher is armed for the pod from launch to down.

## 8. Non-author TEST FITNESS review

TEST FITNESS: PENDING

## 9. Pre-run review

PENDING
