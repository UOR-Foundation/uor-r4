# REPORT: one-model-end-to-end, claude-309

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **(b64L) answers none of `m2-fresh256`: 0 of 256 rows complete, 0 of 128 pairs, on both seeds.** The bar is 104 rows and 52 pairs. Every reply ran to the 64-token cap: 0 EOS stops, 0 BOS stops, 0 context stops. Every stratum is 0 (length2, length4, length8, reassert, update).
- **The miss is the model's, not the scorer's or the serving path's.** Three controls:
  - **Loose match:** I dropped the EOS condition and looked for each row's accepted answer (lower-cased, final period dropped) anywhere in the reply text. 0 of 256 on each seed. So the model does not recall the fact even before it fails to stop.
  - **The float model never stops either:** across the trained float model's 32-token greedy replies on its own oasst2 prompts in the seal (`samples-oasst2-greedy-memory.json`, seed 1), the EOS id 1 occurs 0 times. The served replies on the same prompts (claude-308's seed-1 `i3/`, DEV and oasst2) also never emit it. So not ending a reply is a property of the trained model, not of export or serving.
  - **The scorer can pass a row:** the known-answer test scores an accepted answer followed by EOS as complete. The same reply without EOS scores incomplete. With the EOS condition set aside, that test fails, and it passes again once the condition is restored.
- **The replies are loops, not answers.** On seed 1, 76 of the 256 replies are the same text: " My curopeans (1999…9". Seed 2's most common reply is " My currently live in the 1999…9" (36 rows). The served model copies the prompt's opening words, then repeats a digit.
- **This closes the configuration, not the mechanism (§4.1).** It covers (b64L) at 16,384 steps on D3, under the `one-user-turn/v1` rendering. The other rendering named in the ORDER, with facts in separate turns, is unread. The loose-match control shows recall is absent under this rendering, so a different layout alone is not expected to change the reading.

## What changed

- **New bin `served-reply-panel`** in the item's instrument crate (`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`, feature `cpu-parity`). It works in five steps:
  1. **Inputs:** it checks the panel's `inputs`, `labels` and `plan` and the tokenizer against the manifest's shas, and refuses an artifact that serves the writer head.
  2. **Rendering:** it decodes each packet's `Context` and query ids and skips `Source`. It joins them into one user message and encodes it with `DialogueEncoder::encode_assistant_prefix` (protocol 2), so the prompt ends on the open `Assistant:` marker.
  3. **Generation:** it drives `CpuFbsForward` greedily with the nearest head. The stop rule runs in this order: EOS, then BOS, then the cap, then the context limit. The library's `generate_reply` stops on id 0 (the served BOS), not on EOS id 1, so the bin uses its own rule.
  4. **Scoring:** it drops the reply's leading protocol-2 space and applies `FrozenAnswers::accepts`. A row is complete only when it stopped on EOS, decodes as UTF-8, and is accepted. `pairs_both_complete_distinct` compares reply texts within each `pair_id`. Strata come from `plan.json` and are checked against the manifest's `strata_rows`.
  5. **Output:** a JSON file with totals, per-stratum counts and every row.
- **`Cargo.toml`:** the bin, and `dep:uor-r4-core` added to `cpu-parity`. `Cargo.lock` gains 4 dependency lines under `uor-r4-cpu-train` (it was behind that crate's own dependencies). No new crates.
- **The harness exception, as recorded in the ORDER:** `loop claim` refused on deepseek's `hamilton_sheet.rs` overlap, so the ORDER was written by hand. The builder subagent then got a `loop suborder` scoped to the instrument crate.

## Measurements

Panel `m2-fresh256` (inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`). Bin `424bd917…`. Artifacts as in claude-308.

| Seed | Rows complete | Pairs both complete, distinct | EOS / BOS / cap / context stops | Answer anywhere in reply | Tokens/s | Output sha256 |
|---|---|---|---|---|---|---|
| 1 | 0 of 256 | 0 of 128 | 0 / 0 / 256 / 0 | 0 | 914.5 | `0d6dec9a63e1cb61fb33ce40c859bf8f8790db88f3c868b16ade9679f16c3856` |
| 2 | 0 of 256 | 0 of 128 | 0 / 0 / 256 / 0 | 0 | 905.1 | `066dc827a58c676e755fa74f49b1e10c046df1422138a8c854a9cbd47488f280` |

Each seed generated 16,384 reply tokens in about 18 s. The tokens per second include the artifact parse and each row's prompt read.

## Checks run at the head

- `cargo build --release --offline --locked --features cpu-parity --bin served-reply-panel`: exit 0.
- `cargo test --offline --locked --features cpu-parity --bin served-reply-panel --no-fail-fast`: 4 passed, 0 failed. The tests cover the rendering of a two-context packet with a `Source` between them, the order of the stop rules, acceptance (including the leading-space strip and a non-UTF-8 reply), and the pair metric.
- **Negative gate (the builder's run):** with the EOS condition taken out of `complete`, `known_answer_needs_eos_and_strips_leading_space` failed (exit 101). After the restore and rebuild, all tests pass.
- **Banned literals:** no hit in `git diff f5adca314 -- docs/labs`.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **The model does not end a reply or recall a stated fact.** Both are training problems, not pipeline problems. The pipeline is now measured end to end: train, export, serve, then score whole replies on a held-out M2 panel, laptop-only, about 18 s per seed. The next training decision can be judged by this score. A run is worth its spend only if it moves this score off 0, or at least gets the model emitting EOS on its own corpus.
- **`m2-reply512` is not scored.** Its `inputs.json` and `labels.json` are not located in the store. Only their hashes are recorded (`b9661606…`, `84991e0…`).
- **The per-turn rendering is unread.**
- **claude-308's one-scorer parity** (served against trained on the same scorer) is still open.

## Plain summary

There is now a tool that asks the question that matters: given a short conversation that states some facts, does the served model answer a question about them correctly, and stop? It runs the shipped integer model on the laptop at about 910 tokens per second and scores all 256 held-out questions in about 18 seconds.

The current best model gets 0 of 256 on both seeds. It never names the fact anywhere in its reply, and it never stops talking: it repeats the prompt's first words and then a string of 9s. The trained model, before export, also never stops, so this is a training problem, not a serving problem. With the path from training to serving to scoring now working end to end, the next training run can be judged by whether it moves this number.
