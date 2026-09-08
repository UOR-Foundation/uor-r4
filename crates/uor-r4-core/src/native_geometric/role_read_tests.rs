//! Real-fit state/serialization checks; these are construction laws, not transfer.
use super::*;
#[allow(dead_code)]
mod fixture {
    use crate::native_geometric as native;
    include!("../../tests/support/native_word_copy_fixture.rs");
}

pub(super) fn fitted() -> &'static Model {
    static MODEL: std::sync::OnceLock<Model> = std::sync::OnceLock::new();
    MODEL.get_or_init(|| {
        let mut docs = Vec::new();
        for (i, name) in ["alpha", "bravo", "cedar", "delta"].iter().enumerate() {
            for (j, prompt, response) in [
                (
                    0,
                    format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    "),
                    format!("{name}\n}}\n"),
                ),
                (
                    1,
                    format!("holder in {name}. Where is holder? Answer:"),
                    format!(" {name}.\n"),
                ),
                (
                    2,
                    format!("{name} in city. Where is missing? Answer:"),
                    " Unknown.\n".into(),
                ),
            ] {
                docs.push(ValueExample {
                    id: format!("role-state-{i}-{j}"),
                    prompt,
                    response,
                });
            }
        }
        let (model, report) = fixture::fitted_composed()
            .fit_role_read(
                &docs,
                ResponseEntryFitConfig {
                    epochs: 64,
                    ..ResponseEntryFitConfig::default()
                },
            )
            .unwrap();
        assert_eq!(report["fit_correct"], report["frames"]);
        Model::from_bytes(&model.to_bytes().unwrap()).unwrap()
    })
}

fn begin(model: &Model, prompt: &str) -> Session {
    let mut s = model.session(Control::Full).unwrap();
    s.observe(model, BOS).unwrap();
    for t in model.encode(prompt).unwrap() {
        s.observe(model, t).unwrap();
    }
    s.begin_response(model).unwrap();
    s
}

#[test]
fn native_role_read_commits_only_observed_entry_and_restores_before_byte_zero() {
    let model = fitted();
    for (prompt, action) in [
        (fixture::COPY_PROMPT, WordCopyAction::Read),
        (
            "holder in alpha. Where is holder? Answer:",
            WordCopyAction::Prepare,
        ),
        (
            "alpha in city. Where is missing? Answer:",
            WordCopyAction::NoRead,
        ),
    ] {
        let mut s = begin(model, prompt);
        let before = s.checkpoint().unwrap();
        let first = s.predict(model).unwrap();
        let decision = s.word_copy_decision().unwrap();
        assert_eq!(decision.action, action);
        let before_wire: serde_json::Value = serde_json::from_slice(&before).unwrap();
        let after_wire: serde_json::Value =
            serde_json::from_slice(&s.checkpoint().unwrap()).unwrap();
        assert_eq!(
            after_wire["word_copy"], before_wire["word_copy"],
            "selection is transient; work counters still advance"
        );
        let mut mismatch = model.restore_session(&before).unwrap();
        mismatch.predict(model).unwrap();
        mismatch.observe(model, u32::from(b'?') + 2).unwrap();
        assert!(mismatch.word_copy.as_ref().unwrap().read_commit.is_none());
        assert!(mismatch.word_copy.as_ref().unwrap().origin.is_none());
        s.observe(model, first.token).unwrap();
        let snapshot = s.checkpoint().unwrap();
        let mut restored = model.restore_session(&snapshot).unwrap();
        assert_eq!(restored.word_copy, s.word_copy);
        let mut bad: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
        bad["word_copy"]["read_commit"]["source_end"] = serde_json::json!(u64::MAX);
        assert!(model
            .restore_session(&serde_json::to_vec(&bad).unwrap())
            .is_err());
        let mut missing: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
        missing["word_copy"]
            .as_object_mut()
            .unwrap()
            .remove("read_commit");
        assert!(model
            .restore_session(&serde_json::to_vec(&missing).unwrap())
            .is_err());
        for _ in 0..3 {
            let next = s.predict(model).unwrap();
            assert_eq!(restored.predict(model).unwrap(), next);
            assert_eq!(restored.word_copy_decision(), s.word_copy_decision());
            s.observe(model, next.token).unwrap();
            restored.observe(model, next.token).unwrap();
            assert_eq!(restored.word_copy, s.word_copy);
            model.restore_session(&s.checkpoint().unwrap()).unwrap();
            if next.token == EOS {
                break;
            }
        }
    }
}

#[test]
fn native_role_read_validates_quantized_rows_and_parent_identity() {
    let model = fitted();
    for change in 0..3 {
        let mut bad = model.clone();
        let head = bad
            .response_entry
            .as_mut()
            .unwrap()
            .copy
            .as_mut()
            .unwrap()
            .role_read
            .as_mut()
            .unwrap();
        match change {
            0 => head.rows[0].weight = 1_000_001,
            1 => head.baseline_artifact = "bad".into(),
            _ => head.dictionary[0].prime += 1,
        }
        bad.refresh_identity().unwrap();
        assert!(bad.validate().is_err());
    }
    let mut changed = model.clone();
    changed
        .response_entry
        .as_mut()
        .unwrap()
        .copy
        .as_mut()
        .unwrap()
        .role_read
        .as_mut()
        .unwrap()
        .rows[0]
        .weight += 1;
    changed.refresh_identity().unwrap();
    assert_ne!(changed.artifact_cid(), model.artifact_cid());
    assert_eq!(
        Model::from_bytes(&changed.to_bytes().unwrap()).unwrap(),
        changed
    );
}

#[test]
fn native_no_read_completion_preserves_parent_and_requires_committed_no_read() {
    let parent = fitted();
    let docs: Vec<_> = ["alpha", "bravo", "cedar", "delta"]
        .into_iter()
        .enumerate()
        .map(|(i, name)| ValueExample {
            id: format!("no-read-suffix-{i}"),
            prompt: format!("13; {name} in city. Where is missing? Answer:"),
            response: " Unknown.\n".into(),
        })
        .collect();
    let (model, report) = parent
        .fit_no_read_completion(&docs, ResponseEntryFitConfig::default())
        .unwrap();
    assert_eq!(report["admitted"].as_array().unwrap().len(), docs.len());
    let mut inherited = model.clone();
    inherited.no_read_completion = None;
    inherited.refresh_identity().unwrap();
    assert_eq!(&inherited, parent);
    let model = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    let mut no_offer = model.clone();
    let head = no_offer.no_read_completion.as_mut().unwrap();
    head.rows.clear();
    head.global_postings.clear();
    no_offer.refresh_identity().unwrap();
    no_offer.validate().unwrap();
    let unchanged = no_offer
        .generate(&docs[0].prompt, 32, Control::Full)
        .unwrap();
    let original = parent.generate(&docs[0].prompt, 32, Control::Full).unwrap();
    assert_eq!(unchanged.text, original.text);
    assert_eq!(
        unchanged.response_entry_trace, original.response_entry_trace,
        "no learned offer must preserve inherited pending decisions and prefix fallback"
    );
    for d in &docs {
        let g = model.generate(&d.prompt, 32, Control::Full).unwrap();
        assert_eq!(g.text, d.response);
        assert_eq!(g.stop, "end_of_document");
    }
    for prompt in [
        fixture::COPY_PROMPT,
        "holder in alpha. Where is holder? Answer:",
        "alpha in city. Where is missing? Answer:",
    ] {
        assert_eq!(
            model.generate(prompt, 32, Control::Full).unwrap().text,
            parent.generate(prompt, 32, Control::Full).unwrap().text
        );
    }
    let mut s = begin(&model, &docs[0].prompt);
    let mut scope_values = s.values.as_ref().unwrap().clone();
    assert!(super::word_copy_runtime::literal_no_read_eligible(
        &scope_values,
        &mut Default::default()
    ));
    scope_values.sources[0].derived = true;
    assert!(!super::word_copy_runtime::literal_no_read_eligible(
        &scope_values,
        &mut Default::default()
    ));
    scope_values.sources.clear();
    assert!(!super::word_copy_runtime::literal_no_read_eligible(
        &scope_values,
        &mut Default::default()
    ));
    let p = s.predict(&model).unwrap();
    assert_eq!(
        s.word_copy_decision().unwrap().action,
        WordCopyAction::NoRead
    );
    let before = s.checkpoint().unwrap();
    s.observe(&model, p.token).unwrap();
    let cp = s.checkpoint().unwrap();
    let mut restored = model.restore_session(&cp).unwrap();
    assert_eq!(
        s.predict(&model).unwrap(),
        restored.predict(&model).unwrap()
    );
    let mut mismatch = model.restore_session(&before).unwrap();
    mismatch.predict(&model).unwrap();
    mismatch.observe(&model, u32::from(b'?') + 2).unwrap();
    assert!(mismatch.word_copy.as_ref().unwrap().read_commit.is_none());
    for change in 0..4 {
        let mut bad = model.clone();
        let h = bad.no_read_completion.as_mut().unwrap();
        match change {
            0 => h.baseline_artifact = "bad".into(),
            1 => h.rows[0].scores[0].token = BOS,
            2 => h.rows[0].scores[0].score = 1_000_001,
            _ => h.schema = super::response_entry_types::RESPONSE_ENTRY_SCHEMA.into(),
        }
        bad.refresh_identity().unwrap();
        assert!(bad.validate().is_err());
    }
    assert!(model
        .fit_no_read_completion(&docs, ResponseEntryFitConfig::default())
        .is_err());
}
