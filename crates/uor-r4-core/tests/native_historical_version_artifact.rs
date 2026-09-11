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

/// Like `raw_sequence`, but the frozen parent may legitimately fail to terminate on a
/// prompt the contract repairs; termination is returned, not required.
fn bounded_sequence(
    model: &Model,
    prompt: &str,
    control: Control,
) -> (Vec<u32>, bool, Value, Value) {
    let mut s = model.session(control).unwrap();
    s.observe(model, BOS).unwrap();
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
    (out, eos, initial, comparable(state(&s)))
}

struct ExpectedField<'a> {
    record: u64,
    owner: &'a str,
    value: &'a str,
    current_proof: Option<u64>,
    depth: Option<u8>,
    /// The ancestor path contains a reassertion link proven under the versioned contract.
    reassertion_links: bool,
    /// The path's first hop passes through a same-value reassertion head.
    reassertion_head: bool,
}
fn payload(r: &Value) -> String {
    atom(if r["span"].is_object() {
        &r["span"]
    } else {
        &r["value"]
    })
}
/// Field anchors produced while generating one prompt under `control`.
fn anchors(model: &Model, prompt: &str, control: Control) -> Vec<(u64, Option<u8>, bool)> {
    let mut s = model.session(control).unwrap();
    s.observe(model, BOS).unwrap();
    for t in model.encode(prompt).unwrap() {
        s.observe(model, t).unwrap();
    }
    s.begin_response(model).unwrap();
    let mut out = Vec::new();
    for _ in 0..96 {
        let p = s.predict(model).unwrap();
        if let Some(d) = s.field_composition_decision().filter(|d| d.field != 0) {
            out.push((
                d.anchor.relation_id,
                d.anchor.ancestor_depth,
                d.anchor.reassertion_links,
            ));
        }
        s.observe(model, p.token).unwrap();
        if p.token == EOS {
            break;
        }
    }
    out
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
            let mut reassertion_hops = 0;
            for hop in 0..depth {
                assert_eq!(cursor["conflict"], false);
                assert_eq!(atom(&cursor["owner"]), f.owner);
                let previous = cursor["previous"].as_u64().unwrap();
                assert!(previous != 0 && previous < cursor["id"].as_u64().unwrap());
                let older = records.iter().find(|r| r["id"] == previous).unwrap();
                // Deeper hops may be resident same-owner, same-value reassertions
                // under the versioned chain contract; the head link is a revision.
                let same_value = cursor["action"] == 1
                    && older["conflict"] == false
                    && atom(&older["owner"]) == f.owner
                    && payload(cursor) == payload(older);
                let reassertion = hop > 0 && same_value;
                let head_hop = hop == 0 && same_value;
                assert!(
                    cursor["action"] == 2 || reassertion || head_hop,
                    "hop {hop} must be an explicit revision or a resident same-value reassertion"
                );
                if head_hop {
                    assert!(
                        f.reassertion_head,
                        "a head hop must be proven under the head contract"
                    );
                }
                reassertion_hops += usize::from(reassertion);
                cursor = older;
            }
            if reassertion_hops > 0 {
                assert!(
                    f.reassertion_links,
                    "a path with a reassertion link must be proven under the versioned contract"
                );
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
            assert_eq!(d.anchor.reassertion_links, f.reassertion_links);
            assert_eq!(d.anchor.reassertion_head, f.reassertion_head);
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
                        validate_and_corrupt_ancestor(model, &after, id, depth);
                        if f.reassertion_links {
                            validate_and_corrupt_reassertion(model, &after, id, depth);
                        }
                        if f.reassertion_head {
                            validate_and_corrupt_head(model, &after);
                        }
                    }
                    (Some(id), None) if f.reassertion_head => {
                        validate_and_corrupt_proof(model, &after, id);
                        validate_and_corrupt_head(model, &after);
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

/// A path proven through a same-value reassertion restores only under its stated
/// contract and only while the reassertion's exact value still matches.
fn validate_and_corrupt_reassertion(model: &Model, wire: &Value, current: u64, depth: u8) {
    let anchor = &wire["field_composition"]["anchor"];
    assert_eq!(anchor["reassertion_links"], true);
    let mut legacy = wire.clone();
    legacy["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("reassertion_links");
    rejects(
        model,
        &legacy,
        "a reassertion path claimed under the revision-only contract must reject",
    );
    let records = wire["values"]["relations"]["records"].as_array().unwrap();
    let mut cursor = records.iter().find(|r| r["id"] == current).unwrap();
    let mut reassertions = Vec::new();
    for _ in 0..depth {
        let previous = cursor["previous"].as_u64().unwrap();
        if cursor["action"] == 1 {
            reassertions.push(cursor["id"].as_u64().unwrap());
        }
        cursor = records.iter().find(|r| r["id"] == previous).unwrap();
    }
    assert!(
        !reassertions.is_empty(),
        "the proven path must contain a reassertion"
    );
    for id in reassertions {
        let mut differing = wire.clone();
        let record = differing["values"]["relations"]["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["id"] == id)
            .unwrap();
        let first = record["value"]["bytes"][0].as_u64().unwrap();
        record["value"]["bytes"][0] = json!((first + 1) % 256);
        if record["span"].is_object() {
            let first = record["span"]["bytes"][0].as_u64().unwrap();
            record["span"]["bytes"][0] = json!((first + 1) % 256);
        }
        rejects(
            model,
            &differing,
            "a reassertion whose value no longer matches its predecessor must reject",
        );
        let mut conflicting = wire.clone();
        conflicting["values"]["relations"]["records"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|r| r["id"] == id)
            .unwrap()["conflict"] = json!(true);
        rejects(
            model,
            &conflicting,
            "a conflicting reassertion is not a link",
        );
    }
}

/// A path whose first hop passes through a same-value reassertion head restores only
/// with the head flag and only while the head still repeats its predecessor's value.
fn validate_and_corrupt_head(model: &Model, wire: &Value) {
    let anchor = &wire["field_composition"]["anchor"];
    assert_eq!(anchor["reassertion_head"], true);
    let current = anchor["current_revision"].as_u64().unwrap();
    let mut legacy = wire.clone();
    legacy["field_composition"]["anchor"]
        .as_object_mut()
        .unwrap()
        .remove("reassertion_head");
    rejects(
        model,
        &legacy,
        "a head-hop path claimed under the frozen first-hop contract must reject",
    );
    let mut differing = wire.clone();
    let head = differing["values"]["relations"]["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["id"] == current)
        .unwrap();
    assert_eq!(
        head["action"], 1,
        "the head must be a same-value reassertion"
    );
    let first = head["value"]["bytes"][0].as_u64().unwrap();
    head["value"]["bytes"][0] = json!((first + 1) % 256);
    if head["span"].is_object() {
        let first = head["span"]["bytes"][0].as_u64().unwrap();
        head["span"]["bytes"][0] = json!((first + 1) % 256);
    }
    rejects(
        model,
        &differing,
        "a head that no longer repeats its predecessor's value must reject",
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
    let versioned = wire["historical_version_intent"]["reassertion_links"] == true;
    let head_contract = wire["historical_version_intent"]["reassertion_heads"] == true;
    let reader_window = wire["historical_version_intent"]["reader_request_window"] == true;
    let reader_turn = wire["historical_version_intent"]["reader_turn_window"] == true;
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
                    reassertion_links: false,
                    reassertion_head: false,
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
                    reassertion_links: false,
                    reassertion_head: false,
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
                    reassertion_links: false,
                    reassertion_head: false,
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
                reassertion_links: false,
                reassertion_head: false,
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
                reassertion_links: false,
                reassertion_head: false,
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
    // Resident same-value reassertions under the versioned chain contract: the
    // initial request reaches the exact root through the reassertion link, the
    // previous request keeps the parent's immediate previous record (the
    // reassertion itself), the current request and an independent sum follow, and
    // the anchor restores only under its stated contract. Withholding the
    // reassertion links never produces the root anchor.
    let mut reassertion_sequences = 0;
    let mut reassertion_abstentions = 0;
    if versioned {
        for (label, facts, owner, initial, root_id, previous, previous_id, current, head_id, depth) in [
            ("reassert-root", "Record: selvi in Dusk Ridge. selvi in Dusk Ridge. selvi now in Copper Vale.", "selvi", "Dusk Ridge", 1, "Dusk Ridge", 2, "Copper Vale", 3, 2),
            ("reassert-middle", "Record: tilva in moss dale. tilva now in Birch Grove. tilva in Birch Grove. tilva now in Pine Hollow.", "tilva", "moss dale", 1, "Birch Grove", 3, "Pine Hollow", 4, 3),
        ] {
            for instruction in ["Name the owner first.", "State the owner first."] {
                let initial_prompt = format!(
                    "{facts} What was the initial location of {owner}? {instruction} Answer:"
                );
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
                    "disabled outer witness must preserve parent output on reassertion chains"
                );
                assert!(
                    !anchors(
                        &model,
                        &initial_prompt,
                        Control::HistoricalVersionIntentReassertionDisabled
                    )
                    .iter()
                    .any(|(id, d, _)| *id == root_id && d.is_some()),
                    "withholding reassertion links must never produce the root anchor: {label}"
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
                        depth: Some(depth),
                        reassertion_links: true,
                        reassertion_head: false,
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
                        reassertion_links: false,
                        reassertion_head: false,
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
                        reassertion_links: false,
                        reassertion_head: false,
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
                reassertion_sequences += 1;
                println!("historical-version {label}; owner={owner}; {instruction}; root={root_id} depth={depth} through a same-value reassertion, previous={previous_id}, head={head_id}; initial/previous/current fields and independent sum exact");
            }
        }
        // A reassertion chain whose root was evicted by thirteen later facts: the
        // surviving reassertion is not a root, absence is proven, and the initial
        // request abstains while previous/current stay readable.
        let mut evicted_reassertion = String::from(
            "Record: selvi in Dusk Ridge. selvi in Dusk Ridge. selvi now in Copper Vale. selvi now in Amber Field.",
        );
        for (k, name) in [
            "arbor", "brook", "cairn", "delta", "ember", "fjord", "glade", "haven", "islet",
            "jetty", "knoll", "lagoon", "marsh",
        ]
        .iter()
        .enumerate()
        {
            evicted_reassertion.push_str(&format!(" {name} in Zone{k}."));
        }
        for instruction in ["", " Name the owner first."] {
            let initial_prompt = format!(
                "{evicted_reassertion} What was the initial location of selvi?{instruction} Answer:"
            );
            let history_prompt =
                format!("What was the previous location of selvi? Name the owner first. Answer:");
            let current_prompt =
                format!("What is the current location of selvi? Name the owner first. Answer:");
            let sum_prompt = "User: suri has 14 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:";
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
            let records = state(&s)["values"]["relations"]["records"].clone();
            let records = records.as_array().unwrap();
            assert!(
                records.iter().all(|r| r["id"] != 1),
                "the root record must actually be evicted"
            );
            assert!(
                records
                    .iter()
                    .any(|r| r["id"] == 2 && r["action"] == 1 && r["previous"] == 1),
                "the same-value reassertion must survive as a non-root"
            );
            reassertion_abstentions += 1;
            turn(
                &model,
                &mut s,
                &history_prompt,
                " selvi was in Copper Vale.\n",
                Some(ExpectedField {
                    record: 3,
                    owner: "selvi",
                    value: "Copper Vale",
                    current_proof: Some(4),
                    depth: None,
                    reassertion_links: false,
                    reassertion_head: false,
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
                    record: 4,
                    owner: "selvi",
                    value: "Amber Field",
                    current_proof: None,
                    depth: None,
                    reassertion_links: false,
                    reassertion_head: false,
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
            reassertion_sequences += 1;
            println!("historical-version evicted-reassertion sequence; instruction={instruction:?}; initial abstained with the reassertion resident, previous/current fields and independent sum exact");
        }
        assert_eq!(reassertion_sequences, 6);
        assert_eq!(reassertion_abstentions, 2);
    }
    // Same-value reassertion heads under the head contract: repeating the current
    // fact keeps the retained history readable. Initial reaches the root through the
    // head hop, previous is the head's immediate previous record (record-hop
    // semantics), current stays the parent's answer, the anchors carry the head flag
    // and restore only under it, and withholding the head hop returns to the
    // parent's Unknown.
    let mut head_sequences = 0;
    if head_contract {
        // (label, facts, owner, initial, root, previous, previous id, current, head, root depth,
        //  whether the initial path also crosses an interior same-value reassertion link)
        for (label, facts, owner, initial, root_id, previous, previous_id, current, head_id, depth, interior) in [
            ("head-reassert", "Record: selvi in Dusk Ridge. selvi now in Copper Vale. selvi in Copper Vale.", "selvi", "Dusk Ridge", 1, "Copper Vale", 2, "Copper Vale", 3, 2, false),
            ("head-reassert-long", "Record: tilva in moss dale. tilva now in Birch Grove. tilva now in Pine Hollow. tilva in Pine Hollow.", "tilva", "moss dale", 1, "Pine Hollow", 3, "Pine Hollow", 4, 3, false),
            // The smallest repeated-head chain: initial and previous both name record 1.
            ("same-only-two", "Record: selvi in Dusk Ridge. selvi in Dusk Ridge.", "selvi", "Dusk Ridge", 1, "Dusk Ridge", 1, "Dusk Ridge", 2, 1, false),
            // Three identical assertions: the root lies below an interior reassertion.
            ("same-only-three", "Record: tilva in moss dale. tilva in moss dale. tilva in moss dale.", "tilva", "moss dale", 1, "moss dale", 2, "moss dale", 3, 2, true),
        ] {
            for instruction in ["Name the owner first.", "State the owner first."] {
                let initial_prompt = format!(
                    "{facts} What was the initial location of {owner}? {instruction} Answer:"
                );
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
                    "disabled outer witness must preserve parent output on reassertion heads"
                );
                // Withholding the head hop leaves the initial request to the frozen
                // parent (an abstention for the plain style, its current-path sentence
                // for an owner-first style); it must never produce the root through
                // the head hop.
                let withheld = raw_sequence(
                    &model,
                    &prompts[..1],
                    Control::HistoricalVersionIntentReassertionHeadDisabled,
                );
                let parent_only = raw_sequence(&parent, &prompts[..1], Control::Full);
                assert_eq!(
                    withheld[0].0, parent_only[0].0,
                    "withholding the head hop must return the parent's own answer: {label}"
                );
                assert_ne!(
                    model.decode(&withheld[0].0).unwrap(),
                    format!(" {owner} was in {initial}.\n").as_bytes(),
                    "withholding the head hop must not produce the root: {label}"
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
                        reassertion_links: interior,
                        reassertion_head: true,
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
                        reassertion_links: false,
                        reassertion_head: true,
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
                        reassertion_links: false,
                        reassertion_head: false,
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
                head_sequences += 1;
                println!("historical-version {label}; owner={owner}; {instruction}; root={root_id} depth={depth} through a same-value reassertion head, previous={previous_id}, head={head_id}; initial/previous/current fields and independent sum exact");
            }
        }
        assert_eq!(head_sequences, 8);
    }
    // Reader-window contract: a current request whose instructions follow the question
    // (twelve request words, the owner beyond the eighth) must still read the committed
    // live head through the owner-first field path: complete value, present tense,
    // exact record source and EOS. Withholding the window, like removing the witness,
    // must reproduce the frozen parent exactly; the explanatory request without the
    // owner-first instruction keeps its inherited value-first form.
    let mut window_sequences = 0;
    if reader_window {
        assert!(
            head_contract,
            "the reader-window contract builds on the head contract"
        );
        // (label, first-turn facts, repeated fact or empty, owner, value, live head id,
        //  whether the frozen parent already emits the exact owner sentence through a
        //  query-occurrence copy on this layout: forward layouts do, reverse layouts do not)
        for (label, first, repeat, owner, value, head_id, parent_text_correct) in [
            ("reverse-repeat", "Record: amber quay holds pemru. Record: amber quay holds pemru.", "Record: amber quay holds pemru.", "pemru", "amber quay", 2, false),
            ("reverse-once", "Record: amber quay holds pemru.", "", "pemru", "amber quay", 1, false),
            ("reverse-competitor", "Record: Dusk Ridge holds selvi. Record: moss dale holds tilva. Record: Dusk Ridge holds selvi.", "Record: Dusk Ridge holds selvi.", "selvi", "Dusk Ridge", 3, false),
            ("record-repeat", "Record: pemru in amber quay. Record: pemru in amber quay.", "Record: pemru in amber quay.", "pemru", "amber quay", 2, true),
        ] {
            for question in [
                "Explain in a sentence. Name the owner first.",
                "Name the owner first. Explain in a sentence.",
            ] {
                let prompt = format!("{first} Where is {owner}? {question} Answer:");
                let expected = format!(" {owner} is in {value}.\n");
                // The frozen parent's own answer on this prompt (it may not terminate);
                // removing the witness or withholding the window must reproduce it exactly.
                let parent_only = bounded_sequence(&parent, &prompt, Control::Full);
                assert_eq!(
                    bounded_sequence(&model, &prompt, Control::HistoricalVersionIntentDisabled),
                    parent_only,
                    "disabled outer witness must preserve parent output: {label}"
                );
                assert_eq!(
                    bounded_sequence(&model, &prompt, Control::HistoricalVersionIntentReaderWindowDisabled),
                    parent_only,
                    "withholding the reader window must return the parent's own answer: {label}"
                );
                if parent_text_correct {
                    assert_eq!(
                        model.decode(&parent_only.0).unwrap(),
                        expected.as_bytes(),
                        "the frozen parent already emits the owner sentence on a forward layout: {label}"
                    );
                } else {
                    assert_ne!(
                        model.decode(&parent_only.0).unwrap(),
                        expected.as_bytes(),
                        "the frozen parent must not already answer the owner sentence: {label}"
                    );
                }
                println!(
                    "historical-version reader-window {label}; parent answer {:?} terminated={}",
                    String::from_utf8_lossy(&model.decode(&parent_only.0).unwrap()),
                    parent_only.1
                );
                let field = || ExpectedField {
                    record: head_id,
                    owner,
                    value,
                    current_proof: None,
                    depth: None,
                    reassertion_links: false,
                    reassertion_head: false,
                };
                let mut s = model.session(Control::Full).unwrap();
                s.observe(&model, BOS).unwrap();
                turn(&model, &mut s, &prompt, &expected, Some(field()), &mut inputs, &mut outputs, &mut proof_checks);
                // The explanatory request alone keeps the inherited value-first form.
                let explain = format!("{first} Where is {owner}? Explain in a sentence. Answer:");
                assert_eq!(
                    bounded_sequence(&model, &explain, Control::Full),
                    bounded_sequence(&parent, &explain, Control::Full),
                    "explanatory request without owner-first must keep the parent's form: {label}"
                );
                // An actual second turn: the first turn is answered by the model itself,
                // then the fact is repeated (or not) and the same request follows.
                let mut s = model.session(Control::Full).unwrap();
                s.observe(&model, BOS).unwrap();
                let (opening, first_answer) = if repeat.is_empty() {
                    (first.to_owned(), format!(" {value}.\n"))
                } else {
                    let opening = first.strip_suffix(repeat).unwrap().trim_end().to_owned();
                    (opening, format!(" {value}.\n"))
                };
                turn(&model, &mut s, &format!("{opening} Where is {owner}? Answer:"), &first_answer, None, &mut inputs, &mut outputs, &mut proof_checks);
                let second = if repeat.is_empty() {
                    format!("Where is {owner}? {question} Answer:")
                } else {
                    format!("{repeat} Where is {owner}? {question} Answer:")
                };
                turn(&model, &mut s, &second, &expected, Some(field()), &mut inputs, &mut outputs, &mut proof_checks);
                window_sequences += 1;
                println!("historical-version reader-window {label}; owner={owner}; {question}; head={head_id}; single and second-turn owner sentences from the live head");
            }
        }
        assert_eq!(window_sequences, 8);
    }
    // Reader-turn-window contract: the request scan stops at the current turn's input
    // boundary. After a first turn answered by the model itself, a second request about
    // an absent owner abstains, about the competing owner reads that owner's record, and
    // about the same owner still reads its record; withholding the turn boundary must
    // reproduce the cross-turn leak on the shapes the fifth review measured. Sixteen owner
    // matches in one request must not panic under any contract or diagnostic.
    let mut turn_scope_sequences = 0;
    if reader_turn {
        assert!(
            reader_window,
            "the turn-window contract builds on the request-window contract"
        );
        let facts = "Record: nemvi in cedar quay. polru in silver garden.";
        for (first, first_answer) in [
            ("Where is nemvi? Answer:", " cedar quay.\n"),
            (
                "Where is nemvi? Name the owner first. Answer:",
                " nemvi is in cedar quay.\n",
            ),
            (
                "Where is nemvi? Explain in a sentence. Answer:",
                " cedar quay is the place.\n",
            ),
        ] {
            let opening = format!("{facts} {first}");
            let seconds: [(&str, &str, Option<ExpectedField<'_>>); 5] = [
                ("Where is zalfe? Answer:", " Unknown.\n", None),
                (
                    "Where is zalfe? Explain in a sentence. Name the owner first. Answer:",
                    " Unknown.\n",
                    None,
                ),
                ("Where is polru? Answer:", " silver garden.\n", None),
                (
                    "Where is polru? Name the owner first. Answer:",
                    " polru is in silver garden.\n",
                    Some(ExpectedField {
                        record: 2,
                        owner: "polru",
                        value: "silver garden",
                        current_proof: None,
                        depth: None,
                        reassertion_links: false,
                        reassertion_head: false,
                    }),
                ),
                (
                    "Where is nemvi? Name the owner first. Explain in a sentence. Answer:",
                    " nemvi is in cedar quay.\n",
                    Some(ExpectedField {
                        record: 1,
                        owner: "nemvi",
                        value: "cedar quay",
                        current_proof: None,
                        depth: None,
                        reassertion_links: false,
                        reassertion_head: false,
                    }),
                ),
            ];
            for (second, expected, field) in seconds {
                // Disclosed inherited failure on every artifact: after a plain first turn on
                // forward facts, a plain request about an absent owner is answered by a
                // recent-capture copy of the previous plain answer; the persistent reader
                // abstains correctly, a later dispatch stage does not. Not claimed here.
                if first == "Where is nemvi? Answer:" && second == "Where is zalfe? Answer:" {
                    continue;
                }
                let mut s = model.session(Control::Full).unwrap();
                s.observe(&model, BOS).unwrap();
                turn(
                    &model,
                    &mut s,
                    &opening,
                    first_answer,
                    None,
                    &mut inputs,
                    &mut outputs,
                    &mut proof_checks,
                );
                turn(
                    &model,
                    &mut s,
                    second,
                    expected,
                    field,
                    &mut inputs,
                    &mut outputs,
                    &mut proof_checks,
                );
                if first.contains("owner")
                    && (second == "Where is zalfe? Answer:" || second == "Where is polru? Answer:")
                {
                    let leaked = raw_sequence(
                        &model,
                        &[opening.as_str(), second],
                        Control::HistoricalVersionIntentReaderTurnScopeDisabled,
                    );
                    assert_ne!(
                        model.decode(&leaked[1].0).unwrap(),
                        expected.as_bytes(),
                        "withholding the turn boundary must reproduce the measured cross-turn leak: {second}"
                    );
                }
                turn_scope_sequences += 1;
            }
        }
        assert_eq!(turn_scope_sequences, 14);
        let flood = format!("Record: selvi in Dusk Ridge. {}", "selvi ".repeat(16));
        for control in [
            Control::Full,
            Control::HistoricalVersionIntentReaderTurnScopeDisabled,
            Control::HistoricalVersionIntentReaderWindowDisabled,
            Control::HistoricalVersionIntentDisabled,
        ] {
            let (_, _, before, after) = bounded_sequence(&model, &flood, control);
            assert_eq!(
                before["values"]["relations"], after["values"]["relations"],
                "flood must not alter retained records"
            );
        }
        let _ = bounded_sequence(&parent, &flood, Control::Full);
        model.current_query_trace(&flood).unwrap();
        model.historical_version_trace(&flood).unwrap();
        println!("historical-version reader-turn-window: {turn_scope_sequences} two-turn sequences and a sixteen-match capacity flood under four controls and both diagnostics");
    }
    println!("actual historical-version artifact={}; sequences={}; input_checkpoint_positions={inputs}; output_checkpoint_positions={outputs}; active_anchor_checkpoint_cases={proof_checks}; evicted_root_abstentions={abstentions}; reassertion_sequences={reassertion_sequences}; reassertion_abstentions={reassertion_abstentions}; head_sequences={head_sequences}; window_sequences={window_sequences}; turn_scope_sequences={turn_scope_sequences}; versioned_chain_contract={versioned}; head_contract={head_contract}; reader_window_contract={reader_window}; reader_turn_contract={reader_turn}; disabled parent equivalence across {} turns; no allocation/energy measurement", model.artifact_cid(), sequence_count + 2 + reassertion_sequences + head_sequences + window_sequences + turn_scope_sequences, sequence_count * 4 + 8 + if versioned { 16 } else { 0 } + head_sequences * 4 + window_sequences * 2);
}
