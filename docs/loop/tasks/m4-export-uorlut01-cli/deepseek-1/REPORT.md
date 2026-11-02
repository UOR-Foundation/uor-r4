# REPORT: m4-export-uorlut01-cli, deepseek-1

Lab: deepseek. Item:
[m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md).
Order: [ORDER.md](ORDER.md). References #2859.

## What did not work, first

- Claude-259 and the public protocol-2 baseline found that training expects a
  `System: Memory: {value}.` turn that `uor-chat` could not encode. The
  no-memory product path therefore could not exercise the trained recall
  contract.
- The first no-memory equivalence test was tautological because `reply()`
  delegated to the new entrypoint. It was replaced with a control built from
  `encode_user_prefix` plus `reply_history`.
- A synthetic test's 40-entry tokenizer omitted bytes needed to decode the
  literal `Memory:` layout. The test tokenizer now includes that text, and the
  assertion checks `User: a\nSystem: Memory: b.\nAssistant:` directly.
- The x86 native disassembly audit cannot certify the D11 ARM64 rule: its
  D80/D81 exemption recognizes ARM general-register syntax and flags three
  existing x86 scalar multiply sites. Its self-test passes, but no full audit
  pass is claimed.
- **The explicit memory line changes output but does not produce recall.** On
  both seeds, every memory-on reply id list differed from its matched
  no-memory control. Neither factual reply contained `Pickle` or tomatoes.
  Within each seed, the positive and `Memory: none.` arms collapsed to the same
  malformed reply, so the line perturbs the model without selecting an answer.

## What changed

The integer stack-chat API now accepts an optional explicit memory value and
encodes it as a system turn through the existing literal-role protocol. The
one-shot CLI exposes `--memory <VALUE>`; the REPL exposes `/memory <VALUE>`,
`/memory`, and `/memory clear`. A queued value is consumed only after a
successful turn, and reset clears it. Empty, multiline and literal-special
values are refused before model state changes.

The change touches three Rust files and the integer README. It adds no model,
tokenizer, store, compiler, artifact-format or numeric kernel. The task record,
FIT registration and tracked two-seed evidence add the fixed product probe.

## Measurements

The new executable is
`a7ad4b83547943bd8902bd13b64596cd8699d6bf2c367347ee9c59d13b02f23c`.
Fourteen fresh processes exited 0 with protocol 2. Stops were 11 `eos` and 3
`max_new_tokens`. All 14 reply texts decoded from their ids. The two seed-1
memory-on repeats matched text, ids, stop, history and system line exactly.

| Seed | Factual memory-on changed from off | Requested value present |
|---|---|---|
| 1 | 2 / 2 | 0 / 2 |
| 2 | 2 / 2 | 0 / 2 |

Peak process RSS was 11,612 KiB. The 14 `/usr/bin/time` records sum to 1.52 s.
The evidence root contains 44 files in 32,964 bytes; its sorted SHA-256
manifest digest is
`35e1f521ccc57cf250a288429e3c7a2e1c1ad1dc73e0e92382698cf988d868e7`.
Exact transcripts and ids are in
[the lab record](../../../labs/m4-export-uorlut01-cli/deepseek-1/README.md).

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib
  stack_chat`: 12 passed, including the memory layout, no-memory equivalence
  and invalid-value controls.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin
  uor-chat`: 4 passed, including one-turn pending-memory behavior.
- Targeted `rustfmt --check` for the changed production files: passed.
- `python3 scripts/audit_zero_matmul_serving.py --self-test`: passed.
- Full x86 stack disassembly audit: not admissible; three existing scalar
  multiply sites are misclassified by the ARM-oriented exemption.
- Fresh reviewer: `TEST FITNESS: FIT`, `PRE-RUN: PASS` at
  `3c8b0506032d96c55fa94fafc4f50d7dc9e1c6c0`.
- `uor-harness prep-check`: complete and FIT.
- `git diff --check`: passed.
- The registered 14-process command block: all statuses 0, no nonempty stderr,
  exact repeat, and all output files retained.

## Skills used

None of the available Codex skills applied. The repository's live charter,
current item, task order and relevant training/serving sources were read
directly.

## Still open

- Automatic statement extraction and store lookup remain unimplemented; this
  unit accepts only an explicit value.
- ARM64/WebAssembly audit qualification and M4's held-out quality bar remain
  outside this partial unit.
- The GPU-only served-training and parity work remains tracked separately.

## Plain summary

The chat program can now place the exact memory line used by training between a
question and the assistant response. The model sees that line and changes its
tokens, but it still does not answer with the supplied value. This establishes
that the missing serving format is now connected, while leaving automatic fact
extraction and the trained model's answer quality as separate failures.
