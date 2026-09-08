//! Preserve the first typed operation under mixed-operation formatting requests.
use super::*;
impl Model {
    pub fn fit_action_binding(
        &self,
        docs: &[MixedOperatorExample],
        preservation: &[OperationTransitionExample],
        prompts: &[Document],
        config: SourceRoutingConfig,
    ) -> Result<(Model, serde_json::Value)> {
        if self.action_emission.is_some() || self.mixed_operators.is_none() {
            return Err(Error(
                "action binding requires retained mixed parent".into(),
            ));
        }
        let old_lexical = self
            .lexical_emission
            .as_ref()
            .ok_or_else(|| Error("action binding lexical parent absent".into()))?;
        let old_roles = self
            .typed_roles
            .as_ref()
            .ok_or_else(|| Error("action binding role parent absent".into()))?;
        let old_operation = self
            .operation_transition
            .as_ref()
            .ok_or_else(|| Error("action binding operation parent absent".into()))?;
        let (roles, fit) = self.fit_mixed_initial(docs, preservation, prompts, &config)?;
        let mut model = self.clone();
        model
            .typed_roles
            .as_mut()
            .ok_or_else(|| Error("action binding role router absent".into()))?
            .router = roles;
        model.action_emission = Some(action_emission::ActionEmission {
            parent_artifact: self.artifact_cid.clone(),
            previous_lexical: old_lexical.clone(),
            previous_roles: old_roles.router.clone(),
            previous_operation: old_operation.clone(),
            context_enabled: false,
        });
        model.refresh_identity()?;
        model.validate()?;
        Ok((
            model,
            serde_json::json!({"initial_roles":fit,"lexical_context_enabled":false,
            "scope":"Offline refinement of existing shared initial roles; literal/admission/operation/emission routers and serving features unchanged"}),
        ))
    }
}
