# REPORT: generated complete replies, deepseek-12

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`b45f37b44cba3c8360186233b5b568a13d930dfe`.

## What did not work

- **The first complete-reply attempt required EOS in the panel manifest.**
  These older manifests do not carry EOS; the fail-closed parse exposed that
  before generating anything. The instrument now takes EOS from the pinned
  tokenizer's added `<|eos|>` token and records its id in the report.
- **Most public M1 replies are not correct by exact reference.** None of the 58
  generated replies matched its reference reply.
- **The public M1 model is not ready for useful chat.** Only 12 distinct texts
  appeared across 58 prompts, most commonly "Got it, I'll remember that."
  Two replies repeated a 4-gram, and one decoded text contained a replacement
  character.
- **Four prompts had no remaining context for a new token.** They are recorded
  as context-limit stops rather than counted as completion or empty replies.
- **Three prompts reached the 16-token cap.** They are recorded separately from
  EOS completion.
- **This is not fresh-phrasing or semantic-correctness evidence.** The panel
  and reference index remain the existing public smoke and generated control.

## What changed

- `m2-evidence` accepts `generate-new-tokens=N`.
- For each paired prompt/reply, it starts from the actual available document
  prefix and performs deterministic greedy decoding.
- EOS, token-limit and context-limit stops are distinct.
- Each generated row records prompt/reply indices, complete ids, decoded text,
  exact-reference agreement and repeated-4-gram status.
- Aggregate output records completion, exact-reference count, distinct texts,
  replacement-character texts and generated 4-/8-gram novelty against the
  supplied reference corpus.
- The tokenizer identity now records the validated added `<|eos|>` id.
- Teacher-forced scoring, writer parity and full CPU trunk parity remain in the
  same report and remain exact.

## Measurements

Public seed-1 19-table artifact:
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

| Reading | Result |
|---|---:|
| Paired panel prompts and replies | 58 |
| Maximum new tokens | 16 |
| EOS-complete replies | 51 |
| Token-limit stops | 3 |
| Context-limit stops | 4 |
| Exact-reference replies | 0 |
| Distinct generated texts | 12 |
| Replies containing a repeated 4-gram | 2 |
| Replies containing a decoder replacement character | 1 |
| Generated new / total 4-grams | 51 / 360 |
| Generated new / total 8-grams | 145 / 145 |
| Teacher-forced targets | 616 |
| Teacher-forced admitted / top-1 / top-8 | 365 / 293 / 306 |
| Full CPU trunk parity | 19,712 / 19,712 coordinates exact |

The full generated reply list is in the pinned report. Its SHA-256 is
`48fe585f40106e943c48ab59c199c77d97f23680427baa09401a217dba90e20b`.
The committed summary is
`docs/labs/one-model-end-to-end/evidence-deepseek-12-generated-replies.json`
(SHA-256
`aa6ac6e4ac6de7445ee84d829804450db63c68a965744c8bc22a0c1f0f2ca0f0`).

## Checks run at the head

- `cargo test --offline --features cpu-parity --lib --bin m2-evidence -j1`:
  15 library tests and 3 CLI tests passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- The public 58-prompt, 16-token generation process completed successfully.
- Targeted `rustfmt --check` passed.
- `git diff --check` passed.

## Skills used

No available Codex skill applied. The existing evidence lane, tokenizer,
integer model, public panel and repository delivery rules were read directly.

## Still open

- Semantic reply correctness, rather than exact-reference agreement.
- A genuinely fresh held-out-phrasing and reply-wording panel.
- Flock-rank generation beside greedy.
- Count-table, memory-off and no-read controls.
- A readable trained full-path FBS artifact with learned Hamming, memory and
  point-key weights.

## Plain summary

The CPU evidence tool can now generate bounded complete replies and grade their
stop reasons, repetition, exact-reference agreement and novelty. On the public
M1 model, 51 of 58 responses reached EOS, but none matched the reference reply,
and 58 prompts collapsed to 12 distinct texts. The model can finish short
geometric outputs; it does not yet hold a useful conversation.
