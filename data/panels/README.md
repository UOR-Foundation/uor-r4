# Chat evaluation panels: scale-matched tiers (#820)

References #820. These files add two evaluation tiers to the ladder chat panel
(`~/uor-r4-local/ladder/panel/`, graded by `chat-grade`). The original panel files are
unchanged. Step 0a ([taxonomy](../../docs/research/step0/0a-panel-taxonomy.md)) found that
the 232-request panel is dominated by open-domain knowledge requests and that a large share of
the failing heldout rows are ill-posed. A 30M–100M model trained mostly on TinyStories and
chat data cannot be expected to answer open-domain knowledge requests, so on that panel a
change in conversation or mechanism barely moves the score.

## Tier C: conversational panel (`conversational-v2*`)

- **What it is:** 160 requests that need no world knowledge. Each one is answerable from the
  conversation itself or from general conversational sense.
- **Format:** the `chat-grade` request format (`id`, `category`, `user_turns`). Ids are
  `conv-*`, which is how `chat-grade` assigns tier C.
- **Files:** `conversational-v2.json` holds all 160 rows. A `chat-grade` panel file is capped
  at 128 requests, so `conversational-v2-a.json` (rows 1–80) and `conversational-v2-b.json`
  (rows 81–160) split it for loading; pass both together.
- **Row checks:** `conversational-v2-checks.tsv` (embedded in `chat-grade`; see below).
- **Row order:** rows interleave round-robin by category, so the derangement control's
  neighbouring row is always from a different category. The derangement therefore catches
  off-topic replies; generic replies are caught by the constant-reply control instead.

| Category | Rows | Multi-turn | Row check | Tests |
|---|---|---|---|---|
| `smalltalk_feelings` | 26 | 0 | none | greetings, feelings, small news |
| `multi_turn_memory` | 30 | 30 | `any` (27 recall, 3 derived) | the last turn needs a fact stated in an earlier user turn |
| `self_contained_instruction` | 30 | 0 | `any` on 6 rows with one determinate answer not in the turn | rewrite, list, count, rhyme, order words |
| `clarify_or_on_topic` | 24 | 3 | `question` on the 21 ambiguous single turns; `any` (topic) on the 3 follow-ups | ambiguous requests should get a clarifying question; follow-ups need the earlier topic |
| `story_continuation` | 26 | 0 | none | short TinyStories-register stories and continuations |
| `unknowable_or_impossible` | 24 | 0 | `abstain` | personal facts with no context, physically impossible asks |

Every multi-turn row depends on its earlier turns: no check term is in the last user turn
(`chat-grade check` refuses the panel otherwise), and for a `recall` row a term is in an earlier
user turn. No row's answer depends on the model's own earlier reply.

### Why v2 (v1 withdrawn before any model replied)

`conversational-160*` (commit 3564f9c1, review on #1724) was withdrawn. No model had replied
to it. v2 keeps v1's ids, categories and order and changes these rows:

- **Grader blind to history (review b).** The grader's relevance question asks only whether a
  reply answers the *last* message, so "Your name is Tom." passed a recall row. v2 adds the
  frozen row checks below; v1's keys sidecar was never read by `chat-grade`.
- **Rows that did not depend on the earlier turn (review b):** mem-03/08/13/16/19/24/25 were
  rewritten so the answer is a stated fact (mem-24 no longer depends on the model's own story);
  clar-17/18/23 now ask a follow-up that names nothing without the first turn; story-23/24 and
  unk-21 are single turn now. Multi-turn rows: 36 → 33, all history dependent.
- **Leakage into training text (review d):** rows matching M-world v1 intents (clarify, greet,
  how_are_you, thanks, farewell), relation templates (`My name is {v}.`, `I live in {v}.`,
  `What color is {x}?`, `My lucky number is {v}.`, ...), the AERM dev-mem panel or whole user
  lines of the training corpora were rewritten: talk-02/03/05/09/19/22, clar-01/03/05/08/14/19/21,
  mem-01/02/03/10/14/15/22/26/29, do-02/14, unk-04. clar-12/16 were made ambiguous so a question
  check applies. `chat-grade leak` found 15 leaking v1 rows and finds none in v2 (below).

### Row checks (`conversational-v2-checks.tsv`)

Tab-separated `id kind history terms`. A checked row is acceptable only when the grader judges
it fluent and relevant **and** it passes its check:

- `any`: the reply contains one of the `|`-separated words or phrases (words lowercased, split
  at anything not a letter, digit or apostrophe; a phrase matches consecutive words).
- `abstain`: the reply contains one of the fixed abstain phrases (`don't know`, `not sure`,
  `can't`, `cannot`, `no way to know`, ...; listed in every report).
- `question`: the reply contains `?`.

### Controls

Every graded row is also graded with (a) its reply placed after the next row's conversation
(the derangement) and (b) each of three constant replies in place of the model's
(`DEFAULT_CONSTANTS` in `chat-grade.rs`; `constants=none|FILE` overrides): "I'm not sure. Can you
tell me more about what you mean?", "That sounds nice! Thank you for telling me." and a
TinyStories opening. Controls go through the same checks. Each report gives, per tier and per
category, the paired McNemar of actual against the derangement and against each constant.
The check-only controls (no grader) apply each check to the row's last user turn and to all its
user turns joined.

`chat-grade check` gives the check-only pass counts before any model replies:

| Category | checked | constant 1 | constants 2, 3 | echo last turn | echo history |
|---|---|---|---|---|---|
| `multi_turn_memory` | 30 | 0 | 0 | 0 | 27 |
| `self_contained_instruction` | 6 | 0 | 0 | 0 | 0 |
| `clarify_or_on_topic` | 24 | 21 | 0 | 14 | 14 |
| `unknowable_or_impossible` | 24 | 24 | 0 | 0 | 0 |

So the check alone does not separate a model from constant 1 on `clarify_or_on_topic` and
`unknowable_or_impossible`: a generic "I'm not sure, can you tell me more?" is a correct answer
to most of those rows. Those two categories, and the unchecked `smalltalk_feelings` and
`story_continuation`, are read only through the paired test against the best constant for the
category; a category the model does not beat its constants on says nothing about the model.
An echo of the history passes most memory checks, so a memory pass also needs the grader's
fluent/relevant verdict on a reply that is not a copy.

### Leakage check

```text
chat-grade leak requests=conversational-v2-a.json,conversational-v2-b.json \
  reference=crates/uor-r4-training/src/{milestone_world,milestone_world_v2,stack_aerm,stack_dialogue}.rs,\
crates/uor-r4-core/src/answer_oracle.rs,docs/integration/paraphrase-review-2026-10-01/decisions.tsv \
  whole_only=everyday-32.json,stretch-32.json,chat-v0-20260925/panel-requests-dev.json \
  corpora=ladder/corpora/ft-mixed-mworld-chatv0,ladder/corpora/tinydialogues-train tokenizer=T.json
```

A turn leaks when its words equal a reference text's or a corpus line's (role label stripped;
a `{placeholder}` stands for 1–4 words in a template of at least 3 literal words), or when it
shares 6 or more consecutive words with a `reference=` text. The corpora are the decoded token
stores of the `ft-mixed-mworld-chatv0` fine-tuning arms (M-world paraphrases plus chat-v0 train,
147,359 documents) and TinyDialogues train (110,024 documents); this is the check that was
UNAVAILABLE in the review, made by decoding the local token stores. v1: 15 leaking rows. v2:
none. Four v2 rows share an 8-word run with a corpus line (story-01/24/26, talk-17; TinyStories
phrasing such as "once upon a time there was a little bunny"); that is reported, not counted.
Not checked: TinyStories train (story register by design) and the sealed §8 panel (never read).

**Freeze statement.** v2 was written on 2026-10-05 from v1 and the #1724 review, before any
model had replied to any v1 or v2 request. Only `chat-grade check` and `chat-grade leak`
(tokenizer, references and corpora; no model) have read it. The sha256 values are in
`MANIFEST.sha256` and posted on #820 before the first reply run. Any edit makes a new panel with
a new name.

## Tier K: clean open panel (`heldout-*-ids.txt`)

- **What it is:** the 200 heldout-200 rows, split into K-clean (110) and K-ill-posed (90).
- **Criterion (one rule, request text only, applied to all 200 rows).** A row is ill-posed when
  its request text, read alone, cannot be answered as asked because:
  - `absent`: it points at specific material it does not contain and that is not common
    knowledge ("the following essay", "this code", "the recipe", "the region", "your
    suggestion", "Read the passage below ...");
  - `template`: it has an unfilled slot (`[city]`, `[event]`);
  - `constraints_only`: it holds only output-format constraints and no task;
  - `fragment`: it holds no request and reads as a piece of a document (a letter salutation,
    a post's opening line, a recipe step, a list item, a quotation, a citation, "Given the
    text: ..." with no question, a sentence cut off).
  A request about a named, widely known thing, a role-play set-up, or a request that includes
  its material is clean. Vague or knowledge-heavy requests are clean.
- **No model outcome was used.** v1 combined the Step 0a labels (which covered failing rows
  only) with a screen applied only to rows both models passed, so membership depended on the
  29M/96M outcomes (review e). v2 labels every row from its text alone. As a cross-check only:
  all 55 0a `ill=1` ids and all 16 v1 screen ids are ill-posed under the text rule; heldout-051
  was also in 0a; 19 rows are new (001, 016, 070, 086, 087, 097, 117, 123, 124, 128, 135, 139,
  141, 150, 174, 175, 176, 180, 195). Twins in form now share a tier: 002/117/144 and 030/128.
- **`heldout-ill-posed.tsv`:** each ill-posed id with its reason.
- **`heldout-ill-posed-ids.txt`:** the 90 ids of K-ill-posed. `chat-grade` embeds this file as
  its default list; `ill_posed=` overrides it.
- **`heldout-clean-ids.txt`:** the 110 ids of K-clean.

To grade only the clean rows, pass `exclude=data/panels/heldout-ill-posed-ids.txt` to
`chat-grade grade` or `reply`, or write a new panel file with
`chat-grade filter out=NEW.json requests=heldout-200-a.json,heldout-200-b.json exclude=...`.
Neither changes the original panel files.

## Scoring per tier

`grade` and `grade-replies` reports keep their fields (`actual`, `control_derangement`,
`paired_against_control`, `control_rule`, `per_category`, `rows`) and add `control_constants`,
`check_only_controls`, `per_category_controls`, `per_tier`, `tiers`, `checks` and `constants`.
`acceptable` includes the row check where a row has one; `fluent_and_relevant` is the old count.
Rows without a check (all K and everyday rows) score exactly as before.

```text
chat-grade tiers report=REPORT.json [checks=...] [out=NEW_ROOT]   # re-tier, no regrading
chat-grade compare a=A/report.json b=B/report.json [tier=C] [checks=...] [out=NEW_ROOT]
chat-grade check requests=conversational-v2-a.json,conversational-v2-b.json tokenizer=T.json context=384
```

`compare` pairs rows by id within each tier and gives two-sided exact McNemar results for
acceptable, fluent, relevant and check pass. Rows present in only one report are counted under
`unpaired`. Different graders give a warning and `same_grader: false`. Unknown categories are
an error.

## Copies

The copies in `~/uor-r4-local/ladder/panel/` are byte-identical to these files; see
`MANIFEST.sha256`. The withdrawn v1 files stay there, marked withdrawn in its `README.txt`.
