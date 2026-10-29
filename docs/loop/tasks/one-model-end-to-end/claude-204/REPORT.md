# REPORT: one-model-end-to-end, claude-204 (a held-out-phrasing panel, and its census)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3407. The item stays `ready`, at step 2.

**Speed.** Counts only, on the laptop: each panel takes under a second to write, and the census about 5 s. No serving change.

## What did not work, first

- **The writer's first draft failed its own control.** Under dialogue protocol 1, the default copied from `m-world`, the `train,train` control had an exact-prompt share of 0 and 0 keyed openings.
  - Decoding showed why: TRAIN is in protocol 2, where the role marker's trailing space is merged into the first word (" My", " Got"). Protocol 1 leaves a lone space token.
  - The writer now defaults to protocol 2. All three panels were regenerated under it before any result was read, and the control then matched DEV.
  - Classed as an instrument fault, caught by the ORDER's control.
- **`m-world.rs` cannot carry the writer.**
  - Since the cutover, `crates/uor-r4-training/Cargo.toml` sets `autoexamples = false` and declares no `m-world` example.
  - A scratch compile shows its other modes import modules now configured out (`flock`, `stack_dialogue`, `stack_memory_replies`, `stack_tracking`).
  - The probe screen (`milestone_world_v2_probe`) is preserved under `#[cfg(any())]`.
  - So the writer is a standalone instrument, and the `m-world.rs` scope claude-203 added went unused.
- **No probe screen.** TRAIN was drawn under a sealed-probe screen and the panels are not. That makes no difference for test panels, and it is a small, stated difference for the control.
- **P-dev tests reading a new prompt, not writing a new reply.** Its replies are 0.640 verbatim TRAIN replies, because reply templates are shared by construction (claude-203).
- **245 of P-dev's 1,443 openings are keyed** by TRAIN's first-8 table. A turn's first 8 ids include the "User:" marker, so common short openings collide.

## What changed

- **`docs/labs/one-model-end-to-end/instrument/`** (bin `heldout-panel`, with its own workspace and `Cargo.lock`):
  - `heldout-panel out=… tokenizer=… cell=PHRASING,VALUE [conversations] [seed] [protocol=2]` writes M-world v2 conversations of one cell as a UORT split with a manifest. It never replaces its output.
  - `heldout-panel decode split=… tokenizer=…` prints documents as text, by mask run. It is the diagnostic that found the protocol fault.
- **`train-in-space --census 1 --panel NAME=DIR`** (repeatable) counts extra UORT splits without pins:
  - `load_unpinned_uort` refuses a bad header, a count mismatch, a missing first BOS, or an id past the vocabulary;
  - each split's sha256 is recorded in the JSON's `extra`;
  - `census` takes the extra panels.
- **Three panels** of 400 conversations each, in the HF store at `inputs/claude/heldout-panels-2026-10-29/` with `SHA256SUMS`:
  - P-dev (`dev,dev`, seed 2041, `790b7e12…`);
  - P-ret (`train,dev`, 2042, `b4d3bf42…`);
  - P-train (`train,train`, 2043, `7546c6d7…`).
- **Results:** `census-panels-2026-10-29.json`, a README section, and one notebook row.

## Measurements

| Overlap with TRAIN | DEV | P-train (control) | P-ret | P-dev |
|---|---|---|---|---|
| Prompts equal to a TRAIN prompt | 0.608 | 0.608 | 0.541 | 0.000 |
| Masked prompt templates, K = 256 / 1,024 | 0.644 / 0.642 | 0.644 / 0.638 | 0.604 / 0.585 | 0.049 / 0.000 |
| Prompt 8-grams / 13-grams | 0.521 / 0.282 | 0.527 / 0.287 | 0.478 / 0.244 | 0.159 / 0.006 |
| Replies equal to a TRAIN reply | 0.750 | 0.726 | 0.634 | 0.640 |
| Masked reply templates, K = 1,024 | 0.766 | 0.757 | 0.710 | 0.705 |
| Openings keyed | 1,194 / 1,226 | 1,365 / 1,425 | 1,332 / 1,422 | 245 / 1,443 |
| New templates (rule: ≤ 0.5) | no | no | no | **yes** |

- **The control:** P-train holds no new templates, as the ORDER requires, and it matches DEV within about 0.01 on every prompt measure.
- **The verdict:** P-dev is the held-out-phrasing panel.
- **The risks:**
  - near-duplicate paraphrases: prompt 13-grams at 0.006 (8-grams at 0.159);
  - chat-v0 coincidences: 0 exact prompts.

## Checks run at the head

- `cargo test --locked -p uor-r4-float-train --lib -- census unpinned`: 5 passed, including the unpinned loader's refusals.
- `cargo test --locked -p uor-r4-float-train --bin train-in-space`: 1 passed (`--panel` parsing).
- `cargo test` in `instrument/`: 1 passed (the cell parser).
- The census re-run with the panels: M1 is unchanged (1,226 / 1,194 / 9,655 / 8,822 / 8,089; 3,438 rows).
- `cargo test --locked -p uor-r4-float-train --no-fail-fast`: 145 passed, 0 failed.
- `uor-harness loop gates --full`, at `4dbc586df` with this REPORT: green, 11 gates GREEN.

## The review

Fresh context, at `2a7c143b9`, given the ORDER and the diff: **APPROVE WITH NITS**. Every number it checked with `jq` matched the JSON, and it found no claim overstated.
- **The writer matches `corpus_v2` on:** the variant (V2), `Mix::default()`, the version-1 meter closure, the encoder, the encoding checks and the vocabulary check.
- **P-dev's 0.000 is not an artifact.** P-ret (same writer, protocol and dev values) gives 0.541 / 0.585. That rules out "dev values alone" and any encoding artifact across the whole panel. P-dev's own reply-template share, 0.705, rules out a panel-wide tokenization fault.
- **Its limit:** P-train is close to guaranteed by construction. It checks the encoding and the census, not whether the census can detect novelty. That is what P-ret against P-dev does.
- **Fixed at the next commit:**
  1. **The writer now applies `corpus_v2`'s spaced-meter and 256-token context checks.** All 1,200 conversations pass, and the panels written with the checks are byte-identical to the stored ones.
  2. **The README says the missing probe screen can only lower P-train's and P-ret's shares,** never P-dev's.
  3. **The loader's test has a mask-only refusal case.** Removing the `mask.len() != count` clause turns it red ("a mask shorter than the count"); checked here, then restored.
  4. **`--panel` refuses a duplicate name.**
- **Left as they are:** `Census.extra` serializes as anonymous tuples, and the loader does not read the header's vocabulary field; ids are bounded by `SHEET_VOCAB`.
- **The protocol-2 evidence,** beyond the control: decoding TRAIN gives `"User: My friend is named Justtros.\nAssistant:"` as `[55, 2728, 28, 2496, …]`, and the reply as `" Nice, …"` (2237). The protocol-1 draft gave a lone space, token 223, before the first word.

## Skills used

- None was invoked this unit.

## Still open

- **Step 2, the GPU overturn check,** under a prep sheet:
  - its decision rows are DEV's 3,438;
  - P-dev joins as a descriptive stratum for its first look. It becomes a decision stratum only in the step-3 protocol, once no decision has touched it yet.
- **Step 3, the protocol.** P-dev, and a fresh draw of the same cell with a new seed, give the new-template stratum.
- **The probe screen** is not reachable from a live build. Whether any panel shares 8-grams with the sealed probe is not checked.

## Plain summary

We built a test set whose questions use wording the training data never contains. A control built the same way as the training data looks just like our old test set, which shows the new test set's difference is real. Its answers still use the familiar reply wording, so it tests whether the model reads a new question, not whether it can write a new kind of answer.
