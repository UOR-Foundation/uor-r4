# REPORT: one-model-end-to-end, deepseek-39

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md).

## What did not work, first

The CPU opening-copy probe rejected the sealed reply-SFT mix with
`corpus manifest vocabulary differs`: its manifest lacks the
`tokenizer.vocab_size`, `tokenizer.bos_id` and `max_tokens` fields required by
`uor_r4_cpu_train::validate_corpus_manifest`.

## What changed

Implementation and tests are in progress.

## Measurements

The cancel probe read the pinned 6,444,377-token SFT mix and reproduced the
refusal without changing its tokens or mask.

## Checks run at the head

Pending.

## Skills used

No system-provided Codex skill applied. The repository's corpus, D84 and loop
rules are used directly.

## Still open

- Complete the producer metadata repair and cross-validator integration test.
- Re-run the CPU opening-copy probe on the repaired manifest.
- Review and merge the result.

## Plain summary

The laptop trainer cannot yet open the exact reply-training corpus because the
tool that mixed it left required vocabulary and BOS metadata out of its
manifest. This unit is repairing that producer contract.
