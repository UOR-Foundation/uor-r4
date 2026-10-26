# M1 address-as-read Stage 0: D11 copy non-vacuity and GPU parity

- **Item:** `m1-address-as-read` (#2833), M1/M3 exact memory. **Unit:** kimi-40, `References` only.
- **Decision:** `docs/loop/tasks/m1-address-as-read/kimi-40/DECISION.md`. This supersedes the old 54/80 emission plan and removes BPB from this sheet.
- **Run:** zero training. One RTX 5090 GPU job, estimated <15 min at about $0.56/h (<$0.25 cap), one run, no retry after FIT.
- **Artifact:** T16_fz Seed 1 D11 `trunk-model.d11`, 542,260 bytes, SHA-256 `cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd`. This is the position-stripped Stage T snapshot, not Level 3 trunk tables. Original checkpoint JSON is `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4`.
- **Tokenizer:** `tokenizer.json`, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **DEV:** `ft/dev/tokens.u16` `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; `response_mask.u8` `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 14,287 rows.
- **Copy manifest:** `docs/labs/m1-address-as-read/stage0-copy.json`, 40 rows, SHA-256 `ea36011870e3f6d0add87e9fc1d25115ffc77f2a9206d13bfe304c6785351fa5`.

## Question

Can the pinned D11 artifact and its GPU scorer produce a non-vacuous exact copy of an explicitly supplied value, while agreeing with a second GPU evaluator? If not, a hidden-fact address-oracle panel would be an invalid instrument.

## Stage 0 controls

1. **GPU-only artifact parity.** Run `discrete-stack eval-corpus` on the original checkpoint and `d11-scorer score` on the D11, both on the GPU pod. Do not run `IntegerDiscreteModel::evaluate_prefix` or any other host model forward. Join by ordinal, target position and target token. All 14,287 rows must have status 0; any refusal is an instrument stop. Compare per-row prediction, top-8 and admitted cells. Record top-1/top-8/admission and the aggregate position-strip difference from retained 455/1,484; no row-level baseline is assumed. Two independent runs must produce byte-identical records.
2. **Executable identity at dispatch.** Record the Stage 0 git HEAD, `discrete-stack` and `d11-scorer` executable SHA-256, `cubin_sha256`, `ptx_sha256`, `IMAGE_TARGET`, `RESOURCES`, CUDA/driver version, device capability, source/kernel/geometry/driver identities, and the original checkpoint and D11 hashes above. Stage 3 binary hashes are not reused as Stage 0 identities.
3. **Copy manifest and rendering.** The 40 fixed rows use `supplied value is {letters separated by spaces}.\nQuestion: what is the supplied value?\nAnswer: `; the no-hint arm contains only the question. Exact and random hints are eight distinct lowercase letters, token-length matched at both the answer and full prompt, with fixed deterministic seeds. Prompt length is at most 192 tokens. On the pod, every exact value token sequence must be absent from TRAIN and DEV, and random hints must not equal exact values.
4. **Copy scoring.** Report teacher-forced per-token top-1/top-8 and free-running exact-string recovery, all through the GPU scorer only. Free-running stops after eight lowercase letters; trailing whitespace is allowed, but fewer/more letters, punctuation or a non-ASCII character fails. Teacher-forced top-8 is descriptive oracle assistance, not generation capability. A planted wrong value must produce a different normalized answer from the exact value on every applicable row.
5. **Primary comparisons.** Compare exact-hint free-running recovery against no-hint and random-hint controls. Let `b` be exact-only wins, `c` control-only wins, `d = b - c`. Both comparisons require `d > 24 && d^2 > 4(b+c)`. On 40 pairs the second clause is redundant; this is a conservative absolute non-vacuity margin of at least 25 net wins, not a 2-SE estimate. A control winning at least 8 rows makes the gate impossible by design and blocks panel spend. Teacher-forced top-8 cannot replace the primary comparison.

## Stop rules

- Any artifact/tokenizer/DEV/manifest hash or shape mismatch, nonzero row status, GPU-path disagreement, nondeterminism, missing-value corpus leak or planted-control failure is an instrument stop (D31), not a treatment result.
- A position-strip aggregate difference is recorded before interpreting copy results; it is not silently repaired or hidden.
- A failed non-vacuity floor ends this configuration. No panel-v3, training or changed threshold after FIT.
- No BPB, no fresh-V2 opening, no model training, no new Python, no `Closes #2833`.

## Next branches

- **Both comparisons pass and GPU parity/determinism are exact:** a later unit may draft a hidden-fact panel with matched hint controls and a new FIT/D67 review. That unit still will not claim that the store is the model's read without a separate consumer path.
- **Either comparison fails:** preserve the scoped copy/parity receipt. Do not build panel-v3. Reassess the artifact/instrument before any address-oracle work.

## Cost and storage

Host manifest generation and finite tests: $0. GPU: one pod, estimated <15 min at about $0.56/h (<$0.25 cap), inside 4 pods/$8/h. One run, no retry after FIT. Reports and raw row records <16 MiB, with the compact copy records separate from the full parity records. Preserve all receipts with SHA-256 in the HF store before pod teardown. No energy claim is made. Full preparation, build, review, run and retry costs are recorded in the REPORT.

## TEST FITNESS line

TEST FITNESS: PENDING - M1 #2029 m1-address-as-read Stage 0
sheet: docs/harness/prep/m1-address-as-read-stage0-2026-10-26.md
reviewer: pending fresh non-author review
