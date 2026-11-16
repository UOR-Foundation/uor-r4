# REPORT: one-model-end-to-end, codex-31 (e-means is the routed read's content assignment)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parents: [codex-29](../codex-29/REPORT.md), [codex-30](../codex-30/ORDER.md).

## Why

codex-29's routed-zero read worked only with a *planted* routing: the missing piece was the rule
that maps content to a channel set. e-means is in the tree (Meta, arXiv 2611.09501, integer Q16;
`crates/uor-r4-discrete/src/e_means.rs`) and is exactly a content assignment. This probe asks whether
it carries a paraphrase retrieval, with the two falsifiers the owner requires.

## The probe

`crates/uor-r4-discrete/tests/e_means_assignment_probe.rs`. 8 clusters × 12 items, 8-dimensional
features (a cluster centre plus item noise), each item a value byte. An item's phases are a coarse
4-bit quantisation of its feature, so a small paraphrase leaves them and a different item does not
share them. The query is a noisy copy of one item's feature. The read is: nearest centroid by exact
squared distance (the e-means assignment), then the least circular energy over that cluster's items.
The fit sees two paraphrases of every item; the items are then mapped to their **fitted** clusters
(the first run failed because it compared true cluster ids with centroid indices — a label mismatch,
and its 0.271 was that bug, not the mechanism).

## Measurements

192 queries per cell; the shortlist is 12 of 96 items, so the read costs 8 centroid distances plus 12
energy sums (20 comparisons) against 96 for a global scan.

| Paraphrase noise | routed | shuffled labels | value-swapped | global scan |
|---|---|---|---|---|
| 100 | **0.997** | 0.003 | 0.000 | 1.000 |
| 400 | **0.601** | 0.007 | 0.000 | 0.538 |
| 1,200 | **0.062** | 0.007 | 0.000 | 0.021 |

- **The assignment carries it.** Routed 0.997 against a 0.003 control at a small paraphrase; the
  control collapses when the item-to-cluster labels are permuted (identical occupancy).
- **The value comes from the store.** The value swap is 0.000 at every noise level.
- **The shortlist is a denoiser.** At medium and large paraphrase it is *above* the no-shortlist
  scan (0.601 against 0.538; 0.062 against 0.021): removing other clusters removes the items that
  would win a global energy comparison on a noisy query.

## What it settles, and what it does not

- The missing rule for the routed read is a content codebook, and an integer e-means fit supplies
  it; the two falsifiers are real and both collapse.
- It does **not** yet run on the served reads: the features here are synthetic vectors, and the
  served assignment would be over token codes (the Hamming sign codes or the Q31 addresses). That
  transfer is the next unit.
- The e-means negative in the A8 codebook (2.165× worse than Lloyd) is a *regime* result: the ADR
  records that the paper's scaling laws were fitted at n ≥ 10⁵ and k ≥ 256 and are not optimal for
  small or low-dimensional data, which is what A8 is. At this probe's scale (96 points, k = 8) the
  fit was good enough to carry a 0.997 retrieval.

## Checks

`cargo test -p uor-r4-discrete --test e_means_assignment_probe` — 1 passed; the table is one run and
repeats (seeded).
