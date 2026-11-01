# REPORT: fbs-snap-sort, claude-241 (the icosian snap scores 3 or 5 roots, not 17)

Lab: claude. Item: [fbs-snap-sort](../../../../roadmap/items/fbs-snap-sort.md) (#3470). Order: [ORDER.md](ORDER.md). The item's five done-when are met here, and the item is set `done`. Closes #3470.

**Speed.** On claude-126's snapped export, decode rose from 4,915 to 6,480 tok/s on this M1 (+32%), and the step fell from 204.6 to 156.5 µs. These are medians of six alternating runs each. Every reply is byte-identical to the base binary's.

## What did not work, first

- **The snap is still the largest single kernel.**
  - Under the same sampler, its share fell from 283 of 630 main-thread samples (45%) to 165 of 519 (32%).
  - Its own samples fell by 42%. The candidates it scores fell from 17 to 3 or 5, so the cost that is left is mostly not the scan.
  - This unit did not split that cost. Two likely parts:
    - each comparison's exact-sign branches (`snap_difference_positive`), plus the per-query setup;
    - the queries that keep the scan: a zero coordinate, or two equal magnitudes.
  - The served queries' fast-path rate was not counted.
- **Negative gate B did not fail, and cannot.**
  - It swapped the axis root and the half-root in the fast path's order; both sweeps still pass.
  - On the fast path, an axis root tied with the half-root is never the best score (proof below). So that order never decides the result, and no test can pin it.
  - Gates A and C do fail; see Measurements.
- **The first WASM audit read 5 of 7, an instrument fault.**
  - The module was built with `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS`. That variable merges with `.cargo/config.toml`'s `+simd128`, so `step` carried 340 SIMD instructions (`0xFD`), which the audit does not classify.
  - The base reads the same 340 and 334 under that build, so this change adds none.
  - Built as the 29 Oct prep sheet records, with `RUSTFLAGS="-C target-feature=-bulk-memory"`, both read 7 of 7. `RUSTFLAGS` replaces the config's flags, so that build has no SIMD.
- **The first replies on other prompts were empty.** `fbs-toks --prompt` takes exactly 128 tokens, and the short prompts were refused. Three 128-token prompts were then used.
- **`--context-sweep` cannot run on claude-126's export,** since it times the Hamming read and that export has none. The sweep ran on claude-240's s1 artifact (`7c82d76b…`) instead.
- **The item's second candidate was not tried:** one fixed-point product for the exact sign. It goes to Still open.
- **The laptop was not idle.**
  - The load average read 4.0 to 5.0 during the A/B, from desktop apps and macOS's `dasd`. No build or other timing ran.
  - The runs alternated, and the order swapped every round.
  - The two sides do not overlap: base steps span 203.7 to 207.8 µs, new steps 155.8 to 157.0 µs.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - **`GOLDEN_BLOCK_OF`** (line 1314): the golden block of each ordered pair `(p, q)`. It is built at compile time from `GOLDEN_AXES`, and it asserts that the 12 blocks name 12 distinct pairs.
  - **`SNAP_ORDERS`** (line 1368): for each of the 64 codes of the six pairwise comparisons, the fast path's 3 or 5 slots in the 17-root row. A code that is a cycle keeps count 0, and its query keeps the scan.
  - **The fast path in `stack_snap_select`** (line 1501). It applies when a query has 17 compatible roots (no zero coordinate) and four distinct magnitudes. It ranks the axes and reads the slots from `SNAP_ORDERS`. Then it runs the same strict comparison, in index order, over those roots only. Every other query keeps the scan.
  - **Three tests:**
    - `a_row_without_zeros_has_the_fast_paths_layout`;
    - `the_orders_name_their_sorted_block_and_its_neighbouring_swaps`;
    - `the_fast_snap_is_the_full_scan_on_every_query_from_minus_8_to_8`.
- **`crates/uor-r4-integer/src/stack/tests.rs`:**
  - The independent oracle becomes a function, `snap_oracle` (line 442).
  - A new test holds `stack_snap_select` equal to it on all 83,521 queries in `−8..=8` (line 489).
- **The docs:**
  - `docs/roadmap/items/fbs-snap-sort.md`: `status: done`, with a done note. The roadmap is re-rendered.
  - One notebook row in `m4-serving.md`.
  - A line in the pipeline map's gap 3.

## Why it is exact

Write `s = |raw|` and rank the axes so that `s_r0 > s_r1 > s_r2 > s_r3`. Against a query with no zero coordinate, every compatible root scores `A + Bφ`, where `A` and `B` are sums of the `s`. There are three families:
- **an axis root:** `(2 s_i, 0)`;
- **the half-root:** `(Σ s, 0)`;
- **a golden root of block `[p, q, t]`:** `(s_q − s_t, s_p + s_t)`, that is `φ s_p + s_q + (φ − 1) s_t`.

**The golden roots.** A golden root scores `c · s_π` with `c = (φ, 1, φ − 1, 0)`, for its block's even permutation `π` of the axes.
- **The coefficients are strictly decreasing,** with steps `d = (φ − 1, 2 − φ, φ − 1)`, all positive.
- **Abel summation** gives `loss(π) = Σ_k d_k (S_k − S_k(π))`. Here `S_k` is the sum of the `k + 1` largest `s`, and `S_k(π)` is the sum over `π`'s first `k + 1` places.
- **Each term is at least zero,** and it is at least `d_k (s_rk − s_r(k+1)) > 0` wherever `π` changes the prefix set `k`.
- **So the sorted order is the unique best of all 24.**
  - If that order is even, its block wins the family. An even permutation of 4 axes is fixed by its first two entries, so that block is `GOLDEN_BLOCK_OF[r0][r1]`.
  - If it is odd, the even orders that change exactly one prefix set are the three swaps of neighbouring ranks. Any other order changes two sets or more, and loses at least as much as one of those swaps.
  - So the family's winners are among the three swaps. Ties among them are real (`s_r0 − s_r1 = s_r2 − s_r3`), and the lowest index must win. The candidates are kept in ascending index order for that.

**Across the families.**
- **A golden root never ties an axis root or the half-root:** its `B = s_p + s_t > 0`, theirs is 0, and `φ` is irrational.
- **The axis root of `r0` beats every other axis root.**
- **An axis root tied with the half-root is never the best on the fast path.** If `2 s_r0 = Σ s`, block `(r0, r1)` beats both:
  - by `(φ − 1) s_r1 + (2φ − 3) s_r2 + (φ − 2) s_r3 > 0` when its third axis is `r2`;
  - by `(φ − 1) s_r1 + (φ − 2) s_r2 + (2φ − 3) s_r3 > 0` when it is `r3`.

  That is why negative gate B passes. Both bounds use `s_r1 > s_r2 > s_r3 > 0` and `φ − 1 > |φ − 2|`.

**So the best root is among:**
- the axis root of `r0`;
- the half-root;
- one golden root, or three when the sorted order is odd.

The same strict loop then keeps the lowest index on an exact tie, as the scan does.

## Measurements

**Exactness.**
- `the_fast_snap_is_the_full_scan_on_every_query_from_minus_8_to_8`:
  - it equals `snap_select_full_scan` on all 83,521 queries;
  - the fast path takes 26,880 of them (8 · 7 · 6 · 5 magnitudes, times 16 sign patterns), and some of those are exact ties;
  - it also holds on 200,000 drawn queries of up to 31 bits, built so that `s_r0 − s_r1 = s_r2 − s_r3` (a tie between neighbouring swaps).
- `snap_selection_matches_the_oracle_on_every_query_from_minus_8_to_8` holds the independent oracle equal on the same 83,521.
- The existing snap tests pass unchanged, `snap_selection_matches_an_independent_exact_oracle` included.
- The layout test checks that all 16 patterns without a zero have 17 roots in the fast path's layout, and that every pattern with a zero has more than 17.

**Negative gates,** each a one-line change in `snap_orders()`. Each was restored, and both builds recompiled:

| Gate | Change | Result |
|---|---|---|
| A, a wrong tie rule | golden candidates in descending index order | **fails** both sweeps at `[-8, -7, -2, -3]`, a tie of two neighbouring swaps (8 − 7 = 3 − 2): root 80 against 56 |
| B | the half-root before the axis root | passes, and must (proof above) |
| C | the sorted block only, without the neighbouring swaps (`n = 1`) | **fails** both sweeps at `[-8, -7, -2, -6]`: root 8 against 88 |

After the gates, all 146 stack tests passed on the restored code.

**Replies,** base against new (sha256 of `[prompt, reply]`):

| Artifact | Prompt | Distinct reply tokens | Base | New |
|---|---|---|---|---|
| claude-126 seed 1 (`314c3909…`) | default (`1 + 11i`) | 21 of 65 | `7d0c4890…` | identical |
| claude-126 seed 1 | `(37i + 5) mod 1400` | 13 of 65 | `392e89bb…` | identical |
| claude-126 seed 1 | `(13i² + 101i + 17) mod 1399` | 33 of 65 | `73833c92…` | identical |
| claude-126 seed 1 | `50 (i mod 13) + 3` | 27 of 65 | `2b4d7de1…` | identical |

Every `--parts` run also reports `replies_match: true`, and the four sweeps' replies agree.

**The serving audits:**

| Build | Base | New |
|---|---|---|
| Native `audit_zero_matmul_serving.py --stack`, release `uor-r4-stack` | FULL PASS: 44 ranges, 125 call-graph functions; 47 / 1 multiplies / divides in the ranges, 115 / 14 in the call graph | the same, FULL PASS |
| WASM `audit_zero_matmul_wasm.py --serving --call-graph`, `RUSTFLAGS="-C target-feature=-bulk-memory"` | 7 of 7; `step` 29 / 14 multiplies / divides, 0 float, in 39,966 instructions | 7 of 7; `step` 29 / 14, 0 float, in 40,194 instructions |
| WASM, the config's `+simd128` kept (instrument check) | 5 of 7: `step` 340 SIMD | 5 of 7: `step` 340 SIMD |

There is no vector form and no `fmov` in the native build.

**Speed, `fbs-toks --parts`** on claude-126's seed-1 export. Six rounds alternated base and new, swapping the order each round, on this M1 on AC power. No other build or timing ran.

| | Base | New |
|---|---|---|
| Decode, median of six (tok/s) | 4,915 | 6,480 |
| Decode range | 4,645 – 4,958 | 6,282 – 6,495 |
| Prefill, 128 tokens (tok/s) | 4,833 | 6,438 |
| Step (µs) | 204.6 | 156.5 |
| Six recurrence cores (µs) | 91.4 | 63.3 |
| Six gated products (µs) | 54.3 | 34.2 |

The two parts that run the snap account for the whole saving: 48.3 µs, against 48.1 µs in the step. Every other part is unchanged.

**Decode against context, `fbs-toks --context-sweep`** on claude-240's s1 artifact. Four rounds alternated; the table shows medians in tok/s:

| Context | Base, with the read | New, with the read | Base, without | New, without |
|---|---|---|---|---|
| 96 | 4,083 | 4,997 | 4,854 | 6,242 |
| 128 | 3,978 | 4,898 | 4,861 | 6,312 |
| 160 | 3,967 | 4,829 | 4,877 | 6,314 |
| 192 | 3,884 | 4,778 | 4,839 | 6,272 |

The read now costs 20 to 24% of decode, against 16 to 20% at base, since the rest got cheaper.

**The snap's share,** from `/usr/bin/sample` at 1 ms on a plain run of each binary, as in the cancel probe. These are the main thread's top-of-stack samples:

| | Base | New |
|---|---|---|
| `stack_snap_select` | 283 of 630 (45%) | 165 of 519 (32%) |
| `stack_native_writer` | 60 | 71 |
| `stack_snap_rotation` | 47 | 58 |
| `stack_recurrence_core` | 49 | 37 |

**Binaries,** all built at this unit's code (`6e4b25c79`) or its base (`4367a7fbd`):
- `fbs-toks`: base `ca955e9f…c679`, new `edb2c86e…1de9`;
- `uor-r4-stack`: base `cca1aaab…fa17`, new `033be732…7e2b`;
- the WASM module: base `3fcd181a…8622`, new `a53af4e2…5284`.

`binaries.sha256` in the store has all of them in full.

**Sealed** to `caseyallard/uor-r4-store:results/claude/fbs-snap-sort-2026-11-01/`. It holds 52 files: the replies, the A/B outputs and samples, both audits on both builds, the gate logs, the probe and the patch. The `SHA256SUMS` sha256 is `9fb5b81560981078a644a7ab7fff55e90c4e893e515c03843676b039769f78f8`. On read-back, every file verifies.

**Classification.** This is an exact kernel change measured with qualified instruments, and no reply changes. It is a positive result for the snap's candidate count. The snap's remaining cost is an open reading, not a negative.

## Checks run at the head

The code is unchanged since `6e4b25c79`, the commit every binary above was built at. The later commits are docs only.
- **`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast`:** 612 passed, 35 failed, 4 ignored.
  - All 35 panic on a missing local input, the same count claude-182 and claude-186 recorded: 34 on a bundle or tokenizer file not found, and one on `assert!(tok_path.exists())`.
  - The new tests and every snap test pass.
- **`cargo fmt -p uor-r4-integer -- --check`:** clean.
- **`uor-harness loop gates --full`:** 11 of 11 green.
- **`uor-harness loop report --check` and `loop pr-check --local`:** PASS. The pr-check warnings are the ORDER's size, and FILEMAP rows for folders this unit does not touch.
- **The fresh review:** see below.

## Skills used

- **holding-the-goal.** Each of the item's five bars was checked before the item was set `done`:
  - exactness, with two failing negative gates and a proof for the one that cannot fail;
  - replies, on prompts whose replies vary;
  - both audits, with the base's own readings beside them;
  - speed, alternating against the base binary.
- **The snap's remaining 32%** is reported as open, not as part of the gain.

## Still open

- **The snap's remaining 32%.** A next unit should split it before it changes anything:
  - count how many served queries take the fast path;
  - time one snap's setup against its comparisons.
- **The item's second candidate.** One fixed-point product would give the exact sign of `da + db·φ` without branches. It would cut the cost of every remaining comparison.
- **The WASM build the config gives fails the audit on SIMD, on `main` too.**
  - `.cargo/config.toml` turns on `+simd128` for wasm32. The audit refuses `0xFD`.
  - The recorded audit builds pass because their `RUSTFLAGS` drops the config's flags.
  - Either the audit should classify SIMD integer forms, or the record should say the audit is of the no-SIMD build.
- **`fbs-toks` records no binary hash in its output** (claude-186's review). Base and new are told apart here by folder and step median.
- **Still owed by this lab, from earlier units:**
  - `integer-hamming-read`'s long-context cost (its done-when 3);
  - the one-model `I-D` and `I-G` unit;
  - claude-240's three code minors.

## Plain summary

When the model rotates a vector, it snaps the rotation to the nearest of 120 fixed directions. That snap used to try 17 of them every time, and it was nearly half the cost of producing a word.

The snap now sorts the vector's four sizes first. A short proof shows that the winner is always among 3 or 5 of the 17. The answer is the same as before on every one of 83,521 test inputs, and on every tie. The replies are the same, word for word.

On this laptop the model now produces about 6,500 words a second, up from about 4,900. The snap is still the largest single cost, at about a third of the time, so there is more to find.
