# ORDER: scale-probe, claude-295

Lab: claude. Item: [scale-probe](../../../../roadmap/items/scale-probe.md). Base: `e052abf41`.

## Goal

The item's done-when reads:

> 1. **A prep sheet** `docs/harness/prep/scale-probe-<date>.md` with a fresh FIT and the D67 pre-run review, both posted on this item's issue. It registers:
>    - the arms, the cost and the stop rule;
>    - the panel: the real-text corpus's own held-out splits, cut by source id, with a census of their 8-token overlap with each TRAIN mix (claude-294's decision card). P-dev stays reserved for `one-model-end-to-end`'s run;
>    - the bars, in advance.
> 2. **The runs,** sealed in the store with a SHA256SUMS that was read back. The pods are down.
> 3. **The REPORT** gives, per width and corpus size:
>    - training tokens per second, and served decode tokens per second against context on the exported artifact;
>    - the panel's reply scores, with paired bands across seeds;
>    - the fitted slope, and whether it is distinguishable from flat.
>
>    It also has one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

## Cancel probe (run first)

To fill in.

## Files in scope

- `docs/harness/prep/scale-probe*`
- `docs/labs/scale-probe/**`
- `docs/roadmap/items/scale-probe.md`
- `docs/loop/tasks/scale-probe/**`
- `docs/model/pipeline-map.md`
- `docs/research/notebook/float-training.md`

## Done when

To fill in.

## Stop and ask if

To fill in.
