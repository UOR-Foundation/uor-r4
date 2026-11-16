# REPORT: written-state-count-model, claude-412 (an exact, deterministic CUDA n-gram counter over u16 token streams)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The context record size in the ORDER was wrong** [claude]. The ORDER said 36 B. It is 40 B: a u128 key, a u64 total, then N1+, N1, N2 and N3+ as u32.
- **The first two wick builds failed** [claude]. `wick.sh sync` copies no `docs/` or `archive/` files, but the discrete crate's lib and tests `include_str!` 71 of them. After those files were shipped, the third build passed. This was a fault in the build instrument, not in the counter.
- **The timings are end to end** [claude]. They include the device-to-host copy and the host SHA-256, not only kernel time. The 4090 was shared with another job running at 94–97% utilization. These are readings, not a qualification.
- **Peak device memory was not isolated on the shared card** [claude]. The nvidia-smi peak minus baseline was 1.6–2.2 GiB, which includes the CUDA context and the other job's drift. The peak below is layout arithmetic.
- **The laptop gate ran on wick's host build** [claude]. Partway through the unit the laptop disk filled, for reasons outside it: swap reached 16 GB, held by another program, and has since been relieved. The change adds a cuda-gated test only.
- **The first `loop pr-check --local` failed `banned-presence`** [claude]. Its D32 lexer strips comments and then counts `.digit` after a non-word byte as a float literal. Two tuple indices after a call, `….expect("oracle").0` and `assembler.finish().0`, matched that rule. They are now destructured with `let (x, ..) = …`, and the tests still pass 5 / 5. A first guess, that the cause was `§3.2` and `1e7` in comments, was wrong, but those comments were reworded too.
- **The counter counts across document boundaries** [claude]. R\* needs them respected; that is still open.

## What changed

- `crates/uor-r4-discrete/src/ngram_count.rs` (host) [claude]:
  - `PrimeRouteKey`: the ordered positional key, with slot 1 (the latest token) in the highest bits and the order tag in the top byte. Back-off drops the oldest slot, so contexts that share a suffix sort side by side.
  - The canonical records `NgramTriple` (22 B: key u128 LE, next u16, count u32) and `NgramContext` (40 B).
  - `NgramAssembler`, which produces the per-order SHA-256 of the triple and context streams, and n1..n4.
  - `plan_partitions` and `read_uort_tokens`.
- `ngram_count.cu` [claude]: 16 integer kernels in their own nvcc PTX/cubin module, admitted in `build.rs`. Nothing was added to `discrete.cu`, and there is no NVRTC.
  - The planning histogram uses shared-memory integer `atomicAdd`, so its sum cannot depend on order.
  - The sort is a custom stable LSD radix sort on 6-bit digits, ranked with `__match_any_sync`, followed by a run-length reduce.
- `ngram_count_device.rs` [claude]:
  - `NgramCounter::new`, `plan`, `count_partition` and `count_order`.
  - The whole u16 stream stays resident.
  - Out of core, the counter uses key-range partitions on the most significant sort token. Partitions are joined by concatenation, and only order 0's empty context is summed across them.
- `ngram_count_oracle.rs`: the test-only ordered-map oracle. `ngram_count_cuda.rs`: the CUDA tests, including the ignored real-stream test `ngram_count_cuda_real_stream_matches_the_oracle` (it reads `UOR_NGRAM_STREAM`, runs orders 0..5, and requires two equal GPU runs). The bin is `ngram-count-probe` [claude].
- `docs/research/notebook/m4-serving.md`: one row [claude].
- No driver source changed, so `driver_source_sha256` is unchanged [claude].

## Measurements

On the 15,471,412-token D87 student stream (`c404/stream/tokens.u16`), on a shared RTX 4090, end to end, with one partition [claude]:

| order | grams | time | targets/s |
|---|---|---|---|
| 0 | 3,969 | 21 ms | 737M |
| 1 | 492,324 | 20 ms | 768M |
| 2 | 2,442,377 | 124 ms | 124M |
| 3 | 4,843,692 | 264 ms | 59M |
| 4 | 6,865,301 | 418 ms | 37M |
| 5 | 8,368,363 | 528 ms | 29M |
| 6 | 9,436,821 | 608 ms | 25M |
| 7 | 10,213,917 | 679 ms | 23M |
| 8 | 10,793,117 | 714 ms | 22M |

- With 17 partitions, orders 0..8 take 99, 36, 165, 342, 524, 691, 826, 908 and 999 ms [claude].
- Upload plus histogram takes 113 ms [claude].
- GPU and oracle agree on both digests and on every record for orders 0..5 [claude]. The order-5 triple digest is `31bb8c91bb5bde334f3e889c00a15166d8754adf4c6ba8ca1a5cf068a36c28f9`, and the order-2 digest is `38626d27d545584732e782cda10da60a587dbbc40c5df4a5ae7309badc4138f9`.
- Two probe runs gave byte-identical SHA files for orders 0..8, under both budgets [claude].

## What the numbers say

- **Correctness** [claude]. The counter is exact and replayable on real D87 text. Partitioning changes the time, never the bytes.
- **Full-corpus time** [claude, est]. A linear projection to D87 TRAIN (about 2.09e9 tokens, ×135.1) is an upper bound, because low-order grams grow sublinearly. It gives about **7.6 GPU-minutes** for orders 0..8 in one partition, or about 10.3 at the 17-partition overhead. Per order: 2.8, 2.7, 17, 36, 57, 71, 82, 92 and 97 s.
- **Memory** [claude, est].
  - The resident stream is 4.18 GB.
  - A partition holds at most max(16n + 4g, 8n + 46g) bytes, where n is targets and g is grams. That is at most 54 B per target, or about 40 B at order 8.
  - The copy-out on the host makes the container's 16 GB the tighter limit. So a budget of 100–150M targets is the safe choice, about 14–20 partitions per order.
- **For the plan** [claude]. Counting is not the bottleneck of the written-state track. A full count base for P1 costs minutes, not the GPU-days that scored training took.

## Checks run at the head

- `UOR_NVCC_ARCH=compute_89 cargo +1.97.1 test --release -p uor-r4-discrete --features cuda --lib ngram_count_cuda` on wick: 4 passed, 0 failed, 1 ignored [claude].
- The same command with `UOR_NGRAM_STREAM=…/tokens.u16` and `-- --ignored` on the real-stream test: 1 passed [claude].
- `ngram-count-probe tokens.u16 8 4294967295 1000000`, run twice: per order, both budgets agree, and the SHA files of the two runs are `cmp`-identical [claude].
- `cargo test -p uor-r4-discrete --lib ngram_count`: 5 passed on the laptop (part 1), and 5 passed on wick's host build at this head [claude].
- **Negative gate** [claude]. With the key packed from sorted tokens, `ngram_count_key_preserves_order` fails ("a b c" and "c b a" give the same key). After the restore: 5 / 5.
- The stub-nvcc `cargo check -p uor-r4-discrete --features cuda --all-targets` is clean [claude].
- Fresh review and `loop pr-check --local`: see the PR [claude].

## Skills used

- `native-space-cycle`: one key, one counting operator, and the oracle's SHA as the replay identity [claude].
- `scope-guard`: no pruning, no smoothing and no change to `TrunkTables` in this unit [claude].

## Still open

- For R\* (claude-413) [claude]:
  - continuation counts N1+(•w h);
  - the modified-KN discounts;
  - BPB;
  - document boundaries.
- Count ≥ 2 pruning on the device, and the `TrunkTables` S2 family, for P1 [claude].
- A kernel-only timing that separates host SHA from GPU time [claude].
- `wick.sh sync` should carry the `include_str!` targets under `docs/` and `archive/` (a candidate fix to the kit) [claude].

## Plain summary

The project now has an exact GPU counter for short word sequences [claude]. On 15.5 million tokens of real training text it gives the same counts as a slow, independent CPU count, and it gives them identically every time [claude]. Counting every sequence up to 8 words over the whole 2-billion-token training set should take roughly 8–10 minutes on the 4090. That makes the count base of the new model cheap [claude].
