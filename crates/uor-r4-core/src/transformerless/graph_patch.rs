//! Immutable graph patch deltas, route translation mapping, and Theorem 11 verifier
//! (Phase 9 / PDF §13 / Theorem 11 / Plan §9.20).

use crate::transformerless::{
    score_q::ScoreQ,
    transitions::{Edge, TransitionGraph},
};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteMapping {
    Retained(u32),
    Split(Vec<u32>),
    Merged(u32),
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RouteTranslationMap {
    pub mappings: BTreeMap<u32, RouteMapping>,
}

impl RouteTranslationMap {
    pub fn new() -> Self {
        RouteTranslationMap {
            mappings: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, parent_route_id: u32, mapping: RouteMapping) {
        self.mappings.insert(parent_route_id, mapping);
    }

    pub fn translate_route(&self, parent_route_id: u32) -> Option<Vec<u32>> {
        match self.mappings.get(&parent_route_id)? {
            RouteMapping::Retained(id) => Some(vec![*id]),
            RouteMapping::Split(ids) => Some(ids.clone()),
            RouteMapping::Merged(id) => Some(vec![*id]),
            RouteMapping::Removed => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphPatch {
    pub parent_graph_cid: String,
    pub epoch_id: u64,
    pub patch_cid: String,
    pub added_edges: Vec<Edge>,
    pub residual_updates: Vec<(usize, ScoreQ)>,
    pub tombstone_edge_ids: Vec<usize>,
    pub route_translation: RouteTranslationMap,
}

impl GraphPatch {
    pub fn new(
        parent_graph_cid: impl Into<String>,
        epoch_id: u64,
        added_edges: Vec<Edge>,
        residual_updates: Vec<(usize, ScoreQ)>,
        tombstone_edge_ids: Vec<usize>,
        route_translation: RouteTranslationMap,
    ) -> Self {
        let mut patch = GraphPatch {
            parent_graph_cid: parent_graph_cid.into(),
            epoch_id,
            patch_cid: String::new(),
            added_edges,
            residual_updates,
            tombstone_edge_ids,
            route_translation,
        };
        patch.patch_cid = patch.compute_cid();
        patch
    }

    /// Compute self-referential BLAKE3 CID over patch payload.
pub fn compute_cid(&self) -> String {
    let mut clone = self.clone();
    clone.patch_cid.clear();

    let mut bytes = Vec::new();
    ciborium::into_writer(&clone, &mut bytes)
        .expect("graph patch CBOR serialization must succeed");

    let mut hasher = Hasher::new();
    hasher.update(&bytes);
    format!("kappa:blake3:{}", hasher.finalize().to_hex())
}

    pub fn verify_cid(&self) -> bool {
        self.patch_cid == self.compute_cid()
    }

    /// Apply patch to a transition graph in-place.
    pub fn apply(&self, graph: &mut TransitionGraph) -> Result<(), String> {
        // Update ScoreQ residuals
        for &(edge_idx, ref score) in &self.residual_updates {
            if edge_idx >= graph.edges.len() {
                return Err(format!("Residual update edge index {} out of bounds", edge_idx));
            }
            graph.edges[edge_idx].score = *score;
        }

        // Add new edges
        for edge in &self.added_edges {
            let new_id = graph.add_edge_with_score(
                edge.src,
                edge.dst,
                edge.weight,
                edge.score,
                edge.kind,
            );
            if new_id != edge.id {
                return Err(format!(
                    "Added edge ID mismatch: patch expected {}, graph assigned {}",
                    edge.id, new_id
                ));
            }
        }

        // Remove tombstones (mark weight = 0)
        for &tombstone_idx in &self.tombstone_edge_ids {
            if tombstone_idx < graph.edges.len() {
                graph.edges[tombstone_idx].weight = 0;
            }
        }

        // Rebuild reverse index and verify Theorem 7
        graph
            .build_reverse_index()
            .map_err(|e| format!("Post-patch reverse index rebuild failed: {}", e))?;
        graph
            .verify_theorem_7()
            .map_err(|e| format!("Post-patch Theorem 7 verification failed: {}", e))?;

        Ok(())
    }

    pub fn to_cbor_bytes(&self) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub fn from_cbor_bytes(bytes: &[u8]) -> Result<Self, String> {
        let patch: GraphPatch = ciborium::from_reader(bytes).map_err(|e| e.to_string())?;
        if !patch.verify_cid() {
            return Err("GraphPatch CID verification failed".to_string());
        }
        Ok(patch)
    }
}

pub struct Theorem11Verifier;

impl Theorem11Verifier {
    /// Formally verify Theorem 11 route translation and score consistency.
    pub fn verify_theorem_11(
        parent: &TransitionGraph,
        patched: &TransitionGraph,
        map: &RouteTranslationMap,
    ) -> Result<(), String> {
        parent
            .verify_theorem_7()
            .map_err(|e| format!("Parent graph Theorem 7 failed: {}", e))?;
        patched
            .verify_theorem_7()
            .map_err(|e| format!("Patched graph Theorem 7 failed: {}", e))?;

        for (&parent_route_id, mapping) in &map.mappings {
            if let RouteMapping::Retained(patched_id) = mapping {
                let parent_edge = parent
                    .edges
                    .get(parent_route_id as usize)
                    .ok_or_else(|| format!("Parent route ID {} missing", parent_route_id))?;
                let patched_edge = patched
                    .edges
                    .get(*patched_id as usize)
                    .ok_or_else(|| format!("Patched route ID {} missing", patched_id))?;

                if parent_edge.score != patched_edge.score {
                    return Err(format!(
                        "Theorem 11 score mismatch for route {}: parent {:?} != patched {:?}",
                        parent_route_id, parent_edge.score, patched_edge.score
                    ));
                }
            }
        }

        Ok(())
    }
}
