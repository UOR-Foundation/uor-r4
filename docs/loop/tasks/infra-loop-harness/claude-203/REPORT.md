# REPORT: infra-loop-harness, claude-203 (the scope a held-out-phrasing panel needs)

Lab: claude. Order: [ORDER.md](ORDER.md). Refs #3407.

**Speed.** No serving change: a scope decision.

## What did not work, first

- **The panel this enables tests held-out *prompts*, not held-out reply wording.** M-world v2's phrasing split covers user-turn templates and fillers only. The assistant's reply templates are shared across splits: `MQAR_ACKS`, `MQAR_REPLIES`, `UPDATE_ACKS`, and v1's closers and absent replies. So a `dev_phrasing × dev_value` panel's replies differ from TRAIN's only by their values. Its census is expected to show a high masked reply-template share. The ORDER's new-template bar is on prompts, so the panel can still pass it.
- **The split can leak in two ways, which only the panel's census can measure:**
  - a TRAIN paraphrase near-equal to a dev template. The generator refuses only exact equality (`milestone_world_v2.rs:514-515`);
  - chat-v0 turns that coincide with short dev phrasings. chat-v0 is screened against the probe, not against dev templates.
- **Not confirmed:** which `paraphrases=` file, if any, built `mw-balp`. The panel unit's census must record it.

## What changed

- **`docs/roadmap/items/one-model-end-to-end.md`:**
  - **`scope:`** adds `crates/uor-r4-training/examples/m-world.rs`. With it, the next unit can give `m-world corpus world=v2` a `cell=` option, writing M-world v2's `dev_phrasing × dev_value` cell as a UORT split for `train-in-space --census`.
  - **"What is missing"** names the held-out-phrasing panel, the writer it needs, what the split does and does not cover, and the two risks its census counts.
- **No other file is needed.** `conversation_in(rng, cell)` and `Cell::new` are public (`milestone_world_v2.rs:137-170, 1398`). `corpus_v2` hard-codes `Split::Train` (`m-world.rs:1210-1217`).
- **`roadmap sync --apply`** updated #3407's body.

## Measurements

None: a decision.

## The decision review

Fresh context, at `e074bde69`. It read:
- the item and the census REPORT;
- the generator's cell code: `Cell`, `Split`, `split_of`, `conversation_in`, `Phrasings`, `install_train_paraphrases`;
- the example's `corpus_v2`;
- the open PRs, and the other items' scopes.

- **DECISION REVIEW: PASS.**
  - **The file is necessary and enough:** the example can wrap `conversation_in` as it already wraps `conversation`, with no library change.
  - **The split holds by construction:** TRAIN draws only `Split::Train`, through both v1 and v2 lineages; paraphrases equal to any template are refused; development picks are never paraphrased.
  - **Its residual risks,** above, are what the item's census already checks.
  - **Nothing is the owner's call, and there is no collision:** no other item lists `m-world.rs`, and the only other open PR is #3323, in the object store.
- **DIFF REVIEW: APPROVE WITH NITS.** Its two nits were fixed at the next commit:
  - the split's coverage, user turns only;
  - the risks the panel's census counts.

  Its third note: the ORDER's library stop does not fire.

The owner log is on #3407 (`6508925426`).

## Checks run at the head

- `cargo xtask roadmap check`: 111 items OK.
- `roadmap sync --apply`: #3407 current.
- `loop report --check` and `loop pr-check --local`: PASS at this REPORT's commit.

## Skills used

- None was invoked this unit.

## Still open

- **The panel unit, under `one-model-end-to-end`:**
  - a `cell=` option on `corpus world=v2`;
  - a dev-cell panel of about 400 documents, the size of DEV;
  - a `--panel DIR` flag on `train-in-space --census`;
  - the panel's census, judged by claude-202's rule: masked prompt-template share at K = 1,024 at most 0.5.

## Plain summary

The tool that writes our training data can now also be changed to write test conversations in wording the training data never used. The next step builds that test set, and checks it really is new.
