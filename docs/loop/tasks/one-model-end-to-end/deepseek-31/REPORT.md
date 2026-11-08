# REPORT: one-model-end-to-end, deepseek-31

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Preparation only so far; no model, pod
or spend.

## What did not work, first

- The 2,048-step extension repeated row 6: stopping moved, but complete accepted
  replies stayed at zero.
- The current mix gives M-World only about 20% of tokens in the old mixed store.
  Repeating the excluded M-World store three times raises that to about 68% and
  raises scored EOS exposure from 19,625 to 39,136 per million.
- The exact `m2-reply512` bytes remain unavailable. Its deterministic lookalike
  fails both registered hashes, so it is not used.

## What changed

- Built a new 65,071-document mixed store: OASST2 once, then the same excluded
  M-World store three times.
- The store has 11,786,293 tokens, 5,710,018 scored targets and 223,469 scored
  EOS with median reply length 11.
- The panel-overlap audit is zero for exact prompts, accepted answers and shared
  six-grams.
- Added a closed `content-ratio/2048/s1-content-ratio` recipe to the job and
  launcher. Legacy 1,024 and the registered legacy 2,048 extension remain
  unchanged, and arbitrary data-root, hash, sheet, report or output overrides
  are refused.
- The job now selects the new input root and hashes only for `content-ratio`,
  validates the four-source manifest and compares the overlap audit against the
  registered document and assistant-reply counts.
- Added static-gate and mutation coverage for the recipe pair, corpus hashes,
  split counts, label, reviewed sheet/report paths and output root.

## Measurements

| file | SHA-256 |
|---|---|
| `tokens.u16` | `3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34` |
| `response_mask.u8` | `83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf` |
| `manifest.json` | `8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd` |

The EOS census is `e641c81ec8621511b68213cf8322372162ba59504d3f42f72ef1c95b2c5ed003`;
the overlap audit is `4db5d14f49ca7edaa1d44944cec842f3876c6f23a801ceacc4181bd59c14d5ee`.
The projected run cost remains about $0.50 at the observed $0.563/hour rate.

## Checks run at the head

- `gate.sh`: 276 checks passed.
- `test-mutants.sh`: 118 of 118 mutations caught.
- `test-hardware-evidence.sh`: passed.
- ShellCheck over the job, launcher and hardware helpers: passed.
- `bash -n` and `git diff --check`: passed.
- `loop report --check` and `loop pr-check --local`: passed.

Fresh FIT and D67 review remain required. Not run: pod creation, training,
export, panel scoring, store write or seal readback.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84, geometry
and delivery rules were used directly.

## Still open

- Obtain fresh exact-head FIT and D67 approval, merge through the queue and
  verify the landed runtime.
- Run the content-ratio intervention only from that merged pin.
- No seed 2 unless row 3 is reached. No second ratio extension.

## Plain summary

This unit tests whether giving the same dialogue source three times more weight can
move complete reply content while retaining stopping and human continuation.
