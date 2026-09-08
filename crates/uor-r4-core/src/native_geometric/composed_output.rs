//! Outer witness for continuing the shared operation router beneath frozen
//! instruction binding and lexical emission. No serving state is stored here.
use super::source_routing::SourceRouting;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ComposedOutput {
    pub parent_artifact: String,
    pub previous_operation: SourceRouting,
}

impl ComposedOutput {
    pub(super) fn validate(&self, model: &Model) -> Result<()> {
        let operation = model
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("composed output operation router absent".into()))?;
        operation.router.validate_shape(model, 3, 7)?;

        // Restore only the changed router. Dictionaries, operation limits,
        // emitters and all earlier witnesses must reconstruct the exact parent.
        let mut parent = model.clone();
        parent.composed_output = None;
        parent
            .operation_transition
            .as_mut()
            .ok_or_else(|| Error("composed output operation parent absent".into()))?
            .router = self.previous_operation.clone();
        parent.refresh_identity()?;
        if parent.artifact_cid() != self.parent_artifact {
            return Err(Error("composed output frozen parent differs".into()));
        }
        parent.validate()?;

        let mut identity = model.clone();
        identity.refresh_identity()?;
        if identity.artifact_cid != model.artifact_cid
            || identity.uor_model_address != model.uor_model_address
        {
            return Err(Error("composed output identity differs".into()));
        }
        Ok(())
    }
}
