//! Candidate-owned context through the existing source selector. Construction
//! and open development run together; fresh identities are a separate command.
use super::*;
use uor_r4_core::native_geometric::SourceRoutingConfig;
fn contrasts(prefix: &str, worlds: &[(&str, &str, &str, i64, i64)]) -> Vec<ValueExample> {
    let mut out = Vec::new();
    for (i, &(a, b, city, x, y)) in worlds.iter().enumerate() {
        for owner in [a, b] {
            for reverse in [false, true] {
                for first in [false, true] {
                    let one = format!("{a} has {x} coins.");
                    let two = format!("{b} has {y} coins.");
                    let nums = if reverse {
                        format!("{two} {one}")
                    } else {
                        format!("{one} {two}")
                    };
                    let place = format!("{owner} lives in {city}.");
                    let facts = if first {
                        format!("{place} {nums}")
                    } else {
                        format!("{nums} {place}")
                    };
                    for query in [a, b] {
                        out.push(ValueExample {id:format!("{prefix}/{i}/{owner}/{reverse}/{first}/{query}"),prompt:format!("User: {facts}\nUser: Where is the location of {query}?\nAssistant:"),response:if owner==query {format!(" {city}.\n")} else {" Unknown.\n".into()}});
                    }
                }
            }
        }
    }
    out
}
fn save_cases(
    out: &Path,
    label: &str,
    model: &Model,
    docs: &[ValueExample],
    control: Control,
) -> ProbeResult<()> {
    let report = source_noread::responses(model, docs, false, control)?;
    write_json(&out.join(format!("{label}.json")), &report)?;
    println!(
        "{}",
        json!({"split":label,"exact":report["exact"],"total":report["total"]})
    );
    Ok(())
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    match args.first().map(String::as_str) {
        Some("fit") if args.len() == 4 => {
            let root = Path::new(&args[2]);
            let out = Path::new(&args[3]);
            fs::create_dir(out)?;
            let model = Model::from_bytes(&fs::read(&args[1])?)?;
            let old: Value =
                serde_json::from_slice(&fs::read(root.join("literal-binding-source.json"))?)?;
            let mut fit: Vec<ValueExample> = serde_json::from_value(old["fit"].clone())?;
            fit.extend(contrasts(
                "context-fit",
                &[
                    ("adara", "cyris", "Rome", 13, 7),
                    ("hesta", "vurin", "Paris", -6, 23),
                ],
            ));
            let open = contrasts("context-open", &[("nelva", "sorvi", "Dover", 17, -5)]);
            write_json(
                &out.join("source.json"),
                &json!({"fit":fit,"development":open,"scope":"631 retained construction plus32 owner/query/order counterfactuals. Original query family; no fresh identities used."}),
            )?;
            save_cases(out, "parent-construction", &model, &fit, Control::Full)?;
            save_cases(out, "parent-development", &model, &open, Control::Full)?;
            let config = SourceRoutingConfig {
                learned_features: 256,
                passes: 2,
                proposals: 8,
                max_seconds: 20,
                role_context_only: true,
                ..Default::default()
            };
            let (candidate, report) = model.fit_retained_source_context(&fit, config)?;
            write_new(&out.join("model.json"), &candidate.to_bytes()?)?;
            write_json(&out.join("fit.json"), &report)?;
            println!("{}", report);
            let mut traces = Vec::new();
            for owner in ["ada", "cyra"] {
                let prompt=format!("User: {owner} lives in Rome. ada has 13 coins. other has 7 coins.\nUser: Where is the location of ada?\nAssistant:");
                traces.push(json!({"owner":owner,"parent":model.source_routing_trace(&prompt)?,"candidate":candidate.source_routing_trace(&prompt)?}));
            }
            write_json(&out.join("owner-counterfactual.json"), &json!(traces))?;
            save_cases(out, "construction", &candidate, &fit, Control::Full)?;
            save_cases(out, "development", &candidate, &open, Control::Full)?;
            save_cases(
                out,
                "disabled-construction",
                &candidate,
                &fit,
                Control::SourceContextDisabled,
            )?;
            save_cases(
                out,
                "window-construction",
                &candidate,
                &fit,
                Control::SourceContextWindowOnly,
            )?;
            for (file, split, label) in [
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
                (
                    "source-noread-source-v2.json",
                    "development",
                    "source-prior-open",
                ),
            ] {
                let source: Value = serde_json::from_slice(&fs::read(root.join(file))?)?;
                let docs: Vec<ValueExample> = serde_json::from_value(source[split].clone())?;
                save_cases(out, label, &candidate, &docs, Control::Full)?;
            }
            let preserve = out.join("preservation");
            fs::create_dir(&preserve)?;
            source_noread::preserve_model(candidate, root, &preserve)?;
        }
        Some("fresh") if args.len() == 3 => {
            let out = Path::new(&args[2]);
            let model = Model::from_bytes(&fs::read(&args[1])?)?;
            let docs = contrasts(
                "context-fresh",
                &[
                    ("velmi", "torva", "Merrow", 41, -18),
                    ("denra", "falvi", "Aster", -32, 59),
                ],
            );
            write_json(&out.join("fresh-source.json"), &json!(docs))?;
            for (label, control) in [
                ("fresh", Control::Full),
                ("fresh-parent", Control::SourceContextDisabled),
                ("fresh-window", Control::SourceContextWindowOnly),
            ] {
                save_cases(out, label, &model, &docs, control)?;
            }
        }
        _ => {
            return Err("source-context fit PARENT ROOT NEW_DIR | fresh MODEL EXISTING_DIR".into())
        }
    }
    Ok(())
}
