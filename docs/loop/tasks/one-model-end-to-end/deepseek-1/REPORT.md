# REPORT: one-model-end-to-end, deepseek-1

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The existing panel writer did not produce the canonical manifest required by the corpus loader.** Its `manifest.json` omitted `mask_schema`, split, byte counts, content hashes and tokenizer identity. The writer now emits those fields; two earlier panel directories were retained under `local/deepseek/m2-evidence/` and are not evidence.
- **Full two-sided CPU-trunk parity is unavailable for the 19-table M1 artifact.** The CPU research path implements the four-table discrete read family, not the additional 15 M1 trunk tables. The evidence lane therefore records a separate independent-writer check from the serving trunk's Q12 query and names full independent trunk parity unavailable rather than calling two calls into serving parity.
- **Generated complete-reply and generated novelty are not measured.** The available public input is a teacher-forced UORT panel. Reporting those metrics as zero would confuse reference overlap with model generation, so they are explicitly unavailable.
- **The first evidence draft accepted empty inputs, could replace an output in a race and did not bind all manifest identity fields.** These were found in independent review. Empty corpora, zero targets, incomplete/unknown arguments, malformed hashes/content and existing output paths now fail closed, and reports use `create_new`.

## What changed

- Added `m2-evidence` to the existing standalone one-model instrument.
- Added a deterministic run index with arena-backed exact prompt, reply and n-gram membership.
- Added canonical UORT plus manifest identity validation, target capping and artifact/corpus hashes.
- Added next-token admission, top-1, top-8 and stratified readings for prompt overlap, first-8 memory keys, reference-reply overlap and reply opening/continuation.
- Added independent `CpuWriterCore` recomputation from the integer serving trunk's Q12 query, with fail-closed admitted-cell, top-8 and token parity.
- Bound manifest bytes, split, M-world cell, seed, conversation count and tokenizer hash into each corpus identity. `train-role` distinguishes canonical TRAIN from a generated control.
- Updated `heldout-panel` to emit the complete corpus manifest needed by the evaluator.
- Added nine library tests and three CLI tests covering exact run indexing, prompt/reply metadata, deterministic capping, 4-/8-gram overlap, malformed UORT/manifests, artifact vocabulary/BOS mismatch, empty inputs, absent-reference strata, a planted writer-metric error, deterministic serialization, argument refusal and create-new output.

Generated panel/token payloads are not committed.

## Measurements

Public smoke run:

- Artifact: `t16-frozen-zero-seed1-trunk.d11`, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
- Panel: generated M-world v2 `dev,dev`, seed 4201, 16 conversations, 1,925 tokens, 616 response targets.
- Reference index: generated M-world v2 `train,train`, seed 4101, 64 conversations, 7,906 tokens, 2,708 response targets.
- This reference index is a **generated control**, not the canonical TRAIN corpus. The public M1 artifact and panel are real; the overlap index is a smoke fixture, not a capability result.
- All 616 response targets: admission 365, top-1 293, top-8 306.
- Reply openings: 58 targets, admission 7, top-1 7, top-8 7.
- Reply continuations: 558 targets, admission 358, top-1 286, top-8 299.
- All 58 prompts, covering the 616 scored targets, were absent from the generated reference index, and all 58 first-8 keys were absent.
- Of 58 reference replies, 19 occurred verbatim in the generated reference index; 241 of 442 reply 4-grams and 158 of 234 reply 8-grams were absent.
- The independent CPU writer matched integer serving on all 616 rows: zero admitted-cell, top-8 and top-1 mismatches.
- Full independent CPU-trunk parity remains `unavailable`; the writer check begins from each serving row's Q12 query.

Command shape:

```sh
target/release/m2-evidence artifact=.../m1-s1.d11 \
  panel-tokens=.../panel/tokens.u16 \
  panel-mask=.../panel/response_mask.u8 \
  panel-manifest=.../panel/manifest.json \
  train-tokens=.../train-control/tokens.u16 \
  train-mask=.../train-control/response_mask.u8 \
  train-manifest=.../train-control/manifest.json \
  train-role=generated_control \
  out=.../evidence-deepseek-1-smoke.json
```

The exact panel and control identities are in `evidence-deepseek-1-smoke.json`.
Report SHA-256: `7e1d0d43a9fad38712f57aeb6ec5b61de82160e23a82ba110251eb78a96a508e`.

## Checks run at the head

- `cargo test --offline --all-targets`: 9 evidence tests, 1 panel-writer test and 3 CLI tests passed.
- `cargo clippy --offline --all-targets -- -D warnings`: passed; only pre-existing path-dependency warnings appeared.
- `rustfmt --check` on the changed Rust files: passed.
- `cargo run --release --locked --bin m2-evidence`: public artifact/panel smoke run completed and wrote the measured report.
- `uor-harness check`: 0 errors and 40 existing/site-wide warnings.
- `cargo run --locked -p xtask -- roadmap check`: 134 items OK.
- `cargo run --locked -p xtask -- links`: completed; it reports repository-wide historical broken links, with no command failure.
- The command reads the integer artifact through `IntegerDiscreteModel::evaluate_prefix`; it does not invoke CUDA or modify model weights.

## Skills used

None.

## Still open

- Run against the canonical public TRAIN split, which is unavailable in this worktree because its HF store is private.
- Add generated-reply input so complete/incorrect/loop and generated n-gram novelty can be scored without conflating them with reference overlap.
- Add the existing value-masked template census as an input, or accept a precomputed stratum file.
- Add a CPU implementation of the 15 additional M1 trunk tables to obtain full two-sided CPU-trunk query parity; the current independent writer check is exact but begins from the serving query.
- Supply count-table, memory-off and no-read arm files to turn their `unavailable` entries into measured controls.
- Run the same report on the active FBS integer artifact after Q5 parity lands.

## Plain summary

This unit adds a laptop-only report that separates a model's next-token accuracy by whether the prompt, its exact first-eight-token memory key, or its reference reply appeared in the supplied training material. On a public M1 model and a generated panel it could identify new prompts and unkeyed openings. It also independently reproduces the serving writer on every scored query, while clearly naming the part of the CPU parity path that does not yet exist. It deliberately says it cannot judge generated complete replies or generated novelty without those outputs. The result is an evaluation instrument, not a claim that the model has learned to converse.
