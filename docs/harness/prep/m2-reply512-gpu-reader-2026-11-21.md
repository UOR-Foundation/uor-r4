# Prep sheet: `m2-reply512` answered by a reader trained and read on wick's 4090 (m2-train-grounded-replies, claude-443, 21 Nov 2026)

Lab: claude. Machine: wick, Casey's own RTX 4090, bare metal. No rental. Milestone: M2 (#2030).

## The question

On 21 Nov the owner was asked whether the saved question reader's CPU-trained, CPU-scored count (512 of 512 on `m2-reply512`) stands as M2's headline, and answered: "No, GPU first". Does a reader whose TRAIN ran as CUDA kernels on the 4090, read by a CUDA kernel on the 4090, give complete correct replies on the M2 development panel?

What this sheet does not claim, whatever the result:

- **It is not a blind test.** `m2-reply512` is a development panel in the wording TRAIN is drawn from, and this reader line was developed on it. The host reader answers it 512 of 512. The GPU TRAIN already wrote reader A's file byte for byte (finite kernel test, `runs/conj/gputrain2/`), and the read kernel already gave the host's reply ids on this panel for the host-trained files. So the expected count is 512 of 512 for both readers. The run qualifies the GPU path for the count; it discovers nothing about the reader.
- **It says nothing about fresh transfer.** `m2-fresh256` is spent for this reader line and is not read. M2's second number (fresh replies) is not touched by this sheet.
- **The trunk takes no part.** The reader is keyed by the question's tokens. It is carried beside the M1 trunk in a D11 artifact; the two are not yet one state.
- **The GPU TRAIN is slower than the host** (55.4 s against 2.1 s for reader A), and the host finds the reply runs, groups pair positions and orders the launches. Both commands also run the host reader, for comparison only: the replies that are counted are the ids the read kernel wrote.

## Base artifact

No model file is loaded as a starting point: each reader is counted from its TRAIN stream on the GPU in this run. The pins:

| | Reader A (TRAIN seed 1) | Reader B (TRAIN seed 2) |
|---|---|---|
| TRAIN tokens sha256 | `bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24` | `c5ecf61ce3f4a1c5b63768648bb361295d6d29b02bba28c3eb036d21bf47b9b6` |
| TRAIN mask sha256 | `89e0732b5858363e31a9729627e55e775107ababd4fda19ecc55e2aa9cb47e0f` | `225bd338e019981fcfa52d2194790f94eb2f95a062ef55e00e9b5387e4a291dd` |
| Host-trained file the GPU bytes must equal | `44574adb84fd0c4cd30e8c182cf0f835b5d0348618b6b248b23c2c5742972c3c` | `078dcfdc05cfa8ab13d14120e46d9c253816cc84b1f4223f3d6cc9fe38810c2b` |

Reader A is the reader the claude-443 decision card selected as M2's saved model (carried in D11 artifact `604fcd03…a7e4dd`); reader B is the second TRAIN seed. Choices, both readers: vocabulary 4096, window 192, reply limit 48, BOS 0, EOS 1, 8 slots, no suffix orders, no token codes, pair stand, open forms, `precise,across`.

Code: `conjunction-probe` built on wick with `--features cuda` from the commit that adds this sheet, saved as `runs/conj/bin/conjunction-probe-v39` (sha256 `d863d3e8c752f80c690f76907889304b3a3c5cae3211f630946f40c74aa6e1b8`). Kernels: `reader_train.cu`, `reader_read.cu`, `reader_common.cuh`, compiled by nvcc in `build.rs` and admitted only with no floating-point instruction. The native-space items (state schema, shared operators, replay oracle, update rule, lossless format) are in [NATIVE-SPACE.md](../../loop/tasks/m2-train-grounded-replies/claude-443/NATIVE-SPACE.md).

## Data split

- TRAIN, each seed: 20,000 synthetic cases from `conjunction-probe m2-format` over the published grammar, with values from a pool of whole-word tokens that no development case uses. These are the streams of the sheet [`m2-fresh256-reader-read-2026-11-21.md`](m2-fresh256-reader-read-2026-11-21.md), unchanged.
- Panel: `m2-reply512` as `m2-format` writes it, `data/m2fmt/panel.json`, sha256 `40e12f873873313bf3fe3bfb2bd25dade49a9040ed8ae11e481b61e7ec297f2e`: 512 prompts, each with the reference reply as token ids.
- The panel is not held out: see the first limit above. No sealed or fresh panel is read, and nothing of the D87 corpus is read.

## Steps

One script on wick, `runs/conj/gpuscore-run.sh score v39 gpuscore1`, in a new folder `runs/conj/gpuscore1/`:

1. Write the sha256 of the binary, both host files, both TRAIN streams and masks and the panel to `inputs.sha256`.
2. For reader A, then reader B: `conjunction-probe reader-gpu-train TOKENS MASK HOST_FILE 4096 192 48 0 1 8 OUT.uorrdr OUT-train.json`. It trains on the GPU, builds a reader from the GPU tables, and exits with an error unless every form, held row, slot and pair row equals the host's and the saved bytes equal `HOST_FILE`. `OUT.uorrdr` is the GPU-trained reader.
3. For each `OUT.uorrdr`: `conjunction-probe reader-gpu OUT.uorrdr data/m2fmt/panel.json 48 OUT-read.json`. One thread a prompt writes the reply ids on the GPU. The command counts those ids against the panel's references with `count_replies_by`, the function `reader-replies` uses for the host: `gpu_count.exact` is the number of replies equal to the reference id for id, `gpu_count.ended` the number ending on EOS. It also reports `gpu_prompts_read` and the same count for each reply set against the next prompt's reference.
4. Write the sha256 of every report and reader file to `outputs.sha256`. Record the counts in the M2 notebook row, the decision card and #2030.

How a reply is graded: a reply is complete and correct when it is the reference, id for id, EOS included. All 512 references decode to an answer the development labels accept (checked at commit f8198c2b5 for the earlier sheet).

## Bar (pre-registered)

- PASS: for reader A, step 2 exits 0 (GPU bytes are the host file), `gpu_prompts_read` is 512, and `gpu_count.exact` is at or above 256 of 512, M2's registered development target. The number reported for the headline is reader A's `gpu_count.exact`.
- Reader B's count is reported beside it as the spread across the TRAIN draw. If reader B's count is below 256, or its step 2 fails, the result is recorded as a pass for one seed only, and the decision card says so.
- Control: `control_gpu_count_against_the_next_prompts_reference.exact` must be under 26 of 512 for both readers. If it is not, the count does not depend on which prompt was asked and the result is void.
- Because the expected count is 512, any `gpu_count.exact` under 512, any prompt not read on the GPU, or any difference from the host reported by either command is recorded as a finding to be explained before the headline is changed, even where the bar is met.
- A step that stops or crashes is an instrument stop for that reader and gives no count.

## Minimum test power

512 prompts; the bar is half of them. The control is expected near 0. Two TRAIN seeds bound the spread from the TRAIN draw; both read the same 512 prompts, so they are not independent samples of the panel. The count carries no sampling claim beyond this panel, and the panel is in-distribution, so its power to show anything about replies outside this wording is none.

## Cost

About eight minutes of the 4090: two readers, each trained three times on the GPU by the command (twice for determinism, once as the reversed-order control) at about a minute each, and two reads of a few milliseconds. No money, no rental, no lease beyond wick. GPU memory is about 2 GB by the sizes of the tables (16.8 million form slots of 16 words, 2.1 million row slots of 68 words); it was not measured.

## Stop rule

One pass of steps 1 to 4. The panel is a development panel, so a stop from a tooling fault may be repaired and the whole script run again in a new folder (`gpuscore2`, …); every run and its outcome is recorded, and no run writes into an earlier run's folder. A change to the reader, to TRAIN or to the grading needs a new sheet.

What each outcome unlocks. PASS: the decision card records the GPU-trained, GPU-read count and `headline.current` for M2 is set to reader A's count; the next units are a fresh panel in other wording with controls, and the reader keyed by the trunk's state. Not PASS: the difference is found with the finite kernel tests and fixed, and the headline stays 0.

TEST FITNESS: PENDING - no reader other than the author has read this sheet yet.
