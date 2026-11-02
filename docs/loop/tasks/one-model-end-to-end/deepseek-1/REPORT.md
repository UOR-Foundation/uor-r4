# REPORT: one-model-end-to-end, deepseek-1

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The existing panel writer did not produce the canonical manifest required by the corpus loader.** Its `manifest.json` omitted `mask_schema`, split, byte counts, content hashes and tokenizer identity. The writer now emits those fields; two earlier panel directories were retained under `local/deepseek/m2-evidence/` and are not evidence.
- **A direct dependency on the CPU training crate produced conflicting `uor-r4-integer` feature builds in the standalone instrument.** The evidence lane uses the integer serving crate directly and records independent CPU-trunk parity as unavailable instead of forcing a broken dependency graph.
- **Generated complete-reply and generated novelty are not measured.** The available public input is a teacher-forced UORT panel. Reporting those metrics as zero would confuse reference overlap with model generation, so they are explicitly unavailable.

## What changed

- Added `m2-evidence` to the existing standalone one-model instrument.
- Added a deterministic run index with arena-backed exact prompt, reply and n-gram membership.
- Added canonical UORT plus manifest identity validation, target capping and artifact/corpus hashes.
- Added next-token admission, top-1, top-8 and stratified readings for prompt overlap, first-8 memory keys, reference-reply overlap and reply opening/continuation.
- Updated `heldout-panel` to emit the complete corpus manifest needed by the evaluator.
- Added four focused tests for lexicographic run matching, prompt/reply metadata, deterministic capping and 4-/8-gram overlap.

Size: implementation and tests are mostly code; the panel-manifest repair is a small code/doc change.

## Measurements

Public smoke run:

- Artifact: `t16-frozen-zero-seed1-trunk.d11`, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
- Panel: generated M-world v2 `dev,dev`, seed 4201, 16 conversations, 1,925 tokens, 616 response targets.
- Reference index: generated M-world v2 `train,train`, seed 4101, 64 conversations, 7,906 tokens, 2,708 response targets.
- This reference index is a **generated control**, not the canonical TRAIN corpus. The public M1 artifact and panel are real; the overlap index is a smoke fixture, not a capability result.
- All 616 response targets: admission 365, top-1 293, top-8 306.
- Reply openings: 58 targets, admission 7, top-1 7, top-8 7.
- Reply continuations: 558 targets, admission 358, top-1 286, top-8 299.
- All 616 prompts were absent from the generated reference index and all 616 first-8 keys were absent.
- Of 58 reference replies, 19 occurred verbatim in the generated reference index; 241 of 442 reply 4-grams and 158 of 234 reply 8-grams were absent.

Command:

```sh
target/release/m2-evidence artifact=.../m1-s1.d11 panel=.../dev-panel-v3 train=.../train-control-v3 out=.../evidence-deepseek-1-smoke.json
```

Report SHA-256: `c8575d290eac2ece0674fca188b712fc21e6e1949be7f9bbb2e80c4f4f883405`.

## Checks run at the head

- `cargo test --locked --quiet`: 4 evidence tests and 1 panel-writer test passed.
- `cargo run --release --locked --bin m2-evidence`: public artifact/panel smoke run completed and wrote the measured report.
- The command reads the integer artifact through `IntegerDiscreteModel::evaluate_prefix`; it does not invoke CUDA or modify model weights.

## Skills used

None.

## Still open

- Run against the canonical public TRAIN split, which is unavailable in this worktree because its HF store is private.
- Add generated-reply input so complete/incorrect/loop and generated n-gram novelty can be scored without conflating them with reference overlap.
- Add the existing value-masked template census as an input, or accept a precomputed stratum file.
- Run the same report on the active FBS integer artifact after Q5 parity lands.

## Plain summary

This unit adds a laptop-only report that separates a model's next-token accuracy by whether the prompt, its exact first-eight-token memory key, or its reference reply appeared in the supplied training material. On a public M1 model and a generated panel it could identify new prompts and unkeyed openings, but it deliberately says it cannot judge generated complete replies or generated novelty without those outputs. The result is an evaluation instrument, not a claim that the model has learned to converse.
