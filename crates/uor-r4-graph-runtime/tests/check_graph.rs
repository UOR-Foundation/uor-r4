use std::collections::BTreeSet;
use std::fs;
use uor_r4_graph_format::{GraphView, SectionId};

#[test]
fn test_dump() {
    let paths = [
        ".uor-models/compiled/smollm2-135m-instruct/compiled.r4g1",
        "../../.uor-models/compiled/smollm2-135m-instruct/compiled.r4g1",
    ];
    let bytes = paths.iter().find_map(|p| fs::read(p).ok()).unwrap();
    let base_graph = GraphView::parse(&bytes).unwrap();
    let num_nodes = base_graph.node_count().unwrap();
    println!("Graph has {} nodes", num_nodes);

    let emit_bytes = base_graph.section(SectionId::EMIT).unwrap();

    let mut all_emitted = BTreeSet::new();

    for n in 0..num_nodes {
        if let Some(node) = base_graph.node(n) {
            let start = node.emission_start as usize;
            let len = node.emission_len as usize;
            if len > 0 && start + len <= emit_bytes.len() {
                let sl = &emit_bytes[start..start + len];
                let entry_size = 8;
                for i in 0..(sl.len() / entry_size) {
                    let offset = i * entry_size;
                    let cand = u32::from_le_bytes([
                        sl[offset],
                        sl[offset + 1],
                        sl[offset + 2],
                        sl[offset + 3],
                    ]);
                    all_emitted.insert(cand);
                }
            }
        }
    }

    println!("Total unique emitted tokens: {}", all_emitted.len());
    println!("Emitted tokens: {:?}", all_emitted);
}
