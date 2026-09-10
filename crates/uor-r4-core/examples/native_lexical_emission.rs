//! Offline shared byte/exact-record emission experiment; labels never serve responses.
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, EmissionExample, EmissionPiece as P, Model, SourceRoutingConfig, BOS, EOS,
};
fn cases(split: &str, groups: usize, offset: i64) -> Vec<EmissionExample> {
    let mut rows = Vec::new();
    for i in 0..groups {
        let a = 13 + offset + i as i64;
        let b = 4 + i as i64 % 3;
        for kind in 0..3 {
            let request = match kind {
                1 => "sentence. ",
                2 => "Rust. ",
                _ => "",
            };
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
            rows.push(EmissionExample{id:format!("{split}-{i}-{kind}"),history:vec![],query:format!("User: suri has {a} coins. orin has {b} coins.\nUser: {request}What is the sum of suri's and orin's coins?\nAssistant:"),prefix:(a+b).to_string(),suffix});
        }
    }
    rows
}
fn save(p: &Path, v: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}

fn rust_source(input: &Path, out: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let report: serde_json::Value = serde_json::from_slice(&fs::read(input)?)?;
    let rows = report["rows"].as_array().ok_or("missing result rows")?;
    let mut program = String::from("fn main() {\n");
    let mut count = 0;
    for r in rows {
        let id = r["id"].as_str().ok_or("missing case identity")?;
        if !id.ends_with("-2") {
            continue;
        }
        let text = r["text"].as_str().ok_or("missing generated text")?;
        if !text
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_whitespace() || b"+-=".contains(&b))
        {
            return Err("generated output outside numeric expression check scope".into());
        }
        program.push_str(&format!("assert!({text}, {id:?});\n"));
        count += 1;
    }
    if count == 0 {
        return Err("no generated Rust expressions".into());
    }
    program.push_str("}\n");
    uor_r4_core::report_output::claim(out)?;
    fs::write(out.join("generated_checks.rs"), program)?;
    save(
        &out.join("source-receipt.json"),
        &serde_json::json!({"source_result":input,"expressions":count,"scope":"Actual generated numeric equalities embedded unchanged into Rust assert statements; compile and run separately"}),
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4
        || ![
            "diagnose",
            "fit",
            "evaluate",
            "stress",
            "fresh",
            "rust-source",
        ]
        .contains(&a[1].as_str())
    {
        return Err(
            "usage: native_lexical_emission <diagnose|fit|evaluate|stress|fresh|rust-source> MODEL_OR_RESULT OUT".into(),
        );
    }
    if a[1] == "rust-source" {
        return rust_source(Path::new(&a[2]), Path::new(&a[3]));
    }
    let model = Model::from_bytes(&fs::read(&a[2])?)?;
    let out = Path::new(&a[3]);
    uor_r4_core::report_output::claim(out)?;
    let mut docs = match a[1].as_str() {
        "fresh" => {
            let mut d = cases("fresh-positive", 4, 109);
            d.extend(cases("fresh-negative", 4, -60));
            d
        }
        "evaluate" => cases("open", 6, 23),
        "stress" => {
            let mut rows = cases("stress-reversed", 2, 43);
            for (i, d) in rows.iter_mut().enumerate() {
                let n = i / 3;
                let a = 56 + n as i64;
                let b = 4 + n as i64 % 3;
                d.query = d.query.replace(
                    &format!("User: suri has {a} coins. orin has {b} coins."),
                    &format!("User: orin has {b} coins. suri has {a} coins."),
                );
            }
            let mut names = cases("stress-names", 2, 53);
            for d in &mut names {
                d.query = d.query.replace("suri", "ada").replace("orin", "cyra");
            }
            rows.extend(names);
            rows
        }
        _ => cases("construction", 6, 0),
    };
    if a[1] == "diagnose" {
        docs = cases("diagnostic", 1, 0);
        let base = docs[0].clone();
        for (i,q) in [
"User: suri has 13 coins. orin has 4 coins.\nUser: In a sentence, what is the sum of suri's and orin's coins?\nAssistant:",
"User: suri has 13 coins. orin has 4 coins.\nUser: In Rust, what is the sum of suri's and orin's coins?\nAssistant:",
"User: suri has 13 coins. orin has 4 coins.\nUser: sentence. What is the sum of suri's and orin's coins?\nAssistant:",
"User: suri has 13 coins. orin has 4 coins.\nUser: Rust. What is the sum of suri's and orin's coins?\nAssistant:",
"User: Answer in a sentence. suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:",
"User: Answer in Rust. suri has 13 coins. orin has 4 coins.\nUser: What is the sum of suri's and orin's coins?\nAssistant:"] .iter().enumerate(){let mut d=base.clone();d.id=format!("diagnostic-position-{i}");d.query=q.to_string();docs.push(d);}
    }
    save(&out.join("cases.json"), &docs)?;
    if a[1] == "diagnose" {
        let mut rows = Vec::new();
        for d in &docs {
            let mut s = model.session(Control::Full)?;
            s.observe(&model, BOS)?;
            for t in model.encode(&d.query)? {
                s.observe(&model, t)?;
            }
            s.begin_response(&model)?;
            let mut tokens = Vec::new();
            let mut eos = false;
            for _ in 0..96 {
                let p = s.predict(&model)?;
                s.observe(&model, p.token)?;
                if p.token == EOS {
                    eos = true;
                    break;
                }
                tokens.push(p.token);
            }
            rows.push(serde_json::json!({"id":d.id,"query":d.query,"text":String::from_utf8(model.decode(&tokens)?)?,"eos":eos}));
        }
        save(&out.join("result.json"), &rows)?;
        return Ok(());
    }
    if a[1] == "fit" {
        let (candidate, fit) = model.fit_lexical_emission(
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
            &candidate.evaluate_lexical_emission(&docs, Control::Full)?,
        )?;
    } else {
        save(
            &out.join("result.json"),
            &model.evaluate_lexical_emission(&docs, Control::Full)?,
        )?;
        if a[1] == "evaluate" {
            for (name, control) in [
                ("disabled", Control::LexicalEmissionDisabled),
                ("read-disabled", Control::LexicalRecordReadDisabled),
                (
                    "geometry-disabled",
                    Control::LexicalEmissionGeometryDisabled,
                ),
            ] {
                save(
                    &out.join(format!("{name}.json")),
                    &model.evaluate_lexical_emission(&docs, control)?,
                )?;
            }
        }
    }
    Ok(())
}
