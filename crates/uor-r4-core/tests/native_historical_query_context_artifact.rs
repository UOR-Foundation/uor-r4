//! Opt-in actual historical query-context and causal checkpoint checks.
//! Expected text belongs only to assertions; no training or allocator instrumentation.
#![forbid(unsafe_code)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::{Control, Model, Session, BOS, EOS};

fn state(s: &Session) -> Value {
    serde_json::from_slice(&s.checkpoint().unwrap()).unwrap()
}
fn same(a: &Session, b: &Session) {
    let mut a = state(a);
    let mut b = state(b);
    a.as_object_mut().unwrap().remove("work");
    b.as_object_mut().unwrap().remove("work");
    assert_eq!(
        a, b,
        "historical query context requires complete causal checkpoint equivalence"
    );
}
fn atom(v: &Value) -> String {
    String::from_utf8(
        v["bytes"].as_array().unwrap()[..v["len"].as_u64().unwrap() as usize]
            .iter()
            .map(|n| n.as_u64().unwrap() as u8)
            .collect(),
    )
    .unwrap()
}
fn comparable(mut v: Value) -> Value {
    for k in ["artifact_cid", "control", "work"] {
        v.as_object_mut().unwrap().remove(k);
    }
    v
}
fn raw_sequence(
    model: &Model,
    prompts: &[&str],
    control: Control,
) -> Vec<(Vec<u32>, Value, Value)> {
    let mut s = model.session(control).unwrap();
    s.observe(model, BOS).unwrap();
    let mut turns = Vec::new();
    for prompt in prompts {
        if s.needs_input_boundary() {
            s.end_response(model).unwrap();
        }
        for t in model.encode(prompt).unwrap() {
            s.observe(model, t).unwrap();
        }
        s.begin_response(model).unwrap();
        let initial = comparable(state(&s));
        let mut out = Vec::new();
        let mut eos = false;
        for _ in 0..96 {
            let t = s.predict(model).unwrap().token;
            s.observe(model, t).unwrap();
            out.push(t);
            if t == EOS {
                eos = true;
                break;
            }
        }
        assert!(eos, "disabled/parent sequence must terminate: {prompt:?}");
        turns.push((out, initial, comparable(state(&s))));
    }
    turns
}

struct ExpectedField<'a> {
    record: u64,
    owner: &'a str,
    value: &'a str,
    current_proof: Option<u64>,
}
fn turn(
    model: &Model,
    s: &mut Session,
    prompt: &str,
    target: &str,
    field: Option<ExpectedField<'_>>,
    inputs: &mut usize,
    outputs: &mut usize,
    proof_checks: &mut usize,
) {
    if s.needs_input_boundary() {
        let before = state(s);
        s.end_response(model).unwrap();
        let after = state(s);
        assert_eq!(after["values"]["relations"], before["values"]["relations"]);
        assert_eq!(after["values"]["query_boundary"], before["values"]["seen"]);
    }
    let tokens = model.encode(prompt).unwrap();
    assert!(tokens.len() <= 512);
    for t in tokens {
        let mut r = model.restore_session(&s.checkpoint().unwrap()).unwrap();
        s.observe(model, t).unwrap();
        r.observe(model, t).unwrap();
        same(s, &r);
        *inputs += 1;
    }
    let mut r = model.restore_session(&s.checkpoint().unwrap()).unwrap();
    s.begin_response(model).unwrap();
    r.begin_response(model).unwrap();
    same(s, &r);
    let initial = state(s);
    let relations = &initial["values"]["relations"];
    let endpoint = field.as_ref().map(|f| {
        let records = relations["records"].as_array().unwrap();
        let r = records.iter().find(|r| r["id"] == f.record).unwrap();
        assert_eq!(atom(&r["owner"]), f.owner);
        assert_eq!(
            atom(if r["span"].is_object() {
                &r["span"]
            } else {
                &r["value"]
            }),
            f.value
        );
        assert_eq!(r["conflict"], false);
        let directory = relations["directory"].as_array().unwrap();
        if let Some(id) = f.current_proof {
            let current = records.iter().find(|r| r["id"] == id).unwrap();
            assert!(directory.contains(&json!(id)));
            assert!(!directory.contains(&json!(f.record)));
            assert_eq!(current["previous"], f.record);
            assert_eq!(current["action"], 2);
            assert_eq!(current["conflict"], false);
            assert_eq!(atom(&current["owner"]), f.owner);
        } else {
            assert!(directory.contains(&json!(f.record)));
        }
        (
            r["value"]["end"].as_u64().unwrap(),
            r["value"]["byte_end"].as_u64().unwrap(),
        )
    });
    let mut out = Vec::new();
    let mut eos = false;
    let mut owner_seen = false;
    let mut value_seen = false;
    let mut checked_proof = false;
    for _ in 0..96 {
        let mut r = model.restore_session(&s.checkpoint().unwrap()).unwrap();
        let predicted = r.predict(model).unwrap();
        assert_eq!(predicted, r.predict(model).unwrap());
        let p = s.predict(model).unwrap();
        assert_eq!(p, s.predict(model).unwrap());
        assert_eq!(p, predicted);
        if let (Some(f), Some(d)) = (
            &field,
            s.field_composition_decision().filter(|d| d.field != 0),
        ) {
            assert_eq!(d.anchor.relation_id, f.record);
            assert_eq!(d.anchor.current_revision, f.current_proof);
            let (end, byte_end) = endpoint.unwrap();
            assert_eq!(d.anchor.source_end, end);
            assert_eq!(d.anchor.source_byte_end, byte_end);
            owner_seen |= d.field == 1;
            value_seen |= d.field == 2;
        }
        s.observe(model, p.token).unwrap();
        r.observe(model, predicted.token).unwrap();
        same(s, &r);
        let after = state(s);
        assert_eq!(after["values"]["relations"], *relations);
        *outputs += 1;
        // Check an active owner-field cursor; no transient prediction is serialized.
        if let Some(f) = &field {
            if f.current_proof.is_some()
                && after["field_composition"]["read"]["field"] == 1
                && !checked_proof
            {
                validate_and_corrupt_proof(model, &after, f.current_proof.unwrap());
                checked_proof = true;
                *proof_checks += 1;
            }
        }
        if p.token == EOS {
            eos = true;
            break;
        }
        out.push(p.token);
    }
    assert!(eos);
    assert_eq!(
        model.decode(&out).unwrap(),
        target.as_bytes(),
        "prompt {prompt:?}"
    );
    if let Some(f) = field {
        assert!(
            owner_seen && value_seen,
            "must read both exact record fields"
        );
        if f.current_proof.is_some() {
            assert!(
                checked_proof,
                "missing mid-owner historical proof checkpoint"
            );
        }
    }
}

fn validate_and_corrupt_proof(model: &Model, wire: &Value, current: u64) {
    assert_eq!(
        wire["field_composition"]["anchor"]["current_revision"],
        current
    );
    let mut missing = wire.clone();
    missing["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("current_revision");
    assert!(
        model
            .restore_session(&serde_json::to_vec(&missing).unwrap())
            .is_err(),
        "missing proof cannot turn an old selected record into a current anchor"
    );
    let mut wrong = wire.clone();
    wrong["field_composition"]["anchor"]["current_revision"] =
        json!(if current == 2 { 4 } else { 2 });
    assert!(
        model
            .restore_session(&serde_json::to_vec(&wrong).unwrap())
            .is_err(),
        "wrong current revision proof must reject"
    );
    let mut link = wire.clone();
    let record = link["values"]["relations"]["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["id"] == current)
        .unwrap();
    record["previous"] = json!(0);
    assert!(
        model
            .restore_session(&serde_json::to_vec(&link).unwrap())
            .is_err(),
        "broken live previous link must reject historical field continuation"
    );
}

#[test]
#[ignore = "requires R4_HISTORICAL_QUERY_CONTEXT_MODEL; charged actual historical query-context artifact"]
fn native_historical_query_context_actual_checkpoint_and_identity() {
    let bytes = std::fs::read(std::env::var("R4_HISTORICAL_QUERY_CONTEXT_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    assert_eq!(model.to_bytes().unwrap(), bytes);
    let parent = model.without_historical_query_context().unwrap();
    assert_eq!(
        parent.artifact_cid(),
        "blake3:8cacb2fe0fbd4a79a8be629096b73458ea5bbd23c3dc84fa871a283437b7df81"
    );
    let parent_bytes = parent.to_bytes().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&parent_bytes)),
        "607e196b880a1bba8b1a1aa1a38224a18ea181ac8ca7138f9ad550ec9bad9cde"
    );
    assert_eq!(
        Model::from_bytes(&parent_bytes)
            .unwrap()
            .to_bytes()
            .unwrap(),
        parent_bytes
    );
    drop(parent_bytes);
    let wire: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["historical_query_context"].is_object());
    assert_eq!(
        wire["historical_query_context"]["parent_artifact"],
        parent.artifact_cid()
    );
    let mut changed = wire.clone();
    changed["historical_query_context"]["unexpected_field"] = true.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err());
    drop(changed);
    let mut changed = wire.clone();
    changed["historical_query_context"]["parent_artifact"] =
        json!(format!("blake3:{}", "0".repeat(64)));
    assert!(
        Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err(),
        "outer historical-query parent commitment must reject mutation"
    );
    drop(changed);
    let mut changed = wire.clone();
    let root = &mut changed["historical_query_context"]["active"]["router"]["codes"][0]["roots"][0];
    let prior = root.as_u64().unwrap();
    assert!(prior < 120);
    *root = json!((prior + 1) % 120);
    let error = Model::from_bytes(&serde_json::to_vec(&changed).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("historical query identity differs")
            || error.contains("historical query changed frozen parameters"),
        "a valid active root change must break artifact identity or exposure-only freezing: {error}"
    );
    drop(changed);
    let mut changed = wire.clone();
    let root = &mut changed["historical_read"]["router"]["codes"][0]["roots"][0];
    let prior = root.as_u64().unwrap();
    assert!(prior < 120);
    *root = json!((prior + 1) % 120);
    let error = Model::from_bytes(&serde_json::to_vec(&changed).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("historical query frozen parent differs"),
        "retained inner reader mutation must reject at the outer parent boundary: {error}"
    );
    drop(changed);
    drop(wire);
    drop(bytes);
    let mut inputs = 0;
    let mut outputs = 0;
    let mut proof_checks = 0;
    let mut cases = vec![
        (
            "reverse",
            "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.",
            String::new(),
            "selvi",
            "Dusk Ridge",
            1,
            "Copper Vale",
            2,
        ),
        (
            "evicted",
            "Record: Dusk Ridge holds selvi. selvi now in Copper Vale.",
            "oak ash elm ".repeat(12),
            "selvi",
            "Dusk Ridge",
            1,
            "Copper Vale",
            2,
        ),
        (
            "two_revisions",
            "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field.",
            String::new(),
            "selvi",
            "Copper Vale",
            2,
            "Amber Field",
            3,
        ),
    ];
    // Independently query both owners in both committed record orders. Distinct
    // values and exact IDs reject first-owner, latest-record and spelling shortcuts.
    for (label, facts, owner, old, previous, current, current_id) in [
        ("selvi_first", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.", "selvi", "Dusk Ridge", 1, "Copper Vale", 2),
        ("tilva_second", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. Silver Cove holds tilva. tilva now in Birch Grove.", "tilva", "Silver Cove", 3, "Birch Grove", 4),
        ("tilva_first", "Record: Silver Cove holds tilva. tilva now in Birch Grove. Dusk Ridge holds selvi. selvi now in Copper Vale.", "tilva", "Silver Cove", 1, "Birch Grove", 2),
        ("selvi_second", "Record: Silver Cove holds tilva. tilva now in Birch Grove. Dusk Ridge holds selvi. selvi now in Copper Vale.", "selvi", "Dusk Ridge", 3, "Copper Vale", 4),
    ] {
        for evicted in [false, true] {
            cases.push((label, facts, if evicted { "oak ash elm ".repeat(12) } else { String::new() }, owner, old, previous, current, current_id));
        }
    }
    let sequence_count = cases.len() * 2;
    assert_eq!(sequence_count, 22);
    for (label, facts, padding, owner, old, previous, current, current_id) in cases {
        for instruction in ["Name the owner first.", "State the owner first."] {
            let prompt = format!(
                "{facts} {padding}What was the previous location of {owner}? {instruction} Answer:"
            );
            let current_prompt = format!("Where is {owner}? Name the owner first. Answer:");
            let sum_prompt = "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
            let prompts = [prompt.as_str(), current_prompt.as_str(), sum_prompt];
            assert_eq!(raw_sequence(&model, &prompts, Control::HistoricalQueryContextDisabled), raw_sequence(&parent, &prompts, Control::Full), "disabled outer witness must preserve parent output and causal state across all three turns");
            let mut s = model.session(Control::Full).unwrap();
            s.observe(&model, BOS).unwrap();
            turn(
                &model,
                &mut s,
                &prompt,
                &format!(" {owner} was in {old}.\n"),
                Some(ExpectedField {
                    record: previous,
                    owner,
                    value: old,
                    current_proof: Some(current_id),
                }),
                &mut inputs,
                &mut outputs,
                &mut proof_checks,
            );
            turn(
                &model,
                &mut s,
                &current_prompt,
                &format!(" {owner} is in {current}.\n"),
                Some(ExpectedField {
                    record: current_id,
                    owner,
                    value: current,
                    current_proof: None,
                }),
                &mut inputs,
                &mut outputs,
                &mut proof_checks,
            );
            turn(
                &model,
                &mut s,
                sum_prompt,
                "18.\n",
                None,
                &mut inputs,
                &mut outputs,
                &mut proof_checks,
            );
            println!("historical-query {label}; evicted={}; owner={owner}; {instruction}; historical record={previous}, current proof={current_id}; historical/current owner fields and independent sum exact", !padding.is_empty());
        }
    }
    assert_eq!(proof_checks, sequence_count);
    println!("actual historical-query-context artifact={}; sequences={sequence_count}; two_owner_sequences=16; input_checkpoint_positions={inputs}; output_checkpoint_positions={outputs}; active_proof_checkpoint_cases={proof_checks}; long previous-location owner-first exact; disabled parent equivalence across {} turns; no allocation/energy measurement", model.artifact_cid(), sequence_count * 3);
}
