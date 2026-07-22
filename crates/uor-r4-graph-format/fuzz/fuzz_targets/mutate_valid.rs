//! Fuzz target: mutate a valid container, then parse.
//!
//! Builds a small stage-2-valid R4G1 container via `ArtifactBuilder`
//! (section payload variation seeded from the fuzz input), applies
//! input-driven byte flips (inside the CID-covered range only: bytes
//! 0..24 of the header carry unprotected fields like the minor version,
//! see RFC §2 and the crate-level CID convention) and truncations, then
//! re-parses. The parser must never panic, every rejection must be a
//! structured `FormatError`, and CID verification must detect every
//! content tampering — no false accepts.

#![no_main]

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use uor_r4_graph_format::{ArtifactBuilder, GraphView, SectionId};

/// Inline xorshift64 PRNG (deterministic build/mutation driver).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

const W: usize = 5; // signature words per region
const SIG_BYTES: usize = 36; // byte-exact signature width (4 padding bytes)

/// Serialize the fixed 224-byte HEAD prefix for `node_count` /
/// `edge_count` (W=5, signature_bytes=36, honest A/E/depth bounds).
fn head_payload(node_count: u32, edge_count: u32, depth_count: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(224);
    out.extend_from_slice(&[0x11; 32]); // teacher_cid
    out.extend_from_slice(&[0x22; 32]); // tokenizer_cid
    out.extend_from_slice(&[0x33; 32]); // corpus_construction_cid
    out.extend_from_slice(&[0x44; 32]); // corpus_certification_cid
    out.extend_from_slice(&[0u8; 20]); // hf_revision
    out.extend_from_slice(&[0x55; 32]); // compiler_version_cid
    out.extend_from_slice(&32u16.to_le_bytes()); // A
    out.extend_from_slice(&16u16.to_le_bytes()); // C
    out.extend_from_slice(&(W as u16).to_le_bytes());
    out.extend_from_slice(&8u16.to_le_bytes()); // K
    out.extend_from_slice(&64u32.to_le_bytes()); // E
    out.extend_from_slice(&64u32.to_le_bytes()); // D
    out.extend_from_slice(&node_count.to_le_bytes());
    out.extend_from_slice(&edge_count.to_le_bytes());
    out.push(depth_count);
    out.extend_from_slice(&[0u8; 5]); // fallback policy
    out.extend_from_slice(&[0u8; 2]); // reserved
    out.extend_from_slice(&(SIG_BYTES as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_major
    out.extend_from_slice(&0u16.to_le_bytes()); // min_runtime_minor
    out.extend_from_slice(&0u16.to_le_bytes()); // feature_bits_required
    out.extend_from_slice(&32000u32.to_le_bytes()); // vocab_size
    debug_assert_eq!(out.len(), 224);
    out
}

/// A small stage-2-valid container in the migration geometry: root +
/// class-style nodes, a refinement chain (plus one optional extra
/// edge), ROUT = `[HALT+pad][prototype words][mask words]`, and an EMIT
/// root prior wired into node 0's emission range.
fn build_valid(rng: &mut Rng) -> Vec<u8> {
    let node_count = 2 + (rng.next() % 7) as u32; // 2..=8
    let n = node_count as usize;

    // Canonical edges sorted by (src, dst): the chain plus, sometimes,
    // a root shortcut to the last node.
    let mut edge_set: BTreeSet<(u32, u32)> = BTreeSet::new();
    for i in 0..node_count - 1 {
        edge_set.insert((i, i + 1));
    }
    if node_count > 2 && rng.next().is_multiple_of(2) {
        edge_set.insert((0, node_count - 1));
    }
    let edges: Vec<(u32, u32)> = edge_set.into_iter().collect();
    let edge_count = edges.len() as u32;

    // Per-node child ranges over the canonical array; reverse index
    // sorted by (dst, src) with per-node forward ranges.
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

    // NODE records.
    let prior_entries = 1 + (rng.next() % 4) as u16; // 1..=4 entries
    let mut node_section = Vec::with_capacity(n * 30);
    for i in 0..n {
        let (emission_start, emission_len) = if i == 0 {
            (0u32, prior_entries * 8)
        } else {
            (0u32, 0u16)
        };
        node_section.extend_from_slice(&child_start[i].to_le_bytes());
        node_section.extend_from_slice(&child_len[i].to_le_bytes());
        node_section.extend_from_slice(&forward_start[i].to_le_bytes());
        node_section.extend_from_slice(&forward_len[i].to_le_bytes());
        node_section.extend_from_slice(&emission_start.to_le_bytes());
        node_section.extend_from_slice(&emission_len.to_le_bytes());
        node_section.extend_from_slice(&(1 + (i as u32) * (W as u32)).to_le_bytes());
        node_section.extend_from_slice(&(1 + (node_count + i as u32) * (W as u32)).to_le_bytes());
        node_section.extend_from_slice(&288u16.to_le_bytes()); // radius
        node_section.push(i as u8); // depth < depth_count
        node_section.push(0); // flags
    }

    // EDGE section: canonical records + reverse index.
    let mut edge_section = Vec::with_capacity(edges.len() * 20);
    for &(src, dst) in &edges {
        edge_section.extend_from_slice(&src.to_le_bytes());
        edge_section.extend_from_slice(&dst.to_le_bytes());
        edge_section.extend_from_slice(&0i32.to_le_bytes()); // score_q
        edge_section.push(0); // kind: E_r
        edge_section.push(0); // flags
        edge_section.extend_from_slice(&0u16.to_le_bytes()); // reserved
    }
    for &id in &reverse {
        edge_section.extend_from_slice(&id.to_le_bytes());
    }

    // ROUT section: padded HALT program, then prototypes and masks.
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

    // EMIT: descriptor + root prior entries (v0 linear-count encoding).
    let mut emit = vec![2u8, 0, 0, 0];
    for i in 0..prior_entries {
        emit.extend_from_slice(&(i as i32).to_le_bytes());
        emit.extend_from_slice(&((i + 1) as i32).to_le_bytes());
    }

    let mut builder = ArtifactBuilder::new(3);
    builder.add_section(
        SectionId::HEAD,
        0,
        &head_payload(node_count, edge_count, node_count as u8),
    );
    builder.add_section(SectionId::NODE, 0, &node_section);
    builder.add_section(SectionId::EDGE, 0, &edge_section);
    builder.add_section(SectionId::ROUT, 0, &rout);
    builder.add_section(SectionId::EMIT, 0, &emit);
    builder.build().expect("seed container must serialize")
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    let mut rng = Rng(u64::from_le_bytes(data[..8].try_into().unwrap()) | 1);
    let mut bytes = build_valid(&mut rng);
    debug_assert!(GraphView::parse(&bytes).is_ok());

    // Input-driven mutations: mostly XOR flips inside the CID-covered
    // range (>= 24), occasionally a truncation. Flips XOR with a nonzero
    // mask, but two flips can cancel at the same position, so the
    // tamper verdict is the final byte comparison, not the flip count.
    let original = bytes.clone();
    for &b in &data[8..] {
        let choice = rng.next();
        match b % 8 {
            0..=5 => {
                let pos = 24 + (choice % (bytes.len() - 24) as u64) as usize;
                bytes[pos] ^= b | 1;
            }
            6 => {
                let keep = 24 + (choice % (bytes.len() - 24) as u64) as usize;
                bytes.truncate(keep);
                break;
            }
            _ => {}
        }
    }
    let tampered = bytes != original;

    // Never panics; every rejection is a structured FormatError.
    if let Ok(view) = GraphView::parse(&bytes) {
        if tampered {
            assert!(
                view.verify_cids().is_err(),
                "CID verification accepted a tampered artifact"
            );
        } else {
            view.verify_cids().expect("untouched build must verify");
        }
    }
});
