//! Open development of the shared geometric typed-role selector on a retained parent.
//! Labels are offline only. Both intermediates and answers are emitted by native inference.
use std::{fs, path::PathBuf};
use uor_r4_core::native_geometric::{Model, SourceRoutingConfig, TypedRoutingExample, ValueAction};
fn examples(split: &str, count: usize, offset: i64) -> Vec<TypedRoutingExample> {
    (0..count)
        .map(|i| {
            let a = 13 + offset + i as i64;
            let b = 4;
            let extra = 3 + (i % 7) as i64;
            let action = if i % 3 == 0 {
                ValueAction::Copy
            } else {
                ValueAction::Add
            };
            let query = if action == ValueAction::Copy {
                "User: Repeat the original total.\nAssistant:".to_owned()
            } else {
                format!("User: There are {extra} extra coins. Add the extra coins to the original total.\nAssistant:")
            };
            TypedRoutingExample {
                id: format!("shared-{split}-{i}"),
                literal_only: false,
                current_query_operand: action == ValueAction::Add,
                current_query_literals: false,
                initial_prompt: format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),
                initial_response: format!("{}.\n", a + b),
                continuation: None,
                refresh: vec![],
                target_intermediate: 0,
                query,
                response: format!(
                    "{}.\n",
                    if action == ValueAction::Copy {
                        a + b
                    } else {
                        a + b + extra
                    }
                ),
                action: Some(action),
                operands: Some(if action == ValueAction::Copy {
                    [a + b, a + b]
                } else {
                    [a + b, extra]
                }),
            }
        })
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: native_shared_transition <fit|fit-replay|fresh|evaluate|export-fresh> MODEL OUTPUT_DIRECTORY".into(),
        );
    }
    let out = PathBuf::from(&args[3]);
    fs::create_dir_all(&out)?;
    if args[1] == "export-fresh" {
        fs::write(
            out.join("source.json"),
            serde_json::to_vec_pretty(&examples("fresh", 12, 93))?,
        )?;
        return Ok(());
    }
    if !["fit", "fit-replay", "fresh", "evaluate"].contains(&args[1].as_str()) {
        return Err("unknown shared transition mode".into());
    }
    let parent = Model::from_bytes(&fs::read(&args[2])?)?;
    if args[1] == "fit" || args[1] == "fit-replay" {
        let mut train = examples("construction", 24, 0);
        if args[1] == "fit-replay" {
            let root = std::path::Path::new(&args[2])
                .parent()
                .and_then(|p| p.parent())
                .ok_or("retained source root absent")?;
            for name in [
                "typed-routing-case-source.json",
                "independent-reachable-source.json",
            ] {
                let source: serde_json::Value =
                    serde_json::from_slice(&fs::read(root.join(name))?)?;
                let mut rows: Vec<TypedRoutingExample> =
                    serde_json::from_value(source["fit"].clone())?;
                let mut seen = std::collections::BTreeSet::new();
                let mut prefixes = Vec::new();
                for row in &mut rows {
                    row.id = format!("replay/{name}/{}", row.id);
                    if name == "independent-reachable-source.json" {
                        if let Some(turn) = &row.continuation {
                            if turn.prompt.starts_with("User: ")
                                && turn.prompt.contains("What is the sum")
                                && seen.insert((row.initial_prompt.clone(), turn.prompt.clone()))
                            {
                                let nums: Vec<i64> = turn
                                    .prompt
                                    .split_whitespace()
                                    .filter_map(|s| s.parse().ok())
                                    .collect();
                                if nums.len() != 2 {
                                    return Err("construction continuation requires exactly two numeric literals".into());
                                }
                                let mut prefix = row.clone();
                                prefix.id = format!("current-literal-prefix/{}", row.id);
                                prefix.query = turn.prompt.clone();
                                prefix.response = turn.response.clone();
                                prefix.continuation = None;
                                prefix.refresh.clear();
                                prefix.target_intermediate = 0;
                                prefix.current_query_literals = true;
                                prefix.action = Some(ValueAction::Add);
                                prefix.operands = Some([nums[0], nums[1]]);
                                prefixes.push(prefix);
                            }
                        }
                    }
                }
                train.extend(rows);
                train.extend(prefixes);
            }
            let source: serde_json::Value =
                serde_json::from_slice(&fs::read(root.join("literal-admission-source.json"))?)?;
            let rows: Vec<TypedRoutingExample> = serde_json::from_value(source["fit"].clone())?;
            for mut row in rows
                .into_iter()
                .filter(|d| d.literal_only && d.action.is_some())
            {
                row.id = format!("query-literal/{}", row.id);
                row.literal_only = false;
                row.current_query_literals = true;
                row.initial_prompt = train[0].initial_prompt.clone();
                row.initial_response = train[0].initial_response.clone();
                train.push(row);
            }
        }
        let open = examples("open", 12, 40);
        fs::write(
            out.join("construction.json"),
            serde_json::to_vec_pretty(&train)?,
        )?;
        fs::write(out.join("open.json"), serde_json::to_vec_pretty(&open)?)?;
        fs::write(
            out.join("parent-open.json"),
            serde_json::to_vec_pretty(&parent.evaluate_typed_routing(&open, false)?)?,
        )?;
        let (candidate, fit) = parent.refine_typed_roles(
            &train,
            SourceRoutingConfig {
                learned_features: 768,
                passes: 8,
                proposals: 24,
                max_seconds: if args[1] == "fit-replay" { 120 } else { 60 },
                ..Default::default()
            },
        )?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        fs::write(out.join("fit.json"), serde_json::to_vec_pretty(&fit)?)?;
        for (name, docs, remove) in [
            ("construction", &train, false),
            ("open", &open, false),
            ("open-ablation", &open, true),
        ] {
            let report = candidate.evaluate_typed_routing(docs, remove)?;
            println!("{name}: {}/{}", report["exact"], report["total"]);
            fs::write(
                out.join(format!("{name}-result.json")),
                serde_json::to_vec_pretty(&report)?,
            )?;
        }
    } else {
        let docs = examples(
            if args[1] == "fresh" { "fresh" } else { "open" },
            12,
            if args[1] == "fresh" { 93 } else { 40 },
        );
        fs::write(out.join("source.json"), serde_json::to_vec_pretty(&docs)?)?;
        let report = parent.evaluate_typed_routing(&docs, false)?;
        println!(
            "{}",
            serde_json::json!({"exact":report["exact"],"total":report["total"]})
        );
        fs::write(out.join("result.json"), serde_json::to_vec_pretty(&report)?)?;
    }
    Ok(())
}
