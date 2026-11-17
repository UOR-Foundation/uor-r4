# ORDER — written-state-count-model (claude-423)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-423, the counter's phase profile. It follows claude-422, which sealed R\*.

## Why

R\*'s sealed run took 2,045 s. 90% of that was counting TRAIN on the GPU, and scoring took about
2 s. Wall time per order is all that exists. Three things are suspected and none is measured:
- Order 10 emits 1.29 B grams and the filter keeps 0.27%.
- `count_order_adjusted_with` counts order k+1 again, over order k's ranges, to get its
  extension runs. So most orders are counted twice.
- The host sink hashes every gram.

Before anything is redesigned (D: profile before long runs, 17 Nov), the time per phase is
measured.

## What it does

1. **Phase timers in `NgramCounter`.** These are integer nanosecond totals in atomics, read with
   `take_phases`:
   - histogram and plan;
   - select;
   - radix sort;
   - boundaries, emit and scans;
   - device-to-host copies;
   - host record building.

   The adjusted counter also times its own count and its order-k+1 extension count apart, and
   the runs and adjust steps. A stream synchronize closes each GPU phase so that its time is its
   own. Outputs are unchanged.
2. **`kn-reference` prints one phase line per order** to stderr, with the sink (assembler push
   and filter keep) timed apart. The JSON is unchanged, so a run's JSON stays `cmp`-stable.
3. **Profile on wick.** DEV-tune at N = 4 in full, then N = 10 on a slice. This is an
   engineering timing on DEV-tune only. DEV-test is not read.

## Gates

- G1, laptop: stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
  instrument build are clean, and the host `kn_` and `ngram_count` tests pass.
- G2, wick: the CUDA tests `ngram_count_cuda kn_score_cuda kn_filter` pass at the head.
- G3, wick: DEV-tune at N = 10, full, gives a JSON `cmp`-identical to claude-422's
  `run1/sweep-10.json` but for `masses_at_floor`. So the timers change no output. Its phase
  lines are the profile.
- G4: the REPORT names the largest phases and the saving each candidate fix could reach,
  bounded by the measured times. No fix is built in this unit.

## Out of scope

No change to what is counted. The pre-sort filter, the shared k / k+1 counting and an index of
TRAIN are later units, chosen from this profile.
