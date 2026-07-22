pub mod manifest;
pub mod reasoning;
pub mod reference;
pub mod vsa;

pub use manifest::{LearningOrigin, SemanticSpaceManifestV1};
pub use reasoning::{
    Constraint, OperatorRegistry, OperatorType, ReasoningPlanV1, SemanticInferenceWitnessV1,
    TypedOperator, WeightedRoute,
};
pub use reference::{FacetRoute, KappaLabel, SemanticRouteReferenceV1};
pub use vsa::{encode_event, encode_graph_edge, encode_statement, expand_atom, Hypervector};
pub mod merkle;
pub use merkle::{compute_merkle_root_and_proof, verify_merkle_proof};
