//! Reuse the accepted extent operator in durable relation values, without fitting.
use super::*;
use uor_r4_core::native_geometric::Session;
fn ingest(s: &mut Session, m: &Model, text: &str) -> ProbeResult<()> {
    for t in m.encode(text)? {
        s.observe(m, t)?;
    }
    Ok(())
}
fn emit(s: &mut Session, m: &Model) -> ProbeResult<String> {
    s.begin_response(m)?;
    let mut out = Vec::new();
    for _ in 0..32 {
        let checkpoint = s.checkpoint()?;
        let mut restored = m.restore_session(&checkpoint)?;
        let p = s.predict(m)?;
        if restored.predict(m)? != p {
            return Err("retained span checkpoint prediction differs".into());
        }
        s.observe(m, p.token)?;
        if p.token == 1 {
            return Ok(String::from_utf8(m.decode(&out)?)?);
        }
        out.push(p.token);
    }
    Err("retained span did not terminate".into())
}
fn turns(m: &Model, a: &str, b: &str, first: &str, second: &str, tail: &str) -> ProbeResult<Value> {
    let mut s = m.session(Control::Full)?;
    s.observe(m, 0)?;
    let padding = "quiet sky. ".repeat(96);
    let trailing = m.encode(&padding)?.len();
    if trailing <= 512 {
        return Err("padding does not evict raw window".into());
    }
    let mut rows = Vec::new();
    for (input, owner, expected) in [
        (
            format!("{a} in {first} {second}. {b} in Cedar Harbor. "),
            a,
            format!(" {first} {second}.\n"),
        ),
        (
            format!("{a} in {first} {second}. "),
            a,
            format!(" {first} {second}.\n"),
        ),
        (
            format!("{a} now in {first} {tail}. "),
            a,
            format!(" {first} {tail}.\n"),
        ),
        (String::new(), b, " Cedar Harbor.\n".into()),
        (
            format!("{a} in {first} {second}. "),
            a,
            " Unknown.\n".into(),
        ),
        (
            format!("{a} now in {first} {second}. "),
            a,
            format!(" {first} {second}.\n"),
        ),
    ] {
        s.end_response(m)?;
        ingest(&mut s, m, &input)?;
        ingest(&mut s, m, &padding)?;
        let query = format!("Where is {owner}? Answer:");
        ingest(&mut s, m, &query)?;
        let text = emit(&mut s, m)?;
        let wire: Value = serde_json::from_slice(&s.checkpoint()?)?;
        let records = wire["values"]["relations"]["records"]
            .as_array()
            .ok_or("records absent")?;
        let current:Vec<_>=records.iter().filter(|r|r["id"].as_u64().unwrap_or(0)>0).map(|r|json!({"id":r["id"],"previous":r["previous"],"action":r["action"],"conflict":r["conflict"],"owner":r["owner"]["bytes"],"value":r["value"]["bytes"],"span":r["span"]})).collect();
        let expected_versions = [2_u64, 3, 4, 4, 5, 6][rows.len()];
        let writes_exact = wire["values"]["relations"]["next_id"] == expected_versions + 1
            && current.len() as u64 == expected_versions;
        rows.push(json!({"input":input,"query":query,"expected":expected,"text":text,"exact":text==expected,"trailing_tokens":trailing,"checkpoint_predictions_equal":true,"writes_exact":writes_exact,"expected_versions":expected_versions,"records":current}));
    }
    let mut isolated = m.session(Control::Full)?;
    isolated.observe(m, 0)?;
    ingest(&mut isolated, m, &format!("Where is {a}? Answer:"))?;
    let isolated = emit(&mut isolated, m)?;
    let exact = rows.iter().filter(|r| r["exact"] == true).count();
    Ok(
        json!({"artifact":m.artifact_cid(),"exact":exact,"total":rows.len(),"turns":rows,"isolated":isolated,"isolation_pass":isolated==" Unknown.\n","writes_exact":rows.iter().all(|r|r["writes_exact"]==true)}),
    )
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    if args.len() != 3 && !(args.len() == 4 && args[3] == "replay") {
        return Err("retained-span PARENT ROOT NEW_DIRECTORY | retained-span MODEL ROOT NEW_DIRECTORY replay".into());
    }
    let root = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    let replay = args.len() == 4;
    let loaded = Model::from_bytes(&fs::read(&args[0])?)?;
    let model;
    let baseline;
    if replay {
        model = loaded;
        baseline = json!({"exact":0,"scope":"Initial matched parent report retained; not rerun"});
    } else {
        let parent = loaded;
        baseline = turns(&parent, "nalia", "evrin", "Gold", "Meadow", "Cove")?;
        write_json(&out.join("parent-sessions.json"), &baseline)?;
        model = parent.with_retained_relation_spans()?;
        write_new(&out.join("model.json"), &model.to_bytes()?)?;
        let mut wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
        let mut old: Value = serde_json::from_slice(&parent.to_bytes()?)?;
        if wire
            .as_object_mut()
            .ok_or("model shape")?
            .remove("relation_spans")
            != Some(json!(parent.artifact_cid()))
        {
            return Err("parent link differs".into());
        }
        for v in [&mut wire, &mut old] {
            let map = v.as_object_mut().ok_or("model shape")?;
            map.remove("artifact_cid");
            map.remove("uor_model_address");
        }
        if wire != old {
            return Err("retained span changed parent parameters".into());
        }
    }
    let model_wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
    let construction = turns(&model, "nalia", "evrin", "Gold", "Meadow", "Cove")?;
    write_json(&out.join("construction-sessions.json"), &construction)?;
    let open = turns(&model, "selvi", "tilva", "Dusk", "Ridge", "Vale")?;
    write_json(&out.join("development-sessions.json"), &open)?;
    let preserve = out.join("preservation");
    fs::create_dir(&preserve)?;
    source_noread::preserve_model_lean(model.clone(), root, &preserve)?;
    let old: Value =
        serde_json::from_slice(&fs::read(root.join("source-order-angular256/source.json"))?)?;
    let docs: Vec<ValueExample> = serde_json::from_value(old["fit"].clone())?;
    let retained = source_noread::lean(source_noread::responses(
        &model,
        &docs,
        false,
        Control::Full,
    )?);
    write_json(&out.join("retained-construction.json"), &retained)?;
    // Earlier extent panels are preservation evidence, never new fitting data.
    for (file, split, name) in [
        (
            "span-context-angular/source.json",
            "fit",
            "prior-span-construction",
        ),
        (
            "span-context-angular/source.json",
            "development",
            "prior-span-development",
        ),
        (
            "span-context-angular/fresh-source.json",
            "fresh",
            "prior-span-fresh",
        ),
    ] {
        let source: Value = serde_json::from_slice(&fs::read(root.join(file))?)?;
        let docs: Vec<ValueExample> = serde_json::from_value(source[split].clone())?;
        let report = source_noread::lean(source_noread::responses(
            &model,
            &docs,
            false,
            Control::Full,
        )?);
        write_json(&preserve.join(format!("{name}.json")), &report)?;
    }
    let mut preserved = true;
    for entry in fs::read_dir(&preserve)? {
        let p = entry?.path();
        if p.extension().is_some_and(|x| x == "json") {
            let j: Value = serde_json::from_slice(&fs::read(p)?)?;
            if j["exact"].is_number() {
                preserved &= j["exact"] == j["total"];
            }
            if j["exact_turns"].is_number() {
                preserved &=
                    j["exact_turns"] == j["total_turns"] && j["isolated_no_shared_records"] == true;
            }
        }
    }
    let selected = construction["exact"] == 6
        && open["exact"] == 6
        && construction["isolation_pass"] == true
        && open["isolation_pass"] == true
        && construction["writes_exact"] == true
        && open["writes_exact"] == true
        && retained["exact"] == 663
        && preserved;
    write_json(
        &out.join("design-selection.json"),
        &json!({"artifact":model.artifact_cid(),"parent":model_wire["relation_spans"],"selected_before_fresh":selected,"parent_parameters_equal":true,"fit":"NOT_RUN_NO_FIT_REQUIRED","preservation":preserved,"construction":construction["exact"],"development":open["exact"],"retained":retained["exact"],"fresh":if replay {"PREVIOUSLY_EXPOSED_PRESERVATION_REPLAY_FOLLOWS"}else{"NOT_RUN_AT_SELECTION"}}),
    )?;
    // A fixed separately authored familiar-template panel, never used for fitting.
    let fresh = turns(&model, "irden", "marvi", "Silken", "Hollow", "River Bank")?;
    write_json(&out.join("fresh-sessions.json"), &fresh)?;
    println!(
        "{}",
        json!({"artifact":model.artifact_cid(),"parent":baseline["exact"],"construction":construction["exact"],"development":open["exact"],"retained":retained["exact"],"preservation":preserved,"selected_before_fresh":selected,"fresh":fresh["exact"],"fresh_total":fresh["total"]})
    );
    Ok(())
}
