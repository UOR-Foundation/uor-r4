use super::reference::KappaLabel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningOrigin {
    pub kind: String, // e.g. "teacher-distillation" or "native-corpus"
    pub teacher_model: Option<String>,
    pub teacher_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticSpaceManifestV1 {
    pub space_name: String,
    pub parent_space_cid: Option<KappaLabel>,
    pub schema_roots: Vec<KappaLabel>,
    pub axis_definitions: Vec<KappaLabel>,
    pub codebook_cids: Vec<KappaLabel>,
    pub threshold_cids: Vec<KappaLabel>,
    pub metric_cids: Vec<KappaLabel>,
    pub operator_registry_cid: KappaLabel,
    pub corpus_root_cids: Vec<KappaLabel>,
    pub compiler_cid: KappaLabel,
    pub quality_certificate_cid: KappaLabel,
    pub epoch: u64,
    pub learning_origin: Option<LearningOrigin>,
}
