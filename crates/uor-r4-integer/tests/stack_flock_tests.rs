//! Tests for allocation-free D11 integer flock selector.

use uor_r4_integer::stack::flock::*;

/// Independent reference implementation using full O(n log n) sorting.
fn reference_flock_select(
    scores: &[i64],
    query: usize,
    select: FlockSelect,
) -> (Vec<FlockEntry>, FlockScan) {
    let window_start = (query + 1).saturating_sub(select.window);
    let mut slots: Vec<Option<FlockSlot>> = vec![None; query + 1];
    for slot in slots[window_start..=query].iter_mut() {
        *slot = Some(FlockSlot::Window);
    }
    slots[select.sink] = Some(FlockSlot::Sink);

    let mut rest: Vec<usize> = (0..window_start)
        .filter(|&pos| slots[pos].is_none())
        .collect();

    rest.sort_by(|&a, &b| scores[b].cmp(&scores[a]).then_with(|| a.cmp(&b)));

    let candidates_scanned = rest.len();
    let short_prefix = (query + 1) <= select.window;
    let top_k_short = candidates_scanned < select.k;
    let cutoff_ties =
        candidates_scanned > select.k && scores[rest[select.k - 1]] == scores[rest[select.k]];

    for &pos in rest.iter().take(select.k) {
        slots[pos] = Some(FlockSlot::TopK);
    }

    let mut entries: Vec<FlockEntry> = slots
        .iter()
        .enumerate()
        .filter_map(|(pos, slot_opt)| {
            slot_opt.map(|slot| FlockEntry {
                position: pos,
                slot,
            })
        })
        .collect();

    entries.sort_by(|a, b| {
        scores[b.position]
            .cmp(&scores[a.position])
            .then_with(|| a.position.cmp(&b.position))
    });

    let mut sink_kept = 0;
    let mut window_kept = 0;
    let mut top_k_kept = 0;
    for entry in &entries {
        match entry.slot {
            FlockSlot::Sink => sink_kept += 1,
            FlockSlot::Window => window_kept += 1,
            FlockSlot::TopK => top_k_kept += 1,
        }
    }

    (
        entries,
        FlockScan {
            visible: query + 1,
            sink_kept,
            window_kept,
            top_k_kept,
            candidates_scanned,
            short_prefix,
            top_k_short,
            cutoff_ties,
        },
    )
}

#[test]
fn test_flock_select_integer_parity_with_reference() {
    let mut scratch = FlockScratch::new(1024);

    // Test across multiple query positions and configurations
    for query in [10, 63, 64, 127, 255, 511] {
        for k in [1, 7, 16, 32] {
            let select = FlockSelect::new(0, 64, k);

            // Synthetic score generator with pseudo-random pattern and ties
            let mut scores = Vec::with_capacity(query + 1);
            for i in 0..=query {
                // Score with deterministic variety and periodic ties
                let val = ((i as i64 * 37 + 13) % 100) - 50;
                scores.push(val);
            }

            let scan = flock_select_integer(&scores, query, select, &mut scratch).unwrap();
            let (ref_entries, ref_scan) = reference_flock_select(&scores, query, select);

            assert_eq!(scan, ref_scan, "FlockScan mismatch at query={query}, k={k}");
            assert_eq!(
                scratch.entries, ref_entries,
                "Entries mismatch at query={query}, k={k}"
            );
        }
    }
}

#[test]
fn test_flock_scratch_allocation_free() {
    let max_context = 512;
    let mut scratch = FlockScratch::new(max_context);

    let initial_slots_cap = scratch.slots.capacity();
    let initial_rest_cap = scratch.rest.capacity();

    let scores: Vec<i64> = (0..max_context).map(|i| (i as i64 * 17) % 200).collect();

    // Step incrementally as in autoregressive inference
    for query in 1..max_context {
        let select = FlockSelect::b0(7);
        let scan = flock_select_integer(&scores, query, select, &mut scratch).unwrap();
        assert_eq!(scan.visible, query + 1);

        // Capacities must never grow during inference
        assert_eq!(
            scratch.slots.capacity(),
            initial_slots_cap,
            "Slots reallocated at query {query}"
        );
        assert_eq!(
            scratch.rest.capacity(),
            initial_rest_cap,
            "Rest buffer reallocated at query {query}"
        );
    }
}

#[test]
fn test_flock_sink_deduplication() {
    let mut scratch = FlockScratch::new(128);
    let scores: Vec<i64> = (0..100).map(|i| i as i64).collect();

    // Query = 30, window = 64: window reaches from 0 to 30. Sink is 0.
    // Sink must override Window slot, no duplicate position 0.
    let select = FlockSelect::new(0, 64, 7);
    let scan = flock_select_integer(&scores, 30, select, &mut scratch).unwrap();

    assert!(scan.short_prefix);
    assert_eq!(scan.sink_kept, 1);
    assert_eq!(scan.window_kept, 30);
    assert_eq!(scan.top_k_kept, 0);
    assert_eq!(scratch.entries.len(), 31);

    let pos_zero_count = scratch.entries.iter().filter(|e| e.position == 0).count();
    assert_eq!(pos_zero_count, 1, "Position 0 must not be duplicated");

    let sink_entry = scratch.entries.iter().find(|e| e.position == 0).unwrap();
    assert_eq!(sink_entry.slot, FlockSlot::Sink);
}

#[test]
fn test_flock_lowest_position_tie_breaking() {
    let mut scratch = FlockScratch::new(256);

    // Query 100, window 20: past rest is 0..81.
    // Sink is 0, so rest candidates are 1..81 (80 candidates).
    // Set all rest scores to equal value 42.
    let mut scores = vec![0i64; 101];
    for s in &mut scores[1..81] {
        *s = 42;
    }

    let select = FlockSelect::new(0, 20, 5);
    let scan = flock_select_integer(&scores, 100, select, &mut scratch).unwrap();

    assert!(scan.cutoff_ties, "Cutoff must register tie");
    assert_eq!(scan.top_k_kept, 5);

    // Filter to top-k entries
    let top_k_positions: Vec<usize> = scratch
        .entries
        .iter()
        .filter(|e| e.slot == FlockSlot::TopK)
        .map(|e| e.position)
        .collect();

    // Must be the 5 lowest available positions: 1, 2, 3, 4, 5
    assert_eq!(top_k_positions, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_top_k_select_integer_pointer() {
    let mut scratch = FlockScratch::new(64);
    let mut scores = vec![10i64; 20];
    scores[7] = 999; // Maximum score at position 7

    let scan = top_k_select_integer(&scores, 19, 1, &mut scratch).unwrap();
    assert_eq!(scan.top_k_kept, 1);
    assert_eq!(scratch.entries.len(), 1);
    assert_eq!(scratch.entries[0].position, 7);
    assert_eq!(scratch.entries[0].slot, FlockSlot::TopK);
}

#[test]
fn test_rank_table_q31_monotonicity_and_sum() {
    let mut weights = [0u32; 64];
    rank_table_q31(16, &mut weights).unwrap();

    // Must be strictly decreasing
    for i in 0..15 {
        assert!(
            weights[i] > weights[i + 1],
            "Weight {} ({}) not > weight {} ({})",
            i,
            weights[i],
            i + 1,
            weights[i + 1]
        );
    }

    // Sum in Q31 should be close to 0x7FFF_FFFF (within integer rounding of 16 terms)
    let sum: u64 = weights[..16].iter().map(|&w| w as u64).sum();
    let expected = 0x7FFF_FFFFu64;
    let diff = (sum as i64 - expected as i64).abs();
    assert!(
        diff < 32,
        "Q31 sum diff {} is larger than roundoff tolerance",
        diff
    );
}

#[test]
fn test_raw_rank_weights_q16() {
    let mut raw = [0u16; 8];
    raw_rank_weights_q16(4, &mut raw).unwrap();

    // 1/1 = 65535, 1/2 = 32768, 1/3 = 21845, 1/4 = 16384
    assert_eq!(raw[0], 0xFFFF);
    assert_eq!(raw[1], 32768);
    assert_eq!(raw[2], 21845);
    assert_eq!(raw[3], 16384);
}
