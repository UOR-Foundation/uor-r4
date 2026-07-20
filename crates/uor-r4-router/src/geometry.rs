use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uor_r4_core::semantic::{KappaLabel, WeightedRoute, expand_atom};
use uor_r4_core::{
    identity_to_qimc_prime, derive_uor_control_plane
};

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
    pub coordinates: HashMap<String, Vec<u32>>,
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
pub struct SpectralGeometry<'a> {
    pub space_cid: KappaLabel,
    pub active_state: Option<&'a [f64]>,
    pub identity: Option<&'a str>,
}

impl<'a> SemanticGeometry for SpectralGeometry<'a> {
    fn space_manifest(&self) -> KappaLabel {
        self.space_cid.clone()
    }

    fn ground(&self, object: &TypedObject) -> Result<GroundedSemantics, GeometryError> {
        if object.content.is_empty() {
            return Err(GeometryError::InvalidObject);
        }
        let mut v = vec![0.0; 1024];
        if let Some(state) = self.active_state {
            v[..512].copy_from_slice(state);
        } else {
            v[..512].copy_from_slice(&vec![1.0 / (512.0f64).sqrt(); 512]);
        }
        Ok(GroundedSemantics {
            vsa_vector: v.iter().map(|&x| x as f32).collect(),
            roles: vec!["spectral-role".to_string()],
        })
    }

    fn encode(&self, grounded: &GroundedSemantics) -> Result<FacetCoordinates, GeometryError> {
        let active_state: Vec<f64> = grounded.vsa_vector[..512].iter().map(|&x| x as f64).collect();
        let identity = self.identity.unwrap_or("");
        let (_qimc_prime, _qimc_index, identity_meta) = identity_to_qimc_prime(identity);
        let uor_control = derive_uor_control_plane(&identity_meta);

        let mut routed_idx = 0;
        let mut best_score = -1.0;
        let mut window_scores = Vec::with_capacity(16);

        for win_idx in 1..=16 {
            let s_idx = (win_idx - 1) * 32;
            let e_idx = win_idx * 32;
            let slice = &active_state[s_idx..e_idx];

            let mut sum_sq = 0.0;
            for &val in slice {
                sum_sq += val * val;
            }
            let norm = sum_sq.sqrt();
            let bias = uor_control
                .window_biases
                .get(&win_idx)
                .copied()
                .unwrap_or(0.0);
            let score = norm * (1.0 + bias);

            if score > best_score {
                best_score = score;
                routed_idx = win_idx;
            }
            window_scores.push(score);
        }

        let mut coords = HashMap::new();
        coords.insert("window".to_string(), vec![routed_idx as u32]);
        
        let mut score_bits = Vec::with_capacity(16);
        for &score in &window_scores {
            score_bits.push((score as f32).to_bits());
        }
        coords.insert("scores".to_string(), score_bits);

        Ok(FacetCoordinates { coordinates: coords })
    }

    fn soft_route(&self, coordinates: &FacetCoordinates, _max_routes: usize) -> Result<Vec<WeightedRoute>, GeometryError> {
        let window_idx = coordinates.coordinates.get("window")
            .and_then(|w| w.first())
            .copied()
            .unwrap_or(1);

        let mut routes = Vec::new();
        if let Some(scores) = coordinates.coordinates.get("scores") {
            for (idx, &bits) in scores.iter().enumerate() {
                let score = f32::from_bits(bits);
                routes.push(WeightedRoute {
                    axis: (idx + 1) as u32,
                    path: vec![(idx + 1) as u32],
                    score,
                });
            }
        } else {
            routes.push(WeightedRoute {
                axis: window_idx,
                path: vec![window_idx],
                score: 1.0,
            });
        }
        Ok(routes)
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
            type_path.push((sum_first_half.abs() as u32) % 100);
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
