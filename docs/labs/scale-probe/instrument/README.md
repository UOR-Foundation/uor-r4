# scale-probe instrument

The laptop side of P5 scale-probe: counting only, integer arithmetic only. Every inconsistency
is a nonzero exit that names the file (or split and row) and the reason. Outputs are never
overwritten. A position is scored iff `p >= 1 && tokens[p] != 0 && mask[p] == 1`.

Build from the repository root:
`~/.cargo/bin/cargo build --offline --manifest-path docs/labs/scale-probe/instrument/Cargo.toml`.

## `mix`

Builds one nested training mix. Per root, takes the shortest document-boundary prefix `[0, L)`
with `L * D >= total * N`, so each root's 1/4 is a prefix of its 1/2, which is a prefix of the
whole. Prefixes are concatenated in the given root order; `out/manifest.json`
(`uor-r4-scale-probe-mix/v1`) records each root's provenance, counts taken and the written shas.

```
scale-probe mix out=local/claude/mix-q1 fraction=1/4 \
  root=oasst2:local/claude/rtc/oasst2/train:<tokens_sha256>:<mask_sha256> \
  root=wiki-00000:local/claude/rtc/wiki-00000/train:<tokens_sha256>:<mask_sha256>
```

## `grade`

Scores one checkpoint's rows files (`{split, memory, positions, targets, nearest, writer}`)
against TRAIN count tables: the unigram top-1 and T2, the previous-token bigram top-1 (ties to
the lower id; an unseen key falls back to the unigram top-1). Writes `out/grade.json`
(`uor-r4-scale-probe-grade/v1`) and `out/nearest-hits.bin` (one bit per row, LSB-first, splits
in argument order). A split's `row_identity_sha256` is SHA-256 over its `(position u64 LE,
target u32 LE)` pairs; the pooled one is SHA-256 over, per split in order, the name length
(u64 LE), the name bytes and the split's 32-byte identity.

```
scale-probe grade out=local/claude/g-w256-q1-s1 train=local/claude/mix-q1:<tok>:<mask> \
  panel=oasst2:local/claude/rtc/oasst2/heldout:<tok>:<mask> rows=oasst2:rows-oasst2.json
```

## `pair`

Compares cell A (seeds 1, 2) with cell B, or with a count table, row by row:
`band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d(s) − min_s d(s))`; a seed is `ahead` if
`d(s) > band(s)`, `behind` if `d(s) < −band(s)`, else `inside`; the verdict is the seeds'
common position, or `mixed`. Inputs must share every row identity; a cell's seeds share TRAIN.

```
scale-probe pair out=local/claude/w512-vs-w256.json a1=g-w512-s1 a2=g-w512-s2 \
  b1=g-w256-s1 b2=g-w256-s2
scale-probe pair out=local/claude/w512-vs-t2.json a1=g-w512-s1 a2=g-w512-s2 table=t2
```
