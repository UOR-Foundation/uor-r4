//! Shared fixture builders for the R4G1 test suites: section payload
//! constructors for the v0 draft-line layouts (224-byte HEAD, 30-byte
//! PackedNode, 16-byte PackedEdge + reverse index, ROUT ops, EMIT/EXCT
//! storage descriptor). Builders write exact little-endian wire bytes so
//! each test can violate exactly one invariant; everything here is
//! deterministic (canonical bytes for identical inputs).

// Both suites share this module; not every helper is used in each one.
#![allow(dead_code)]

use uor_r4_graph_format::{OP_HALT, OP_JMP_FWD, OP_LEAF, OP_TEST_POPCOUNT_LE};

/// HEAD field set with stage-2-valid defaults: no nodes/edges, `W=8`
/// with the honest `signature_bytes = 64`, `depth_count = 1`.
#[derive(Debug, Clone)]
pub struct HeadFields {
    pub teacher_cid: [u8; 32],
    pub tokenizer_cid: [u8; 32],
    pub corpus_construction_cid: [u8; 32],
    pub corpus_certification_cid: [u8; 32],
    pub hf_revision: [u8; 20],
    pub compiler_version_cid: [u8; 32],
    pub max_frontier_width: u16,
    pub max_candidates: u16,
    pub signature_words: u16,
    pub shortlist_size: u16,
    pub max_emission_entries: u32,
    pub max_program_steps: u32,
    pub node_count: u32,
    pub edge_count: u32,
    pub depth_count: u8,
    pub fallback_policies: [u8; 5],
    pub signature_bytes: u16,
    pub min_runtime_major: u16,
    pub min_runtime_minor: u16,
    pub feature_bits_required: u16,
    pub vocab_size: u32,
}

impl Default for HeadFields {
    fn default() -> Self {
        Self {
            teacher_cid: [0x11; 32],
            tokenizer_cid: [0x22; 32],
            corpus_construction_cid: [0x33; 32],
            corpus_certification_cid: [0x44; 32],
            hf_revision: *b"0123456789abcdef0123",
            compiler_version_cid: [0x55; 32],
            max_frontier_width: 32,
            max_candidates: 16,
            signature_words: 8,
            shortlist_size: 8,
            max_emission_entries: 64,
            max_program_steps: 64,
            node_count: 0,
            edge_count: 0,
            depth_count: 1,
            fallback_policies: [0; 5],
            signature_bytes: 64,
            min_runtime_major: 1,
            min_runtime_minor: 0,
            feature_bits_required: 0,
            vocab_size: 100,
        }
    }
}

/// Serialize the fixed 224-byte HEAD prefix (v0 draft line, RFC §4).
pub fn head_payload(f: &HeadFields) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&f.teacher_cid);
    out.extend_from_slice(&f.tokenizer_cid);
    out.extend_from_slice(&f.corpus_construction_cid);
    out.extend_from_slice(&f.corpus_certification_cid);
    out.extend_from_slice(&f.hf_revision);
    out.extend_from_slice(&f.compiler_version_cid);
    out.extend_from_slice(&f.max_frontier_width.to_le_bytes());
    out.extend_from_slice(&f.max_candidates.to_le_bytes());
    out.extend_from_slice(&f.signature_words.to_le_bytes());
    out.extend_from_slice(&f.shortlist_size.to_le_bytes());
    out.extend_from_slice(&f.max_emission_entries.to_le_bytes());
    out.extend_from_slice(&f.max_program_steps.to_le_bytes());
    out.extend_from_slice(&f.node_count.to_le_bytes());
    out.extend_from_slice(&f.edge_count.to_le_bytes());
    out.push(f.depth_count);
    out.extend_from_slice(&f.fallback_policies);
    out.extend_from_slice(&[0u8; 2]); // reserved
    out.extend_from_slice(&f.signature_bytes.to_le_bytes());
    out.extend_from_slice(&f.min_runtime_major.to_le_bytes());
    out.extend_from_slice(&f.min_runtime_minor.to_le_bytes());
    out.extend_from_slice(&f.feature_bits_required.to_le_bytes());
    out.extend_from_slice(&f.vocab_size.to_le_bytes());
    debug_assert_eq!(out.len(), 224);
    out
}

/// One packed node record's fields (30-byte PackedNode, PDF §21).
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeFields {
    pub child_start: u32,
    pub child_len: u16,
    pub forward_start: u32,
    pub forward_len: u16,
    pub emission_start: u32,
    pub emission_len: u16,
    pub prototype_word_start: u32,
    pub mask_word_start: u32,
    pub radius: u16,
    pub depth: u8,
    pub flags: u8,
}

/// Serialize a NODE section: contiguous 30-byte records, no padding.
pub fn node_section(nodes: &[NodeFields]) -> Vec<u8> {
    let mut out = Vec::with_capacity(nodes.len() * 30);
    for n in nodes {
        out.extend_from_slice(&n.child_start.to_le_bytes());
        out.extend_from_slice(&n.child_len.to_le_bytes());
        out.extend_from_slice(&n.forward_start.to_le_bytes());
        out.extend_from_slice(&n.forward_len.to_le_bytes());
        out.extend_from_slice(&n.emission_start.to_le_bytes());
        out.extend_from_slice(&n.emission_len.to_le_bytes());
        out.extend_from_slice(&n.prototype_word_start.to_le_bytes());
        out.extend_from_slice(&n.mask_word_start.to_le_bytes());
        out.extend_from_slice(&n.radius.to_le_bytes());
        out.push(n.depth);
        out.push(n.flags);
    }
    debug_assert_eq!(out.len(), nodes.len() * 30);
    out
}

/// One packed canonical edge's fields (16-byte PackedEdge).
#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeFields {
    pub src: u32,
    pub dst: u32,
    pub score_q: i32,
    pub kind: u8,
    pub flags: u8,
    pub reserved: u16,
}

/// Serialize an EDGE section: the canonical edge array followed by the
/// reverse index (one u32 edge ID per canonical edge).
pub fn edge_section(edges: &[EdgeFields], reverse: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(edges.len() * 20);
    for e in edges {
        out.extend_from_slice(&e.src.to_le_bytes());
        out.extend_from_slice(&e.dst.to_le_bytes());
        out.extend_from_slice(&e.score_q.to_le_bytes());
        out.push(e.kind);
        out.push(e.flags);
        out.extend_from_slice(&e.reserved.to_le_bytes());
    }
    for id in reverse {
        out.extend_from_slice(&id.to_le_bytes());
    }
    debug_assert_eq!(out.len(), edges.len() * 16 + reverse.len() * 4);
    out
}

/// `TEST_POPCOUNT_LE { word u8, mask u64, threshold u16 }` (12 bytes).
pub fn op_test_popcount_le(word: u8, mask: u64, threshold: u16) -> [u8; 12] {
    let mut out = [0u8; 12];
    out[0] = OP_TEST_POPCOUNT_LE;
    out[1] = word;
    out[2..10].copy_from_slice(&mask.to_le_bytes());
    out[10..12].copy_from_slice(&threshold.to_le_bytes());
    out
}

/// `JMP_FWD { delta_ops u16 }` (3 bytes).
pub fn op_jmp_fwd(delta_ops: u16) -> [u8; 3] {
    let mut out = [0u8; 3];
    out[0] = OP_JMP_FWD;
    out[1..3].copy_from_slice(&delta_ops.to_le_bytes());
    out
}

/// `LEAF { shortlist_start u32, shortlist_len u16 }` (7 bytes).
pub fn op_leaf(shortlist_start: u32, shortlist_len: u16) -> [u8; 7] {
    let mut out = [0u8; 7];
    out[0] = OP_LEAF;
    out[1..5].copy_from_slice(&shortlist_start.to_le_bytes());
    out[5..7].copy_from_slice(&shortlist_len.to_le_bytes());
    out
}

/// `HALT` (1 byte).
pub fn op_halt() -> [u8; 1] {
    [OP_HALT]
}

/// Serialize an EMIT/EXCT section: the 4-byte storage descriptor
/// `{width u8, shift i8, zero_point i16}` plus the opaque remainder.
pub fn storage_section(width: u8, shift: i8, zero_point: i16, remainder: &[u8]) -> Vec<u8> {
    let mut out = vec![width, shift as u8];
    out.extend_from_slice(&zero_point.to_le_bytes());
    out.extend_from_slice(remainder);
    out
}
