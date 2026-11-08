# REPORT: cpu-training-pilot, deepseek-241

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](./ORDER.md). Base: `973bc0750`.

**Forcing the role-correct value exposes the current model's main opening.** On
the sealed Claude-322 artifacts, ordinary generation produced **0/256**
complete replies under every seed and rendering. Forcing only the correct first
token produced **20-41/256**. Forcing the complete correct value produced
**200-250/256** and **75-122/128** complete-distinct pairs. Forcing the paired
other-relation or a shuffled value produced **0/256** and **0/128** in every
case. The dominant gap is selecting and copying the role-correct source value;
continuation supports most forced correct values, with a smaller residual
termination/continuation gap.

## What did not work, first

- **Free generation still completes nothing.** The one-user controls reproduce
  Claude-322 exactly: EOS stops are 109/256 and 86/256 at seeds 1 and 2,
  first-line asked values 14 and 2, and other-relation values 11 and 6. None is
  a complete accepted answer.
- **The correct first token is insufficient.** Forcing only that token reaches
  41 complete rows at seed 1 and 20-21 at seed 2. Most replies still finish with
  the wrong continuation, so an opening-only patch does not clear the bar.
- **Forced correct content is not a perfect oracle.** The model still misses
  26/256 and 56/256 correct-value rows on the one-user renderings and 6/256 and
  43/256 on per-turn. Some adds text before EOS; others hit the cap.
- **Wrong values terminate without becoming answers.** Other-relation values
  reach 215-254 EOS stops and shuffled values 211-252, but both have 0 complete
  and 0 complete pairs. Stopping is not correctness-aware.

## What changed

- Added `cpu-forced-copy-panel`, a CPU development-research instrument with five
  matched arms: free, correct-first-token, complete correct value, paired
  other-relation value and deterministic shuffled value.
- Reused the exact protocol-v2 panel rendering and nearest served head. The
  local stop contract uses token 1 as EOS, matching `served-reply-panel`; the
  first attempt incorrectly reused a legacy helper that stops on token 0 and is
  retained as an instrument fault.
- Reused one parsed CPU forward across 32 independent row workers. Each final
  full five-arm sweep takes about 2.4-2.7 seconds.
- Added fail-closed artifact/tokenizer/panel hashes, unique case and pair checks,
  exact frozen-answer scoring, exact per-stratum pair metrics, paired controls,
  CPU identity and executable SHA-256.

## Measurements

The selected artifacts are seed 1
`86e51d5ee2c00a4041bcde4d2183884e351099f28f155fb1e6a7516f13064dfe`
and seed 2
`8838cd1044ae7152934d8ec09016e268f2dabb0c702dd45e6ab06d7b5198bdae`.
The executable is
`82052efdf55e5973ac13a9dd7a7538cae4f32479a0a0cb9904b25992a4700ed8`.

| Seed | Rendering | Free complete/pairs | First-target complete/pairs | Correct-value complete/pairs | Other value complete | Shuffled value complete |
|---|---|---:|---:|---:|---:|---:|
| 1 | `one-user-turn/v1` | 0/0 | 41/12 | **230/102** | 0 | 0 |
| 1 | `per-turn/v1` | 0/0 | 41/12 | **250/122** | 0 | 0 |
| 2 | `one-user-turn/v1` | 0/0 | 20/0 | **200/75** | 0 | 0 |
| 2 | `per-turn/v1` | 0/0 | 21/0 | **213/86** | 0 | 0 |

Every paired comparison passed its `max(2, ceil(2*sqrt(discordant)))` threshold:

- first-target vs free: gains 41 and 20/21;
- correct-value vs first-target: gains 189, 209, 180 and 192;
- correct-value vs other-relation: gains 230, 250, 200 and 213;
- correct-value vs shuffled-value: gains 230, 250, 200 and 213.

Correct-value completion by stratum, seed 1/seed 2:

| Rendering | `length2` | `length4` | `length8` | `reassert` | `update` |
|---|---:|---:|---:|---:|---:|
| `one-user-turn/v1` | 56/45 | 58/44 | 58/58 | 30/27 | 28/26 |
| `per-turn/v1` | 62/50 | 64/54 | 64/50 | 28/31 | 32/28 |

First-target completion by stratum, seed 1/seed 2:

| Rendering | `length2` | `length4` | `length8` | `reassert` | `update` |
|---|---:|---:|---:|---:|---:|
| `one-user-turn/v1` | 27/14 | 0/0 | 0/0 | 14/6 | 0/0 |
| `per-turn/v1` | 29/13 | 0/0 | 0/0 | 12/8 | 0/0 |

The seed-1 one-user repeat differs only in wall time and peak RSS; every
decision, reply, tally and paired count is identical.

The cumulative D84 receipt is
[`evidence/resources.json`](./evidence/resources.json): 14 retained reports,
54.553 seconds measured probe time, 137,456 KiB maximum peak RSS,
555,631,880 bytes of retained evidence/inputs, 3,674,715,698 bytes of
regenerable target, and 147.602 GiB free. The canonical shared ledger
`.uor-models/native-joint-learning-2026-09-04/model-time.json` was `ABSENT`.
Downloads, export, builds, tests and delivery are explicitly unmeasured.

## Checks run at the head

- `cargo test --offline --locked -p uor-r4-cpu-train --bin
  cpu-forced-copy-panel`: 7 passed.
- `cargo test --offline --locked -p uor-r4-cpu-train --lib`: 455 passed,
  4 ignored.
- `cargo test --release --offline --locked --features cpu-parity --bin
  served-reply-panel`: 15 passed.
- `cargo check --offline --locked -p uor-r4-cpu-train --bin
  cpu-forced-copy-panel`: passed.
- Targeted `rustfmt --check`, driver Clippy with no changed-code warnings,
  `git diff --check`, report check, local PR policy and full loop gates pass.
- Five final reports cover two seeds, both renderings and the exact-schema
  repeat. All 256 rows and 128 pairs are present in every arm.

## Skills used

No system-provided Codex skill applied. The repository's geometry, D84 CPU
research and loop-delivery rules were used directly.

## Still open

- Implement a lawful role-conditioned value reader/pointer. Correct-value
  forcing shows a large capability ceiling, but it does not provide the
  mechanism that finds that value without a label.
- Preserve continuation after the located value. The residual 6-56 failures
  show that opening/copy alone is not sufficient.
- Do not tune on these forced prefixes or merge them into serving. The panel is
  now consumed and this unit is an attribution diagnostic only.
- Claude-325 and Claude-323 remain the main training-track experiments. This
  branch neither waits for nor modifies them.
- Fresh exact-head review, protected queue and fresh-main verification remain.

## Plain summary

The model already has most of the machinery needed to finish an answer after it
is handed the right value: 200 to 250 of 256 replies become complete. Handing it
only the first correct token is not enough, and handing it the other relation's
value or a shuffled value never counts. So the next useful mechanism is not
another global point table or more stop tuning. It is a lawful way to choose the
role-correct value from the prompt and copy it into the reply, followed by the
existing geometric continuation and termination path.
