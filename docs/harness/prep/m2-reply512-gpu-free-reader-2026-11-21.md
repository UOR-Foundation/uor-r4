# Prep sheet: `m2-reply512` answered by the order-free reader, trained and read on wick's 4090 (m2-train-grounded-replies, claude-443, 21 Nov 2026)

Lab: claude. Machine: wick, Casey's own RTX 4090, bare metal. No rental. Milestone: M2 (#2030).

## The question

M2's headline (512 of 512, development) rests on the in-order TRAIN run as CUDA kernels, under the sheet [`m2-reply512-gpu-reader-2026-11-21.md`](m2-reply512-gpu-reader-2026-11-21.md). That TRAIN took 55 s for 20,000 replies and gave no result on the corpus-worded stream. The order-free TRAIN (`conjunction/free.rs`, two passes; kernels `free_walk`, `free_rows`, `free_best`, `free_keep`, `free_sort`) writes a different reader file in about 1 s. Does the reader written by the order-free TRAIN on the 4090, read by the CUDA read kernel on the 4090, give complete correct replies on the M2 development panel, so that it may be the trainer later sheets rest on?

What this sheet does not claim, whatever the result:

- **The headline number does not move.** It is 512 of 512 now. A PASS changes which TRAIN rule the count rests on, nothing else. No result here raises any milestone number.
- **This panel was already read by this reader, twice, under no sheet.** Finite kernel tests on 21 Nov (`runs/conj/gputrain8/` with binary v41, `runs/conj/gputrain10/` with binary v42) ended with a development read of this panel: 512 of 512 for both readers, control 0 of 512. So the count is known before the run. The run registers the path; it discovers nothing.
- **It is not a blind test.** `m2-reply512` is a development panel in the wording TRAIN is drawn from, and this reader line was developed on it.
- **It says nothing about fresh transfer.** `m2-fresh256` is spent for this reader line and is not read. M2's second number (fresh replies) stays 0.
- **The trunk takes no part.** The reader is keyed by the question's tokens and is carried beside the M1 trunk; the two are not yet one state.
- **The host still does part of the work.** It finds the reply runs and the pair positions and loads the tables; the kernels do the offers, the kept rows and the counts. The commands also run the host reader for comparison only: the replies that are counted are the ids the read kernel wrote.
- **The order-free rule needs two passes.** One pass answered 384 of 512 on the host (notebook row 20). The sheet reads the two-pass reader only.

## Base artifact

No model file is loaded as a starting point: each reader is counted from its TRAIN stream on the GPU in this run. The pins:

| | Reader A (TRAIN seed 1) | Reader B (TRAIN seed 2) |
|---|---|---|
| TRAIN tokens sha256 | `bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24` | `c5ecf61ce3f4a1c5b63768648bb361295d6d29b02bba28c3eb036d21bf47b9b6` |
| TRAIN mask sha256 | `89e0732b5858363e31a9729627e55e775107ababd4fda19ecc55e2aa9cb47e0f` | `225bd338e019981fcfa52d2194790f94eb2f95a062ef55e00e9b5387e4a291dd` |
| Host order-free file the GPU bytes must equal | `9db7e8348bb0bdadee7502a35716d39dacb8e9c25e57f76338fe3fea91e1227b` | `ce9615680f561344af65446b75ea7bb2657634881d34928c09a9a1d32fb89578` |

The host files are written in the run by `reader-save` with the update `free2`; their sha256 must be the ones above, which are the values of the two earlier finite kernel tests. Choices, both readers: vocabulary 4096, window 192, reply limit 48, BOS 0, EOS 1, 8 slots, no suffix orders, no token codes, pair stand, open forms, `precise,across,free2`.

Code: `conjunction-probe` as built on wick with `--features cuda` at commit `1429ea74214edc20a8b08d2b5b420278db928e0e`, saved as `runs/conj/bin/conjunction-probe-v42` (sha256 `dff3fe4280681038a38d1cb5bcb7ee860433788f3f1e46d84ec9f338bc79c617`); kernel sha256 `6b69a25f46e517cc9d99872db6c52257e8b8dd1543d092e3381ce949f673a1a8`. No build is made for this run. Kernels: `reader_train.cu`, `reader_read.cu`, `reader_common.cuh`, compiled by nvcc in `build.rs` and admitted only with no floating-point instruction. The native-space items are in [NATIVE-SPACE.md](../../loop/tasks/m2-train-grounded-replies/claude-443/NATIVE-SPACE.md).

## Data split

- TRAIN, each seed: the 20,000 synthetic cases of the earlier sheet, unchanged.
- Panel: `m2-reply512` as `m2-format` writes it, `data/m2fmt/panel.json`, sha256 `40e12f873873313bf3fe3bfb2bd25dade49a9040ed8ae11e481b61e7ec297f2e`: 512 prompts, each with the reference reply as token ids.
- The panel is not held out and has been read by this reader before: see the limits above. No sealed or fresh panel is read, and nothing of the D87 corpus is read.

## Steps

One script on wick, `runs/conj/gpufree-score.sh v42 gpuscore2`, in a new folder `runs/conj/gpuscore2/`:

1. Write the sha256 of the binary, both TRAIN streams and masks and the panel to `inputs.sha256`.
2. For reader A, then reader B: `conjunction-probe reader-save TOKENS MASK HOST.uorrdr 4096 192 48 0 1 8 - 1 - pair open precise,across,free2` (the host's order-free TRAIN), then `conjunction-probe reader-gpu-train TOKENS MASK HOST.uorrdr 4096 192 48 0 1 8 OUT.uorrdr OUT-train.json free2`. The second command trains on the GPU, and exits with an error unless every form, held row, slot and pair row equals the host's, the saved bytes equal `HOST.uorrdr`, and a run with the replies in reverse order gives the same bytes. A second GPU run is made and reported as `gpu_reader_bytes_are_the_same_on_a_second_run`; the command does not exit on it, so the PASS line asks for it by name. The script compares the six input sha256 and each host file's sha256 with this sheet's pins and prints `PINS OK` or `PIN DIFFERS`.
3. For each `OUT.uorrdr`: `conjunction-probe reader-gpu OUT.uorrdr data/m2fmt/panel.json 48 OUT-read.json`, graded as in the earlier sheet: `gpu_count.exact` is the number of replies equal to the reference id for id, EOS included.
4. Write the sha256 of every report and reader file to `outputs.sha256`. Record the counts in the M2 notebook row and the decision card.

## Bar (pre-registered)

- PASS: for reader A, every input sha256 is the one pinned in this sheet, the host file's sha256 is the one pinned, step 2 exits 0 with `gpu_reader_bytes_are_the_file` and `gpu_reader_bytes_are_the_same_on_a_second_run` true and `update` equal to `{"free":{"passes":2}}`, step 3 exits 0 with `equal` true, `gpu_prompts_read` is 512, and `gpu_count.exact` is at or above 256 of 512, M2's registered development target.
- An input sha256 that differs from this sheet voids the run. The script goes on after a failed step, so a read of a reader whose step 2 did not exit 0 is not counted, whatever it prints.
- Reader B's count is reported beside it as the spread across the TRAIN draw. If reader B's count is below 256, or its step 2 fails, the result is recorded as a pass for one seed only.
- Control: `control_gpu_count_against_the_next_prompts_reference.exact` must be under 26 of 512 for both readers; otherwise the count does not depend on which prompt was asked and the result is void. This control is not a no-reader control.
- Because the count is known to be 512, any `gpu_count.exact` under 512, any prompt not read on the GPU, or any difference from the host is a finding to be explained before the order-free reader is named as the trainer, even where the bar is met.
- A step that stops or crashes is an instrument stop for that reader and gives no count.

## Minimum test power

512 prompts; the bar is half of them. The panel is in-distribution and already read by this reader, so the run has no power to show anything about replies outside this wording, and none to show anything not already seen in the two unsheeted reads. Its power is against a regression of the path: a wrong binary, a changed input, a nondeterministic kernel.

## Cost

Under two minutes of the 4090: for each reader, a host TRAIN of about 5 s, three GPU TRAINs of about 1 s each inside one command, and a read of a few milliseconds. No money, no rental. GPU memory: 33.6 million offer slots, 16.8 million form slots and 1.0 million row slots, about 4 GB by the sizes of the tables; not measured.

## Stop rule

One pass of steps 1 to 4. A stop from a tooling fault may be repaired and the whole script run again in a new folder (`gpuscore3`, …); every run and its outcome is recorded, and no run writes into an earlier run's folder. A change to the reader, to TRAIN or to the grading needs a new sheet.

What each outcome unlocks. PASS: the decision card names the order-free TRAIN as the GPU trainer of the M2 reader, with reader A's order-free file as the saved reader for later sheets; `headline.current` stays 512; the next unit is a fresh panel in other wording, drawn by a reader that is not the author, read on the GPU with controls. Not PASS: the difference is found with the finite kernel tests; the headline stays 512 on the in-order reader of the earlier sheet, which this run does not touch.

Fitness read. Read 1 (fresh-context reader, not the author, same lab), which checked the sheet's field names and exit conditions against `conjunction-probe.rs`: FIT WITH NOTES, no blocker. Its notes are written in above: step 2 said the command exits on a differing second run, which it does not (the PASS line already asked for the field); the script now checks the pins itself and records the host files' sha256 in `outputs.sha256`. It did not trace how `open` and `precise,across,free2` are parsed, and it inferred the JSON form of `update` from the derive; the two earlier finite kernel tests printed `{"free":{"passes":2}}`.

TEST FITNESS: FIT - read 1, a fresh-context reader that is not the author (same lab, claude), with notes and no blocker. No other lab has read this sheet.
