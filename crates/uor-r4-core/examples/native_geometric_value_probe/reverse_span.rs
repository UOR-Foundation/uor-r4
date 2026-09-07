//! Reuse the writer's endpoint and learned edges for reverse phrase retention.
use super::*;
use uor_r4_core::native_geometric::Session;
fn ingest(s: &mut Session, m: &Model, text: &str) -> ProbeResult<()> {
    for token in m.encode(text)? {
        s.observe(m, token)?;
    }
    Ok(())
}
fn emit(s: &mut Session, m: &Model) -> ProbeResult<String> {
    s.begin_response(m)?;
    let mut tokens = Vec::new();
    for _ in 0..32 {
        let mut restored = m.restore_session(&s.checkpoint()?)?;
        let p = s.predict(m)?;
        if restored.predict(m)? != p {
            return Err("reverse span checkpoint differs".into());
        }
        s.observe(m, p.token)?;
        if p.token == 1 {
            return Ok(String::from_utf8(m.decode(&tokens)?)?);
        }
        tokens.push(p.token);
    }
    Err("reverse span did not terminate".into())
}
pub(super) fn panel(
    m: &Model,
    names: [&str; 2],
    values: [&str; 3],
    prefix: &str,
) -> ProbeResult<Value> {
    let [a, b] = names;
    let [first, changed, other] = values;
    let mut s = m.session(Control::Full)?;
    s.observe(m, 0)?;
    let padding = "quiet sky. ".repeat(96);
    let trailing = m.encode(&padding)?.len();
    if trailing <= 512 {
        return Err("source not evicted".into());
    }
    let mut rows = Vec::new();
    for (input, owner, expected, versions) in [
        (
            format!("{prefix}{first} holds {a}. {prefix}{other} holds {b}. "),
            a,
            format!(" {first}.\n"),
            2,
        ),
        (
            format!("{prefix}{first} holds {a}. "),
            a,
            format!(" {first}.\n"),
            3,
        ),
        (
            format!("{a} now in {changed}. "),
            a,
            format!(" {changed}.\n"),
            4,
        ),
        (String::new(), b, format!(" {other}.\n"), 4),
        (
            format!("{prefix}{first} holds {a}. "),
            a,
            " Unknown.\n".into(),
            5,
        ),
        (
            format!("{a} now in {first}. "),
            a,
            format!(" {first}.\n"),
            6,
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
            .ok_or("records missing")?;
        let records: Vec<_> = records
            .iter()
            .filter(|r| r["id"].as_u64().unwrap_or(0) > 0)
            .cloned()
            .collect();
        let count_ok = wire["values"]["relations"]["next_id"] == versions + 1
            && records.len() == versions as usize;
        rows.push(json!({"input":input,"query":query,"expected":expected,"text":text,"exact":text==expected,"version_count_exact":count_ok,"expected_versions":versions,"records":records,"checkpoint_predictions_equal":true}));
    }
    let mut isolated = m.session(Control::Full)?;
    isolated.observe(m, 0)?;
    ingest(&mut isolated, m, &format!("Where is {a}? Answer:"))?;
    let isolated = emit(&mut isolated, m)?;
    Ok(
        json!({"artifact":m.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"version_counts_exact":rows.iter().all(|r|r["version_count_exact"]==true),"isolation":isolated==" Unknown.\n","trailing_tokens":trailing,"turns":rows}),
    )
}
fn accepted(j: &Value) -> bool {
    j["exact"] == j["total"] && j["version_counts_exact"] == true && j["isolation"] == true
}
pub(super) fn short(m: &Model) -> ProbeResult<Value> {
    let mut rows = Vec::new();
    for (input, owner, value) in [
        ("Orin Grove holds nelra.", "nelra", "Orin Grove"),
        ("Record: Copper Vale holds serin.", "serin", "Copper Vale"),
        ("Ash Field holds merla.", "merla", "Ash Field"),
    ] {
        let mut s = m.session(Control::Full)?;
        s.observe(m, 0)?;
        let prompt = format!("{input} Where is {owner}? Answer:");
        ingest(&mut s, m, &prompt)?;
        let text = emit(&mut s, m)?;
        let expected = format!(" {value}.\n");
        rows.push(json!({"prompt":prompt,"text":text,"expected":expected,"exact":text==expected}));
    }
    Ok(
        json!({"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"cases":rows}),
    )
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    if args.len() != 3 {
        return Err("reverse-span PARENT ROOT NEW_DIRECTORY".into());
    }
    let root = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    let parent = Model::from_bytes(&fs::read(&args[0])?)?;
    let names = ["nelra", "vesk"];
    let values = ["Orin Grove", "Amber Grove", "Silver Cape"];
    let before = panel(&parent, names, values, "")?;
    write_json(&out.join("parent.json"), &before)?;
    let model = parent.with_reverse_relation_spans()?;
    write_new(&out.join("model.json"), &model.to_bytes()?)?;
    let mut wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
    let mut original: Value = serde_json::from_slice(&parent.to_bytes()?)?;
    if wire
        .as_object_mut()
        .ok_or("shape")?
        .remove("relation_reverse_spans")
        != Some(json!(parent.artifact_cid()))
    {
        return Err("reverse parent link differs".into());
    }
    for value in [&mut wire, &mut original] {
        let m = value.as_object_mut().ok_or("shape")?;
        m.remove("artifact_cid");
        m.remove("uor_model_address");
    }
    if wire != original {
        return Err("reverse span changed learned parameters".into());
    }
    let construction = panel(&model, names, values, "")?;
    write_json(&out.join("construction.json"), &construction)?;
    let development = panel(
        &model,
        ["serin", "mavra"],
        ["Copper Vale", "Cobalt Vale", "Ivory Pier"],
        "Record: ",
    )?;
    write_json(&out.join("development.json"), &development)?;
    let preserved = preserve(&model, root, out)?;
    let short = short(&model)?;
    write_json(&out.join("short.json"), &short)?;
    let selected = accepted(&construction)
        && accepted(&development)
        && preserved
        && short["exact"] == short["total"];
    write_json(
        &out.join("selection.json"),
        &json!({"artifact":model.artifact_cid(),"parent":parent.artifact_cid(),"parent_parameters_equal":true,"fit":"NOT_RUN_NO_FIT_REQUIRED","selected_before_fresh":selected,"preservation":preserved,"construction":construction["exact"],"development":development["exact"],"fresh":"NOT_RUN_AT_SELECTION"}),
    )?;
    let fresh = panel(
        &model,
        ["belvi", "norvi"],
        ["Quiet River Bend", "Silver River Bend", "Violet Quay"],
        "",
    )?;
    write_json(&out.join("fresh.json"), &fresh)?;
    let mut boundaries = Vec::new();
    for (text, expected) in [
        ("Records show Orin Grove holds nelra.", " Orin Grove.\n"),
        ("Orin  Grove holds nelra.", " Orin  Grove.\n"),
    ] {
        let mut s = model.session(Control::Full)?;
        s.observe(&model, 0)?;
        ingest(&mut s, &model, text)?;
        ingest(&mut s, &model, &"quiet sky. ".repeat(96))?;
        ingest(&mut s, &model, "Where is nelra? Answer:")?;
        let actual = emit(&mut s, &model)?;
        boundaries
            .push(json!({"input":text,"expected":expected,"text":actual,"exact":actual==expected}));
    }
    write_json(
        &out.join("boundaries.json"),
        &json!({"cases":boundaries,"scope":"Post-selection diagnostics only: unseen plain intro and two-space gap; no fitting or tuning"}),
    )?;
    println!(
        "{}",
        json!({"artifact":model.artifact_cid(),"parent":before["exact"],"construction":construction["exact"],"development":development["exact"],"preservation":preserved,"selected":selected,"fresh":fresh["exact"]})
    );
    Ok(())
}

pub(super) fn preserve(model: &Model, root: &Path, out: &Path) -> ProbeResult<bool> {
    let preserve = out.join("preservation");
    fs::create_dir(&preserve)?;
    source_noread::preserve_model_lean(model.clone(), root, &preserve)?;
    for (file, split, name) in [
        (
            "source-order-angular256/source.json",
            "fit",
            "retained-construction",
        ),
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
    for (name, a, b, first, second, tail) in [
        (
            "forward-construction",
            "nalia",
            "evrin",
            "Gold",
            "Meadow",
            "Cove",
        ),
        ("forward-open", "selvi", "tilva", "Dusk", "Ridge", "Vale"),
        (
            "forward-prior-fresh",
            "irden",
            "marvi",
            "Silken",
            "Hollow",
            "River Bank",
        ),
    ] {
        let report = retained_span::turns(&model, a, b, first, second, tail)?;
        write_json(&preserve.join(format!("{name}.json")), &report)?;
    }
    let mut preserved = true;
    for item in fs::read_dir(&preserve)? {
        let j: Value = serde_json::from_slice(&fs::read(item?.path())?)?;
        if j["exact"].is_number() {
            preserved &= j["exact"] == j["total"];
        }
        if j["writes_exact"].is_boolean() {
            preserved &= j["writes_exact"] == true;
        }
        if j["writes_exact"].is_number() {
            preserved &= j["writes_exact"] == j["total"];
        }
        if j["isolation_pass"].is_boolean() {
            preserved &= j["isolation_pass"] == true;
        }
        if j["exact_turns"].is_number() {
            preserved &=
                j["exact_turns"] == j["total_turns"] && j["isolated_no_shared_records"] == true;
        }
    }
    Ok(preserved)
}
