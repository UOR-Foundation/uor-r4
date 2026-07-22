//! Stage-2 semantic validation (RFC §6 items 4–9): packed-range
//! resolution, edge endpoints and the reverse index, HEAD-bound
//! honesty, ROUT bytecode, and EMIT/EXCT storage descriptors.
//!
//! Stage 2 runs from [`GraphView::parse`](crate::GraphView::parse)
//! after stage 1, and only when a HEAD section is present: a container
//! without HEAD stays stage-1-only so bootstrap fixtures and pure
//! section-carrier artifacts remain parseable (their sections stay
//! opaque bytes). Completeness note: the RFC marks CODE/PROV mandatory,
//! but this draft slice validates the sections listed above only;
//! mandatory-section completeness is a later Phase-1 slice.
//!
//! v0 approximations (recorded in RFC §9):
//!
//! - Theorem 7 reverse-index wiring is checked as *existence*: every
//!   reverse entry is `< edge_count` and every canonical edge ID
//!   appears at least once in the reverse block. Per-node reverse
//!   ranges are not yet wired. The existence scan is O(edge_count²)
//!   worst case — acceptable at draft-line load time and replaced by
//!   the O(E) per-node range check when the wiring lands.
//! - Node emission ranges are byte ranges over the EMIT remainder
//!   (after the 4-byte storage descriptor); entry-typed resolution
//!   waits for the EMIT table layout.
//! - Prototype/mask word starts are checked against the ROUT section
//!   size in u64 words; the W-word extent per node is not yet
//!   cross-checked (needs a ROUT region layout).

use crate::error::{BoundKind, FormatError, RangeField};
use crate::head::Head;
use crate::header::read_u32_le;
use crate::records::{
    self, StorageDescriptor, PACKED_EDGE_LEN, PACKED_NODE_LEN, REVERSE_INDEX_ENTRY_LEN,
    STORAGE_DESCRIPTOR_LEN,
};
use crate::rout;
use crate::types::SectionId;
use crate::view::GraphView;

/// Run stage 2 over the sections of a stage-1-validated view. Returns
/// the decoded [`Head`] when a HEAD section is present; `None` keeps
/// the container stage-1-only (see module docs).
pub(crate) fn validate(view: &GraphView) -> Result<Option<Head>, FormatError> {
    let head = match view.section(SectionId::HEAD) {
        Some(bytes) => Head::parse(bytes)?,
        None => return Ok(None),
    };

    // HEAD-internal honesty: signature_bytes must equal W u64 words.
    let honest_signature_bytes = u32::from(head.signature_words()) * 8;
    if u32::from(head.signature_bytes()) != honest_signature_bytes {
        return Err(FormatError::DishonestBounds {
            bound: BoundKind::SignatureBytes,
            declared: u32::from(head.signature_bytes()),
            observed: honest_signature_bytes,
        });
    }

    // EMIT/EXCT storage descriptors (RFC §6 item 8). Node emission
    // ranges resolve against the EMIT remainder, so the descriptor is
    // validated first.
    let emit_remainder = match view.section(SectionId::EMIT) {
        Some(bytes) => {
            StorageDescriptor::parse(SectionId::EMIT, bytes)?;
            bytes.len() - STORAGE_DESCRIPTOR_LEN
        }
        None => 0,
    };
    if let Some(bytes) = view.section(SectionId::EXCT) {
        StorageDescriptor::parse(SectionId::EXCT, bytes)?;
    }

    // NODE is present iff node_count > 0, and its record count must
    // equal the declaration (RFC §6 item 4).
    let node_bytes = match view.section(SectionId::NODE) {
        Some(bytes) => {
            let expected = u64::from(head.node_count()) * PACKED_NODE_LEN as u64;
            if bytes.len() as u64 != expected {
                return Err(FormatError::NodeCountMismatch {
                    declared: head.node_count(),
                    section_len: bytes.len() as u64,
                });
            }
            Some(bytes)
        }
        None => {
            if head.node_count() > 0 {
                return Err(FormatError::MissingNodeSection);
            }
            None
        }
    };

    // EDGE holds edge_count canonical edges plus the reverse index.
    let edge_bytes = match view.section(SectionId::EDGE) {
        Some(bytes) => {
            let expected =
                u64::from(head.edge_count()) * (PACKED_EDGE_LEN + REVERSE_INDEX_ENTRY_LEN) as u64;
            if bytes.len() as u64 != expected {
                return Err(FormatError::EdgeCountMismatch {
                    declared: head.edge_count(),
                    section_len: bytes.len() as u64,
                });
            }
            Some(bytes)
        }
        None => {
            if head.edge_count() > 0 {
                return Err(FormatError::MissingEdgeSection);
            }
            None
        }
    };

    // ROUT size in u64 words, for prototype/mask word-start resolution.
    let rout_words = view
        .section(SectionId::ROUT)
        .map_or(0, |bytes| bytes.len() as u64 / 8);

    // Per-node range resolution and HEAD-bound honesty (RFC §6
    // items 4 and 7).
    if let Some(bytes) = node_bytes {
        for i in 0..head.node_count() {
            // The section size was validated above, so the record is
            // in bounds.
            let node = records::decode_node(&bytes[i as usize * PACKED_NODE_LEN..]);
            let child_end = u64::from(node.child_start) + u64::from(node.child_len);
            if child_end > u64::from(head.edge_count()) {
                return Err(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Child,
                });
            }
            let forward_end = u64::from(node.forward_start) + u64::from(node.forward_len);
            if forward_end > u64::from(head.edge_count()) {
                return Err(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Forward,
                });
            }
            let emission_end = u64::from(node.emission_start) + u64::from(node.emission_len);
            if emission_end > emit_remainder as u64 {
                return Err(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Emission,
                });
            }
            if u64::from(node.prototype_word_start) > rout_words {
                return Err(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Prototype,
                });
            }
            if u64::from(node.mask_word_start) > rout_words {
                return Err(FormatError::RangeOutOfBounds {
                    node: i,
                    field: RangeField::Mask,
                });
            }
            if node.child_len > head.max_frontier_width() {
                return Err(FormatError::DishonestBounds {
                    bound: BoundKind::FrontierWidth,
                    declared: u32::from(head.max_frontier_width()),
                    observed: u32::from(node.child_len),
                });
            }
            if u32::from(node.emission_len) > head.max_emission_entries() {
                return Err(FormatError::DishonestBounds {
                    bound: BoundKind::EmissionEntries,
                    declared: head.max_emission_entries(),
                    observed: u32::from(node.emission_len),
                });
            }
            if node.depth.0 >= head.depth_count() {
                return Err(FormatError::DishonestBounds {
                    bound: BoundKind::DepthCount,
                    declared: u32::from(head.depth_count()),
                    observed: u32::from(node.depth.0),
                });
            }
        }
    }

    // Edges: endpoints, reverse-index bounds, and the Theorem-7
    // existence approximation (RFC §6 item 5).
    if let Some(bytes) = edge_bytes {
        let edge_count = head.edge_count();
        for i in 0..edge_count {
            // The section size was validated above, so the record is
            // in bounds.
            let edge = records::decode_edge(&bytes[i as usize * PACKED_EDGE_LEN..]);
            if edge.src.0 >= head.node_count() || edge.dst.0 >= head.node_count() {
                return Err(FormatError::EdgeEndpointOutOfBounds {
                    edge: i,
                    src: edge.src.0,
                    dst: edge.dst.0,
                });
            }
        }
        let reverse = &bytes[edge_count as usize * PACKED_EDGE_LEN..];
        for (j, entry) in reverse.chunks_exact(REVERSE_INDEX_ENTRY_LEN).enumerate() {
            let edge_id = read_u32_le(entry, 0);
            if edge_id >= edge_count {
                return Err(FormatError::ReverseIndexOutOfBounds {
                    index: j as u32,
                    edge_id,
                });
            }
        }
        // Existence scan (v0): every canonical edge ID must appear at
        // least once in the reverse block. O(edge_count^2) worst case;
        // see the module docs.
        for i in 0..edge_count {
            let found = reverse
                .chunks_exact(REVERSE_INDEX_ENTRY_LEN)
                .any(|entry| read_u32_le(entry, 0) == i);
            if !found {
                return Err(FormatError::ReverseIndexMissing { edge: i });
            }
        }
    }

    // ROUT bytecode (RFC §6 item 6).
    if let Some(bytes) = view.section(SectionId::ROUT) {
        rout::validate(bytes, &head)?;
    }

    Ok(Some(head))
}
