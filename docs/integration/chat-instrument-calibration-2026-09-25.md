# Chat instrument calibration — frozen panel v0 (CAR-LM, evaluation-only)

September 25, 2026. Branch `codex/canonical-address-routing-20260925`, base commit
`5c931a94379781547fd2ef8657fbc33ecd5214ac`. **Evaluation/data instrument only.** No model was
trained or changed; no capability, geometric-advantage, integer-serving or energy claim is made
here. The retained integer bundle used as the negative control is a read-only local artifact.

## Verdict

**Instrument valid.** The frozen chat panel v0 plus the Rust scorer separate a passing scripted
responder from a known-degenerate retained generation set and from the C1 template/retrieval
control, at the predeclared v0 thresholds and separations. The `fresh` array was not opened.

## Deliverables

| Artifact | Path |
|---|---|
| Frozen panel (JSON) | `docs/integration/chat-panel-v0-2026-09-25.json` |
| Scorer crate | `crates/uor-r4-chat-eval/` (`lib.rs`, `bin/chat-panel-score.rs`, `tests/panel_calibration.rs`) |
| Calibration evidence | `docs/evidence/chat-instrument-calibration-2026-09-25.json` |
| This record | `docs/integration/chat-instrument-calibration-2026-09-25.md` |

Panel identity: canonical-content SHA-256
`4e12e1bd607edd0db5d93f6550c1968562d0490f481ec2ca5d5c2facae79e979`; raw committed file SHA-256
`10dfc0e7c529feecfa61e393b22a404ffee561339281abf5ea03f53964c6cfd2`. The canonical digest is
taken over the panel JSON with the `panel_sha256` key removed, keys sorted and whitespace
stripped, so writing the digest into the file does not change it. The scorer recomputes and
compares it (`panel-hash`).

## Panel composition

| Group | Development | Sealed fresh |
|---|---:|---:|
| Core single-turn (greeting 5, factual 8, instruction 7) | 20 | 13 (greeting 3, factual 6, instruction 4) |
| Memory probes, 3 turns, scored on turn 3 | 10 | 6 |
| Refusal / clarification | 8 | 5 |
| **Scored rows** | **38** | **24** |

Presentation is the literal transcript `<|user|>…\n<|assistant|>…\n`; greedy (T=0, top-k 1) is
the pass/fail generation, with two fixed-seed T=0.8 samples recorded for `cycle_rate` and
`trunc_rate`, `max_new=64`, stop at EOS. Memory rows assert a seeded entity/attribute on turn 1
with a distractor on turn 2 and query on turn 3; only turn 3 is scored. Every fresh row names a
`mirrors_dev` counterpart.

## Metric definitions

`on_topic` (core rows) = ≥2 distinct topic-lexicon content tokens **and** content overlap ≥0.10
**and** word length ≥5. `content_overlap` = on-lexicon generated content tokens / all generated
content tokens (fixed English stopword list). `fact_retained` = every token of `required_fact`
appears. `complies` (instruction rows) = at most one sentence terminator, ≤30 word tokens,
contains `must_contain`, and ≥2 generated content words absent from the prompt. `refusal_ok`
= a question mark or a declared cue, and no declared forbidden fact. `cycle_rate` = fraction of
(row, generation) pairs with a word 4-gram occurring more than twice. `trunc_rate` = fraction
that stopped at `maximum_new_tokens` rather than EOS. All proportions carry Wilson 95% CIs.

Word-level tokenization is used for content; the shared 4096-vocab byte-level BPE is reused for
surface length and truncation accounting when a `tokenizer.json` is supplied.

## Calibration result

Committed panel, development rows (all 38), scorer release binary
`ad10d483d598ef16da5d824f992dcc015c27b418d500d03e6c78bd89cef731e8`:

| Arm | core on_topic | memory fact | instruction complies | refusal ok | cycle_rate | trunc_rate | passes v0 |
|---|---:|---:|---:|---:|---:|---:|---|
| Scripted (panel-embedded responder) | **20/20** | **10/10** | **7/7** | **8/8** | 0.000 | 0.000 | **yes** |
| Degenerate (retained integer, quaternion, greedy, read enabled) | **1/20** | 5/10 | 0/7 | 0/8 | **0.763** | 0.974 | no |
| C1 template/retrieval (nearest prompt by token Jaccard) | **0/20** | 0/10 | 0/7 | 6/8 | 0.000 | 0.000 | no |

Wilson 95% CIs (k/n): scripted core 20/20 [0.839, 1.000], memory 10/10 [0.722, 1.000],
complies 7/7 [0.646, 1.000], refusal 8/8 [0.676, 1.000]; degenerate core 1/20 [0.009, 0.236],
cycle 29/38 [0.570, 0.889]; C1 core 0/20 [0.000, 0.161], refusal 6/8 [0.409, 0.929].

| Calibration check | Observed | Requirement | Result |
|---|---|---:|---|
| scripted passes all v0 thresholds | 20/20, 10/10, 7/7, 8/8, cycle 0.000, trunc 0.000 | all v0 | PASS |
| degenerate on_topic low | 1/20 | ≤4/20 | PASS |
| degenerate cycle high | 0.763 | ≥0.50 | PASS |
| C1 does not pass v0 | 0/20 core, `passes_v0=false` | false | PASS |
| on_topic separation | scripted − degenerate = 19 | ≥16 | PASS |
| cycle separation | degenerate − scripted = 0.763 | ≥0.30 | PASS |

`instrument_valid = true`. Scripted/degenerate/C1 per-row detail (including every generated
text) is in the evidence JSON.

**Degenerate source.** Retained `bundle-quaternion-1` integer session
(`/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/`,
bundle SHA-256 `b52c9cdba9800a601435f3c798f88d798bb742ff1a0adce97572c46f36d79412`), 38 panel
development prompts, greedy, `max_new_tokens=64`, read enabled, run with the prebuilt
`uor-r4-integer` (`3811173d…b182`). It was selected before scoring as the strongest retained
served artifact, not after seeing the score. Output repeatedly cycles
(e.g. `You are a good girl. You are a good girl. …`) and 37/38 generations hit the token cap.

## Exact commands

Calibrate the instrument (passes; exits 0):

```sh
target/release/chat-panel-score calibrate \
  --panel docs/integration/chat-panel-v0-2026-09-25.json \
  --degenerate target/chat-eval/degenerate-quaternion-1/generations.jsonl \
  --degenerate-format integer \
  --tokenizer /Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json \
  --out target/chat-eval/calibration-release.json
```

**One-line command to score any arm on the panel** (generations JSONL: one object per row,
`{"id": "<panel id>", "generations": [{"sample":"greedy","text":"…","stop":"eos"}]}`; extra
T=0.8 samples may be appended for cycle/truncation):

```sh
target/release/chat-panel-score score --panel docs/integration/chat-panel-v0-2026-09-25.json --generations <arm>.jsonl --tokenizer <tokenizer.json> --label <arm> --out <report>.json
```

Panel digest: `target/release/chat-panel-score panel-hash --panel docs/integration/chat-panel-v0-2026-09-25.json`.
Build and tests: `cargo build -p uor-r4-chat-eval --release --offline`; `cargo test -p uor-r4-chat-eval --offline`
(6 unit + 1 committed-panel calibration test, all pass).

## Deviations from the task packet (recorded, not concealed)

1. **Development row count.** The packet's aggregate "28 development prompts" conflicts with its
   own per-category counts (A 20 + B 10 + C 8 = 38) and with the threshold denominators
   (`on_topic ≥12/20`, `memory ≥8/10`, `complies ≥4/7`, `refusal ≥4/8`). The panel therefore has
   **38 scored development rows**: the 28 single-turn prompts (A 20 + C 8) plus the 10 three-turn
   memory probes. This is the interpretation under which the thresholds are well-defined.
2. **Fresh row count.** 24 sealed rows, as stated, matched to a named development row across all
   three groups (13 core / 6 memory / 5 refusal) rather than a 1:1 mirror of all 38 development
   rows, which the 24-row quantity forbids.
3. **Degenerate prompt format.** The retained raw BPE has no `<|user|>` special token, so the
   degenerate control joins each row's user turns with a single space. This is a negative-control
   convenience, not the panel's presentation contract.
4. **C1 refusal threshold.** C1 scores 6/8 on refusals because neighbouring prompts share
   clarification forms; it still fails v0 overall (core 0/20). `c1_does_not_pass_v0` is scoped to
   the full v0 verdict, not to every sub-threshold.
5. **Home.** The scorer is a new evaluation-only crate `crates/uor-r4-chat-eval` (no model,
   training or numerical dependency; it depends only on the shared `uor-r4-tokenizer`). This is
   lighter than adding an evaluation binary to the serving or training crates.

## Limits

This is a small authored panel and a proxy scorer. `on_topic` can fail a fluent tangential
answer and can be satisfied by a degenerate echo of topic words (the calibration separation is
what controls that). `refusal_ok` cannot detect an invented fact outside its declared forbidden
list. The degenerate control is a raw-continuation artifact scored on chat prompts; its failure
is expected and says nothing about chat-tuned models. A passing v0 result on this panel would
qualify only "answered this panel", never general conversation. The fresh rows remain sealed;
no final held-out evaluation was run.

## Next decision

The instrument is ready for T1/R0: score the retained artifacts and the count control on this
panel, and decide between a training run and a mechanism change. Open the sealed `fresh` array
only after design selection. Delivered for lead review; no commit or push performed here.
