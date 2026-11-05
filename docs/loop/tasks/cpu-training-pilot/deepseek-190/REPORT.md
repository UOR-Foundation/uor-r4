# REPORT: cpu-training-pilot, deepseek-190

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `3ac40d6d5`.

## Status

Complete at the CPU-preflight scope. All 18 registered cells passed exact
export/load/CPU/serving parity and the run completed in **9.196 s** with
**78,980 KiB** peak RSS. This is instrument, byte-cost and decode-throughput
evidence for the scale-probe widths; it is not corpus training, a scale slope,
a quality result or a statement that any width fits GPU memory.

## What did not work

- The automated `loop claim` rolled back because its generated commit lacked
  the required `Lab:` and `Order:` trailers. The documented hand-claim path
  was used.
- The first prep-shape check found the missing `Base artifact` and `Data split`
  blocks. Both are now explicit.
- The first fresh review found an invalid full source hash, missing synthetic
  shape constants in JSON evidence and fail-open peak-RSS parsing. The review
  rejected FIT/GO before compute.
- The second fresh review found that provenance of the full-path tables,
  complete shape assertions, the exactly-2-GiB boundary and visible per-repeat
  hashes were not yet enforced. All four were repaired before the registered
  run.
- The first `--help` probe and the first attempt to reuse a deleted worktree
  changed no project state.
- There is no GPU-memory result. A width-2,048 artifact is refused by the
  current writer cap, and this CPU run cannot show whether 1,024 fits GPU
  training.

## What changed

- Added `cpu-scale-serving-bench`, which constructs deterministic synthetic
  FBS trunks at widths 256, 512 and 1,024 and contexts 16, 64 and 192.
- Exports every cell through `UORLUT01`, repeats the complete
  `audit_served_forward` comparison three times, and requires exact query,
  token and kept-cell parity plus matching CPU/serving decision hashes.
- Records all three decision hashes, artifact and table bytes, shape constants,
  decode throughput, RSS and all four negative controls.
- Requires projection artifacts to carry no side tables and full artifacts to
  retain their expected Hamming heads/depths and nonempty opening/reply memory.
- Validates the reloaded vocabulary, width, context, MLP, layer count and
  kept-cell count; peak-RSS parsing fails closed and exactly 2 GiB is rejected.
- Added the pre-registration, FIT/GO record, compact evidence and README entry.

No serving, CUDA, float-training, exporter or other Claude-owned source
changed.

## Measurements

Registered binary:
`65c6fba90cfc36a6e55ec4d97276ad8574602b719cce7faa322af8ccc1101fe8`.
Summary SHA-256:
`a2453028e5039d635921e4052854bdd8bba15ef3baa5aa4b6f3f351f9b019214`.

| Width | Arm | Context | Artifact bytes | Resident bytes | Serving tok/s |
|---:|---|---:|---:|---:|---:|
| 256 | projection | 16 | 2,145,024 | 2,147,296 | 6,969.3 |
| 256 | projection | 64 | 2,145,024 | 2,147,296 | 7,102.4 |
| 256 | projection | 192 | 2,145,088 | 2,147,296 | 7,097.8 |
| 256 | full | 16 | 2,160,576 | 2,291,056 | 6,560.6 |
| 256 | full | 64 | 2,164,608 | 2,292,208 | 6,596.2 |
| 256 | full | 192 | 2,175,360 | 2,295,280 | 6,253.8 |
| 512 | projection | 16 | 4,280,384 | 4,288,832 | 3,953.8 |
| 512 | projection | 64 | 4,280,384 | 4,288,832 | 3,999.0 |
| 512 | projection | 192 | 4,280,384 | 4,288,832 | 3,997.0 |
| 512 | full | 16 | 4,308,224 | 4,575,984 | 3,719.8 |
| 512 | full | 64 | 4,312,256 | 4,577,136 | 3,674.7 |
| 512 | full | 192 | 4,323,008 | 4,580,208 | 3,527.2 |
| 1,024 | projection | 16 | 8,551,424 | 8,571,904 | 2,125.8 |
| 1,024 | projection | 64 | 8,551,424 | 8,571,904 | 2,138.2 |
| 1,024 | projection | 192 | 8,551,424 | 8,571,904 | 2,115.3 |
| 1,024 | full | 16 | 8,603,840 | 9,145,840 | 2,031.2 |
| 1,024 | full | 64 | 8,607,872 | 9,146,992 | 1,988.1 |
| 1,024 | full | 192 | 8,618,624 | 9,150,064 | 1,884.1 |

The read-free artifact grows almost exactly linearly with width:
`2,145,024 -> 4,280,384 -> 8,551,424` bytes, ratios **1.995** and
**1.998**. The full path adds 14.4 KiB, 78.7 KiB and 67.2 KiB at contexts
16, 64 and 192. Single-pass serving throughput approximately halves when width
doubles; the 16-to-192 context change costs 5%, 5% and 7% in the full arm.

- 18 cells, 54 complete audit repeats, 4,896 token decisions, 2,924,544 Q12
  query-coordinate comparisons and 39,168 kept-cell comparisons;
- zero query-coordinate, token, kept-cell or decision-hash mismatch;
- every cell's three decision hashes are present and equal;
- all four negative controls refused: truncated artifact, out-of-vocabulary
  token, sequence over context and width above the 1,024 writer cap;
- process wall **9.196 s**, peak RSS **78,980 KiB**;
- AMD Ryzen 9 9950X, 16 cores / 32 logical CPUs.

Compact evidence:
[scale-serving-summary.json](evidence/scale-serving-summary.json), bound by
[PUBLISHED-SHA256SUMS](evidence/PUBLISHED-SHA256SUMS).

## Checks run at the head

- Fresh round-3 reviewer: **TEST FITNESS: FIT** and **D67 PRE-RUN: GO** at
  `2b077afbd`;
- focused `cpu-scale-serving-bench` release tests: **3 passed**;
- full `uor-r4-cpu-train` release suite: **351 passed**, 4 expected ignores,
  all binary tests passed;
- strict scoped Clippy:
  `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`;
- `cargo fmt --check`, `git diff --check`, D23/D25/D32 ratchets and claim
  wording passed;
- `uor-harness check --only prep.sheet_shape`, `prep-check` and
  `loop pr-check --local` passed;
- source/binary pins and absent output root were checked before the run;
- the registered output passed its schema, cell-count, repeat-hash, parity,
  control and RSS predicates after read-back;
- registered wall time and RSS remained below the 20-minute and 2-GiB stops.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
evidence and loop-delivery procedures were used.

## Still open

- Claude's scale-probe still has to establish quality growth on real text and
  GPU training-memory feasibility at these widths.
- This preflight covers the exact output path, not real trained artifacts. The
  same audit should consume Claude's exports when they exist.
- Exact-head delivery review, protected merge, fresh-main verification and
  claim release remain.

## Plain summary

All three scale-probe widths export and serve exactly at every registered
context on this CPU path. Artifact bytes scale linearly, while single-thread
decode throughput approximately halves each time width doubles and changes
only modestly with context. The result removes output-path uncertainty from the
upcoming scale run but does not answer whether width improves quality or fits
on the GPU.
