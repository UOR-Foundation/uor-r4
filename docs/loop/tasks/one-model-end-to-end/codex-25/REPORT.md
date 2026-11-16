# REPORT: one-model-end-to-end, codex-25 (the slide on the product path)

Lab: codex. Order: [ORDER.md](ORDER.md). Parent: [codex-24](../codex-24/REPORT.md).

## What did not work, first

- **The store is a dataset repository.** `hf download caseyallard/uor-r4-store <path>` fails as a
  model ("File not found"); the same path with `--repo-type dataset` fetches, and
  `w2-inputs/2026-10-25/driver-data/tokenizer.json` is byte-exact at
  `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, the sha the contracts pin.
  Without it the product path cannot be exercised at all (`--stack` requires `--tokenizer`).
- **The help-text edit merged an `eprintln!` argument** and broke the bin's compile; the flag's line
  and the `--prompt-ids` line are separate `eprintln!`s now.
- **This is not the bound `/17` test.** The artifact served here is the probe's `/14`; the bound
  form is proved at the library level (codex-23) and still waits for a run's contract.
- The text path is exercised for plumbing and determinism only: the probe is not a language model,
  so its replies are meaningless. No quality claim.

## What changed

- `crates/uor-r4-integer/src/stack/chat.rs`: `StackChat::set_context_slide`, forwarding to the
  session.
- `crates/uor-r4-integer/src/bin/uor-chat.rs`: `--context-slide` (raw-id path and chat path), the
  `--stack` requirement, and the help line.
- `crates/uor-r4-integer/tests/context_slide.rs`: `under_the_bound_the_slide_changes_nothing` — the
  tokens and the read's picks are identical with and without the flag until the window fills.

## Measurements

Product path, `uor-chat --stack ext-s1.uorlut01 --tokenizer d36d3e87…`, 200 raw ids, 4 generated
tokens:

| Run | Result |
|---|---|
| without `--context-slide` | `error: the session is full (192 positions)`, exit 1 |
| with `--context-slide` | serves: position **203**, ids `[208, 209, 200, 113]`, session sha `8f2fcf50…`, **1,473 tok/s** |
| the same run again | identical ids, identical session sha, 1,480 tok/s |

The read-bearing model serves past its context and the run is deterministic. The rate is in line
with claude-384's served pair-rows reads (1,505–1,648 tok/s).

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm,fbs-synthetic --lib` — 700 passed,
  1 ignored.
- `cargo test … --test context_slide` — **4 passed**.
- `uor-chat` at the head against the probe artifact, the two runs above.

## Still open

1. The bound `/17` under a real contract (needs the run's pins).
2. The report-to-strata mapping for the verdict (needs one `m2-evidence` report).
3. The 1B run itself, which claude is on.
