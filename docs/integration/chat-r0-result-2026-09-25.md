# R0 result — retained artifacts on frozen chat-panel v0 (CAR-LM, evaluation-only)

September 25, 2026. Branch `codex/canonical-address-routing-20260925`, base commit
`b7162e607272af1b9c950eb531f77d26b2bf06b3`. **Evaluation-only.** No model was trained or
changed. This is rung **R0** of the [CAR-LM direction update](car-lm-direction-update-2026-09-25.md#4-reordered-ladder-to-chat)
(§5 T1) and the [chat instrument calibration](chat-instrument-calibration-2026-09-25.md). The
sealed `fresh` array was not opened.

## Verdict

**No retained arm approaches chat-v0, and the gap is objective/data at the measured scope.** Both
retained integer artifacts — the quaternion model and the matched ordinary (Householder-pair)
control — fail **every** v0 threshold by a wide margin and are the same kind of artifact: raw
TinyStories continuation that ignores the prompt and emits repetitive story text. The panel's own
scripted responder passes all v0 thresholds, so the failure is not an unreachable metric. The
literal R0 criterion "does any arm beat the count reference on the same tokens" is **NOT_RUN**
because no order-2/order-5 count **generation** arm is producible from existing evaluator-v2 data
(§5); that comparison belongs to R1's response-NLL screen.

## Deliverables

| Artifact | Path |
|---|---|
| This record | `docs/integration/chat-r0-result-2026-09-25.md` |
| Sealed evidence (commands, hashes, full arm reports) | `docs/evidence/chat-r0-2026-09-25.json` |
| Frozen panel (unchanged) | `docs/integration/chat-panel-v0-2026-09-25.json` |
| Request file | `target/chat-eval/r0-requests.json` |
| Scored reports | `target/chat-eval/r0-{quaternion,ordinary}-report.json`, `r0-calibration.json` |

Panel identity: canonical-content SHA-256
`4e12e1bd607edd0db5d93f6550c1968562d0490f481ec2ca5d5c2facae79e979`; raw committed file SHA-256
`10dfc0e7c529feecfa61e393b22a404ffee561339281abf5ea03f53964c6cfd2`. 38 development rows
(core 20 / memory 10 / refusal 8); 24 fresh rows sealed. Evidence JSON SHA-256
`b024fdfa76b57c0b12a4eaf83c72791cc9c26db6101d103dd67618580b648d4a`.

## Arms and artifacts

- **r0-quaternion** — retained `bundle-quaternion-1` integer session, transport `quaternion`
  (`/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/`), bundle SHA-256
  `b52c9cdba9800a601435f3c798f88d798bb742ff1a0adce97572c46f36d79412`.
- **r0-ordinary** — retained `bundle-householder_pair-1`, matched ordinary control, transport
  `householder_pair`; bundle SHA-256
  `521f2a4c3dc68c835db61d01e512412772b24f61722d08edc7cc2bc1cdd9d960`.
- **c1_retrieval** — instrument's template/retrieval control (nearest development prompt by token
  Jaccard, emit that neighbour's stored first expected answer); no model.
- **scripted** — panel-embedded responder that emits each row's expected answer; the passing upper
  control.
- Binaries (reused, no build): `uor-r4-integer` `3811173d…b182`, `chat-panel-score` `ad10d483…1e8`.

## Per-category results against the v0 thresholds

Development rows (38). Greedy, read enabled, `max_new_tokens=64`. Wilson 95% CIs are in the
evidence JSON.

| Arm | core on_topic /20 | memory /10 | complies /7 | refusal /8 | cycle_rate | trunc_rate | passes v0 |
|---|---:|---:|---:|---:|---:|---:|---|
| scripted (passing upper control) | **20/20** | **10/10** | **7/7** | **8/8** | 0.000 | 0.000 | **yes** |
| c1_retrieval (no-training control) | 0/20 | 0/10 | 0/7 | 6/8 | 0.000 | 0.000 | no |
| **r0-quaternion (integer)** | **1/20** | 5/10 | 0/7 | 0/8 | **0.763** | **0.974** | no |
| **r0-ordinary (integer, matched)** | **0/20** | 5/10 | 0/7 | 3/8 | **0.474** | **1.000** | no |
| v0 threshold | ≥12/20 | ≥8/10 | ≥4/7 | ≥4/8 | ≤0.20 | ≤0.35 | — |

Finer category breakdown (same runs):

| Arm | greeting on_topic /5 | factual on_topic /8 | instruction on_topic /7 | memory on_topic /10 | memory fact /10 | refusal ok /8 |
|---|---:|---:|---:|---:|---:|---:|
| r0-quaternion | 0 | 1 | 0 | 0 | 5 | 0 |
| r0-ordinary | 0 | 0 | 0 | 0 | 5 | 3 |
| c1_retrieval | 0 | 0 | 0 | 0 | 0 | 6 |

Row-by-row, the two integer arms differ only where noted: quaternion gains the single `factual`
on-topic row (`dev-core-fact-08`) and loses the three refusal rows (`dev-ref-05/07/08`) that
ordinary happens to satisfy; memory fact swaps two rows. Neither dominates and both fail v0.

**The memory 5/10 is not stateful recall.** `fact_retained` is lexical presence, and the required
fact (a name/word) already appears in the joined prompt that the raw-continuation model is
continuing; it echoes the word rather than retrieving turn 1. The passing scripted control (10/10)
shows the metric *can* be satisfied by a genuine answer.

## Exact commands and results

Run from the worktree; the worktree has no built `target/`, so the reused owner-checkout binaries
are invoked by absolute path (identity in the evidence JSON).

```sh
# panel digest
/Users/casey.allard/uor-r4/target/release/chat-panel-score panel-hash --panel docs/integration/chat-panel-v0-2026-09-25.json
# -> 4e12e1bd607edd0db5d93f6550c1968562d0490f481ec2ca5d5c2facae79e979

# request file: 38 development rows, user_turns joined by one space, greedy, enabled, max_new=64
python3 target/chat-eval/r0-build-requests.py
# -> requests=38; content-identical to the calibration degenerate-requests.json; sha256 f9f343a0…

# generation (fresh sealed attempt dirs; each writes generations.jsonl + summary.json)
/Users/casey.allard/uor-r4/target/release/uor-r4-integer generate \
  /Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1 \
  target/chat-eval/r0-requests.json target/chat-eval/r0-quaternion-1
/Users/casey.allard/uor-r4/target/release/uor-r4-integer generate \
  /Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1 \
  target/chat-eval/r0-requests.json target/chat-eval/r0-ordinary-1

# integer report -> panel-format JSONL (index order -> development order, response_text, stop.reason)
python3 target/chat-eval/r0-convert.py target/chat-eval/r0-quaternion-1 target/chat-eval/r0-quaternion-panel.jsonl
python3 target/chat-eval/r0-convert.py target/chat-eval/r0-ordinary-1  target/chat-eval/r0-ordinary-panel.jsonl

# scoring
/Users/casey.allard/uor-r4/target/release/chat-panel-score score --panel docs/integration/chat-panel-v0-2026-09-25.json \
  --generations target/chat-eval/r0-quaternion-panel.jsonl --label r0-quaternion \
  --out target/chat-eval/r0-quaternion-report.json
/Users/casey.allard/uor-r4/target/release/chat-panel-score score --panel docs/integration/chat-panel-v0-2026-09-25.json \
  --generations target/chat-eval/r0-ordinary-panel.jsonl --label r0-ordinary \
  --out target/chat-eval/r0-ordinary-report.json

# C1 retrieval control + instrument re-validation
/Users/casey.allard/uor-r4/target/release/chat-panel-score calibrate --panel docs/integration/chat-panel-v0-2026-09-25.json \
  --degenerate target/chat-eval/r0-quaternion-1/generations.jsonl --degenerate-format integer \
  --tokenizer /Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1/tokenizer.json \
  --out target/chat-eval/r0-calibration.json
# -> instrument_valid = true (all six calibration checks PASS; c1 core 0/20, refusal 6/8)
```

Scored stdout is captured verbatim in `target/chat-eval/r0-cmdlogs/` and embedded in the evidence
JSON. Re-running the two `score` commands reproduced identical report SHA-256s
(`c1682a37…`, `c22cddd4…`), so the scoring step is deterministic. The R0 quaternion generation is
text- and stop-identical to the earlier calibration degenerate run (same bundle, same request
content, greedy) — a consistency check, not a second independent draw. All 38 requests generated
within budget: `max_prompt_tokens = 25` (limit 191), `max_session_tokens_including_pending = 89`
(limit 256), **zero truncated prompts**, zero budget overruns. Stop reasons: quaternion
`maximum_new_tokens` 36 / `eos` 1 / `short_cycle` 1; ordinary `maximum_new_tokens` 38.

## Count control — NOT_RUN

Requested: an order-2/order-5 count generation arm on the same 38 rows. A fitted evaluator-v2
order-5 Kneser-Ney model **does** exist
(`/Users/casey.allard/uor-r4/.uor-models/investigations/reference-baselines-20260924/count-fit/model.ng5`,
369,454,767 bytes, SHA-256 `2e7002c6…b5211627`, bound to evaluator SHA `d2432fbb…`), but it is a
**scoring** artifact. `uor-r4-training` exposes only `ngram-fit`/`ngram-evaluate`;
`KneserNey5Gram` exposes `fit`/`probability` (NLL) and no next-token decoder; there is no
prompt-conditioned sampler, no panel stop/session policy, no generations emitter, and the integer
runtime cannot consume `model.ng5`. No already-built binary produces panel-format generations from
counts.

What it would require: a new Rust decoder that loads (or refits) `model.ng5`, tokenizes each panel
prompt through the shared BPE, greedily selects the KN5 next token under the same
`max_new=64`/EOS/short-cycle rules, emits the generations JSONL, and is then scored — a new
instrument, not a cheap reuse, and outside this task's "already-built binaries" constraint.
The R0 "beats the count reference" comparison is therefore untested here; R1's predeclared
held-out response-NLL screen is where the count reference is properly exercised.

## R0 verdict and negative scope

**Question:** does any retained arm approach chat-v0, or is the gap objective/data?

**Answer:** no retained arm approaches chat-v0 (quaternion core 1/20, ordinary 0/20; both fail all
six thresholds). The scripted control passing 20/20, 10/10, 7/7, 8/8 shows the panel is passable.
Both retained artifacts are raw-continuation TinyStories models with no dialogue objective and no
support for the panel's presentation contract, so the observed gap is consistent with an
**objective/data** gap, not a mechanism limit. The count-generation comparison is NOT_RUN.

**Negative scope (exact).** This rejects the retained quaternion and matched ordinary integer
artifacts as chat-v0 candidates at this panel, prompt convention, greedy/`max_new=64` budget and
proxy scorer. It does **not**: exercise an order-2/order-5 count generation arm; open or tune on
the 24 sealed fresh rows; measure general conversation, instruction following or reasoning;
evaluate the geometric mechanism (the mechanism was not the variable under test — the ordinary
control fails the same way); or qualify any integer/energy claim. The quaternion arm reproduces the
calibration degenerate arm exactly, so it adds a reproducibility check rather than a new draw.

## Limitations

- Proxy scorer: `on_topic` can be satisfied by echoing topic words; `refusal_ok` checks only the
  declared forbidden list.
- `fact_retained` is lexical presence, not stateful recall (see above); memory 5/10 is an
  over-estimate of memory behavior.
- Prompts join user turns with a single space because the retained raw BPE has no
  `<|user|>`/`<|assistant|>` tokens; the panel's literal presentation is not representable by this
  artifact (recorded deviation in the calibration).
- One greedy generation per row, so `cycle_rate` is over one generation per row and `trunc_rate`
  uses the integer stop reason; no T=0.8 samples were produced.
- Development rows only; no final held-out evaluation. A passing v0 would only mean "answered this
  panel", never general chat.

## Cost

Evaluation-only. No training, no build (reused binaries). Two generation runs: quaternion 21.18 s
wall / 20.32 s nested model step / 68.5 ms load; ordinary 18.12 s / 17.38 s / 58.8 ms. Scoring
sub-second per arm. New storage ~1.3 MiB (sealed generations) plus ~0.1 MiB (scored reports). Runs
executed while a concurrent D8 cycle loaded the host, so wall times are not idle-machine timings;
peak RSS was not separately instrumented (1.68 M-parameter artifacts, 1.1 MiB model + 1.0 MiB
tables). No external compute.

## Next decision

**Proceed to R1** (already authorized by the direction update §4/§7): train the first
response-masked dense dialogue model (5–15 M) with a predeclared held-out response-NLL margin over
the count reference, then quantize into the integer session and re-run this panel (R3/R4). Do not
change mechanism on this evidence. The R1 screen supplies the count comparison R0 could not run.
The sealed `fresh` array stays closed until design selection.
