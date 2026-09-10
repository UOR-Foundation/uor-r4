//! Opt-in actual historical-version-intent artifact, ancestor-path proof and checkpoint checks.
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
        "historical version intent requires complete causal checkpoint equivalence"
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
    depth: Option<u8>,
}
#[allow(clippy::too_many_arguments)]
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
            assert!(directory.contains(&json!(id)));
            assert!(!directory.contains(&json!(f.record)));
            // Walk the exact descending chain from the live head to the selected record.
            let depth = f.depth.unwrap_or(1);
            let mut cursor = records.iter().find(|r| r["id"] == id).unwrap();
            for hop in 0..depth {
                assert_eq!(
                    cursor["action"], 2,
                    "hop {hop} must be an explicit revision"
                );
                assert_eq!(cursor["conflict"], false);
                assert_eq!(atom(&cursor["owner"]), f.owner);
                let previous = cursor["previous"].as_u64().unwrap();
                assert!(previous != 0 && previous < cursor["id"].as_u64().unwrap());
                cursor = records.iter().find(|r| r["id"] == previous).unwrap();
            }
            assert_eq!(cursor["id"], f.record);
            if f.depth.is_some() {
                assert_eq!(
                    cursor["previous"], 0,
                    "initial selection must reach the root"
                );
                assert_eq!(cursor["action"], 1);
            }
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
            assert_eq!(d.anchor.ancestor_depth, f.depth);
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
        if let Some(f) = &field {
            if after["field_composition"]["read"]["field"] == 1 && !checked_proof {
                match (f.current_proof, f.depth) {
                    (Some(id), Some(depth)) => {
                        validate_and_corrupt_ancestor(model, &after, id, depth)
                    }
                    (Some(id), None) => validate_and_corrupt_proof(model, &after, id),
                    (None, _) => validate_and_corrupt_current(model, &after, f.record),
                }
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
        assert!(
            checked_proof,
            "missing mid-owner field checkpoint for record {}",
            f.record
        );
    }
}

fn rejects(model: &Model, wire: &Value, why: &str) {
    assert!(
        model
            .restore_session(&serde_json::to_vec(wire).unwrap())
            .is_err(),
        "{why}"
    );
}

fn validate_and_corrupt_current(model: &Model, wire: &Value, current: u64) {
    assert!(wire["field_composition"]["anchor"]["current_revision"].is_null());
    assert!(wire["field_composition"]["anchor"]["ancestor_depth"].is_null());
    let mut wrong = wire.clone();
    wrong["field_composition"]["anchor"]["relation_id"] = json!(0);
    rejects(model, &wrong, "zero relation");
    let mut endpoint = wire.clone();
    let end = endpoint["field_composition"]["anchor"]["source_byte_end"]
        .as_u64()
        .unwrap();
    endpoint["field_composition"]["anchor"]["source_byte_end"] = json!(end + 1);
    rejects(model, &endpoint, "moved endpoint");
    let mut proof = wire.clone();
    proof["field_composition"]["anchor"]["current_revision"] = json!(current);
    rejects(model, &proof, "spurious revision proof on a current anchor");
    let mut depth = wire.clone();
    depth["field_composition"]["anchor"]["ancestor_depth"] = json!(2);
    rejects(model, &depth, "ancestor depth without a revision proof");
}

fn validate_and_corrupt_proof(model: &Model, wire: &Value, current: u64) {
    assert_eq!(
        wire["field_composition"]["anchor"]["current_revision"],
        current
    );
    assert!(wire["field_composition"]["anchor"]["ancestor_depth"].is_null());
    let mut missing = wire.clone();
    missing["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("current_revision");
    rejects(
        model,
        &missing,
        "missing proof cannot turn an old record into a current anchor",
    );
    let mut wrong = wire.clone();
    wrong["field_composition"]["anchor"]["current_revision"] = json!(current + 1);
    rejects(model, &wrong, "wrong current revision proof must reject");
    let mut one = wire.clone();
    one["field_composition"]["anchor"]["ancestor_depth"] = json!(1);
    rejects(
        model,
        &one,
        "explicit depth one is not the canonical one-link proof",
    );
    let mut deeper = wire.clone();
    deeper["field_composition"]["anchor"]["ancestor_depth"] = json!(2);
    rejects(
        model,
        &deeper,
        "a deeper path claim must not validate an immediate-previous anchor",
    );
    let mut link = wire.clone();
    let record = link["values"]["relations"]["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["id"] == current)
        .unwrap();
    record["previous"] = json!(0);
    rejects(
        model,
        &link,
        "broken live previous link must reject historical field continuation",
    );
}

fn validate_and_corrupt_ancestor(model: &Model, wire: &Value, current: u64, depth: u8) {
    let anchor = &wire["field_composition"]["anchor"];
    assert_eq!(anchor["current_revision"], current);
    assert_eq!(anchor["ancestor_depth"], depth);
    let selected = anchor["relation_id"].as_u64().unwrap();
    let mut missing_depth = wire.clone();
    missing_depth["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("ancestor_depth");
    rejects(
        model,
        &missing_depth,
        "an ancestor anchor without its depth claims a one-link proof",
    );
    let mut missing_proof = wire.clone();
    missing_proof["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("current_revision");
    rejects(
        model,
        &missing_proof,
        "ancestor depth without a live head proof",
    );
    for bad in [0_u64, 1, u64::from(depth) + 1, 16] {
        let mut wrong = wire.clone();
        wrong["field_composition"]["anchor"]["ancestor_depth"] = json!(bad);
        rejects(model, &wrong, "wrong ancestor depth must reject");
    }
    let mut wrong_head = wire.clone();
    wrong_head["field_composition"]["anchor"]["current_revision"] = json!(current + 1);
    rejects(model, &wrong_head, "wrong live head must reject");
    // Break each link of the proven path in turn, including the deepest one.
    let records = wire["values"]["relations"]["records"].as_array().unwrap();
    let mut cursor = current;
    for _ in 0..depth {
        let mut broken = wire.clone();
        let record = broken["values"]["relations"]["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["id"] == cursor)
            .unwrap();
        let next = record["previous"].as_u64().unwrap();
        record["previous"] = json!(0);
        rejects(
            model,
            &broken,
            "broken ancestor path link must reject continuation",
        );
        cursor = next;
    }
    assert_eq!(cursor, selected);
    let root = records.iter().find(|r| r["id"] == selected).unwrap();
    assert_eq!(root["previous"], 0);
    assert_eq!(root["action"], 1);
    let mut evicted = wire.clone();
    let record = evicted["values"]["relations"]["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["id"] == selected)
        .unwrap();
    record["id"] = json!(selected + 16);
    rejects(
        model,
        &evicted,
        "an overwritten root slot must reject the ancestor anchor",
    );
}

#[test]
#[ignore = "requires UOR_HISTORICAL_VERSION_MODEL; charged actual historical-version artifact"]
fn native_historical_version_actual_checkpoint_and_identity() {
    let bytes = std::fs::read(std::env::var("UOR_HISTORICAL_VERSION_MODEL").unwrap()).unwrap();
    let model = Model::from_bytes(&bytes).unwrap();
    assert_eq!(model.to_bytes().unwrap(), bytes);
    let parent = model.without_historical_version_intent().unwrap();
    assert_eq!(
        parent.artifact_cid(),
        "blake3:f0901dadf87f82cd283c8cae56c41a3048ca7b839c1810b3577e589a18af38e3"
    );
    let parent_bytes = parent.to_bytes().unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&parent_bytes)),
        "9e126c2f7d8af95164e6b6ace81fea63feb3b9431f211120bd7e4ab881058435"
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
    assert!(wire["historical_version_intent"].is_object());
    assert_eq!(
        wire["historical_version_intent"]["parent_artifact"],
        parent.artifact_cid()
    );
    assert_eq!(
        wire["historical_version_intent"]["committed_query_scope"],
        true
    );
    let mut changed = wire.clone();
    changed["historical_version_intent"]["unexpected_field"] = true.into();
    assert!(Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err());
    let mut changed = wire.clone();
    changed["historical_version_intent"]["parent_artifact"] =
        json!(format!("blake3:{}", "0".repeat(64)));
    assert!(
        Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err(),
        "outer parent commitment must reject mutation"
    );
    let mut changed = wire.clone();
    let root = &mut changed["historical_version_intent"]["router"]["codes"][0]["roots"][0];
    let prior = root.as_u64().unwrap();
    assert!(prior < 120);
    *root = json!((prior + 1) % 120);
    let error = Model::from_bytes(&serde_json::to_vec(&changed).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("historical version identity differs"),
        "a valid active root change must break artifact identity: {error}"
    );
    let mut changed = wire.clone();
    let root = &mut changed["current_query_handoff"]["router"]["codes"][0]["roots"][0];
    let prior = root.as_u64().unwrap();
    *root = json!((prior + 1) % 120);
    let error = Model::from_bytes(&serde_json::to_vec(&changed).unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("historical version parent differs"),
        "inner parent mutation must reject at the outer parent boundary: {error}"
    );
    let mut changed = wire.clone();
    changed["historical_version_intent"]["dictionary"][0]["prime"] = json!(0);
    assert!(Model::from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err());
    drop(changed);
    drop(wire);
    drop(bytes);

    let mut inputs = 0;
    let mut outputs = 0;
    let mut proof_checks = 0;
    // (label, facts, owner, initial, root id, previous, previous id, current, head id, depth)
    let cases = [
        ("selvi_first", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field. Silver Cove holds tilva. tilva now in Birch Grove. tilva now in Pine Hollow.", "selvi", "Dusk Ridge", 1, "Copper Vale", 2, "Amber Field", 3, 2),
        ("tilva_second", "Record: Dusk Ridge holds selvi. selvi now in Copper Vale. selvi now in Amber Field. Silver Cove holds tilva. tilva now in Birch Grove. tilva now in Pine Hollow.", "tilva", "Silver Cove", 4, "Birch Grove", 5, "Pine Hollow", 6, 2),
        ("tilva_first_forward", "Record: tilva in Silver Cove. tilva now in Birch Grove. tilva now in Pine Hollow. tilva now in Cedar Point. selvi in Dusk Ridge. selvi now in Copper Vale.", "tilva", "Silver Cove", 1, "Pine Hollow", 3, "Cedar Point", 4, 3),
        ("selvi_second_two", "Record: tilva in Silver Cove. tilva now in Birch Grove. tilva now in Pine Hollow. tilva now in Cedar Point. selvi in Dusk Ridge. selvi now in Copper Vale.", "selvi", "Dusk Ridge", 5, "Dusk Ridge", 5, "Copper Vale", 6, 1),
    ];
    let sequence_count = cases.len() * 2;
    assert_eq!(sequence_count, 8);
    for (label, facts, owner, initial, root_id, previous, previous_id, current, head_id, depth) in
        cases
    {
        for instruction in ["Name the owner first.", "State the owner first."] {
            let initial_prompt =
                format!("{facts} What was the initial location of {owner}? {instruction} Answer:");
            let history_prompt =
                format!("What was the previous location of {owner}? {instruction} Answer:");
            let current_prompt =
                format!("What is the current location of {owner}? {instruction} Answer:");
            let sum_prompt = "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
            let prompts = [
                initial_prompt.as_str(),
                history_prompt.as_str(),
                current_prompt.as_str(),
                sum_prompt,
            ];
            assert_eq!(
                raw_sequence(&model, &prompts, Control::HistoricalVersionIntentDisabled),
                raw_sequence(&parent, &prompts, Control::Full),
                "disabled outer witness must preserve parent output and causal state across all four turns"
            );
            let mut s = model.session(Control::Full).unwrap();
            s.observe(&model, BOS).unwrap();
            turn(
                &model,
                &mut s,
                &initial_prompt,
                &format!(" {owner} was in {initial}.\n"),
                Some(ExpectedField {
                    record: root_id,
                    owner,
                    value: initial,
                    current_proof: Some(head_id),
                    depth: (depth > 1).then_some(depth),
                }),
                &mut inputs,
                &mut outputs,
                &mut proof_checks,
            );
            turn(
                &model,
                &mut s,
                &history_prompt,
                &format!(" {owner} was in {previous}.\n"),
                Some(ExpectedField {
                    record: previous_id,
                    owner,
                    value: previous,
                    current_proof: Some(head_id),
                    depth: None,
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
                    record: head_id,
                    owner,
                    value: current,
                    current_proof: None,
                    depth: None,
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
            println!("historical-version {label}; owner={owner}; {instruction}; root={root_id} depth={depth}, previous={previous_id}, head={head_id}; initial/previous/current fields and independent sum exact");
        }
    }
    assert_eq!(proof_checks, sequence_count * 3);
    // A chain whose root was evicted by fourteen later facts: the initial request
    // must abstain, the immediate previous version stays readable, and the same
    // session then answers the current version and an independent sum.
    let mut evicted = String::from(
        "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
    );
    for (k, name) in [
        "arbor", "brook", "cairn", "delta", "ember", "fjord", "glade", "haven", "islet", "jetty",
        "knoll", "lagoon", "marsh", "nadir",
    ]
    .iter()
    .enumerate()
    {
        evicted.push_str(&format!(" {name} in Zone{k}."));
    }
    let mut abstentions = 0;
    for instruction in ["", "Name the owner first."] {
        let suffix = if instruction.is_empty() {
            String::new()
        } else {
            format!(" {instruction}")
        };
        let initial_prompt =
            format!("{evicted} What was the initial location of selvi?{suffix} Answer:");
        let history_prompt =
            format!("What was the previous location of selvi? Name the owner first. Answer:");
        let current_prompt =
            format!("What is the current location of selvi? Name the owner first. Answer:");
        let sum_prompt = "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
        let prompts = [
            initial_prompt.as_str(),
            history_prompt.as_str(),
            current_prompt.as_str(),
            sum_prompt,
        ];
        assert_eq!(
            raw_sequence(&model, &prompts, Control::HistoricalVersionIntentDisabled),
            raw_sequence(&parent, &prompts, Control::Full),
            "disabled outer witness must preserve parent output on the evicted-root sequence"
        );
        let withheld = raw_sequence(
            &model,
            &prompts,
            Control::HistoricalVersionIntentAbstainDisabled,
        );
        let tokens: Vec<_> = withheld[0]
            .0
            .iter()
            .copied()
            .filter(|t| *t != EOS)
            .collect();
        assert_ne!(
            model.decode(&tokens).unwrap(),
            b" Unknown.\n",
            "withholding the abstention candidate must fall back to the parent's answer, not abstain"
        );
        let mut s = model.session(Control::Full).unwrap();
        s.observe(&model, BOS).unwrap();
        turn(
            &model,
            &mut s,
            &initial_prompt,
            " Unknown.\n",
            None,
            &mut inputs,
            &mut outputs,
            &mut proof_checks,
        );
        assert!(
            state(&s)["values"]["relations"]["records"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["id"] != 1),
            "the root record must actually be evicted"
        );
        abstentions += 1;
        turn(
            &model,
            &mut s,
            &history_prompt,
            " selvi was in Copper Vale.\n",
            Some(ExpectedField {
                record: 2,
                owner: "selvi",
                value: "Copper Vale",
                current_proof: Some(3),
                depth: None,
            }),
            &mut inputs,
            &mut outputs,
            &mut proof_checks,
        );
        turn(
            &model,
            &mut s,
            &current_prompt,
            " selvi is in Amber Field.\n",
            Some(ExpectedField {
                record: 3,
                owner: "selvi",
                value: "Amber Field",
                current_proof: None,
                depth: None,
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
        println!("historical-version evicted-root sequence; instruction={instruction:?}; initial abstained, previous/current fields and independent sum exact");
    }
    assert_eq!(abstentions, 2);
    println!("actual historical-version artifact={}; sequences={}; input_checkpoint_positions={inputs}; output_checkpoint_positions={outputs}; active_anchor_checkpoint_cases={proof_checks}; evicted_root_abstentions={abstentions}; disabled parent equivalence across {} turns; no allocation/energy measurement", model.artifact_cid(), sequence_count + 2, sequence_count * 4 + 8);
}
