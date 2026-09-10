//! Offline contextual-instruction construction and freely generated behavior checks.
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Document, EmissionExample, EmissionPiece as P, InstructionExample, Model,
    SourceRoutingConfig, ValueAction,
};
fn cases(split: &str, groups: usize, offset: i64) -> Vec<EmissionExample> {
    let mut rows = Vec::new();
    for g in 0..groups {
        let a = 13 + offset + g as i64;
        let b = 4 + g as i64 % 3;
        for (i, (before, after, kind)) in [
            ("", "", 0),
            ("sentence. ", "", 1),
            ("Rust. ", "", 2),
            ("In a sentence, ", "", 1),
            ("In Rust, ", "", 2),
            ("", " Explain in a sentence.", 1),
            ("", " Write a Rust equality.", 2),
            ("Explain in a sentence. ", "", 1),
            ("Write a Rust equality. ", "", 2),
            ("Please answer in a sentence. ", "", 1),
            ("Please answer in Rust. ", "", 2),
            ("", " Please explain the result in a sentence.", 1),
        ]
        .iter()
        .enumerate()
        {
            let suffix = match kind {
                1 => Some(vec![
                    P::Text(" is ".into()),
                    P::Operand(0),
                    P::Text(" plus ".into()),
                    P::Operand(1),
                    P::Text(".\n".into()),
                ]),
                2 => Some(vec![
                    P::Text(" == ".into()),
                    P::Operand(0),
                    P::Text(" + ".into()),
                    P::Operand(1),
                    P::Text("\n".into()),
                ]),
                _ => None,
            };
            rows.push(EmissionExample{id:format!("{split}-{g}-{i}-{kind}"),history:vec![],query:format!("User: suri has {a} coins. orin has {b} coins.\nUser: {before}What is the sum of suri's and orin's coins?{after}\nAssistant:"),prefix:(a+b).to_string(),suffix});
        }
    }
    rows
}
fn save(p: &Path, v: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn evaluate(
    model: &Model,
    docs: &[EmissionExample],
    control: Control,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut result = model.evaluate_lexical_emission(docs, control)?;
    let rows = result["rows"]
        .as_array_mut()
        .ok_or("evaluation rows absent")?;
    for row in rows.iter_mut() {
        // The construction declares ordered source identities independently of
        // the model's chosen operands; equal sums do not excuse changed bindings.
        let ordered = row["writes"][0]["action"] == "add"
            && row["writes"][0]["operands"][0]["id"] == 1
            && row["writes"][0]["operands"][1]["id"] == 0;
        row["ordered_operands_exact"] = ordered.into();
        row["exact"] = (row["exact"] == true && ordered).into();
    }
    let exact = rows.iter().filter(|r| r["exact"] == true).count();
    result["exact"] = exact.into();
    Ok(result)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 5
        || !["fit", "evaluate", "fresh", "stress", "preserve-lexical"].contains(&a[1].as_str())
    {
        return Err("usage: native_instruction_binding <fit|evaluate|fresh|stress|preserve-lexical> MODEL PRESERVED_RESULT OUT".into());
    }
    let model = Model::from_bytes(&fs::read(&a[2])?)?;
    let out = Path::new(&a[4]);
    uor_r4_core::report_output::claim(out)?;
    if a[1] == "preserve-lexical" {
        let baseline: serde_json::Value = serde_json::from_slice(&fs::read(&a[3])?)?;
        let wire: serde_json::Value = serde_json::from_slice(&model.to_bytes()?)?;
        let parent = wire["instruction_binding"]["parent_artifact"]
            .as_str()
            .ok_or("instruction parent absent")?;
        if baseline["artifact"].as_str() != Some(parent) {
            return Err("lexical baseline is not the frozen parent".into());
        }
        let docs: Vec<EmissionExample> = serde_json::from_slice(&fs::read(
            Path::new(&a[3])
                .parent()
                .ok_or("baseline parent absent")?
                .join("cases.json"),
        )?)?;
        let mut result = model.evaluate_lexical_emission(&docs, Control::Full)?;
        let prior = baseline["rows"].as_array().ok_or("baseline rows absent")?;
        if docs.is_empty() || prior.is_empty() {
            return Err("empty lexical preservation population".into());
        }
        let rows = result["rows"].as_array_mut().ok_or("current rows absent")?;
        if rows.len() != prior.len() {
            return Err("baseline row count differs".into());
        }
        for (row, old) in rows.iter_mut().zip(prior) {
            if row["id"] != old["id"] || row["query"] != old["query"] {
                return Err("baseline case differs".into());
            }
            let writes = |r: &serde_json::Value| {
                r["writes"].as_array().map(|w| w.iter().map(|w| serde_json::json!({"action":w["action"],"value":w["value"],"operands":w["operands"].as_array().map(|o| o.iter().map(|o| serde_json::json!({"id":o["id"],"value":o["value"]})).collect::<Vec<_>>())})).collect::<Vec<_>>())
            };
            let preserved = row["text"] == old["text"]
                && row["reads"] == old["reads"]
                && row["eos"] == old["eos"]
                && writes(row) == writes(old);
            row["baseline_preserved"] = preserved.into();
            row["exact"] = (row["exact"] == true && old["exact"] == true && preserved).into();
        }
        let exact = rows.iter().filter(|r| r["exact"] == true).count();
        result["exact"] = exact.into();
        result["baseline_artifact"] = baseline["artifact"].clone();
        save(&out.join("result.json"), &result)?;
        return Ok(());
    }
    let docs = match a[1].as_str() {
        "evaluate" => cases("open", 2, 23),
        "fresh" => {
            let mut r = cases("fresh-positive", 1, 109);
            r.extend(cases("fresh-negative", 1, -60));
            r
        }
        "stress" => {
            let mut r = cases("changed-names", 1, 43);
            for d in &mut r {
                d.query = d.query.replace("suri", "ada").replace("orin", "cyra");
            }
            let mut other = cases("reversed", 1, 53);
            for d in &mut other {
                d.query = d.query.replace(
                    "suri has 66 coins. orin has 4 coins.",
                    "orin has 4 coins. suri has 66 coins.",
                );
            }
            r.extend(other);
            r
        }
        _ => {
            let mut rows = cases("construction", 3, 0);
            let mut reversed = cases("construction-reversed", 3, 0);
            for (g, group) in reversed.chunks_mut(12).enumerate() {
                let a = 13 + g as i64;
                let b = 4 + g as i64 % 3;
                for d in group {
                    d.query = d.query.replace(
                        &format!("suri has {a} coins. orin has {b} coins."),
                        &format!("orin has {b} coins. suri has {a} coins."),
                    );
                }
            }
            rows.extend(reversed);
            rows
        }
    };
    save(&out.join("cases.json"), &docs)?;
    if a[1] == "fit" {
        let raw: serde_json::Value = serde_json::from_slice(&fs::read(&a[3])?)?;
        if raw["artifact"].as_str() != Some(model.artifact_cid()) {
            return Err("main preservation artifact differs from parent".into());
        }
        let mut preserve: Vec<Document> = raw["cases"]
            .as_array()
            .ok_or("preservation cases absent")?
            .iter()
            .map(|r| {
                Ok(Document {
                    id: format!("preserve/{}", r["id"].as_str().ok_or("case id absent")?),
                    text: r["prompt"]
                        .as_str()
                        .ok_or("case prompt absent")?
                        .to_string(),
                })
            })
            .collect::<Result<_, Box<dyn std::error::Error>>>()?;
        // These are already-open parent preservation panels. Include their
        // actual initial/literal contexts, not just the 663 main responses.
        let parent_dir = Path::new(&a[3])
            .parent()
            .ok_or("preservation parent absent")?;
        let mut prompts: std::collections::BTreeSet<_> =
            preserve.iter().map(|d| d.text.clone()).collect();
        for name in ["preservation.json", "prior-chains.json", "prior-fresh.json"] {
            let prior: serde_json::Value =
                serde_json::from_slice(&fs::read(parent_dir.join(name))?)?;
            if prior["artifact"].as_str() != Some(model.artifact_cid()) {
                return Err("training preservation artifact differs from parent".into());
            }
            for row in prior["cases"]
                .as_array()
                .ok_or("prior preservation cases absent")?
            {
                let prompt = row["prompt"].as_str().or_else(|| {
                    if row["input"]["literal_only"] == true {
                        row["input"]["query"].as_str()
                    } else {
                        row["input"]["initial_prompt"].as_str()
                    }
                });
                if let Some(prompt) = prompt.filter(|p| !p.is_empty()) {
                    if prompts.insert(prompt.to_string()) {
                        preserve.push(Document {
                            id: format!(
                                "prior/{name}/{}",
                                row["id"].as_str().ok_or("prior case id absent")?
                            ),
                            text: prompt.to_string(),
                        });
                    }
                }
            }
        }
        save(&out.join("preservation-training.json"), &preserve)?;
        let training: Vec<_> = docs
            .iter()
            .map(|d| InstructionExample {
                id: d.id.clone(),
                prompt: d.query.clone(),
                action: ValueAction::Add,
                operands: [1, 0],
            })
            .collect();
        save(&out.join("training.json"), &training)?;
        let (candidate, fit) = model.fit_instruction_binding(
            &training,
            &preserve,
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
            &evaluate(&candidate, &docs, Control::Full)?,
        )?;
    } else {
        save(
            &out.join("result.json"),
            &evaluate(&model, &docs, Control::Full)?,
        )?;
        if a[1] == "evaluate" {
            save(
                &out.join("disabled.json"),
                &evaluate(&model, &docs, Control::InstructionBindingDisabled)?,
            )?;
        }
    }
    Ok(())
}
