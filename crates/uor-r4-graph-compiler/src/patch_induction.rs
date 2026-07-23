use std::collections::HashSet;
use uor_r4_graph_format::records::{PackedRouteTranslation, PackedTombstone};
use uor_r4_graph_format::{ArtifactCid, GraphView};

/// Emits the `PTCH` section payload.
///
/// The payload starts with a 32-byte parent CID, followed by packed tombstones.
pub fn emit_patch_section(parent_cid: &ArtifactCid, tombstones: &[PackedTombstone]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&parent_cid.0);
    for ts in tombstones {
        payload.extend_from_slice(&ts.id.to_le_bytes());
        payload.push(ts.kind);
        payload.push(ts.flags);
        payload.extend_from_slice(&ts.reserved.to_le_bytes());
    }
    payload
}

/// Emits the `RTNX` section payload.
///
/// The payload consists of an array of packed route translations.
pub fn emit_rtnx_section(translations: &[PackedRouteTranslation]) -> Vec<u8> {
    let mut payload = Vec::new();
    for r in translations {
        payload.extend_from_slice(&r.src_region.0.to_le_bytes());
        payload.extend_from_slice(&r.dst_region.0.to_le_bytes());
        payload.push(r.map_kind);
        payload.push(r.flags);
        payload.extend_from_slice(&r.reserved.to_le_bytes());
    }
    payload
}

/// Computes a structural diff between an older `base` graph and a `newer` state
/// (represented abstractly here as sets of valid IDs) to produce tombstones.
pub fn compute_tombstones(
    base: &GraphView,
    active_nodes: &HashSet<u32>,
    active_edges: &HashSet<u32>,
) -> Vec<PackedTombstone> {
    let mut tombstones = Vec::new();

    if let Some(n) = base.node_count() {
        for id in 0..n {
            if !active_nodes.contains(&id) {
                tombstones.push(PackedTombstone {
                    id,
                    kind: 0,
                    flags: 0,
                    reserved: 0,
                });
            }
        }
    }

    if let Some(e) = base.edge_count() {
        for id in 0..e {
            if !active_edges.contains(&id) {
                tombstones.push(PackedTombstone {
                    id,
                    kind: 1,
                    flags: 0,
                    reserved: 0,
                });
            }
        }
    }

    tombstones
}
