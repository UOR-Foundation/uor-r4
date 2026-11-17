//! CUDA tests of the modified Kneser–Ney scorer (claude-421): the GPU masses
//! equal `KnTables::scored_masses` element by element, for N = 0..=4, on the
//! training stream, on another seed's stream and on a stream holding BOS in
//! every context slot. Run on wick's 4090 with `--release`.

use crate::kn_score::KnTables;
use crate::kn_score_device::KnScorer;
use crate::ngram_count::NGRAM_BOS;
use crate::ngram_count_oracle::oracle_adjusted;

/// The stream generator of kn_score.rs's tests, copied: a heavy head, a long
/// tail, one draw in 61 a BOS and one in 9 a copy of an earlier span.
fn kn_score_stream(seed: u64, len: usize) -> Vec<u16> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 16) as u32
    };
    let mut stream: Vec<u16> = vec![NGRAM_BOS];
    while stream.len() < len {
        let r = next();
        if (r >> 16) % 9 == 0 && stream.len() > 16 {
            let from = next() as usize % (stream.len() - 9);
            let run = 2 + (r >> 20) as usize % 8;
            for at in from..from + run {
                stream.push(stream[at]);
            }
            continue;
        }
        let product = (r % 16) * ((r >> 4) % 16) * ((r >> 8) % 16) * ((r >> 12) % 16);
        stream.push(if (r >> 24) % 61 == 0 {
            NGRAM_BOS
        } else {
            (5 + product / 16) as u16
        });
    }
    stream.truncate(len);
    stream[0] = NGRAM_BOS;
    stream
}

fn tables(tokens: &[u16], highest: u32) -> KnTables {
    let orders = (0..=highest)
        .map(|k| oracle_adjusted(tokens, k, highest).expect("adjusted counts"))
        .collect();
    KnTables::new(highest, orders).expect("tables")
}

/// A query with BOS at every distance 1..=5 before some target: a run of
/// training tokens, then BOS followed by 0..=5 training tokens, repeated.
fn bos_query(train: &[u16]) -> Vec<u16> {
    let mut query = vec![NGRAM_BOS];
    for (round, gap) in (0..60usize).zip((0..=5usize).cycle()) {
        let from = 1 + (round * 37) % (train.len() - 16);
        query.extend(train[from..from + 6].iter().filter(|&&t| t != NGRAM_BOS));
        query.push(NGRAM_BOS);
        query.extend(
            train[from + 6..from + 6 + gap]
                .iter()
                .filter(|&&t| t != NGRAM_BOS),
        );
    }
    query.push(7);
    query
}

#[test]
fn kn_score_cuda_matches_the_host_scorer() {
    let train = kn_score_stream(0x421a, 20_000);
    let other = kn_score_stream(0x421b, 20_000);
    let bos = bos_query(&train);
    for k in 1..=5usize {
        assert!(
            bos.windows(k + 1)
                .any(|w| w[0] == NGRAM_BOS && w[1..].iter().all(|&t| t != NGRAM_BOS)),
            "the BOS query lacks BOS at distance {k}"
        );
    }
    let mut by_order = Vec::new();
    for highest in 0..=4u32 {
        let host = tables(&train, highest);
        let scorer = KnScorer::new(0, &host).expect("scorer");
        for (name, query) in [("train", &train), ("other seed", &other), ("bos", &bos)] {
            let want = host.scored_masses(query).expect("host masses");
            let got = scorer.score(query).expect("gpu masses");
            assert_eq!(got.len(), want.len(), "N {highest} {name}: length");
            for (i, (g, w)) in got.iter().zip(&want).enumerate() {
                assert_eq!(g, w, "N {highest} {name}: scored target {i}");
            }
            eprintln!("N {highest} {name}: {} masses equal", want.len());
        }
        by_order.push(scorer.score(&other).expect("gpu masses"));
    }
    let (order_zero, order_two) = (&by_order[0], &by_order[2]);
    let differing = order_zero
        .iter()
        .zip(order_two)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing * 2 > order_zero.len(),
        "N = 2 differs from N = 0 on only {differing} of {} targets",
        order_zero.len()
    );
}

#[test]
fn kn_score_cuda_empty_and_bos_only_queries_score_nothing() {
    let train = kn_score_stream(0x421c, 6_000);
    let scorer = KnScorer::new(0, &tables(&train, 2)).expect("scorer");
    assert!(scorer
        .score(&[NGRAM_BOS, NGRAM_BOS])
        .expect("bos only")
        .is_empty());
    assert!(scorer.score(&[4096]).is_err(), "a 13-bit id is refused");
}
