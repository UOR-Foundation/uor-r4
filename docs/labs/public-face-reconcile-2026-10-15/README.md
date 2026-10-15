# Public-face reconciliation, 15 October 2026

**Scope:** docs-only status annotation of the repository's public face (README and
`docs/figures/`) against the current decisions. No compute, no pod, no capability claim.

## What changed

- [README.md](../../../README.md) D29 row: records that NVRTC was removed from the module set by #2344.
- [docs/figures/geometry/hamming-rank-read.svg](../../figures/geometry/hamming-rank-read.svg):
  "today's read" and "vs today" wording now say the exp-table softmax read is the historical
  pre-D23 float read. The discrete `ReadSpec::Flock` read is the active one.

## Checked, no change needed

- Pages deploy (D36): README does not mention GitHub Pages. `architecture.svg` and
  `serving-pipeline.svg` already label the WASM Studio as FROZEN.
- Softmax, sigmoid, tanh, GELU, matmul in README: already carried as `dormant` or
  `archived-negative (D22 scope)` in the mechanism tables.
- `architecture.svg`, `serving-pipeline.svg`, `geometry-stack.svg`, `milestones.svg`: retired terms
  appear only as historical, rule, or replacement references.
- `next-token.svg`: its committed "historical (pre-D25)" banner is kept.

## Open

- Starling line: the README does not state its M4 count. `current-state` (2026-10-13) records 3/3 and
  a posted D21 pivot card continuing with one decisive run. The PM assignment says "pending pivot
  card". Not annotated, pending Kimi PM answer on #2028.
- `docs/figures/geometry/generate.py` does not reproduce the committed SVGs. Running it changes
  `600-cell-icosians.svg` and `next-token.svg` and drops the next-token historical banner. The
  generator was not run for this change; the committed SVG was hand-edited.

Line: public-face docs · count unchanged · M1 headline 37/232 unchanged. No compute.
