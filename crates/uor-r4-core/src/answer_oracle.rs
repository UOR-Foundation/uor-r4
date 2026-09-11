//! Offline answer oracle for authored evaluation cases.
//!
//! Accepted answer strings are authored once from the request's typed intent and
//! frozen in the case file, so every evaluator (the core driver and the native API
//! check) judges the same list. The list is membership only: no evaluator may derive
//! an alternative from the response, and a previous-location request never accepts a
//! present-tense statement of its old value.
use serde::{Deserialize, Serialize};

/// What the request asks for; the accepted forms follow from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// The first stored version: `VALUE.` or `OWNER was in VALUE.`
    Initial,
    /// The immediate previous version: `VALUE.` or `OWNER was in VALUE.`
    Previous,
    /// The live version: `VALUE.` or `OWNER is in VALUE.`
    Current,
    /// A proven-evicted chain: `Unknown.` only.
    Abstain,
}

/// Accepted complete answer strings for one request. A plain request (no owner-first
/// instruction) accepts the bare value or the frozen emitter's owner sentence in the
/// tense of the intent; an owner-first request accepts only that sentence.
pub fn accepted(intent: Intent, plain: bool, owner: &str, value: &str) -> Vec<String> {
    match intent {
        Intent::Abstain => vec![" Unknown.\n".to_owned()],
        Intent::Initial | Intent::Previous => sentence(plain, owner, "was", value),
        Intent::Current => sentence(plain, owner, "is", value),
    }
}

fn sentence(plain: bool, owner: &str, verb: &str, value: &str) -> Vec<String> {
    let owner_form = format!(" {owner} {verb} in {value}.\n");
    if plain {
        vec![format!(" {value}.\n"), owner_form]
    } else {
        vec![owner_form]
    }
}

/// Accepted complete answers for a current request that carries the explanatory
/// instruction (`Explain in a sentence.`). With an explicit owner-first instruction only
/// the present-tense owner sentence is accepted. Without it the inherited
/// `VALUE is the place.` form is a legitimate declared form and the owner sentence
/// remains acceptable; the bare value is not an explanation.
pub fn accepted_explanatory(owner_first: bool, owner: &str, value: &str) -> Vec<String> {
    let owner_form = format!(" {owner} is in {value}.\n");
    if owner_first {
        vec![owner_form]
    } else {
        vec![format!(" {value} is the place.\n"), owner_form]
    }
}

/// Membership in the frozen list; nothing else.
pub fn accepts(accepted: &[String], text: &str) -> bool {
    accepted.iter().any(|a| a == text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_oracle_explanatory_owner_first_requires_present_tense_owner_sentence() {
        let both = accepted_explanatory(true, "pemru", "amber quay");
        assert_eq!(both, vec![" pemru is in amber quay.\n".to_owned()]);
        assert!(!accepts(&both, " amber quay is the place.\n"));
        assert!(!accepts(&both, " pemru was in amber quay.\n"));
        assert!(!accepts(&both, " quay holds pemru is the place.\n"));
        assert!(!accepts(&both, " amber quay.\n"));
    }

    #[test]
    fn answer_oracle_explanatory_only_accepts_inherited_place_form_or_owner_sentence() {
        let explain = accepted_explanatory(false, "pemru", "amber quay");
        assert!(accepts(&explain, " amber quay is the place.\n"));
        assert!(accepts(&explain, " pemru is in amber quay.\n"));
        assert!(!accepts(&explain, " amber quay.\n"));
        assert!(!accepts(&explain, " pemru was in amber quay.\n"));
    }

    #[test]
    fn answer_oracle_previous_request_rejects_present_tense_old_value() {
        let plain = accepted(Intent::Previous, true, "selvi", "Copper Vale");
        assert_eq!(
            plain,
            vec![" Copper Vale.\n", " selvi was in Copper Vale.\n"]
        );
        assert!(!accepts(&plain, " selvi is in Copper Vale.\n"));
        assert!(accepts(&plain, " Copper Vale.\n"));
        let owner_first = accepted(Intent::Previous, false, "selvi", "Copper Vale");
        assert_eq!(owner_first, vec![" selvi was in Copper Vale.\n"]);
        assert!(!accepts(&owner_first, " Copper Vale.\n"));
        assert!(!accepts(&owner_first, " selvi is in Copper Vale.\n"));
    }

    #[test]
    fn answer_oracle_current_initial_and_abstain_forms_are_typed() {
        assert_eq!(
            accepted(Intent::Current, true, "selvi", "Amber Field"),
            vec![" Amber Field.\n", " selvi is in Amber Field.\n"]
        );
        assert!(!accepts(
            &accepted(Intent::Current, false, "selvi", "Amber Field"),
            " selvi was in Amber Field.\n"
        ));
        assert_eq!(
            accepted(Intent::Initial, false, "selvi", "Dusk Ridge"),
            vec![" selvi was in Dusk Ridge.\n"]
        );
        let abstain = accepted(Intent::Abstain, true, "selvi", "Dusk Ridge");
        assert_eq!(abstain, vec![" Unknown.\n"]);
        assert!(!accepts(&abstain, " Dusk Ridge.\n"));
        assert!(!accepts(&abstain, " selvi was in Dusk Ridge.\n"));
        assert!(!accepts(&[], "anything"));
    }
}
