# REPORT: lawful M-world oracle-corpus support, deepseek-8

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](ORDER.md). Base:
`5e50f3e193746996c6157aa7b82cb6b6bc27584f`.

## What did not work, first

- **Claude-259's stated corpus route is not buildable on current `main`.**
  `crates/uor-r4-training/examples/m-world.rs` has no `[[example]]` target:
  D29 set `autoexamples = false`, and that retained source imports historical
  compiler types. Running `cargo test -p uor-r4-training --example m-world`
  fails with “no example target named `m-world`”. Adding coverage only to that
  source would not execute in the lawful build.
- **The real pinned-tokenizer smoke did not run.** This host has no Hugging Face
  token and the pinned `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
  tokenizer is in the private `caseyallard/uor-r4-store`. The public upstream
  SmolLM2 tokenizer does not carry the project's atomic `<|bos|>`, `<|eos|>` and
  `<|unk|>` tokens, so the writer correctly refused it. No substitute
  vocabulary was used and no real-tokenizer count is claimed.
- **The bounded control is a finite corpus smoke, not model capability.** It
  uses the repository test's deterministic byte tokenizer and a relation-only
  M-world mix because byte-level character counting cannot hit MQAR's short
  distance bands. It proves encoding, mask length, memory-line counts,
  matched controls, determinism and report sealing. It does not prove real
  tokenizer parity, default-mix MQAR distances or training benefit.
- **Probe decontamination is not duplicated in the new writer.** The historical
  probe module is excluded from the native graph. Callers can pass an
  `exclude=PANEL.json` reply panel, and the output remains composable with
  `mix-chat-corpus`; automatic sealed-probe screening needs its own lawful
  implementation.

## What changed

- Added `m-world-recall-corpus`, an explicitly compiled Rust writer. It imports
  only `milestone_world`, `milestone_world_v2`, the tokenizer's literal-role
  protocol, the UORT corpus writer and report sealing.
- Modes:
  - `recall=off` is the default matched control and emits no memory turns.
  - `recall=oracle` inserts one `System: Memory: ...` turn for every MQAR or
    relation query, at `recall_at=reply` or `recall_at=query`.
- The manifest distinguishes value-bearing lines from `Memory: none.` lines:
  `value_lines` and `none_lines` sum to `lines`.
- The writer records the M-world version, revision, digest, mix, seed, protocol,
  context, per-kind and per-category counts, exclusion-panel identity, token and
  mask SHA-256 values, and the sealed wrapper root.
- Added the binary target and widened the item's file scope. Historical
  `examples/m-world.rs` is byte-unchanged.

## Measurements

At the focused test head, the bounded byte-tokenizer oracle control generated:

| Reading | Value |
|---|---:|
| Conversations | 24 |
| Turns | 66 |
| Tokens | 4,793 |
| Response tokens | 1,853 |
| Recall lines | 10 |
| Value-bearing recall lines | 8 |
| `Memory: none.` lines | 2 |
| Longest episode | 281 |

The off control had the same 24 conversations and 66 turns, no memory turns and
no recall block. Two independent oracle generations produced byte-identical
token stores, masks and train manifests. Decoding the token store yielded
exactly the 10 recorded `Memory:` lines.

No training, model evaluation, GPU run or capability claim was made.

## Checks run at the head

- `cargo test --locked -p uor-r4-training --bin m-world-recall-corpus`:
  **4 passed**, including byte determinism, the matched off/oracle census and
  report-root claim/seal/verify.
- `cargo test --locked -p uor-r4-training --lib`: **80 passed**.
- `cargo check --locked -p uor-r4-training --bin m-world-recall-corpus`: passed.
- `rustfmt --check` on the new binary: passed.
- `uor-harness ratchet d23`: passed, 4,451 unchanged.
- `uor-harness ratchet d25`: passed, 93 unchanged.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The repository charter, D29 target boundary
and admitted M-world/tokenizer APIs were read directly.

## Still open

- Obtain the pinned private tokenizer, generate matched TRAIN-scale off/oracle
  stores with the default mix, and record their SHA-256 values and decoded
  census. Uploading those stores requires private-store write authority.
- Decide whether the next training unit composes the M-world source with the
  pinned chat store through `mix-chat-corpus` or consumes an integrated writer.
- Bring the sealed English probe's exclusion rule into the lawful writer before
  calling a generated TRAIN fully decontaminated.
- Claude-260 deliberately keeps TRAIN fixed and tests the two-token key alone,
  so this corpus is ready for a later isolated key-plus-memory comparison.

## Plain summary

The old M-world source still documents an oracle-memory corpus option, but it is
not compiled after the D29 cleanup. This unit adds a small lawful writer that
can make the missing off/oracle training pair without importing the old
compiler. Its finite controls prove the generated oracle corpus contains real
memory turns and repeats byte for byte. The actual private-tokenizer corpus and
training run remain open because this host lacks the pinned tokenizer and model
store credentials.
