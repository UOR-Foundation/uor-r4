use serde::{Deserialize, Serialize};

pub type KappaLabel = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticRouteReferenceV1 {
    pub object_cid: KappaLabel,
    pub schema_cid: KappaLabel,
    pub semantic_space_cid: KappaLabel,
    pub geometry_manifest_cid: KappaLabel,
    pub routes: Vec<FacetRoute>,
    pub bindings_cid: KappaLabel,
    pub grounding_witness_cid: KappaLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetRoute {
    pub axis: u32,
    pub path: Vec<u16>,
    pub confidence_q16: u16,
    pub valid_from_epoch: u64,
    pub evidence_root_cid: KappaLabel,
}
