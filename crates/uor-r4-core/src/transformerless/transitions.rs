//! Forward transition edges ($E_f$) and reverse edge indexes ($E_b$) compiled
//! from teacher corpus streams.
//!
//! Enforces Theorem 7: Reverse edge indexes ($E_b$) reference exact canonical
//! edge IDs in $E_f$ sorted by destination region.

use super::compiler::Corpus;
use super::score_q::ScoreQ;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeKind {
    Refinement = 0,
    Overlap = 1,
    Forward = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub weight: u32,
    pub score: ScoreQ,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Default)]
pub struct TransitionGraph {
    /// Canonical edge array (E_f + E_r + E_o). Canonical Edge ID = index.
    pub edges: Vec<Edge>,
    /// Map from src node ID to list of canonical edge IDs originating from src.
    pub forward_map: HashMap<u32, Vec<u32>>,
    /// Reverse index array E_b: canonical edge IDs sorted by dst node ID.
    pub reverse_index: Vec<u32>,
    /// Map from dst node ID to (start_offset, count) in `reverse_index`.
    pub reverse_offsets: HashMap<u32, (usize, usize)>,
}

impl TransitionGraph {
    pub fn new() -> Self {
        TransitionGraph::default()
    }

    /// Add a canonical edge to the graph with explicit weight and ScoreQ fixed-point score.
    pub fn add_edge_with_score(
        &mut self,
        src: u32,
        dst: u32,
        weight: u32,
        score: ScoreQ,
        kind: EdgeKind,
    ) -> u32 {
        let id = self.edges.len() as u32;
        let edge = Edge {
            id,
            src,
            dst,
            weight,
            score,
            kind,
        };
        self.edges.push(edge);
        self.forward_map.entry(src).or_default().push(id);
        id
    }

    /// Add a canonical edge to the graph.
    pub fn add_edge(&mut self, src: u32, dst: u32, weight: u32, kind: EdgeKind) -> u32 {
        let raw = weight.min(i32::MAX as u32) as i32;
        self.add_edge_with_score(src, dst, weight, ScoreQ::from_raw(raw), kind)
    }

    /// Build and sort the reverse edge index $E_b$, validating Theorem 7 consistency.
    pub fn build_reverse_index(&mut self) -> Result<(), &'static str> {
        let mut edge_ids: Vec<u32> = (0..self.edges.len() as u32).collect();
        // Sort edge IDs by (dst, src, kind, id)
        edge_ids.sort_by_key(|&id| {
            let e = &self.edges[id as usize];
            (e.dst, e.src, e.kind as u8, e.id)
        });

        let mut reverse_offsets = HashMap::new();
        let total = edge_ids.len();
        let mut idx = 0;
        while idx < total {
            let dst = self.edges[edge_ids[idx] as usize].dst;
            let start = idx;
            while idx < total && self.edges[edge_ids[idx] as usize].dst == dst {
                idx += 1;
            }
            let count = idx - start;
            reverse_offsets.insert(dst, (start, count));
        }

        self.reverse_index = edge_ids;
        self.reverse_offsets = reverse_offsets;

        self.verify_theorem_7()
    }

    /// Verify Theorem 7 consistency:
    /// For every dst node, all reverse index entries in its slice MUST refer to
    /// valid canonical edge IDs whose destination equals dst.
    pub fn verify_theorem_7(&self) -> Result<(), &'static str> {
        for (&dst, &(start, count)) in &self.reverse_offsets {
            if start + count > self.reverse_index.len() {
                return Err("Theorem 7 violation: reverse index range out of bounds");
            }
            for i in start..start + count {
                let edge_id = self.reverse_index[i];
                let edge = self
                    .edges
                    .get(edge_id as usize)
                    .ok_or("Theorem 7 violation: invalid canonical edge ID in reverse index")?;
                if edge.dst != dst {
                    return Err("Theorem 7 violation: reverse index target mismatched edge dst");
                }
            }
        }
        Ok(())
    }
}

/// Compile forward region transitions ($E_f$) and reverse edge indexes ($E_b$)
/// from a teacher corpus.
///
/// `region_assigner`: maps a token ID / context index to a region ID.
/// `max_transitions_per_node`: maximum forward transitions to keep per region node.
pub fn compile_transitions_from_corpus<F>(
    corpus: &Corpus,
    region_assigner: F,
    max_transitions_per_node: usize,
) -> Result<TransitionGraph, &'static str>
where
    F: Fn(u32) -> u32,
{
    let mut transition_counts: HashMap<(u32, u32), u32> = HashMap::new();

    // Iterate over sequential positions in the corpus
    let n = corpus.n;
    if n > 1 {
        for i in 0..(n - 1) {
            // Keep transitions within the same story sequence
            if corpus.story[i] == corpus.story[i + 1] {
                let src_node = region_assigner(corpus.input[i]);
                let dst_node = region_assigner(corpus.next[i]);
                *transition_counts.entry((src_node, dst_node)).or_default() += 1;
            }
        }
    }

    // Group transitions by src node
    let mut transitions_by_src: HashMap<u32, Vec<(u32, u32)>> = HashMap::new();
    for ((src, dst), weight) in transition_counts {
        transitions_by_src.entry(src).or_default().push((dst, weight));
    }

    let mut graph = TransitionGraph::new();

    // Add forward edges bounded to top `max_transitions_per_node` per src node
    for (src, mut dsts) in transitions_by_src {
        // Sort descending by transition weight, then ascending by dst node ID
        dsts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let limit = dsts.len().min(max_transitions_per_node);
        for &(dst, weight) in &dsts[..limit] {
            graph.add_edge(src, dst, weight, EdgeKind::Forward);
        }
    }

    // Build and verify Theorem 7 reverse index
    graph.build_reverse_index()?;
    Ok(graph)
}
