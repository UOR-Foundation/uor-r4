//! Opt-in actual-artifact checks for lowercase reverse-start span transfer.
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
    let mut initial = snapshot(&session);
    for key in ["artifact_cid", "control", "work"] {
        initial.as_object_mut().unwrap().remove(key);
    }
    let mut output = Vec::new();
    for _ in 0..96 {
        let token = session.predict(model).unwrap().token;
        session.observe(model, token).unwrap();
        output.push(token);
        if token == EOS {
            break;
        }
    }
    let mut final_state = snapshot(&session);
    for key in ["artifact_cid", "control", "work"] {
        final_state.as_object_mut().unwrap().remove(key);
    }
    (output, initial, final_state)
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
        let stored = if record["span"].is_object() {
            &record["span"]
        } else {
            &record["value"]
        };
        assert_eq!(atom(stored), value);
        if value.split_ascii_whitespace().count() > 1 {
            assert!(
                record["span"].is_object(),
                "multiword old value requires a complete stored span"
            );
        }
        assert_eq!(record["conflict"], false);
        let phrase = format!("{value} holds {owner}");
        let source_start = prompt
            .find(&phrase)
            .expect("authored reverse occurrence absent");
        assert_eq!(
            record["value"]["byte_end"],
            (source_start + value.len() - 1) as u64,
            "refining start must retain the exact original reverse endpoint"
        );
        assert_eq!(
            record["owner"]["byte_end"],
            (source_start + phrase.len() - 1) as u64,
            "refining start must retain the exact original owner occurrence"
        );
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

fn check_records(session: &Session, chains: &[(&str, &str, &str)]) {
    let wire = snapshot(session);
    let relations = &wire["values"]["relations"];
    let records: Vec<_> = relations["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
        .collect();
    assert_eq!(records.len(), chains.len() * 2);
    for (i, (owner, old, new)) in chains.iter().enumerate() {
        for (offset, value) in [(1, *old), (2, *new)] {
            let id = (i as u64) * 2 + offset;
            let record = records.iter().find(|r| r["id"] == id).unwrap();
            let stored = if record["span"].is_object() {
                &record["span"]
            } else {
                &record["value"]
            };
            assert_eq!(atom(&record["owner"]), *owner);
            assert_eq!(atom(stored), value);
            assert_eq!(record["action"], offset);
            assert_eq!(record["previous"], if offset == 1 { 0 } else { id - 1 });
            assert_eq!(record["conflict"], false);
        }
    }
    let directory: Vec<_> = relations["directory"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_u64)
        .filter(|id| *id != 0)
        .collect();
    assert_eq!(
        directory,
        (1..=chains.len())
            .map(|i| (i as u64) * 2)
            .collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "requires R4_REVERSE_START_MODEL; charged full-artifact reverse-start cases"]
fn native_reverse_start_actual_checkpoint_and_identity() {
    let bytes = std::fs::read(std::env::var("R4_REVERSE_START_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    assert_eq!(model.to_bytes().unwrap(), bytes);
    let wire: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["relation_start_refinement"].is_object());
    let parent = model.without_relation_start_refinement().unwrap();
    assert_eq!(
        parent.artifact_cid(),
        "blake3:f76e745256c4589ab1e6180e065e047e8be06bb1e13eddb811bc598737cb3a8f"
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
    assert_eq!(
        wire["relation_start_refinement"]["parent_artifact"],
        parent.artifact_cid()
    );
    let old_bias = wire["relation_start"]["biases"][0].as_i64().unwrap();
    for (pointer, new, expected_error) in [
        (
            "/relation_start_refinement/parent_artifact",
            serde_json::json!(format!("blake3:{}", "0".repeat(64))),
            Some("relation start refinement frozen parent differs"),
        ),
        (
            "/relation_start/biases/0",
            serde_json::json!(if old_bias == 32 {
                old_bias - 1
            } else {
                old_bias + 1
            }),
            Some("relation start refinement changed frozen parameters"),
        ),
        (
            "/relation_start/codes/0/roots/0",
            serde_json::json!(120),
            None,
        ),
    ] {
        let mut changed = wire.clone();
        *changed
            .pointer_mut(pointer)
            .expect("required refinement field") = new;
        let error = Model::from_bytes(&serde_json::to_vec(&changed).unwrap())
            .err()
            .expect("structural mutation must reject");
        if let Some(expected) = expected_error {
            assert!(
                error.to_string().contains(expected),
                "wrong rejection at {pointer}: {error}"
            );
        }
    }
    drop(wire);
    drop(bytes);
    let mut cases = Vec::new();
    for (words, old) in ["moss", "moss dale", "moss dale bank"]
        .into_iter()
        .enumerate()
    {
        for evicted in [false, true] {
            cases.push((
                format!("lowercase_{}_evicted_{evicted}", words + 1),
                vec![("selvi", old, "Copper Vale")],
                0,
                evicted,
            ));
        }
    }
    cases.push((
        "competing_first".into(),
        vec![
            ("selvi", "moss dale", "Copper Vale"),
            ("tilva", "pine cove bank", "Birch Grove"),
        ],
        0,
        false,
    ));
    cases.push((
        "competing_second_evicted".into(),
        vec![
            ("selvi", "moss dale bank", "Copper Vale"),
            ("tilva", "pine cove", "Birch Grove"),
        ],
        1,
        true,
    ));
    let mut inputs = 0;
    let mut outputs = 0;
    for (index, (label, chains, queried, evicted)) in cases.into_iter().enumerate() {
        let facts = chains
            .iter()
            .map(|(o, old, new)| format!("{old} holds {o}. {o} now in {new}."))
            .collect::<Vec<_>>()
            .join(" ");
        let (owner, old, current) = chains[queried];
        let previous = (queried as u64) * 2 + 1;
        let query = if index % 2 == 0 {
            format!("Where was {owner} before? Answer:")
        } else {
            format!("What was the previous location of {owner}? Answer:")
        };
        let padding = if evicted {
            "oak ash elm ".repeat(12)
        } else {
            String::new()
        };
        let prompt = format!("Record: {facts} {padding}{query}");
        assert_eq!(raw(&model,&prompt,Control::RelationStartRefinementDisabled),raw(&parent,&prompt,Control::Full),
            "disabled refinement must equal full f76 output/EOS and complete causal states on {label}");
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        turn(
            &model,
            &mut session,
            &prompt,
            &format!(" {old}.\n"),
            Some((previous, owner, old)),
            &mut inputs,
            &mut outputs,
        );
        check_records(&session, &chains);
        turn(
            &model,
            &mut session,
            &format!("Where is {owner}? Answer:"),
            &format!(" {current}.\n"),
            None,
            &mut inputs,
            &mut outputs,
        );
        check_records(&session, &chains);
        turn(&model,&mut session,"User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:","18.\n",None,&mut inputs,&mut outputs);
        println!("reverse-start case={label}; old={old:?}; previous_record={previous}; current={current:?}; complete span/versions/checkpoint identities; independent sum=18");
    }
    println!("actual reverse-start artifact={}; parent={}; sequences=8; input_checkpoint_positions={inputs}; output_checkpoint_positions={outputs}; disabled causal-state comparisons=8; no allocation/energy measurement",model.artifact_cid(),parent.artifact_cid());
}
