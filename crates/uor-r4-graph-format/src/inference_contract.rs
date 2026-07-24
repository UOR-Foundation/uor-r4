//! Versioned machine-readable inference operation contract.
//!
//! Normative document source:
//! `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`.

/// Semantic version of the normative inference operation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ContractVersion {
    pub const fn as_tuple(self) -> (u16, u16, u16) {
        (self.major, self.minor, self.patch)
    }

    /// Stable packed u32 form: `major<<20 | minor<<10 | patch`.
    pub const fn encode_packed(self) -> u32 {
        ((self.major as u32) << 20) | ((self.minor as u32) << 10) | self.patch as u32
    }

    pub fn decode_packed(raw: u32) -> Result<Self, InferenceContractError> {
        let major = ((raw >> 20) & 0x0fff) as u16;
        let minor = ((raw >> 10) & 0x03ff) as u16;
        let patch = (raw & 0x03ff) as u16;
        if major == 0 && minor == 0 && patch == 0 {
            return Err(InferenceContractError::InvalidPackedVersion(raw));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

/// Current contract version shared by docs, scans, and proof obligations.
pub const INFERENCE_OPERATION_CONTRACT_VERSION: ContractVersion = ContractVersion {
    major: 0,
    minor: 1,
    patch: 0,
};

/// Runtime boundary activities governed by the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryActivity {
    IncrementalContextSignatureUpdate,
    SemanticRegionRouting,
    CandidateVerification,
    ActiveFrontierUpdate,
    TransitionScoring,
    GoalConstraintScoring,
    TokenCandidateScoringAndShortlist,
    FixedWidthPlanning,
    ScoreQDescriptorDecode,
}

/// Allowed operation classes for contract-bound runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedOperationClass {
    BitwiseWordLogic,
    ShiftAndRotate,
    Popcount,
    IntegerAddSub,
    IntegerAddSubSaturatingChecked,
    IntegerComparison,
    IntegerMinMax,
    FixedCapacitySelection,
    BoundedBranchOrBranchlessSelect,
    TableReads,
    CompilerGeneratedConstantOffsetAddressing,
}

/// Forbidden operation classes for contract-bound runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForbiddenOperationClass {
    ScalarIntegerMultiplication,
    SimdVectorMultiplication,
    FloatingPointArithmetic,
    DivisionAndRemainder,
    FusedMultiplyAdd,
    DotProductInstructions,
    DenseTensorOrMatrixMultiply,
    RuntimeNormalizationWithMulDiv,
    DynamicHeapAllocation,
}

/// Activities intentionally outside the deployed runtime contract boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplicitExclusion {
    Training,
    TeacherExecution,
    CompilerOptimization,
    Clustering,
    GraphInduction,
    Quantization,
    ArtifactGeneration,
    OfflineCertification,
    TestOnlyReferenceImplementations,
}

/// Owning module path for each contract boundary activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityOwner {
    pub activity: BoundaryActivity,
    pub module_path: &'static str,
}

/// Focused errors for contract registry lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceContractError {
    UnknownBoundaryActivity(BoundaryActivity),
    InvalidPackedVersion(u32),
}

impl core::fmt::Display for InferenceContractError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownBoundaryActivity(activity) => {
                write!(
                    f,
                    "missing owner mapping for boundary activity: {activity:?}"
                )
            }
            Self::InvalidPackedVersion(raw) => write!(f, "invalid packed contract version: {raw}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InferenceContractError {}

pub const BOUNDARY_ACTIVITIES: [BoundaryActivity; 9] = [
    BoundaryActivity::IncrementalContextSignatureUpdate,
    BoundaryActivity::SemanticRegionRouting,
    BoundaryActivity::CandidateVerification,
    BoundaryActivity::ActiveFrontierUpdate,
    BoundaryActivity::TransitionScoring,
    BoundaryActivity::GoalConstraintScoring,
    BoundaryActivity::TokenCandidateScoringAndShortlist,
    BoundaryActivity::FixedWidthPlanning,
    BoundaryActivity::ScoreQDescriptorDecode,
];

pub const ALLOWED_OPERATION_CLASSES: [AllowedOperationClass; 11] = [
    AllowedOperationClass::BitwiseWordLogic,
    AllowedOperationClass::ShiftAndRotate,
    AllowedOperationClass::Popcount,
    AllowedOperationClass::IntegerAddSub,
    AllowedOperationClass::IntegerAddSubSaturatingChecked,
    AllowedOperationClass::IntegerComparison,
    AllowedOperationClass::IntegerMinMax,
    AllowedOperationClass::FixedCapacitySelection,
    AllowedOperationClass::BoundedBranchOrBranchlessSelect,
    AllowedOperationClass::TableReads,
    AllowedOperationClass::CompilerGeneratedConstantOffsetAddressing,
];

pub const FORBIDDEN_OPERATION_CLASSES: [ForbiddenOperationClass; 9] = [
    ForbiddenOperationClass::ScalarIntegerMultiplication,
    ForbiddenOperationClass::SimdVectorMultiplication,
    ForbiddenOperationClass::FloatingPointArithmetic,
    ForbiddenOperationClass::DivisionAndRemainder,
    ForbiddenOperationClass::FusedMultiplyAdd,
    ForbiddenOperationClass::DotProductInstructions,
    ForbiddenOperationClass::DenseTensorOrMatrixMultiply,
    ForbiddenOperationClass::RuntimeNormalizationWithMulDiv,
    ForbiddenOperationClass::DynamicHeapAllocation,
];

pub const EXPLICIT_EXCLUSIONS: [ExplicitExclusion; 9] = [
    ExplicitExclusion::Training,
    ExplicitExclusion::TeacherExecution,
    ExplicitExclusion::CompilerOptimization,
    ExplicitExclusion::Clustering,
    ExplicitExclusion::GraphInduction,
    ExplicitExclusion::Quantization,
    ExplicitExclusion::ArtifactGeneration,
    ExplicitExclusion::OfflineCertification,
    ExplicitExclusion::TestOnlyReferenceImplementations,
];

pub const ACTIVITY_OWNERS: [ActivityOwner; 9] = [
    ActivityOwner {
        activity: BoundaryActivity::IncrementalContextSignatureUpdate,
        module_path: "uor-r4-core::transformerless::runtime",
    },
    ActivityOwner {
        activity: BoundaryActivity::SemanticRegionRouting,
        module_path: "uor-r4-graph-runtime::routing",
    },
    ActivityOwner {
        activity: BoundaryActivity::CandidateVerification,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::ActiveFrontierUpdate,
        module_path: "uor-r4-core::transformerless::reference_state",
    },
    ActivityOwner {
        activity: BoundaryActivity::TransitionScoring,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::GoalConstraintScoring,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::TokenCandidateScoringAndShortlist,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::FixedWidthPlanning,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::ScoreQDescriptorDecode,
        module_path: "uor-r4-wasm-router::r4g1::{encode_into,decode_into,generate_into}",
    },
];

pub fn owner_for_activity(
    activity: BoundaryActivity,
) -> Result<&'static str, InferenceContractError> {
    ACTIVITY_OWNERS
        .iter()
        .find(|entry| entry.activity == activity)
        .map(|entry| entry.module_path)
        .ok_or(InferenceContractError::UnknownBoundaryActivity(activity))
}

#[cfg(test)]
mod tests {
    use super::{
        owner_for_activity, ContractVersion, ACTIVITY_OWNERS, BOUNDARY_ACTIVITIES,
        INFERENCE_OPERATION_CONTRACT_VERSION,
    };

    #[test]
    fn every_boundary_activity_has_owner_mapping() {
        for activity in BOUNDARY_ACTIVITIES {
            let owner = owner_for_activity(activity).expect("owner mapping");
            assert!(!owner.is_empty());
        }
    }

    #[test]
    fn owner_mapping_is_unique_per_activity() {
        for (i, left) in ACTIVITY_OWNERS.iter().enumerate() {
            for right in ACTIVITY_OWNERS.iter().skip(i + 1) {
                assert_ne!(left.activity, right.activity, "duplicate owner mapping");
            }
        }
    }

    #[test]
    fn contract_version_is_nonzero_minor_or_major() {
        let (major, minor, _) = INFERENCE_OPERATION_CONTRACT_VERSION.as_tuple();
        assert!(major > 0 || minor > 0);
    }

    #[test]
    fn contract_version_packed_round_trip() {
        let version = INFERENCE_OPERATION_CONTRACT_VERSION;
        let packed = version.encode_packed();
        let decoded = ContractVersion::decode_packed(packed).expect("packed decode");
        assert_eq!(decoded, version);
    }
}
