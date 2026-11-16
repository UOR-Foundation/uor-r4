# ORDER — m2-student-corpus-ingest (claude-403)

Lab: claude. Unit: claude-403. Task: `m2-student-corpus-ingest`.

## Goal

A Rust instrument, `student-ingest`, that turns donor-corpus JSONL shards into the native
trainer's input files with per-segment coverage counters, so the D87 donor corpus can be read
by the native trainer and its tokenizer coverage is known before any training run.

## Scope

- Slice A (this PR): the standalone package `docs/labs/m2-student-corpus-ingest/instrument/`
  (its own `[workspace]`), binary `student-ingest --tokenizer <tokenizer.json> --out <dir>
  <shard>...`, and its tests on hand-written fixtures. No training, no GPU, no real-data run.
- Slice B (later): run the release binary on pod 3's shards, read-only, and write `REPORT.md`.

Inputs: `.jsonl` or `.jsonl.zst` shards. The format is detected per record by its keys:
(a) the D41 pilot (`id`, `kind`, `messages[{role,content}]`, `answers[1..4]`; the ingest uses
`answers[0]`); (b) D87 production (`id`, `kind` in `single`|`two-turn`, `source`,
`messages[{role:"user",content}]`, `turns[{reasoning,answer}]`, one turn per user message).
A record with an unknown key is refused, naming its id.

Outputs in `<dir>`: `tokens.u16` (64-byte UORT header, version 1, vocabulary 4,096, first
token BOS = 0, every id < 4,096), `response_mask.u8` and `segment.u8` (one byte per token),
and `summary.json`: the tokenizer's sha256, each input file's sha256 pinned at read time,
records per format and kind, and per segment the text bytes, tokens, dropped bytes and
fallback tokens, plus the top 20 runs of dropped-or-fallback bytes as hex strings with
counts. Ratios are integer numerator/denominator pairs; no floating point.

Coverage counting does not use `encode_lossy` (it reports no losses): a dropped byte is a
text byte outside the set of bytes the vocabulary can emit, cross-checked against
`text_len − Σ token_byte_len(emitted ids)`; a fallback token is an emitted id that decodes
to one byte ≥ 0x80.

## Open decision: turn framing (the lab's call, reversible)

No new tokens and no reasoning marker. Per record: BOS; then per turn the user marker and
the user content, the assistant marker, the reasoning text, `"\n\n"`, the answer text; EOS
at the end of the record. The markers are the ones `turn_census::Marks` reads. Segment tags
in `segment.u8`: 0 frame (BOS, EOS, markers, separator), 1 prompt, 2 reasoning, 3 answer.
`response_mask` is 1 on reasoning and answer (D87 makes both hard targets), 0 elsewhere.
A reviewer or slice B may change this framing; the instrument keeps it in one place.

## Gates

- G1: release build of the instrument finishes.
- G2: `cargo test` on the instrument passes with more than zero tests.
- G3: this ORDER commit is the first commit on the branch.
- G4: `git diff --check origin/main...HEAD` is empty.

No GPU; no pod; no corpus or binary is committed.
