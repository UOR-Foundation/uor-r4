//! Source-cue/next-prime continuation using the existing H4 operator and cursor.
use super::*;
fn panels(prefix: &str, worlds: &[(&str, &str, &str, &str)]) -> Vec<ValueExample> {
    let mut docs = Vec::new();
    for (i, &(owner, city, second, object)) in worlds.iter().enumerate() {
        for (kind,prompt,response) in [
            ("stop",format!("Record: {city} holds {owner}. Where is {owner}? Answer:"),format!(" {city}.\n")),
            ("place",format!("User: {owner} lives in {city} {second}.\nUser: Where is {owner}?\nAssistant:"),format!(" {city} {second}.\n")),
            ("object",format!("User: {owner} carries {object} holds.\nUser: What does {owner} carry?\nAssistant:"),format!(" {object} holds.\n")),
        ] { docs.push(ValueExample{id:format!("{prefix}/{i}/{kind}"),prompt,response}); }
    }
    docs
}
fn save(
    out: &Path,
    name: &str,
    model: &Model,
    docs: &[ValueExample],
    control: Control,
) -> ProbeResult<()> {
    let report = source_noread::lean(source_noread::responses(model, docs, false, control)?);
    write_json(&out.join(format!("{name}.json")), &report)?;
    println!(
        "{}",
        json!({"split":name,"exact":report["exact"],"total":report["total"]})
    );
    Ok(())
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("fit") if args.len() == 4 => {
            let root = Path::new(&args[2]);
            let out = Path::new(&args[3]);
            fs::create_dir(out)?;
            let parent = Model::from_bytes(&fs::read(&args[1])?)?;
            let old: Value = serde_json::from_slice(&fs::read(
                root.join("contextual-emission-angular/source.json"),
            )?)?;
            let mut fit: Vec<ValueExample> = serde_json::from_value(old["fit"].clone())?;
            fit.extend(panels(
                "extent-context-fit",
                &[
                    ("nalia", "Gold", "Meadow", "stone"),
                    ("evrin", "Dusk", "Ridge", "metal"),
                    ("selvi", "Pearl", "Cove", "plastic"),
                ],
            ));
            let open = panels(
                "extent-context-open",
                &[
                    ("tilva", "Ash", "Court", "wooden"),
                    ("savin", "Mist", "Vale", "climbing"),
                ],
            );
            write_json(
                &out.join("source.json"),
                &json!({"fit":fit,"development":open,"scope":"Reuse the five earlier construction pairs plus nine new raw same-space context contrasts. No old fresh cases in fitting."}),
            )?;
            save(out, "parent-construction", &parent, &fit, Control::Full)?;
            save(out, "parent-development", &parent, &open, Control::Full)?;
            let (candidate, report) = parent.fit_contextual_source_span(&fit)?;
            write_new(&out.join("model.json"), &candidate.to_bytes()?)?;
            write_json(&out.join("fit.json"), &report)?;
            println!("{}", report);
            for (name, docs) in [("construction", &fit), ("development", &open)] {
                save(out, name, &candidate, docs, Control::Full)?;
                save(
                    out,
                    &format!("{name}-no-context"),
                    &candidate,
                    docs,
                    Control::SourceSpanContextDisabled,
                )?;
                save(
                    out,
                    &format!("{name}-no-pair"),
                    &candidate,
                    docs,
                    Control::SourceSpanPairDisabled,
                )?;
                save(
                    out,
                    &format!("{name}-parent"),
                    &candidate,
                    docs,
                    Control::SourceSpanDisabled,
                )?;
            }
            for (file, split, name) in [
                (
                    "source-order-angular256/source.json",
                    "fit",
                    "retained-construction",
                ),
                (
                    "contextual-emission-angular/source.json",
                    "development",
                    "prior-open",
                ),
                (
                    "source-order-angular256/fresh-source.json",
                    "",
                    "source-context-prior-fresh",
                ),
                (
                    "literal-binding-source.json",
                    "first_use",
                    "literal-prior-fresh",
                ),
                (
                    "joint-admission-source.json",
                    "first_use",
                    "admission-prior-fresh",
                ),
                (
                    "source-noread-source-v2.json",
                    "first_use",
                    "source-prior-fresh",
                ),
            ] {
                let s: Value = serde_json::from_slice(&fs::read(root.join(file))?)?;
                let docs: Vec<ValueExample> = serde_json::from_value(if split.is_empty() {
                    s
                } else {
                    s[split].clone()
                })?;
                save(out, name, &candidate, &docs, Control::Full)?;
            }
            let preserve = out.join("preservation");
            fs::create_dir(&preserve)?;
            source_noread::preserve_model_lean(candidate, root, &preserve)?;
        }
        Some("fresh") if args.len() == 3 => {
            let out = Path::new(&args[2]);
            let model = Model::from_bytes(&fs::read(&args[1])?)?;
            let docs = panels(
                "extent-context-fresh",
                &[
                    ("irden", "Silken", "Hollow", "rubber"),
                    ("tulvi", "Quiet", "River Bank", "resin"),
                    ("marvi", "Ivory", "Glen", "ceramic"),
                ],
            );
            let boundaries=vec![ValueExample{id:"extent-boundary/unseen-connector".into(),prompt:"User: irden lives in Silken Hollow beside Brook.\nUser: Where is irden?\nAssistant:".into(),response:" Silken Hollow.\n".into()},ValueExample{id:"extent-boundary/wide-gap".into(),prompt:"User: tulvi lives in Quiet  River.\nUser: Where is tulvi?\nAssistant:".into(),response:" Quiet  River.\n".into()}];
            write_json(
                &out.join("fresh-source.json"),
                &json!({"fresh":docs,"boundary_diagnostics":boundaries,"scope":"Fixed separately authored cases first executed after design selection, familiar relation/query templates. Boundary diagnostics report scope limits separately."}),
            )?;
            for (name, control) in [
                ("fresh", Control::Full),
                ("fresh-no-context", Control::SourceSpanContextDisabled),
                ("fresh-no-pair", Control::SourceSpanPairDisabled),
                ("fresh-parent", Control::SourceSpanDisabled),
            ] {
                save(out, name, &model, &docs, control)?;
            }
            save(
                out,
                "boundary-diagnostics",
                &model,
                &boundaries,
                Control::Full,
            )?;
        }
        _ => return Err("span-context fit PARENT ROOT NEW_DIR | fresh MODEL EXISTING_DIR".into()),
    }
    Ok(())
}
