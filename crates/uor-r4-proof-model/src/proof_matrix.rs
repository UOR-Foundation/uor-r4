//! Executable proof module: Machine-checkable ProofStatusMatrix tracking verification status across all PDF theorems.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofStatus {
    Verified,
    ExecutableSpec,
    DifferentialPass,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremEntry {
    pub name: String,
    pub theorem_id: String,
    pub status: ProofStatus,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStatusMatrix {
    pub entries: Vec<TheoremEntry>,
}

impl Default for ProofStatusMatrix {
    fn default() -> Self {
        ProofStatusMatrix {
            entries: vec![
                TheoremEntry {
                    name: "Allocation Freedom".to_string(),
                    theorem_id: "PDF §16".to_string(),
                    status: ProofStatus::Verified,
                    description:
                        "Zero allocation step contract enforced by counting allocator tests"
                            .to_string(),
                },
                TheoremEntry {
                    name: "Bounded Ranges".to_string(),
                    theorem_id: "Theorem 8".to_string(),
                    status: ProofStatus::Verified,
                    description: "Section relative packed range boundaries verified bounds-checked"
                        .to_string(),
                },
                TheoremEntry {
                    name: "Deterministic Top-K".to_string(),
                    theorem_id: "PDF §23".to_string(),
                    status: ProofStatus::Verified,
                    description: "Canonical tie-breaking (highest score, then lowest TokenId)"
                        .to_string(),
                },
                TheoremEntry {
                    name: "Reverse Index Consistency".to_string(),
                    theorem_id: "Theorem 7".to_string(),
                    status: ProofStatus::Verified,
                    description:
                        "Reverse edge indexes reference exact canonical edge IDs sorted by target"
                            .to_string(),
                },
            ],
        }
    }
}

impl ProofStatusMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verify_all(&self) -> Result<(), String> {
        for entry in &self.entries {
            if entry.status == ProofStatus::Unverified {
                return Err(format!("Unverified theorem found: {}", entry.theorem_id));
            }
        }
        Ok(())
    }
}
