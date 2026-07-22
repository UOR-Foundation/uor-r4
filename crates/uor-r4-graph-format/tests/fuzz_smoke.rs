//! Stable-toolchain deterministic mutation smoke test: the same
//! no-panic / structured-error assertions as the cargo-fuzz targets
//! (`fuzz/fuzz_targets/{parse_arbitrary,mutate_valid}.rs`), driven by an
//! inline xorshift64 PRNG so CI without a nightly toolchain gets the
//! same shake. ~20k iterations per target.

mod common;

use common::{edge_section, head_payload, node_section, EdgeFields, HeadFields, NodeFields};
use uor_r4_graph_format::{ArtifactBuilder, GraphView, SectionId};

/// xorshift64 PRNG (identical stepping to the fuzz targets' driver).
struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

const W: usize = 5;
const SIG_BYTES: usize = 36;
const ITERATIONS: usize = 20_000;

/// Exercise every typed accessor of a parsed view; must never panic.
fn exercise(view: &GraphView) {
    for section in view.sections() {
        let _ = (section.id, section.flags, section.payload.len());
    }
    if let Some(head) = view.head() {
        let _ = (
            head.node_count(),
            head.edge_count(),
            head.signature_words(),
            head.signature_bytes(),
            head.depth_count(),
        );
    }
    for node in view.nodes() {
        let _ = node;
    }
    for edge in view.edges() {
        let _ = edge;
    }
    if let Some(count) = view.edge_count() {
        for i in 0..count {
            let _ = view.reverse_edge_id(i);
        }
    }
    let _ = view.verify_cids();
}

/// Small stage-2-valid container in the migration geometry (W=5 words
/// per 36-byte signature), built through the canonical serializer with
/// PRNG-varied section payloads — the same construction as the
/// `mutate_valid` fuzz target.
fn build_valid(rng: &mut XorShift64) -> Vec<u8> {
    let node_count = 2 + (rng.next() % 7) as u32; // 2..=8
    let n = node_count as usize;

    let mut edge_set: std::collections::BTreeSet<(u32, u32)> = Default::default();
    for i in 0..node_count - 1 {
        edge_set.insert((i, i + 1));
    }
    if node_count > 2 && rng.next().is_multiple_of(2) {
        edge_set.insert((0, node_count - 1));
    }
    let edges: Vec<(u32, u32)> = edge_set.into_iter().collect();
    let edge_count = edges.len() as u32;

    let mut child_start = vec![0u32; n];
    let mut child_len = vec![0u16; n];
    for (i, &(src, _)) in edges.iter().enumerate() {
        if child_len[src as usize] == 0 {
            child_start[src as usize] = i as u32;
        }
        child_len[src as usize] += 1;
    }
    let mut reverse: Vec<u32> = (0..edge_count).collect();
    reverse.sort_by_key(|&id| (edges[id as usize].1, edges[id as usize].0));
    let mut forward_start = vec![0u32; n];
    let mut forward_len = vec![0u16; n];
    for (i, &id) in reverse.iter().enumerate() {
        let dst = edges[id as usize].1;
        if forward_len[dst as usize] == 0 {
            forward_start[dst as usize] = i as u32;
        }
        forward_len[dst as usize] += 1;
    }

    let prior_entries = 1 + (rng.next() % 4) as u16; // 1..=4
    let nodes: Vec<NodeFields> = (0..n)
        .map(|i| NodeFields {
            child_start: child_start[i],
            child_len: child_len[i],
            forward_start: forward_start[i],
            forward_len: forward_len[i],
            emission_start: 0,
            emission_len: if i == 0 { prior_entries * 8 } else { 0 },
            prototype_word_start: 1 + (i as u32) * (W as u32),
            mask_word_start: 1 + (node_count + i as u32) * (W as u32),
            radius: 288,
            depth: i as u8,
            flags: 0,
        })
        .collect();
    let canonical: Vec<EdgeFields> = edges
        .iter()
        .map(|&(src, dst)| EdgeFields {
            src,
            dst,
            score_q: 0,
            kind: 0,
            flags: 0,
            reserved: 0,
        })
        .collect();

    let mut rout = Vec::with_capacity(8 + n * W * 8 * 2);
    rout.extend_from_slice(&[0u8; 8]); // HALT + program padding
    for _ in 0..n {
        let mut words = [0u8; W * 8];
        for byte in words[..SIG_BYTES].iter_mut() {
            *byte = rng.next() as u8;
        }
        rout.extend_from_slice(&words);
    }
    for _ in 0..n {
        let mut words = [0u8; W * 8];
        words[..SIG_BYTES].fill(0xFF);
        rout.extend_from_slice(&words);
    }

    let mut emit = vec![2u8, 0, 0, 0];
    for i in 0..prior_entries {
        emit.extend_from_slice(&(i as i32).to_le_bytes());
        emit.extend_from_slice(&((i + 1) as i32).to_le_bytes());
    }

    let head = head_payload(&HeadFields {
        signature_words: W as u16,
        signature_bytes: SIG_BYTES as u16,
        node_count,
        edge_count,
        depth_count: node_count as u8,
        ..HeadFields::default()
    });

    let mut builder = ArtifactBuilder::new(3);
    builder.add_section(SectionId::HEAD, 0, &head);
    builder.add_section(SectionId::NODE, 0, &node_section(&nodes));
    builder.add_section(SectionId::EDGE, 0, &edge_section(&canonical, &reverse));
    builder.add_section(SectionId::ROUT, 0, &rout);
    builder.add_section(SectionId::EMIT, 0, &emit);
    builder.build().expect("seed container must serialize")
}

/// Mirrors `fuzz_targets/parse_arbitrary.rs`: arbitrary bytes must
/// never panic the parser; rejections are structured `FormatError`s.
#[test]
fn parse_arbitrary_smoke() {
    let mut rng = XorShift64(0x243F_6A88_85A3_08D3);
    for i in 0..ITERATIONS {
        let len = (rng.next() % 300) as usize;
        let mut bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
        // Every fourth input carries the magic so the run also reaches
        // past the first stage-1 check.
        if i % 4 == 0 && bytes.len() >= 4 {
            bytes[..4].copy_from_slice(b"R4G1");
        }
        match GraphView::parse(&bytes) {
            Ok(view) => exercise(&view),
            Err(error) => {
                let _ = format!("{error}"); // structured Display, no panic
            }
        }
    }
}

/// Mirrors `fuzz_targets/mutate_valid.rs`: a valid container with
/// PRNG-driven byte flips (CID-covered range only) and truncations must
/// never panic the parser, and CID verification must detect every
/// content tampering — no false accepts.
#[test]
fn mutate_valid_smoke() {
    let mut rng = XorShift64(0x9E37_79B9_7F4A_7C15);
    let mut accepted_tampered = 0usize;
    let mut rejected = 0usize;
    for _ in 0..ITERATIONS {
        let mut bytes = build_valid(&mut rng);
        assert!(
            GraphView::parse(&bytes).is_ok(),
            "seed container must be stage-2 valid"
        );
        // Two flips can cancel at the same position, so the tamper
        // verdict is the final byte comparison, not the flip count.
        let original = bytes.clone();
        for _ in 0..(rng.next() % 24) {
            match rng.next() % 8 {
                0..=5 => {
                    let pos = 24 + (rng.next() % (bytes.len() - 24) as u64) as usize;
                    let mask = (rng.next() as u8) | 1; // always a real change
                    bytes[pos] ^= mask;
                }
                6 => {
                    let keep = 24 + (rng.next() % (bytes.len() - 24) as u64) as usize;
                    bytes.truncate(keep);
                    break;
                }
                _ => {}
            }
        }
        let tampered = bytes != original;
        match GraphView::parse(&bytes) {
            Ok(view) => {
                exercise(&view);
                if tampered {
                    assert!(
                        view.verify_cids().is_err(),
                        "CID verification accepted a tampered artifact"
                    );
                    accepted_tampered += 1;
                } else {
                    view.verify_cids().expect("untouched build must verify");
                }
            }
            Err(error) => {
                let _ = format!("{error}");
                rejected += 1;
            }
        }
    }
    // Sanity: the shake actually exercised both outcomes — tampered
    // artifacts that still parse (caught by the CIDs) and ones rejected
    // by the two validation stages.
    assert!(
        accepted_tampered > 0,
        "no tampered artifact reached CID verification"
    );
    assert!(rejected > 0, "no mutated artifact was rejected");
}
