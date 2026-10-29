# ORDER: infra-loop-harness, claude-201 (the item for scoring one model end to end)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md), the umbrella that adds roadmap items (as claude-162 did). Base: `d28142f9e`.

Setup row: 2 (item-file maintenance: a new gap item).

## Goal

Add the roadmap item `one-model-end-to-end` for the last part of the open direction's order: "`turn-memory-reply` first, then the integer serving of the parts, R7, and one model end to end" (`PLAN.md:86`). The item's work comes from the whole-project assessment Casey asked for on 29 Oct (about 20:05Z): "in a smart way that considers the whole project and outside research ... instead of making quick (and wrong) decisions".
- **The assessment:** a fresh decision review passed it in its third round. The owner log is on #2028 (`6507593846`).
- **Where it lives:** the unit lands it as [ASSESSMENT.md](ASSESSMENT.md) in this folder.
- **The item's first three steps:**
  1. a data census on the laptop: the TRAIN size conflict, a TRAIN reply index, and which panel holds new templates;
  2. a small GPU check, (r) against (r0) on DEV's 3,438 unnamed rows with a band fixed in advance, sharing a pod with integer-hamming-read's step 5;
  3. the end-to-end scoring protocol, written before any training run.

## Cancel probe (run first)

- **Does an item already cover this?** No.
  - `in-space-trunk-to-text` is done.
  - `m2-fresh-heldout-panel` and `m1-fresh-panel-transfer` are done, and each serves another milestone's bar.
  - No item holds the census or the end-to-end protocol. Checked by `git grep -l -i "end to end"` over `docs/roadmap/items` at `d28142f9e`.
- **Classed:** instrument. Nothing cancels the unit.

## Files in scope

- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/roadmap/issues.json`
- `docs/roadmap/README.md` (the rendered list)
- `docs/loop/tasks/infra-loop-harness/claude-201/**`

## Done when

1. **The item exists.** It is `status: ready` and `needs_gpu: true`. Its scope covers the census instrument, the GPU check's scoring, and their records. Its done-when holds the three steps above. It links ASSESSMENT.md.
2. **`cargo xtask roadmap check`** passes, and the render is current.
3. **`roadmap sync --apply`** gives the item an issue, which is recorded in `issues.json`.
4. **A fresh review** of the item against the assessment, and `loop report --check` and `loop pr-check --local`, pass.

## Stop and ask if

Stop and decide (never wait) if:
- the item's scope would overlap an open ORDER: then narrow it;
- the census instrument needs a workspace crate change outside the item: then it is a standalone instrument under `docs/labs/`, as `m2-fresh-heldout-panel`'s is.
