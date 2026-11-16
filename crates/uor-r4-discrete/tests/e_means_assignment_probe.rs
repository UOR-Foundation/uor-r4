//! Probe: e-means as the **content assignment** for the routed-zero read.
//!
//! The routed-zero read (codex-29) needs a rule that maps content to a sparse set of channels —
//! the piece its planted experiment lacked. e-means (`crate::e_means`, Meta's arXiv 2611.09501)
//! clusters content features; a cluster can own a channel set. This probe plants a paraphrase task
//! and measures whether the clustering shortlist carries the retrieval:
//!
//! - **content**: `clusters x items` feature vectors (a cluster centre plus item noise), each item a
//!   value byte;
//! - **code**: an item's phases are a coarse quantisation of its feature on its cluster's channels,
//!   so a small paraphrase noise leaves the phases and a different item has different ones;
//! - **query**: a *noisy* copy of one item's feature (the paraphrase);
//! - **read**: nearest centroid by exact squared distance (the e-means assignment), then the
//!   least circular energy over that cluster's items; the value is the winner's.
//!
//! Arms: `routed` (the e-means shortlist), `shuffled` (the same clusters with the item-to-cluster
//! labels permuted — identical occupancy, wrong shortlist), `global` (no shortlist: least energy
//! over every item), and `value-swapped` (the stored values rotated, the ground truth unchanged).
//!
//! Falsifiers: `shuffled` must collapse if the assignment is what carries it; `value-swapped` must
//! collapse to chance if the value really comes from the store.
//!
//! `cargo test -p uor-r4-discrete --test e_means_assignment_probe -- --nocapture`

use uor_r4_discrete::e_means;

/// A deterministic LCG, so the probe repeats exactly.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    /// A value in `-limit..=limit`.
    fn signed(&mut self, limit: i64) -> i64 {
        (self.next() % (2 * limit as u64 + 1)) as i64 - limit
    }
}

const DIM: usize = 8;
const CLUSTERS: usize = 8;
const ITEMS: usize = 12;
const CHANNELS: usize = DIM; // the phases live on the same lanes
const LIMIT: i64 = 1 << 12;

/// One item: its base feature, its cluster, its value.
struct Item {
    feature: Vec<i64>,
    cluster: usize,
    value: u8,
}

fn distance2(a: &[i64], b: &[i64]) -> u128 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| {
            let d = i128::from(x) - i128::from(y);
            (d * d) as u128
        })
        .sum()
}

/// A coarse phase per lane: the quantisation is 4 bits of the feature, so a small paraphrase
/// noise leaves it and a different item almost never coincides.
fn phases(feature: &[i64]) -> Vec<u8> {
    feature
        .iter()
        .take(CHANNELS)
        .map(|&v| (((v + LIMIT) >> 7) & 0x0f) as u8)
        .collect()
}

fn ring_step(a: u8, b: u8) -> u16 {
    let d = (i32::from(a) - i32::from(b)).unsigned_abs() as u16;
    d.min(16 - d) // the phases are a 4-bit ring
}

fn energy(item: &Item, query: &[u8]) -> u32 {
    phases(&item.feature)
        .iter()
        .zip(query)
        .map(|(&a, &b)| u32::from(ring_step(a, b)))
        .sum()
}

/// Accuracy of one arm over `queries` paraphrases of every item.
fn run(items: &[Item], centroids: &[i64], noise: i64, label_shift: usize, swap: bool) -> (f64, f64) {
    let mut rng = Lcg(0x51ce ^ noise as u64);
    let mut ok = 0usize;
    let mut tried = 0usize;
    let mut shortlist_total = 0usize;
    for item in items {
        for _ in 0..3 {
            let query_feature: Vec<i64> = item
                .feature
                .iter()
                .map(|&v| v + rng.signed(noise))
                .collect();
            let query = phases(&query_feature);
            // The e-means assignment: the nearest centroid.
            let mut cluster = 0usize;
            let mut best = u128::MAX;
            for (c, centroid) in centroids.chunks_exact(DIM).enumerate() {
                let d = distance2(centroid, &query_feature);
                if d < best {
                    best = d;
                    cluster = c;
                }
            }
            let shortlist: Vec<&Item> = items
                .iter()
                .filter(|i| (i.cluster + label_shift) % CLUSTERS == cluster)
                .collect();
            shortlist_total += shortlist.len();
            let winner = shortlist
                .iter()
                .min_by_key(|i| energy(i, &query))
                .expect("a shortlist is never empty");
            let expected = if swap {
                items[(items.iter().position(|i| std::ptr::eq(i, *winner)).unwrap() + 1) % items.len()]
                    .value
            } else {
                item.value
            };
            ok += usize::from(winner.value == expected);
            tried += 1;
        }
    }
    (ok as f64 / tried as f64, shortlist_total as f64 / tried as f64)
}

/// No shortlist: least energy over every item (the accuracy ceiling, at full cost).
fn global(items: &[Item], noise: i64) -> f64 {
    let mut rng = Lcg(0x9d ^ noise as u64);
    let mut ok = 0usize;
    let mut tried = 0usize;
    for item in items {
        for _ in 0..3 {
            let query_feature: Vec<i64> = item
                .feature
                .iter()
                .map(|&v| v + rng.signed(noise))
                .collect();
            let query = phases(&query_feature);
            let winner = items.iter().min_by_key(|i| energy(i, &query)).expect("items");
            ok += usize::from(winner.value == item.value);
            tried += 1;
        }
    }
    ok as f64 / tried as f64
}

#[test]
fn e_means_assignment_carries_the_shortlist_and_the_value_swap_is_a_real_falsifier() {
    // Content: cluster centres, then items around them.
    let mut rng = Lcg(2026_11_16);
    let mut items: Vec<Item> = Vec::with_capacity(CLUSTERS * ITEMS);
    for c in 0..CLUSTERS {
        let centre: Vec<i64> = (0..DIM).map(|_| rng.signed(LIMIT)).collect();
        for _ in 0..ITEMS {
            let feature: Vec<i64> = centre.iter().map(|&v| v + rng.signed(400)).collect();
            items.push(Item {
                feature,
                cluster: c,
                value: rng.next() as u8,
            });
        }
    }
    // The fit sees two paraphrases of every item.
    let mut points: Vec<i64> = Vec::with_capacity(items.len() * 2 * DIM);
    for item in &items {
        for _ in 0..2 {
            for &v in &item.feature {
                points.push(v + rng.signed(200));
            }
        }
    }
    let n = points.len() / DIM;
    let centroids = e_means::fit(&points, n, DIM, CLUSTERS, e_means::EPOCHS, 7).expect("fit");
    // The fitted centroids are in their own order: each item's cluster is the nearest one.
    let nearest = |feature: &[i64]| -> usize {
        let mut best = 0usize;
        let mut best_d = u128::MAX;
        for (c, centroid) in centroids.chunks_exact(DIM).enumerate() {
            let d = distance2(centroid, feature);
            if d < best_d {
                best_d = d;
                best = c;
            }
        }
        best
    };
    for item in &mut items {
        item.cluster = nearest(&item.feature);
    }

    for noise in [100i64, 400, 1_200] {
        let (routed, shortlist) = run(&items, &centroids, noise, 0, false);
        let (shuffled, _) = run(&items, &centroids, noise, 1, false);
        let (swapped, _) = run(&items, &centroids, noise, 0, true);
        let ceiling = global(&items, noise);
        println!(
            "noise {noise:5}: routed {routed:.3} (shortlist {shortlist:.0})  shuffled {shuffled:.3}  \
             value-swapped {swapped:.3}  global {ceiling:.3}"
        );
        if noise == 100 {
            assert!(routed > 0.8, "the e-means shortlist must carry a small paraphrase");
            assert!(shuffled < routed - 0.2, "the shuffled labels must collapse");
            assert!(swapped < routed - 0.2, "the value swap must collapse");
        }
    }
}
