//! Host-only reconstruction of learned exact-field selection. Serialized
//! cursors and relation IDs are checked against the actual bounded output;
//! they do not authenticate source material outside the retained window.

use super::completion_types::CompletionWork;
use super::field_composition::FieldState;
use super::response_entry_types::ResponseEntryState;
use super::word_copy_types::{WordCopyState, WordCopyWork};
use super::*;

fn invalid(message: &str) -> Error {
    Error(format!("session field composition {message}"))
}

pub(super) fn validate_field_presence(model: &Model, wire: &serde_json::Value) -> Result<()> {
    let field = wire.get("field_composition");
    if model.field_composition.is_some() {
        if !field.is_some_and(serde_json::Value::is_object) {
            return Err(invalid("state is required by this artifact"));
        }
    } else if field.is_some() {
        return Err(invalid("state is absent from this artifact"));
    }
    if field.is_some_and(|state| state.get("pending").is_some()) {
        return Err(invalid("contains a transient prediction"));
    }
    Ok(())
}

impl Session {
    /// Returns true only when complete field replay also establishes that
    /// the inherited copy state is empty. Otherwise its normal validator
    /// must still run. A serialized field origin never grants that bypass.
    pub(super) fn restore_field_composition_state(
        &mut self,
        model: &Model,
        saved: FieldState,
    ) -> Result<bool> {
        let entry = self
            .response_entry
            .as_ref()
            .ok_or_else(|| invalid("requires restored response entry"))?;
        let values = self
            .values
            .as_ref()
            .ok_or_else(|| invalid("requires restored typed state"))?;
        if saved.pending.is_some() {
            return Err(invalid("contains a transient prediction"));
        }
        if !entry.active {
            if saved != FieldState::default() {
                return Err(invalid("inactive entry retains field state"));
            }
            self.field_composition = Some(saved);
            return Ok(false);
        }
        let anchor = entry
            .boundary
            .ok_or_else(|| invalid("active entry has no boundary"))?;
        // The entry validator already establishes the complete geometric
        // trajectory and its agreement with retained ordinary tokens.
        if values.seen.checked_sub(anchor.at_seen) != Some(u64::from(entry.steps))
            || entry.steps == 0
            || entry.steps >= super::response_entry_types::RESPONSE_ENTRY_STEPS
        {
            return Err(invalid("active prefix is outside the replay window"));
        }
        let mut boundary = values.clone();
        boundary.seen = anchor.at_seen;
        boundary.pose = anchor.pose;
        boundary.phases = anchor.phases;
        boundary.pending = None;
        let mut initial = ResponseEntryState {
            boundary: Some(anchor),
            last: values.queries[0].token,
            previous: if values.query_len > 1 {
                values.queries[1].token
            } else {
                BOS
            },
            seen: anchor.at_seen,
            ..ResponseEntryState::default()
        };
        let mut fields = FieldState::default();
        let mut copy = WordCopyState::default();
        let mut work = WordCopyWork::default();
        let baseline = Candidate {
            token: BOS,
            score: 0,
        };
        // Exactly reproduce the parent read first. The adapter may only
        // expose owner/value fields belonging to that already-selected read.
        let inherited = super::role_read::offer(
            &mut copy,
            model,
            &mut initial,
            &boundary,
            baseline,
            self.control,
            &mut work,
        );
        let first = inherited.and_then(|candidate| {
            super::field_composition::offer(
                model,
                &mut fields,
                &mut copy,
                &mut initial,
                &boundary,
                candidate,
                self.control,
                &mut work,
            )
        });
        if first.is_none() {
            if saved != FieldState::default() {
                return Err(invalid("origin differs from selected Base path"));
            }
            self.field_composition = Some(saved);
            return Ok(false);
        }
        for step in 0..entry.steps {
            let sequence = anchor.at_seen + u64::from(step);
            if sequence >= values.seen || values.seen - sequence > values.recent_len as u64 {
                return Err(invalid("actual output evidence is absent"));
            }
            let observed = values.recent[(sequence & 31) as usize];
            if observed.sequence != sequence {
                return Err(invalid("actual output sequence differs"));
            }
            let choice = if step == 0 {
                first
            } else {
                super::field_composition::offer(
                    model,
                    &mut fields,
                    &mut copy,
                    &mut initial,
                    &boundary,
                    baseline,
                    self.control,
                    &mut work,
                )
            };
            if choice.is_none_or(|candidate| candidate.token != observed.token) {
                return Err(invalid(
                    "observed prefix differs from learned field selection",
                ));
            }
            boundary.seen = sequence + 1;
            boundary.pose = observed.pose;
            boundary.phases = observed.phases;
            initial.observe(
                model,
                &boundary,
                observed.token,
                self.control,
                &mut CompletionWork::default(),
            );
            fields.observe(model, &mut initial, &boundary, observed.token, &mut work);
        }
        if fields != saved || initial != *entry {
            return Err(invalid(
                "saved field or entry state differs from causal replay",
            ));
        }
        if copy != WordCopyState::default() || self.word_copy != Some(WordCopyState::default()) {
            return Err(invalid("active fields retain inherited copy state"));
        }
        self.field_composition = Some(saved);
        Ok(true)
    }
}
