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
- The trained-model probe is not yet run; it remains gated on FIT/PRE-RUN and
  the fixed output root.

## What changed

The integer stack-chat API now accepts an optional explicit memory value and
encodes it as a system turn through the existing literal-role protocol. The
one-shot CLI exposes `--memory <VALUE>`; the REPL exposes `/memory <VALUE>`,
`/memory`, and `/memory clear`. A queued value is consumed only after a
successful turn, and reset clears it. Empty, multiline and literal-special
values are refused before model state changes.

The change touches three Rust files and the integer README. It adds no model,
tokenizer, store, compiler, artifact-format or numeric kernel. The task record
and pre-registration add the fixed two-seed product probe.

## Measurements

No trained-model measurement has been taken. The pre-registered probe is
defined in
`docs/harness/prep/m4-export-uorlut01-cli-memory-bridge-2026-11-02.md` and uses
the new debug executable SHA-256
`a7ad4b83547943bd8902bd13b64596cd8699d6bf2c367347ee9c59d13b02f23c`.

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
- `uor-harness prep-check`: complete, pending FIT.
- `git diff --check`: passed.

## Skills used

None of the available Codex skills applied. The repository's live charter,
current item, task order and relevant training/serving sources were read
directly.

## Still open

- Obtain an independent FIT/PRE-RUN review, execute the 14 fixed product
  prompts, and record exact transcript/evidence results.
- Automatic statement extraction and store lookup remain unimplemented; this
  unit accepts only an explicit value.
- ARM64/WebAssembly audit qualification and M4's held-out quality bar remain
  outside this partial unit.
- The GPU-only served-training and parity work remains tracked separately.

## Plain summary

The chat program can now place the exact memory line used by training between a
question and the assistant response. This is the missing plumbing needed before
an automatic fact finder can feed answers into the model. It does not yet find
facts by itself, and it does not show that the model can answer correctly.
