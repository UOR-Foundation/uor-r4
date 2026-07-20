use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uor_r4_core::semantic::{KappaLabel, WeightedRoute, expand_atom};

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
    pub coordinates: HashMap<String, Vec<u16>>,
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

// 1. Spectral Geometry (Heuristic Baseline)
pub struct SpectralGeometry {
    pub space_cid: KappaLabel,
}

impl SemanticGeometry for SpectralGeometry {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Result<GroundedSemantics, GeometryError> {
        if object.content.is_empty() {
            return Err(GeometryError::InvalidObject);
        }
        Ok(GroundedSemantics {
            vsa_vector: vec![0.0; 1024],
            roles: vec!["spectral-role".to_string()],
        })
    }

    fn encode(&self, _grounded: &GroundedSemantics) -> Result<FacetCoordinates, GeometryError> {
        let mut coords = HashMap::new();
        coords.insert("type".to_string(), vec![1, 2]);
        coords.insert("entity".to_string(), vec![10, 20]);
        Ok(FacetCoordinates { coordinates: coords })
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Result<Vec<WeightedRoute>, GeometryError> {
        let mut routes = Vec::new();
        for (facet, path) in &coordinates.coordinates {
            let axis = match facet.as_str() {
                "type" => 1,
                "entity" => 2,
                _ => 0,
            };
            routes.push(WeightedRoute {
                axis,
                path: path.clone(),
                score: 1.0,
            });
        }
        Ok(routes.into_iter().take(max_routes).collect())
    }

    fn apply_operator(&self, route: &WeightedRoute, operator: &Operator) -> Result<Vec<WeightedRoute>, GeometryError> {
        if operator.name == "identity" {
            Ok(vec![route.clone()])
        } else {
            Err(GeometryError::OperatorMismatch)
        }
    }
}

// 2. VsaGeometry (Proof-Carrying)
pub struct VsaGeometry {
    pub space_cid: KappaLabel,
}

impl SemanticGeometry for VsaGeometry {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Result<GroundedSemantics, GeometryError> {
        if object.content.is_empty() {
            return Err(GeometryError::InvalidObject);
        }
        // Expand atom deterministically from content hash
        let hv = expand_atom(&object.object_type, &object.content, &self.space_cid);
        let mut float_vec = Vec::with_capacity(1024);
        for i in 0..16 {
            let val = hv.0[i];
            for bit in 0..64 {
                let bit_val = if (val & (1u64 << bit)) != 0 { 1.0 } else { -1.0 };
                float_vec.push(bit_val);
            }
        }
        Ok(GroundedSemantics {
            vsa_vector: float_vec,
            roles: vec!["grounded-vsa-role".to_string()],
        })
    }

    fn encode(&self, grounded: &GroundedSemantics) -> Result<FacetCoordinates, GeometryError> {
        let mut coords = HashMap::new();
        // Induce simple path codes by binning the grounded VSA dimensions
        let mut type_path = Vec::new();
        if !grounded.vsa_vector.is_empty() {
            let sum_first_half: f32 = grounded.vsa_vector[0..512].iter().sum();
            type_path.push((sum_first_half.abs() as u16) % 100);
        }
        coords.insert("type".to_string(), type_path);
        coords.insert("entity".to_string(), vec![100, 200]);
        Ok(FacetCoordinates { coordinates: coords })
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, max_routes: usize) -> Result<Vec<WeightedRoute>, GeometryError> {
        let mut routes = Vec::new();
        for (facet, path) in &coordinates.coordinates {
            let axis = match facet.as_str() {
                "type" => 1,
                "entity" => 2,
                _ => 0,
            };
            routes.push(WeightedRoute {
                axis,
                path: path.clone(),
                score: 0.95,
            });
        }
        Ok(routes.into_iter().take(max_routes).collect())
    }

    fn apply_operator(&self, route: &WeightedRoute, operator: &Operator) -> Result<Vec<WeightedRoute>, GeometryError> {
        if operator.name == "vsa-identity" {
            Ok(vec![route.clone()])
        } else {
            Err(GeometryError::OperatorMismatch)
        }
    }
}
