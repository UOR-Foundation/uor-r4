//! Bounded same-query shared operator transition experiment.
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, OperationTransitionExample, SourceRoutingConfig, TypedRoutingTurn, ValueAction,
};
fn cases(split: &str, count: usize, offset: i64) -> Vec<OperationTransitionExample> {
    let mut out = Vec::new();
    for i in 0..count {
        let a = 13 + offset + i as i64;
        let b = 4;
        let extra = 3 + i as i64 % 5;
        out.push(OperationTransitionExample { id:format!("{split}-{i}-sum-stop"), history:vec![], query:format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"), first_response:format!("{}.\n",a+b), next:None });
        for again in [false, true] {
            out.push(OperationTransitionExample{
            id:format!("{split}-{i}-{again}"),
            history:vec![TypedRoutingTurn{prompt:format!("User: suri has {a} coins. orin has {b} coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"),response:format!("{}.\n",a+b)}],
            query:format!("User: There are {extra} extra coins. Add the extra coins to the original total.{}\nAssistant:",if again{" Again."}else{""}),
            first_response:format!("{}.\n",a+b+extra),next:again.then_some((ValueAction::Add,extra)),
        });
        }
    }
    out
}
fn save(path: &Path, value: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: native_operation_transition <diagnose|fit|evaluate|fresh> MODEL OUT".into(),
        );
    }
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    let out = Path::new(&args[3]);
    fs::create_dir_all(out)?;
    let docs = if args[1] == "fresh" {
        {
            let mut v = cases("fresh-positive", 4, 109);
            v.extend(cases("fresh-negative", 4, -60));
            v
        }
    } else if args[1] == "evaluate" {
        cases("open", 8, 23)
    } else {
        cases("construction", 12, 0)
    };
    save(&out.join("cases.json"), &docs)?;
    if args[1] == "fit" {
        let (candidate, fit) = model.fit_operation_transition(
            &docs,
            SourceRoutingConfig {
                learned_features: 768,
                passes: 8,
                proposals: 24,
                max_seconds: 120,
                ..Default::default()
            },
        )?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &fit)?;
        save(
            &out.join("result.json"),
            &candidate.evaluate_operation_transition(&docs, Control::Full)?,
        )?;
    } else {
        save(
            &out.join("result.json"),
            &model.evaluate_operation_transition(&docs, Control::Full)?,
        )?;
        if args[1] == "evaluate" {
            save(
                &out.join("disabled.json"),
                &model
                    .evaluate_operation_transition(&docs, Control::OperationTransitionDisabled)?,
            )?;
            save(
                &out.join("intermediate-disabled.json"),
                &model.evaluate_operation_transition(
                    &docs,
                    Control::OperationTransitionIntermediateDisabled,
                )?,
            )?;
        }
    }
    Ok(())
}
