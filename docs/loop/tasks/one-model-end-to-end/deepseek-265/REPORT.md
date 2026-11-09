# REPORT: one-model-end-to-end, deepseek-265

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `ea3398701`.

## What did not work, first

- **The first value locator was not a value locator.** The initial two/three-token
  repeated-context-span heuristic looked plausible in aggregate, but decoded
  examples selected keys, articles and template fragments such as `"wid is"`,
  `" capital of"` and `" script"`. It is retained only as a control explicitly
  labelled `not-semantic`. It is not used to claim that TRAIN values are inside
  or outside the carry.
- **The existing seed-2 ranked files were not official-read artifacts.** They
  named the discarded direct-snapshot artifact audit (`852c4721…` and
  `e3fd234c…`), not the published `2741d938…` and `391392f5…` pair. The tool
  refused them by artifact identity. I regenerated both first-rank reports on
  CPU from the official artifacts with the merged `served-reply-panel`, then
  required all 256 cases to agree.
- **The carry does not make the asked value first on any official row.** At the
  first reply position it is top-1 on 0 of 256 rows at both seeds, with or
  without the carry.
- **Seed 2's two first-line successes are not carry successes.** The exact `−C`
  control has the same two successes. Their first-token ranks are 3,420 versus
  3,422 and 3,242 versus 3,232; one improves by 2 ranks and the other worsens by
  10. The value appears later in the line, not at the first predicted position.
- **This unit cannot identify a semantic answer-value row in the generic mixed
  text.** The published mask and tokenizer identify question and reply
  boundaries, but do not identify which reply tokens are the asked value. That
  measurement remains unavailable and is not inferred from proximity.

## What changed

Added `real-carry-geometry` to the one-model CPU instrument:

- `train-census` validates the UORT token file, mask and manifest hashes,
  identifies masked assistant replies, counts question-to-first-scored-row and
  question-to-first-answer-token distances, reports source/component coverage,
  and runs shifted and same-document shuffle controls.
- `first-reply` renders the official panel prompts, replays carried and exact
  `−C` artifacts through `CpuFbsForward`, verifies the cut byte for byte, checks
  both existing/regenerated official first-rank reports case by case, and writes
  each readout's Hamming selections plus the exact carried query terms.
- `receipt` verifies byte-identical reruns and emits a small committed summary.
  Full 14 MB traces stay in the project store.

At the code head the size is **1,867 code/build lines** including tests, **271
task/document/notebook lines**, the generated **807-line receipt**, and a
**7-line claim record**: 2,952 lines in all. The roadmap item has no numeric line
estimate, so no ratio is available. No model weight, serving arithmetic, panel
bar, run or lease changed.

## Measurements

### TRAIN d3mwcap6

Inputs were the published `0273173e…` tokens, `da506e85…` mask and `442ead2c…`
manifest. The full run and a byte-identical repeat both hash to
`215155331284b58abf260cd5a6aacd4991ebafd6fa0d07161407e1daadfd8423`.

- **176,299 questions** were identified across the manifest's **378,016
  documents**.
- **176,299 / 176,299 question ends are exactly 5 rows before the first scored
  reply input.** The first masked answer target is exactly 6 rows after the
  question end. Every observed value is 5 or 6, respectively.
- The distance from the **start** of the user question to the scored input has
  p50 13, p90 17, p95 19 and p99 34. **174,353 / 176,299 (98.9%) are within 32
  rows**; 176,259 are inside one 192-token window.
- Coverage is 171,834 `mw-balp` questions and 4,465 OASST2 questions. Each
  component has 100% first-reply coverage at 32 rows.
- The control that shifts a question 32 rows earlier leaves **0 / 176,299**
  inside the carry. Same-document next-question pairing leaves 6,544 / 11,010
  inside, while 165,289 documents have only one question and are excluded.
- The nonsemantic repeated-span control is retained in the receipt with decoded
  examples. It supports no value-reach claim.
- The most common **first assistant reply tokens** are `" It"` 39,860,
  `" Your"` 38,861, `" That"` 27,551, `" You"` 12,780 and `" The"` 7,457. These
  are sentence/template openings, while the panel's expected first token is the
  asked value itself.

### Official first-reply traces

Both seeds use the published step-16,384 artifact and exact `−C`. Both traces
reproduce the official 256 first-token ranks, and byte-identical repeats match.

| seed | receipt SHA-256 | query changed | type-head rows changed | asked top-1 carried / `−C` | asked-rank p50 carried / `−C` | first-line answered carried / `−C` |
|---|---:|---:|---:|---:|---:|---:|
| 1 | `01e57efe…` | 168 / 256 | 256 / 256 | 0 / 0 | 1,423 / 1,454 | 0 / 0 |
| 2 | `b0e79f3f…` | 215 / 256 | 256 / 256 | 0 / 0 | 2,447 / 2,450 | 2 / 2 |

At seed 1, carry changes 156 complete first-position decisions; asked rank
improves on 60 rows, ties on 120 and worsens on 76. At seed 2 it changes 208
rows; asked rank improves on 95, ties on 88 and worsens on 73. The shift moves
the final Q12 query and the readout's selected positions, but not the asked
token to top-1.

The asked value's spelling is present in every panel prompt, but only 159 / 256
prompts have its last occurrence inside the final 32 rows. The trace records the
per-position carry gain and term magnitude; the exact `−C` remains the negative
control. Neither seed's first-line count changes under that control.

**Diagnosis:** the first-reply failure is not a carry-reach failure. The model
is trained to put a pronoun/template first (`"Your friend…"`, `"It…"`,
`"That's…"`) while this panel grades a value-first reply. Moving the question
closer cannot by itself turn the first trained target into the panel target.
This is a target/layout mismatch supported by both the exact TRAIN first-token
census and the official first-position trace; it is not evidence that geometry,
the ask heads or the carry mechanism cannot work.

## Checks run at the head

- `cargo test --offline --features cpu-parity --lib`: **23 passed**, including
  four new distance/inference/overwrite/input-render tests.
- Strict changed-target Clippy:
  `cargo clippy --offline --features cpu-parity --bin real-carry-geometry --no-deps -- -D warnings`:
  passed.
- Changed-file `rustfmt --check`: passed.
- Full TRAIN census and repeat: byte-identical, **176,299** questions.
- Official seed-1 trace and repeat: byte-identical, **256 / 256** official rank
  parity.
- Official seed-2 trace and repeat: byte-identical, **256 / 256** official rank
  parity; the rank reports were regenerated from the published artifacts.
- Exact `−C` reproduction, panel hashes and writer-table equality: passed on
  both seeds.
- Store upload and read-back: **6 / 6 files** hash-identical.
- No GPU, pod, lease, training run or Claude artifact was touched.

## Skills used

No Codex skill was used. This work used the project loop, Rust crates, published
artifacts, `served-reply-panel`, and the project store.

## Still open

1. **Semantic value-row distance remains unmeasured.** A future corpus with
   answer-span tags could count it directly; token proximity cannot.
2. **The stopping loss is outside this unit.** It appears without the carry and
   was not assigned to ask heads, pull or trainer commit.
3. **The next model unit should match the intended reply layout.** A reasonable
   next comparison is a response layout whose first scored reply token is the
   asked value, or a separately registered training-protocol reader that stops
   grading the current template opening as if it were a value-first answer.
   That is a new training/decision unit and does not alter this registered
   outcome.

## Plain summary

The trained memory is close enough to the question: every identified TRAIN
question ends only five rows before the row that predicts its first reply, so a
32-row carry can reach the question in training. On the official test prompts,
the carry changes many later choices and positions, but the first token is never
the asked value on either seed. The two seed-2 lines that eventually name the
right value do so later in the line and still do so when the carry is removed.
The stronger mismatch is in the training data: its replies usually begin with a
pronoun or sentence template, while the panel expects the value first. This
rules out carry reach as the explanation for this failure. It does not prove
that the carry cannot work, and it does not change Claude's outcome or the
serving path.
