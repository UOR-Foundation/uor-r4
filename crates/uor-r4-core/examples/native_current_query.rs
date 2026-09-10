//! Offline contrasts for current literals versus multiple actual computed results.
//! No labels or query classification enter the serving path.
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Model, SourceRoutingConfig, TypedRoutingExample, TypedRoutingTurn, ValueAction,
};

fn prompt(a: i64, b: i64) -> String {
    format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:")
}

fn contrasts(split: &str, groups: usize, offset: i64) -> Vec<TypedRoutingExample> {
    let mut rows = Vec::new();
    for i in 0..groups {
        let a = 13 + offset + i as i64;
        let b = 4;
        let extra = 3 + (i % 7) as i64;
        for kind in 0..6 {
            let (query, response, action, operands) = match kind {
                0 => (prompt(a, b), a + b, ValueAction::Add, [a, b]),
                1 => (
                    prompt(a + 2, b + 3),
                    a + b + 5,
                    ValueAction::Add,
                    [a + 2, b + 3],
                ),
                2 => (
                    "User: Repeat the original total.\nAssistant:".into(),
                    a + b,
                    ValueAction::Copy,
                    [a + b, a + b],
                ),
                3 => (
                    format!("User: There are {extra} extra coins. Add the extra coins to the original total.\nAssistant:"),
                    a + b + extra,
                    ValueAction::Add,
                    [a + b, extra],
                ),
                4 => (
                    format!("User: orin has {} coins. suri has {} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:", b + 3, a + 2),
                    a + b + 5,
                    ValueAction::Add,
                    [a + 2, b + 3],
                ),
                _ => (prompt(a + b, 0), a + b, ValueAction::Add, [a + b, 0]),
            };
            rows.push(TypedRoutingExample {
                id: format!("current-query-{split}-{i}-{kind}"),
                literal_only: false,
                current_query_operand: kind == 3,
                current_query_literals: kind < 2 || kind >= 4,
                initial_prompt: prompt(a, b),
                initial_response: format!("{}.\n", a + b),
                continuation: Some(TypedRoutingTurn {
                    prompt: prompt(a + 1, b),
                    response: format!("{}.\n", a + b + 1),
                }),
                refresh: Vec::new(),
                target_intermediate: 0,
                query,
                response: format!("{response}.\n"),
                action: Some(action),
                operands: Some(operands),
            });
        }
    }
    rows
}

fn stress() -> Vec<TypedRoutingExample> {
    let mut rows = Vec::new();
    for history in 1..=4 {
        for kind in 0..4 {
            let mut row = contrasts("stress", 1, 0)[0].clone();
            row.id = format!("history-{history}-variant-{kind}");
            if history == 1 {
                row.continuation = None;
            }
            for i in 2..history {
                row.refresh.push(TypedRoutingTurn {
                    prompt: prompt(13 + i as i64, 4),
                    response: format!("{}.\n", 17 + i),
                });
            }
            match kind {
                0 => {}
                1 => {
                    row.query = "User: orin has 7 coins. suri has 19 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:".into();
                    row.operands = Some([19, 7]);
                    row.response = "26.\n".into();
                }
                2 => {
                    row.query = prompt(17, 0);
                    row.operands = Some([17, 0]);
                }
                _ => {
                    row.query = "User: Repeat the original total.\nAssistant:".into();
                    row.action = Some(ValueAction::Copy);
                    row.operands = Some([17, 17]);
                    row.current_query_literals = false;
                }
            }
            rows.push(row);
        }
    }
    rows
}

fn save(
    path: impl AsRef<Path>,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: native_current_query <fit|evaluate|controls|stress|fresh> MODEL PRIOR_CONSTRUCTION_JSON OUT"
                .into(),
        );
    }
    let out = Path::new(&args[4]);
    uor_r4_core::report_output::claim(out)?;
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    let open = contrasts("open", 8, 40);
    if args[1] == "fit" {
        let mut training: Vec<TypedRoutingExample> = serde_json::from_slice(&fs::read(&args[3])?)?;
        training.extend(contrasts("construction", 12, 0));
        save(out.join("construction.json"), &training)?;
        save(out.join("open.json"), &open)?;
        save(
            out.join("parent-open.json"),
            &model.evaluate_typed_routing(&open, false)?,
        )?;
        let (candidate, report) = model.refine_typed_roles(
            &training,
            SourceRoutingConfig {
                learned_features: 768,
                passes: 8,
                proposals: 24,
                max_seconds: 120,
                ..Default::default()
            },
        )?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(out.join("fit.json"), &report)?;
        for (name, rows, control) in [
            ("construction", &training, false),
            ("open", &open, false),
            ("open-intermediate-removed", &open, true),
        ] {
            let result = candidate.evaluate_typed_routing(rows, control)?;
            println!("{name}: {}/{}", result["exact"], result["total"]);
            save(out.join(format!("{name}-result.json")), &result)?;
        }
    } else if ["evaluate", "controls", "stress", "fresh"].contains(&args[1].as_str()) {
        let rows = if args[1] == "fresh" {
            let mut fresh = contrasts("fresh-positive", 2, 109);
            fresh.extend(contrasts("fresh-negative", 2, -60));
            fresh
        } else if args[1] == "stress" {
            stress()
        } else {
            open
        };
        save(out.join("source.json"), &rows)?;
        let result = model.evaluate_typed_routing(&rows, args[1] == "controls")?;
        println!("{}/{}", result["exact"], result["total"]);
        save(out.join("result.json"), &result)?;
    } else {
        return Err("unknown mode".into());
    }
    Ok(())
}
