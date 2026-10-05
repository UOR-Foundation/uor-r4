# Chat evaluation panels: scale-matched tiers (#820)

References #820. These files add two evaluation tiers to the ladder chat panel
(`~/uor-r4-local/ladder/panel/`, graded by `chat-grade`). The original panel files are
unchanged. Step 0a ([taxonomy](../../docs/research/step0/0a-panel-taxonomy.md)) found that
the 232-request panel is dominated by open-domain knowledge requests and that about a quarter
of the failing heldout rows are ill-posed. A 30M–100M model trained mostly on TinyStories and
chat data cannot be expected to answer open-domain knowledge requests, so on that panel a
change in conversation or mechanism barely moves the score.

## Tier C: conversational panel (`conversational-160*.json`)

- **What it is:** 160 new requests that need no world knowledge. Each one is answerable from
  the conversation itself or from general conversational sense.
- **Format:** the `chat-grade` request format (`id`, `category`, `user_turns`). Ids are
  `conv-*`, which is how `chat-grade` assigns tier C.
- **Files:** `conversational-160.json` holds all 160 rows. A `chat-grade` panel file is capped
  at 128 requests, so `conversational-160-a.json` (rows 1–80) and `conversational-160-b.json`
  (rows 81–160) split it for loading; pass both together.
- **Row order:** rows interleave round-robin by category, so the derangement control's
  neighbouring row is always from a different category. Otherwise two greetings could stand
  as each other's "relevant" control.
- **Multi-turn rows:** 36 rows have 2–4 user turns. `reply_panel` answers each turn with the
  model's own greedy reply, so the last reply is graded on the model's real history.

| Category | Rows | Multi-turn | Tests |
|---|---|---|---|
| `smalltalk_feelings` | 26 | 0 | greetings, feelings, small news |
| `multi_turn_memory` | 30 | 30 | the last turn needs a fact or plan stated in an earlier user turn |
| `self_contained_instruction` | 30 | 0 | rewrite politer, list three, count words in a given sentence, rhyme, yes/no about a fact stated in the turn |
| `clarify_or_on_topic` | 24 | 3 | an ambiguous request should get a sensible clarifying question; on-topic follow-ups |
| `story_continuation` | 26 | 2 | short TinyStories-register stories and continuations |
| `unknowable_or_impossible` | 24 | 1 | personal facts with no context ("what did I eat yesterday?") or physically impossible asks; the reply should say it does not know or cannot |

`conversational-160-keys.tsv` is the author's note of the expected content of each
`multi_turn_memory` last reply. `chat-grade` does not read it. The grader judges fluency and
relevance only, not whether a recalled fact is correct, so this file is there for a later
exact-recall check.

**Freeze statement.** The panel was authored on 2026-10-05 from the category specification
alone, before any model had replied to any of these requests. The author had read the Step 0a
labels and notes on the old panel, but did not see any model output for these requests.
`chat-grade check` (tokenizer and context only, no model) is the only tool that has read them.
The sha256 values are in `MANIFEST.sha256`. Any edit makes a new panel with a new name.

## Tier K: clean open panel (`heldout-*-ids.txt`)

- **What it is:** the 200 heldout-200 rows, split into K-clean and K-ill-posed. A row is
  ill-posed when its request refers to material it does not contain: a letter salutation
  only, "the following X:" with X missing, a `[placeholder]` template, an
  "edit this essay" with no essay, or a fragment of a document.
- **`heldout-ill-posed.tsv`:** each ill-posed id with its source.
  - `0a-29m` / `0a-96m` / both: the Step 0a label `ill=1` (55 ids). 0a labelled failing rows
    only, and the two models' flags never conflict on a row both labelled.
  - `screen`: 16 further ids. These rows were accepted by both 0a models, so 0a never
    labelled them. I applied the same criterion to the request text alone, without reading
    the replies; the note column gives the reason for each.
- **`heldout-ill-posed-ids.txt`:** the 71 ids of tier K-ill-posed. `chat-grade` embeds this
  file as its default list; `ill_posed=` overrides it.
- **`heldout-clean-ids.txt`:** the 129 ids of tier K-clean.

To grade only the clean rows, pass `exclude=data/panels/heldout-ill-posed-ids.txt` to
`chat-grade grade` or `reply`. Alternatively, write a new panel file with
`chat-grade filter out=NEW.json requests=heldout-200-a.json,heldout-200-b.json exclude=...`.
Neither changes the original panel files.

## Scoring per tier

The `grade` and `grade-replies` reports keep every field they had and add two more:
`per_tier` (the actual, control and paired-against-control results per tier, with per-category
results inside each tier) and `tiers` (the tier rule, plus the source and sha256 of the
ill-posed list). The other subcommands:

```text
chat-grade tiers report=REPORT.json [out=NEW_ROOT]           # re-tier an existing report, no regrading
chat-grade compare a=A/report.json b=B/report.json [tier=C] [out=NEW_ROOT]
chat-grade check requests=conversational-160-a.json,conversational-160-b.json tokenizer=T.json context=384
```

`compare` pairs the rows of the two reports by id within each tier. It gives two-sided exact
McNemar results for acceptable, fluent and relevant, and per-category acceptable counts. A
row present in only one report is counted under `unpaired` and left out of the test. If the
two reports were graded by different graders, `compare` warns and records
`same_grader: false`.

## Copies

The copies in `~/uor-r4-local/ladder/panel/` are byte-identical to these files; see
`MANIFEST.sha256`.
