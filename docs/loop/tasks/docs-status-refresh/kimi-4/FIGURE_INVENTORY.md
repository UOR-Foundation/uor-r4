# Figure inventory

Lab: kimi. Base: `ec9a790fe5be73cd673f082eba86ced551b22d12`.

There are **68 tracked SVGs: 17 generated outputs and 51 hand-maintained figures**. Generator outputs are regenerated, never hand-edited. Hand-maintained status figures are edited only with their captions and accessible descriptions; conceptual and mathematical figures without stale status stay unchanged.

## Generated outputs (17)

- **8 `figures.rs` outputs:** `architecture.svg`, `architecture-narrow.svg`, `milestones.svg`, `training-pipeline.svg`, `serving-pipeline.svg`, `geometry-stack.svg`, `geometry/compliant-path.svg`, `geometry/operators.svg`.
- **4 `figures_charts.rs` outputs:** `start/status-vs-baselines.svg`, `research/results-share-by-week.svg`, `research/timeline.svg`, `research-lines-timeline.svg`.
- **5 `figures_math.rs` outputs:** `geometry/e8-coxeter-plane.svg`, `geometry/600-cell-coxeter.svg`, `mechanisms/flock-read.svg`, `geometry/spiralcore-ternary-tree.svg`, `mechanisms/route-read.svg`.

## Hand-maintained figures (51)

Every remaining path below is included in the status/caption review.

On 30 October a second pass re-reviewed all 51 paths' status text, captions and accessible descriptions. Six mechanism statuses were corrected, the lineage was labelled historical with current statuses, and the Hamilton, transformer and LUT descriptions were reconciled. The changed status figures and DEV chart were inspected at desktop and 390 px; SpiralCore's status moved to its own line after an actual overflow was found and fixed. `research/timeline.svg` is explicitly a historical audit view of phases A–E; later Phase F and Phase G are in `docs/research/history.md`, not presented as part of that chart. All 17 generated figures were regenerated, with the DEV target 6,324 visible. A later exact-head review nevertheless caught stale Hamilton and BVG-K labels; those labels and their accessible text are corrected in the final repair rather than treating the source review as infallible. The `d9b9b4d8f` review then caught the main pipeline's mixed M1/FBS composition and a solid planned browser cache. The pipeline is now two explicit lanes with the correct M1 position/read/normalization/readout path and visible FBS Hamming/memory nodes; the browser model cache and its arrow are dashed and distinct from verified synthetic session storage. Complete now has a stronger green fill than Working. These repairs were rendered and visually inspected, including a focused pipeline export-stage crop.

docs/figures/geometry/600-cell-icosians.svg
docs/figures/geometry/connections.svg
docs/figures/geometry/exact-2i-lanes.svg
docs/figures/geometry/golden-integers.svg
docs/figures/geometry/hamming-rank-read.svg
docs/figures/geometry/hopf-fibration.svg
docs/figures/geometry/native-input-access.svg
docs/figures/geometry/next-token.svg
docs/figures/geometry/octonion-signed-binding.svg
docs/figures/geometry/prime-addressing.svg
docs/figures/geometry/protected-joint-learning.svg
docs/figures/geometry/quaternion-rotation.svg
docs/figures/geometry/s3-and-transport.svg
docs/figures/geometry/vsa-hypervectors.svg
docs/figures/geometry/zeta-phases.svg
docs/figures/mechanisms-family.svg
docs/figures/mechanisms/address-cells.svg
docs/figures/mechanisms/admission.svg
docs/figures/mechanisms/carrier-credit.svg
docs/figures/mechanisms/discrete-learning.svg
docs/figures/mechanisms/e8-orbit-memory.svg
docs/figures/mechanisms/exact-memory.svg
docs/figures/mechanisms/geometric-stack.svg
docs/figures/mechanisms/hamilton-gate.svg
docs/figures/mechanisms/hopf-observation.svg
docs/figures/mechanisms/legal-construction.svg
docs/figures/mechanisms/lut-serving.svg
docs/figures/mechanisms/native-geometric.svg
docs/figures/mechanisms/object-store.svg
docs/figures/mechanisms/pointer-key.svg
docs/figures/mechanisms/prime-zeta-codebook.svg
docs/figures/mechanisms/r4g1-graph.svg
docs/figures/mechanisms/rank-margin.svg
docs/figures/mechanisms/read-binding.svg
docs/figures/mechanisms/recurrence-gates.svg
docs/figures/mechanisms/relation-read.svg
docs/figures/mechanisms/spin-weights.svg
docs/figures/mechanisms/spiralcore-fbs.svg
docs/figures/mechanisms/ternary-xor.svg
docs/figures/mechanisms/tokenizer.svg
docs/figures/mechanisms/vsa.svg
docs/figures/model/discrete-learning-loop.svg
docs/figures/model/integer-export.svg
docs/figures/model/pipeline.svg
docs/figures/products/desktop-client-architecture.svg
docs/figures/products/product-family.svg
docs/figures/products/web-client-architecture.svg
docs/figures/projects/projects-map.svg
docs/figures/start/hero-how-it-thinks.svg
docs/figures/start/roadmap.svg
docs/figures/start/transformer-vs-geometry.svg
