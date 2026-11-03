# REPORT: batched scorer, deepseek-9

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base: `b6ebf7002`.

## What did not work

- **CUDA exact parity did not run.** This workstation has no GPU, and the pod listing was unavailable. The batched path therefore remains opt-in and off by default. No decode speedup is claimed.
- **Scoped, forced-opening and served-key arms remain serial.** Their batch state is not yet rectangular or independent per sample, and weakening exactness for them would defeat the unit.
- **Mixed prompt lengths are never padded.** They remain separate groups. Padding changes recurrent history and memory-row indices, so a larger apparent batch is not worth a different model.

## What changed

`train-in-space --score-rows --batched-decode 1` now groups full-scope nearest decodes by exact prompt length. Each group is split under the existing evaluation batch/context budget and runs one rectangular recurrent forward per decode step. Results are restored to prompt order, and Flock randomness still uses each prompt's original global sample index.

Terminated rows receive a BOS placeholder for tensor shape only and are never read again. Active rows keep their exact histories. `Greedy`, `FlockRank`, `FlockCell` and `CycleStep` share the path; every unsupported arm falls back to the existing one-at-a-time decoder.

## Measurements

- Pinned DEV produced 310 prompts over 75 distinct lengths.
- Exact-length grouping reduces them to 75 groups; the largest has 32 prompts and the mean is 4.13.
- Host grouping controls: **2 passed**. Every prompt appears once, all members of a group have the same length, grouping is deterministic, and wide groups respect the `16 × 192` evaluation-token budget.
- The complete float-train library suite passed: **210 passed, 0 failed**.

## Checks run at the head

- `CARGO_TARGET_DIR=target cargo test --locked -p uor-r4-float-train --lib`: 210 passed.
- `cargo check --locked -p uor-r4-float-train --bin train-in-space`: passed.
- `git diff --check`: passed.
- CUDA test `exact_length_batches_match_one_at_a_time_decodes` is defined but **NOT_RUN** on this host. It compares complete `DecodedReply` fields under all four rules with Hamming reads and opening/reply memory rows active, and requires a non-empty reply.

## Skills used

No available Codex skill applied. The serial decoder, failure surface, prompt/corpus controls and integer serving path were read directly.

## Still open

- Run the CUDA equality test on an RTX 5090.
- If equality holds, compare cumulative decode time for serial and batched scoring on the same prompts and both panels.
- Rebase/merge forward after Claude's #3579 and rerun parity on the merged head.
- Make batching the default only after GPU equality and a reviewed speed reading both hold.

## Plain summary

The scorer can now batch prompts of exactly equal length without padding, while every uncertain arm stays on the original serial path. Host grouping and the full float library pass. Exact CUDA equality and speed remain unrun, so the feature is opt-in and makes no performance claim yet.
