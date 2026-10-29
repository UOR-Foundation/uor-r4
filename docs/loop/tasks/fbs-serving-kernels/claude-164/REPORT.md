# REPORT: fbs-serving-kernels, claude-164 (an exact sign-compatible icosian snap)

Lab: claude. Order: [ORDER.md](ORDER.md). The item stays `ready`. This unit gives a faster exact kernel for the largest part. Done-when 2's vector kernel, and its switch, stay open.

**Speed.** On this M1, the served FBS model now decodes at **749–752 tok/s**, against **270** with claude-163's binary in the same session: **2.8×**. That is 1.33 ms a token instead of 3.70 ms, on claude-126's seed-1 export at a 192-token context. Every reply is byte-identical.

## What did not work, first

- **The first build failed the serving audit.** With 120-byte table rows, the audit's call-graph check failed in `stack_snap_select` on one `umaddl`, a multiply instruction, which computes the row offset `pattern × 120`. Rows of 128 bytes make the offset a shift, and the audit then passed with 0 forbidden instructions.
  - That failing audit's log was printed in this session but not kept. The sealed log is the passing one.
  - A table indexed in an audited kernel needs a power-of-two row stride. That is a candidate convention, noted in FRICTION.
- **The candidate scan is still about 29% of the step.** The step is 1.33 ms, of which the core is 54.3% and the product 36.0%. Together the core and the product take a third of the time they did (3.57 ms → 1.20 ms).
  - A two-point estimate splits each part's time linearly in the number of candidates scored. It is an estimate from two runs (120 and 17 candidates), not a measurement.
  - On it, the candidate scan was about 2.75 ms of the old 3.70 ms step, and is about 0.39 ms of the new 1.33 ms.
  - The rest of the core and the product, about 0.81 ms, is the per-snap fixed cost, the Hamilton products and the lanes.
- **This unit does not meet done-when 2 as worded.** The done-when names a vector kernel in `stack/fbs_simd.rs` behind a runtime switch. This change is exact scalar code that runs on every target, so it needs no switch. Its "without" arm is claude-163's sealed binary, run alternately in the same session.
- **The item still has no GitHub issue** (noted in claude-163's REPORT and in FRICTION). The claim is noted on #2028.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:**
  - **`stack_snap_select`** now reads the query's sign pattern (two bits a coordinate) and scores only the roots listed for that pattern, in index order. It uses the same `small_mul` dot product and the same strict `snap_difference_positive` comparison as before. The first candidate starts as the best, as root 0 did in the full scan.
  - **`SNAP_CANDIDATES`** is a `static` built at compile time by `snap_candidates()`. For each of the 256 two-bit patterns (81 occur), it lists the roots whose nonzero coordinates agree in sign with the query's nonzero ones, in rows of 128 bytes. A zero query coordinate leaves that root sign free.
  - **`root_coordinate_sign`** gives a root coordinate's sign: `b`'s when `b ≠ 0`, else `a`'s. That holds for every coordinate of the table: 0, ±1, ±½, ±φ/2 and ±(φ−1)/2.
- **Why it is exact.** The 120 unit icosians are closed under the sign flip of any coordinate. Take a root whose nonzero coordinate `j` opposes the query's nonzero `r_j`. Its flip is another root, scoring higher by `2|r_j ρ_j| > 0`, so the first root is never a maximizer.
  - The compatible roots therefore hold every maximizer, and the max over them is the global max.
  - Scanning them in index order with a strict comparison returns the lowest-index maximizer, as the full scan does.
- **Four new tests in `kernels.rs`:**
  - `the_icosian_roots_are_closed_under_coordinate_sign_flips`: the argument's premise, over the table itself.
  - `a_root_coordinate_sign_is_its_exact_sign`: the sign rule against `snap_difference_positive`.
  - `each_sign_pattern_lists_its_compatible_roots_in_index_order`: every pattern's list, 17 for each of the 16 patterns without a zero, and 120 for the zero query.
  - `the_sign_compatible_snap_is_the_full_scan`, against the old scan, kept as the test's `snap_select_full_scan`:
    - all 28,561 queries in `[-6, 6]^4`;
    - 200,000 pseudo-random queries at magnitudes from 1 up to the full `i32` range, with coordinates zeroed at random;
    - 7,260 queries on each root and halfway between every pair, at scale `2^20`.

  With the rule broken (roots with a zero coordinate dropped for a positive query), the last test fails (`0 passed; 1 failed`). It passes once restored. The existing exact-oracle test, `snap_selection_matches_an_independent_exact_oracle`, still passes.
- **`docs/research/notebook/m4-serving.md`:** one row.
- **`docs/roadmap/items/fbs-serving-kernels.md`:** the Progress section, with this result and what is next.

## Measurements

**The A/B.** One artifact: claude-126's seed-1 export, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`. The machine is this M1 on AC power. Each run is `fbs-toks --parts`: plain, staged and parted protocols, each 1 warm-up and 20 timed runs of a 128-token prefill and 64 decode steps.

- **Old:** claude-163's binary, sha256 `9c1f1778602c6eb9f593119656891db35b5c0e7d7b574f39bc9578164fac6f9a`, code `c9c5fd238`.
- **New:** this unit's binary at `b55fd9461`, sha256 `b0ed9537917f00fc41a89f1b44166c44b49da9533657dbc7be0400f5f5848270`.
- **Order:** old, new, old, new.

| Run | Decode tok/s | Step (ns) | Recurrence core, 6 layers (ns) | Gated product, 6 layers (ns) | Blocks within 10% |
|---|---|---|---|---|---|
| old 1 | 270 | 3,705,458 | 2,090,417 | 1,476,874 | yes |
| new 1 | **749** | 1,328,312 | 721,894 | 478,020 | yes |
| old 2 | 270 | 3,696,437 | 2,084,853 | 1,471,915 | yes |
| new 2 | **752** | 1,326,917 | 720,979 | 477,875 | yes |

- **The core:** 2.9× faster, 118–123 µs a layer in new 2.
- **The product:** 3.1× faster, 77–81 µs a layer.
- **The two norms** are unchanged: each takes 37.4–37.5 µs over six layers, so each one's share rose from 1.0% to 2.8%.
- **Each block's parts** lie at 997–1,003 per mille of the block in both new runs.

**Instrument checks:**

| Check | Result |
|---|---|
| The artifact's sha256 against claude-158's and claude-163's | match |
| Replies of all four runs, and claude-163's sealed reply | one hash: SHA-256 `97c8ad41e1cb5cee…` of `jq -r '.timing.reply\|tostring'` |
| `replies_match` (staged and parted against plain), every run | true |
| Old binary's decode against claude-163's 272 | 270, 270 (−0.7%) |
| Serving audit, `python3 scripts/audit_zero_matmul_serving.py <release uor-r4-stack> --stack` (sha256 `162ad79c…3fef`) | FULL PASS: 0 multiplier, divider or float instructions over 44 symbol ranges and 93 reachable functions |

**Sealed:** `caseyallard/uor-r4-store:results/claude/fbs-serving-kernels-snap-2026-10-28/` holds `old1.json`, `new1.json`, `old2.json`, `new2.json`, `audit-uor-r4-stack.txt`, `binaries.sha256`, `COMMAND.txt` and `SHA256SUMS`, read back with `shasum -c` OK.

## Checks run at the head

- `cargo test -p uor-r4-integer --features serving-wasm --lib --no-fail-fast`: `334 passed; 0 failed`.
- `cargo fmt -p uor-r4-integer -- --check`: clean.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets`: no finding in the changed lines. The crate's existing findings elsewhere are unchanged.
- The serving audit, above: FULL PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --full`: in the review record.

## Skills used

- **measure-before-optimizing.** The target came from claude-163's split, and the A/B ran old and new alternately in one session on one artifact.
- **writing-meaningful-tests.** The exactness test is shown to fail when the compatibility rule is broken, and the argument's premise (closure under sign flips) is itself a test.

## Still open

- **Fewer candidates.** Of the 17, the 12 golden roots can be cut further by the query's magnitude order. That must stay exact, lowest-index tie included.
- **Done-when 2's vector kernel** for what is left of the core and product: the Hamilton products, the lanes and the per-snap fixed cost. Done-when 3's speed is measured against this unit's 749 tok/s.

## Plain summary

The model on the laptop now writes about 750 words a second, up from 270, with exactly the same output. Most of the time went to snapping directions to the nearest of 120 fixed points, checking all 120 each time. Because the 120 points are symmetric under sign changes, only 17 of them can ever be the nearest for a given direction, so the model now checks just those.
