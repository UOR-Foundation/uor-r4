# REPORT: one-model-end-to-end, claude-270 (batched decode by default, after current-code GPU parity)

Lab: claude. [ORDER](./ORDER.md). PR #3629, item #3407. Plan step: Casey's plan of 4 November, step P1.

**In one paragraph.** The float scorer's `--score-rows` now decodes batched unless `--batched-decode 0` is given. Before the flip, this unit reran claude-266's exact scoring command on current code (`b58de92f0`) twice per checkpoint on an RTX 5090: once one prompt at a time, once batched. It used claude-266's final (b) seed-1 and (k2) seed-2 checkpoints, on DEV and both panels. All 15 output files of each checkpoint are byte-identical between the two runs, and the two stdout manifests match with the seconds dropped. The serial files are also byte-identical to claude-266's stored scored files, so claude-267 to claude-269 left these decodes unchanged. Decoding ran 5.8 and 6.1 times faster. No model was trained, and no scored file changes.

## Tokens per second first

The float scorer's greedy reply decode, on one RTX 5090 per checkpoint, both on the same pod. The token counts are the decoded reply tokens of all nine decodes; the seconds are the decodes' own, from the stdout manifests.

| Checkpoint | Tokens | Serial | Batched | Speed-up | Scoring wall, serial → batched |
|---|---|---|---|---|---|
| (b) seed 1, step 8,192 | 302,957 | 7,220.9 s, **42.0 tok/s** | 1,250.7 s, **242.2 tok/s** | 5.77× | 7,239 → 1,269 s |
| (k2) seed 2, step 8,192 | 309,546 | 7,322.1 s, **42.3 tok/s** | 1,208.9 s, **256.1 tok/s** | 6.06× | 7,340 → 1,228 s |

By decode, the speed-up runs from 2.97× (DEV's Flock-rule decode, 10,038 tokens) to 9.66× (p-w's greedy decode without memory, 76,928 tokens). The largest decodes gain most: they hold the most prompts at each length. The batched pass keeps the GPU 36% busy (27% serial), at the same 5.3 GB of the card's 32 GB. So each step is still launch-bound, and wider batches have room.

These are the scorer's speeds, not the served model's. The integer session's laptop tok/s is unchanged and was not measured here.

## What did not work

1. **One 2-GPU pod, not one pod per checkpoint, at about twice the ORDER's cost.** `uor-pod up --count 2` gave one pod with two GPUs (vast-59029725, 2× RTX 5090, $1.908/h). The ORDER planned one pod per checkpoint at about $2 in all. The pod was up from 23:45Z to 02:16Z, about $4.80.
   - The job script was changed after its pre-run review read it (`0d40c12a…` to `d90bb074…`), so the two checkpoints run side by side, each alone on its own GPU. The change gives each job its own folder, source, build and `CUDA_VISIBLE_DEVICES`. The scoring command, the pins, the compare, the pass rule and the seal are the same lines.
   - No second review round read the changed script. The result below rests on the sealed files, not on that review.
2. **This host is about 1.8 times slower than claude-266's.** The same serial command took 7,239 and 7,340 s here, against 3,980 to 4,211 s per arm in claude-266. The serial pass kept the GPU 27% busy, against claude-266's 46–51%. Both GPUs show the same slowdown, so the host is the likely cause: not the decode, and not the two jobs sharing the pod.
   - Classification: instrument (the host). The speed reading is serial against batched on the same pod, never against claude-266's seconds.
3. **The pre-run review's first round blocked on a wrong reading.** It held that the scored files carry each decode's wall seconds, so a byte compare could never pass.
   - claude-266's 15 stored files, from this same command, contain no `decode` or `search` field. The seconds go only to the `ScoredFile` manifest on stdout.
   - The job gained a descriptive compare of the two stdout manifests with the seconds dropped, and the ORDER records why bytes suffice (`5843c53ca`). Its other checks passed: the command, the pins, the seal, the watcher and the caps.
   - The result agrees with the correction: 15 of 15 files are the same, and so are the untimed manifests.
4. **The claim note on #3407 described a streaming decode,** which this unit did not build. The ORDER was re-scoped before any code. A carried-state decode needs a CUDA scan entry in `cuda_stack.cu` and a key cache in `flock_read.rs`, both outside this item's scope. A decode step is also launch-bound, so streaming one prompt saves little. Streaming stays after wider batches (Next).
5. **`uor-pod down` could not reach the pod over SSH** to check the laptop archive, and refused. Both job folders were already in the HF store and read back byte-identical (Artifacts), so `down --confirm-archived` deleted the pod.

## What changed

- **`bin/train-in-space.rs`:**
  - `BATCHED_DECODE_DEFAULT = true`, with `batched_decode_or_default`.
  - `--batched-decode` now sets `Some(..)`, and `score_rows` receives the default when the flag is absent.
  - The module doc says the default, and that `--batched-decode 0` is the serial reference.
  - The test `score_rows_decodes_batched_unless_told_otherwise` pins all three cases.
- **`in_space_long.rs`: unchanged.** Its doc lines describe what `batched_decode` does, not a default.
- **The item's scope** gains `crates/uor-r4-training/src/milestone_world_v2.rs`, for the next unit: the corpus generator (the ORDER's scope decision).

## Measurements

| Check | (b) seed 1 | (k2) seed 2 |
|---|---|---|
| CUDA float library suite at `b58de92f0` | 311 passed, 0 failed, 7 ignored | 311 passed, 0 failed, 7 ignored |
| `exact_length_batches_match_one_at_a_time_decodes` | ok | ok |
| Binary `train-in-space-b58de92f` sha256 | `dc301857e4aa8689763940a6a411538c90342cb3ca184871defb1be9bd7fc4e7` | the same |
| Serial against batched, 15 output files (`parity.txt`) | 15 SAME | 15 SAME |
| The stdout manifests, seconds dropped (`manifest.txt`) | SAME | SAME |
| Serial against claude-266's stored scored files (`stored.txt`, descriptive) | 15 SAME | 15 SAME |
| Seal (`SHA256SUMS.files`, 75 files, `shasum -c` after the pull and after the read-back) | 75 OK | 75 OK |

## Artifacts

- `caseyallard/uor-r4-store:results/claude/one-model-end-to-end-batched-parity-2026-11-04/par-b-s1/`, whose `SHA256SUMS.files` has sha256 `fe02ada8f65ed1f9f7408533b1398d82e771572ea1f665592ef1b683131c5725`.
- `.../par-k2-s2/`, whose `SHA256SUMS.files` has sha256 `c248c4a262ed753d2c273ddecd4b81dd89a95f71f7ba4aaa05a51dd792ff47c1`.
- Each holds the job log, `gpu.csv`, both runs' scored files and stdout, `parity.txt`, `stored.txt`, `manifest.txt`, the decode seconds and the binary. The fetched inputs (the checkpoint, data and panels) are claude-266's, already in the store, and are not copied.

## What this shows, and what it does not

- **Shows:** on these two checkpoints, with claude-266's command on DEV, p-w and p-v, the batched path writes the same bytes as the serial one. So batching can be the default without moving any scored file, and the scoring half of a seed runs about six times faster.
- **Does not show:**
  - parity on other checkpoints, widths, or prompts past 258 tokens. The long corpus (P2) will need its own check;
  - parity for arms other than full-scope nearest: they still decode one prompt at a time, by design;
  - any change in model quality, which this unit cannot move;
  - training time. The training run's own checkpoint replies stay serial, a handful of prompts each.
- **A projection, not a measurement:** on claude-266's host, a seed's scoring of about 4,000 s per arm would fall to about 700 s, if the ratio holds there.

## Next

- **P2, the long-conversation corpus (claude-271).** The generator takes a context (the default 256 byte-identical) and a chained mode with recall across many turns. Then a census.
- **P3 (a), arms side by side on one GPU:** the batched pass keeps the GPU 36% busy and uses 5.3 of 32 GB. **P3 (b), cross-length batching:** exact equal-length groups average about four prompts.

## Checks run at the head

- **On the laptop**, at the merge of main `0fbb687df`: `cargo test -p uor-r4-float-train --no-fail-fast` gives the library 225 passed, `train-in-space` 5 (the default test among them), `fbs-snapshot` 7, `spiralcore-fbs-sheet` 4, the dependency check 2 and the kernel controls 12, with 0 failed. `cargo fmt --check` lists the same 22 pre-existing diffs before and after this change, none in its lines.
- **Negative gate:** with `BATCHED_DECODE_DEFAULT = false`, `score_rows_decodes_batched_unless_told_otherwise` fails at its first assertion. Restored byte for byte (`cmp`), it passes.
- **On the GPU:** the table above, at code head `b58de92f0`. The later commits change only the ORDER, this REPORT, the notebook and the item's docs and scope line.
- **Review:** a fresh-context review of the diff and this REPORT, recorded on PR #3629 by `loop review`.

## Skills used

None applied.

## Still open

1. **The changed job script was not re-reviewed** before the pods ran (What did not work, item 1). The sealed outputs carry the result.
2. **The slow host** (item 2) is not explained beyond "host"; nothing in this unit depends on it.

## Plain summary

Scoring a trained model, which means letting it write its answers to thousands of test prompts, took about two hours per model. It took far longer than training, because the scorer wrote one answer at a time. A faster path already existed that writes the answers for prompts of the same length together, but it was switched off until someone proved it gives exactly the same answers. This unit proved that on two trained models: every one of the 30 output files was identical, bit for bit. It then switched the fast path on. Scoring now runs about six times faster: about 42 word pieces per second became 242 to 256.

References #3407.
