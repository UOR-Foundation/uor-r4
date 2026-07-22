//! Fuzz target: arbitrary bytes → `GraphView::parse`.
//!
//! The parser must never panic — every rejection is a structured
//! `FormatError`. On the rare accept, the typed accessors and the CID
//! verifier are exercised too (same no-panic requirement).

#![no_main]

use libfuzzer_sys::fuzz_target;
use uor_r4_graph_format::GraphView;

fuzz_target!(|data: &[u8]| {
    if let Ok(view) = GraphView::parse(data) {
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
});
