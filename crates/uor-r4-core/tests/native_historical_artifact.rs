//! Opt-in actual-artifact checks for authored before/previous queries.
//! No fitting, serving templates, allocator instrumentation or general-language claim.
#![forbid(unsafe_code)]

use serde_json::Value;
use uor_r4_core::native_geometric::{Control, Model, Session, WordCopyAction, BOS, EOS};

fn snapshot(session: &Session) -> Value {
    serde_json::from_slice(&session.checkpoint().unwrap()).unwrap()
}

fn same(left: &Session, right: &Session) {
    let mut left = snapshot(left);
    let mut right = snapshot(right);
    left.as_object_mut().unwrap().remove("work");
    right.as_object_mut().unwrap().remove("work");
    assert_eq!(left, right, "full causal checkpoint state differs");
}

fn atom(value: &Value) -> String {
    let bytes = value["bytes"].as_array().unwrap()[..value["len"].as_u64().unwrap() as usize]
        .iter()
        .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}

fn raw(model: &Model, prompt: &str, control: Control) -> (Vec<u32>, Value, Value) {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(prompt).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    let initial = snapshot(&session)["values"]["relations"].clone();
    let mut output = Vec::new();
    for _ in 0..96 {
        let token = session.predict(model).unwrap().token;
        session.observe(model, token).unwrap();
        output.push(token);
        if token == EOS {
            break;
        }
    }
    (
        output,
        initial,
        snapshot(&session)["values"]["relations"].clone(),
    )
}

fn turn(
    model: &Model,
    session: &mut Session,
    prompt: &str,
    target: &str,
    historical: Option<(u64, &str, &str)>,
    input_positions: &mut usize,
    output_positions: &mut usize,
) {
    if session.needs_input_boundary() {
        let before_boundary = snapshot(session);
        session.end_response(model).unwrap();
        let after_boundary = snapshot(session);
        assert_eq!(
            after_boundary["values"]["query_boundary"], before_boundary["values"]["seen"],
            "input boundary must advance to the actual observed token position"
        );
        assert_eq!(
            after_boundary["values"]["relations"], before_boundary["values"]["relations"],
            "scoping the next query must preserve exact committed history"
        );
        assert_eq!(
            after_boundary["values"]["lexemes"]["recent"],
            before_boundary["values"]["lexemes"]["recent"],
            "query scoping must retain the source window"
        );
    }
    let input = model.encode(prompt).unwrap();
    assert!(input.len() <= 512);
    for token in input {
        let mut restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        session.observe(model, token).unwrap();
        restored.observe(model, token).unwrap();
        same(session, &restored);
        *input_positions += 1;
    }
    let mut restored = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    session.begin_response(model).unwrap();
    restored.begin_response(model).unwrap();
    same(session, &restored);
    let initial = snapshot(session);
    let relations = &initial["values"]["relations"];
    let endpoint = historical.map(|(id, owner, value)| {
        let record = relations["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(atom(&record["owner"]), owner);
        assert_eq!(atom(&record["span"]), value);
        assert_eq!(record["conflict"], false);
        let current_ids: Vec<_> = relations["directory"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_u64)
            .filter(|n| *n != 0)
            .collect();
        assert!(
            !current_ids.contains(&id),
            "historical target must not be current"
        );
        let current = relations["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| {
                current_ids.contains(&r["id"].as_u64().unwrap()) && atom(&r["owner"]) == owner
            })
            .unwrap();
        assert_eq!(
            current["previous"], id,
            "target is the exact previous version of queried owner's current record"
        );
        (
            id,
            record["value"]["end"].as_u64().unwrap(),
            record["value"]["byte_end"].as_u64().unwrap(),
        )
    });
    let mut selected = false;
    let mut output = Vec::new();
    let mut eos = false;
    for _ in 0..96 {
        let mut restored = model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
        let predicted = restored.predict(model).unwrap();
        assert_eq!(predicted, restored.predict(model).unwrap());
        let actual = session.predict(model).unwrap();
        assert_eq!(actual, session.predict(model).unwrap());
        assert_eq!(actual, predicted);
        if let Some((id, end, byte_end)) = endpoint {
            selected |= session.word_copy_decision().is_some_and(|d| {
                matches!(d.action, WordCopyAction::Prepare | WordCopyAction::Read)
                    && d.source_end == end
                    && d.source_byte_end == byte_end
            }) || session.field_composition_decision().is_some_and(|d| {
                d.field != 0
                    && d.anchor.relation_id == id
                    && d.anchor.source_end == end
                    && d.anchor.source_byte_end == byte_end
            });
        }
        session.observe(model, actual.token).unwrap();
        restored.observe(model, predicted.token).unwrap();
        same(session, &restored);
        assert_eq!(
            snapshot(session)["values"]["relations"],
            *relations,
            "generation changed committed records"
        );
        *output_positions += 1;
        if actual.token == EOS {
            eos = true;
            break;
        }
        output.push(actual.token);
    }
    assert!(eos, "response did not terminate");
    assert_eq!(
        model.decode(&output).unwrap(),
        target.as_bytes(),
        "prompt {prompt:?}"
    );
    if historical.is_some() {
        assert!(
            selected,
            "historical output did not expose the exact previous occurrence endpoint"
        );
    }
}

#[test]
#[ignore = "requires R4_HISTORICAL_MODEL; charged actual-artifact before/previous cases"]
fn native_historical_actual_checkpoint_and_identity() {
    let bytes = std::fs::read(std::env::var("R4_HISTORICAL_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    assert_eq!(
        model.to_bytes().unwrap(),
        bytes,
        "actual artifact roundtrip"
    );
    let wire: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["historical_read"].is_object());
    assert_eq!(
        wire["historical_read"]["query_scope"], 2,
        "current-turn qualification requires the explicit scope-2 artifact"
    );
    let parent = model.without_historical_read().unwrap();
    assert_eq!(
        parent.artifact_cid(),
        "blake3:4d81c3b003499648e47248fb8d5d4a50622f6cc0fbad83f0baad2f6cf25b3fac"
    );
    assert_eq!(
        wire["historical_read"]["router"]["parent_artifact"],
        parent.artifact_cid()
    );
    let parent_bytes = parent.to_bytes().unwrap();
    assert_eq!(
        Model::from_bytes(&parent_bytes)
            .unwrap()
            .to_bytes()
            .unwrap(),
        parent_bytes
    );
    drop(parent_bytes);
    for (pointer, new) in [
        ("/historical_read/query_scope", serde_json::json!(3)),
        (
            "/historical_read/router/parent_artifact",
            serde_json::json!(format!("blake3:{}", "0".repeat(64))),
        ),
        (
            "/historical_read/router/codes/0/roots/0",
            serde_json::json!(120),
        ),
    ] {
        let mut changed = wire.clone();
        *changed
            .pointer_mut(pointer)
            .expect("required historical witness field") = new;
        assert!(
            Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "malformed historical field must reject: {pointer}"
        );
    }
    drop(wire);
    drop(bytes);
    let mut input_positions = 0;
    let mut output_positions = 0;
    for (label, facts, padding, owner, previous, old, current) in [
        ("reverse", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.", String::new(), "selvi", 1, "Dusk Ridge", "Copper Vale"),
        ("evicted", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.", "oak ash elm ".repeat(12), "selvi", 1, "Dusk Ridge", "Copper Vale"),
        ("two_owners", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.", String::new(), "selvi", 1, "Dusk Ridge", "Copper Vale"),
        ("two_revisions", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field.", String::new(), "selvi", 2, "Copper Vale", "Amber Field"),
    ] {
        for query in [format!("Where was {owner} before? Answer:"), format!("What was the previous location of {owner}? Answer:")] {
        let prompt = format!("{facts} {padding}{query}");
        if label == "reverse" {
            assert_eq!(raw(&model, &prompt, Control::HistoricalReadDisabled), raw(&parent, &prompt, Control::Full),
                "disabled historical contribution must preserve parent output/EOS and records");
        }
        let mut session = model.session(Control::Full).unwrap(); session.observe(&model, BOS).unwrap();
        turn(&model, &mut session, &prompt, &format!(" {old}.\n"), Some((previous, owner, old)), &mut input_positions, &mut output_positions);
        turn(&model, &mut session, &format!("Where is {owner}? Answer:"), &format!(" {current}.\n"), None, &mut input_positions, &mut output_positions);
        turn(&model, &mut session, "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", "18.\n", None, &mut input_positions, &mut output_positions);
        println!("historical case={label}; query={query:?}; previous_record={previous}; previous={old:?}; current={current:?}; independent sum=18; checkpoint identity exact");
        }
    }
    println!("actual historical artifact={}; input_checkpoint_positions={input_positions}; output_checkpoint_positions={output_positions}; exact previous occurrence and records; no allocation or energy measurement", model.artifact_cid());
}

/// Optional preserved scope-1 artifact: exact legacy bytes and its recorded
/// before -> bare-current negative stay readable under the scope-2 implementation.
#[test]
#[ignore = "requires R4_HISTORICAL_LEGACY_MODEL; preserved scope-1 artifact replay"]
fn native_historical_legacy_byte_roundtrip_and_observed_negative() {
    let bytes = std::fs::read(std::env::var("R4_HISTORICAL_LEGACY_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    let wire: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["historical_read"].is_object());
    assert!(
        wire["historical_read"].get("query_scope").is_none(),
        "preserved legacy artifact must keep omitted default scope exactly"
    );
    assert_eq!(
        model.to_bytes().unwrap(),
        bytes,
        "legacy bytes must not acquire scope-2 fields"
    );
    drop(wire);
    drop(bytes);
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    let mut inputs = 0;
    let mut outputs = 0;
    turn(
        &model,
        &mut session,
        "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Where was selvi before? Answer:",
        " Dusk Ridge.\n",
        Some((1, "selvi", "Dusk Ridge")),
        &mut inputs,
        &mut outputs,
    );
    turn(
        &model,
        &mut session,
        "Where is selvi? Answer:",
        " Dusk Ridge.\n",
        None,
        &mut inputs,
        &mut outputs,
    );
    println!("legacy historical artifact={}; byte-exact roundtrip; preserved before->bare-current negative Dusk Ridge instead of Copper Vale; input_checkpoint_positions={inputs}; output_checkpoint_positions={outputs}; negative preservation is not current-answer correctness",model.artifact_cid());
}
