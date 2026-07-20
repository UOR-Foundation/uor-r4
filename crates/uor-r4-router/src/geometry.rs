use serde::{Deserialize, Serialize};
use uor_r4_core::semantic::{KappaLabel, WeightedRoute};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypedObject {
    pub object_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroundedSemantics {
    pub vsa_vector: Vec<f32>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FacetCoordinates {
    pub coordinates: std::collections::HashMap<String, Vec<u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Operator {
    pub name: String,
    pub space_cid: KappaLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeometryError {
    InvalidObject,
    EncodingFailed,
    RouteResolutionFailed,
    OperatorMismatch,
}

pub trait SemanticGeometry {
    fn space_manifest(&self) -> KappaLabel;
    fn ground(&self, object: &TypedObject) -> Result<GroundedSemantics, GeometryError>;
    fn encode(&self, grounded: &GroundedSemantics) -> Result<FacetCoordinates, GeometryError>;
    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Result<Vec<WeightedRoute>, GeometryError>;
    fn apply_operator(&self, route: &WeightedRoute, operator: &Operator) -> Result<Vec<WeightedRoute>, GeometryError>;
}
