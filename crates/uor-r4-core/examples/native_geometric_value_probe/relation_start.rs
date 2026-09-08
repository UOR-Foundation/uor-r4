//! Fit only the start scorer; endpoint, admission, payload and parent stay fixed.
use super::*;
fn examples(tag: &str, worlds: &[(&str, &str, &str, &str)], long: bool) -> Vec<ValueExample> {
    let mut out = Vec::new();
    for (i, &(intro, name, full, single)) in worlds.iter().enumerate() {
        for (j, (prefix, value)) in [(intro, full), ("", full), (intro, single), ("", single)]
            .into_iter()
            .enumerate()
        {
            let padding = if long {
                "quiet sky. ".repeat(96)
            } else {
                String::new()
            };
            out.push(ValueExample {
                id: format!("{tag}/{i}/{j}"),
                prompt: format!("{prefix}{value} holds {name}. {padding}Where is {name}? Answer:"),
                response: format!(" {value}.\n"),
            });
        }
    }
    out
}
fn save(out: &Path, name: &str, m: &Model, docs: &[ValueExample]) -> ProbeResult<Value> {
    let report = source_noread::lean(source_noread::responses(m, docs, false, Control::Full)?);
    write_json(&out.join(format!("{name}.json")), &report)?;
    Ok(report)
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    if args.len() != 3 {
        return Err("relation-start PARENT ROOT NEW_DIRECTORY".into());
    }
    let root = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    fs::create_dir(out)?;
    let fit = examples(
        "start-fit",
        &[
            ("Notes say ", "telra", "Amber Meadow", "Amber"),
            ("Log tells ", "mevin", "Dawn River Bend", "Dawn"),
        ],
        false,
    );
    let open = examples(
        "start-open",
        &[
            ("A ledger says ", "tesvi", "Pearl Cove", "Pearl"),
            ("Report notes ", "vorin", "Cobalt Field", "Cobalt"),
        ],
        false,
    );
    write_json(
        &out.join("source.json"),
        &json!({"fit":fit,"development":open,"scope":"New authored raw prompt/response contrasts. Existing Records show diagnostic and former fresh panels excluded from fitting."}),
    )?;
    let parent = Model::from_bytes(&fs::read(&args[0])?)?;
    let before = save(out, "parent-construction", &parent, &fit)?;
    let before_open = save(out, "parent-development", &parent, &open)?;
    let (model, report) = parent.fit_relation_start(&fit)?;
    write_json(&out.join("fit.json"), &report)?;
    write_new(&out.join("model.json"), &model.to_bytes()?)?;
    let mut wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
    let mut old: Value = serde_json::from_slice(&parent.to_bytes()?)?;
    let block = wire
        .as_object_mut()
        .ok_or("shape")?
        .remove("relation_start")
        .ok_or("start block absent")?;
    if block["parent_artifact"] != parent.artifact_cid() {
        return Err("wrong start parent".into());
    }
    for x in [&mut wire, &mut old] {
        let x = x.as_object_mut().ok_or("shape")?;
        x.remove("artifact_cid");
        x.remove("uor_model_address");
    }
    if wire != old {
        return Err("parent parameters changed".into());
    }
    let construction = save(out, "construction", &model, &fit)?;
    let development = save(out, "development", &model, &open)?;
    let mut preserved = reverse_span::preserve(&model, root, out)?;
    for (name, names, values, prefix) in [
        (
            "reverse-construction",
            ["nelra", "vesk"],
            ["Orin Grove", "Amber Grove", "Silver Cape"],
            "",
        ),
        (
            "reverse-open",
            ["serin", "mavra"],
            ["Copper Vale", "Cobalt Vale", "Ivory Pier"],
            "Record: ",
        ),
        (
            "reverse-prior-fresh",
            ["belvi", "norvi"],
            ["Quiet River Bend", "Silver River Bend", "Violet Quay"],
            "",
        ),
    ] {
        let j = reverse_span::panel(&model, names, values, prefix)?;
        preserved &=
            j["exact"] == j["total"] && j["version_counts_exact"] == true && j["isolation"] == true;
        write_json(&out.join("preservation").join(format!("{name}.json")), &j)?;
    }
    let short = reverse_span::short(&model)?;
    preserved &= short["exact"] == short["total"];
    write_json(&out.join("preservation/reverse-short.json"), &short)?;
    let selected = construction["exact"] == construction["total"]
        && development["exact"] == development["total"]
        && preserved;
    write_json(
        &out.join("selection.json"),
        &json!({"artifact":model.artifact_cid(),"parent":parent.artifact_cid(),"parent_parameters_equal":true,"selected_before_fresh":selected,"construction":construction["exact"],"development":development["exact"],"preservation":preserved,"fresh":"NOT_RUN_AT_SELECTION"}),
    )?;
    if !selected {
        println!(
            "{}",
            json!({"artifact":model.artifact_cid(),"parent_construction":before["exact"],"parent_open":before_open["exact"],"construction":construction["exact"],"development":development["exact"],"preservation":preserved,"selected":false,"fresh":"NOT_RUN_SELECTION_FAILED"})
        );
        return Ok(());
    }
    let fresh = examples(
        "start-fresh",
        &[
            ("File notes ", "remvi", "Opal Coast", "Opal"),
            ("A witness says ", "sarvi", "Lilac Bay", "Lilac"),
        ],
        true,
    );
    write_json(
        &out.join("fresh-source.json"),
        &json!({"fresh":fresh,"scope":"Familiar template, new names/values/intro wording; source evicted before query, first executed after selection."}),
    )?;
    let fresh = save(out, "fresh", &model, &fresh)?;
    let diagnostics = vec![
        ValueExample {
            id: "start-boundary/prior-intro".into(),
            prompt: format!(
                "Records show Orin Grove holds nelra. {}Where is nelra? Answer:",
                "quiet sky. ".repeat(96)
            ),
            response: " Orin Grove.\n".into(),
        },
        ValueExample {
            id: "start-boundary/prior-gap".into(),
            prompt: format!(
                "Orin  Grove holds nelra. {}Where is nelra? Answer:",
                "quiet sky. ".repeat(96)
            ),
            response: " Orin  Grove.\n".into(),
        },
        ValueExample {
            id: "start-boundary/lowercase".into(),
            prompt: format!(
                "File notes fine sand holds remvi. {}Where is remvi? Answer:",
                "quiet sky. ".repeat(96)
            ),
            response: " fine sand.\n".into(),
        },
    ];
    save(out, "diagnostics", &model, &diagnostics)?;
    println!(
        "{}",
        json!({"artifact":model.artifact_cid(),"parent_construction":before["exact"],"parent_open":before_open["exact"],"construction":construction["exact"],"development":development["exact"],"preservation":preserved,"selected":selected,"fresh":fresh["exact"],"fresh_total":fresh["total"]})
    );
    Ok(())
}
