//! Outer witness for jointly refining literal selection and numeric admission.
//! Restoring these two routers reconstructs the complete accepted parent,
//! including its unchanged lexical emitter, dictionary and earlier witnesses.
use super::source_routing::SourceRouting;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InstructionBinding {
    pub parent_artifact: String,
    pub previous_literals: SourceRouting,
    pub previous_admission: SourceRouting,
}

impl InstructionBinding {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let literals = model
            .typed_literals
            .as_ref()
            .ok_or_else(|| Error("instruction binding literal router absent".into()))?;
        let admission = model
            .joint_admission
            .as_ref()
            .ok_or_else(|| Error("instruction binding admission router absent".into()))?;
        literals.router.validate_shape(model, 3, 6)?;
        admission.router.validate_shape(model, 2, 8)?;

        // This outer witness must be peeled before the frozen lexical parent:
        // changing any other field prevents exact parent reconstruction.
        let mut parent = model.clone();
        parent.instruction_binding = None;
        parent
            .typed_literals
            .as_mut()
            .ok_or_else(|| Error("instruction binding literal parent absent".into()))?
            .router = self.previous_literals.clone();
        parent
            .joint_admission
            .as_mut()
            .ok_or_else(|| Error("instruction binding admission parent absent".into()))?
            .router = self.previous_admission.clone();
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("instruction binding frozen parent differs".into()));
        }
        parent.validate()?;

        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("instruction binding identity differs".into()));
        }
        Ok(())
    }
}
