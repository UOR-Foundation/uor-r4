# REPORT: m3-qualify-memory, deepseek-1

Lab: deepseek. Item:
[m3-qualify-memory](../../../../roadmap/items/m3-qualify-memory.md).
Order: [ORDER.md](ORDER.md). References #2031.

## What did not work, first

- The preceding explicit-memory probe showed that both public artifacts consume
  `System: Memory:` but do not answer with the supplied `Pickle` or tomatoes.
  This unit therefore tests extraction and protocol integration only; it cannot
  turn that model-quality failure into a capability.
- The initial `call` extraction path treated "call me Captain Sam" as an
  ordinary value lookup and retained `me`. It now recognizes `call` as a name
  request and skips the indirect object.
- Automatic memory is intentionally not default-on. Enabling it by default
  would change the merged chat baseline before the new behavior has a product
  result.

## What changed

Added `stack/dialogue_memory.rs`, a deterministic serving-side identity-channel
sieve. It reads prior user turns only, splits them into clauses, ranks clauses
by shared normalized content atoms and recency, then extracts bounded name,
temporal, locative, age or default predicate values.

The stack API exports `recall` and `recall_value`; `uor-chat` gains the opt-in
`--memory-sieve` flag and `/memory auto` / `/memory manual`. Explicit
`/memory <VALUE>` overrides the sieve for one turn. The fact log is cleared by
`/reset`, and generated assistant messages are never logged as facts.

The code change is confined to `crates/uor-r4-integer` plus task documentation.
It adds no training, tokenizer, artifact-format or numeric-kernel change.

## Measurements

No trained-model measurement has been taken. Pure extraction controls cover
the two fixed public probes, a correction, named/called forms, where, when,
age, color and rename questions, no-match questions and pronoun-only queries.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib
  dialogue_memory`: 5 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin
  uor-chat`: 5 passed.
- Targeted `rustfmt --check` on the new sieve and changed CLI: passed.
- `uor-harness prep-check`: sheet complete, pending FIT.
- `git diff --check`: passed.

## Skills used

None of the available Codex skills applied. The current charter, M3 item,
training sieve, serving bridge and product code were read directly.

## Still open

- Obtain an independent FIT/PRE-RUN review, then run the six fixed automatic
  memory conversations and record their exact evidence.
- Automatic extraction coverage beyond the bounded lexical forms remains
  unmeasured.
- Model answer quality and M3 qualification remain open.
- ARM64/WebAssembly audit qualification and GPU export/parity work remain
  outside this host-only support unit.

## Plain summary

The chat program can now search what the user said earlier, select the most
relevant clause, extract a short value, and feed that value into the memory
format the model expects. This only works for the bounded patterns covered by
the deterministic tests; it is not a general language reader, and it cannot
compensate for the model's observed failure to answer from memory.
