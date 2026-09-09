//! Offline typed-field supervision; free generation receives prompt tokens only.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, FieldCompositionExample, FieldPiece, Model, RoutingMode, SourceRoutingConfig, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Serialize, Deserialize)]
struct Case {
    example: FieldCompositionExample,
    expected: Option<String>,
    provenance: String,
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn group(owner: &str, value: &str, index: usize, split: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (layout, fact) in [
        format!("Record: {owner} in {value}."),
        format!("Record: tilva in Oak Bay. Record: {owner} in {value}."),
    ]
    .into_iter()
    .enumerate()
    {
        for (style, instruction) in ["Name the owner first.", "State the owner first."]
            .into_iter()
            .enumerate()
        {
            cases.push(Case {
                example: FieldCompositionExample {
                    id: format!("{split}/{index}/{layout}/{style}"),
                    prompt: format!("{fact} Where is {owner}? {instruction} Answer:"),
                    owner: owner.into(),
                    value: value.into(),
                    pieces: vec![
                        FieldPiece::Bytes(" ".into()),
                        FieldPiece::Owner,
                        FieldPiece::Bytes(" is in ".into()),
                        FieldPiece::Value,
                        FieldPiece::Bytes(".\n".into()),
                    ],
                    inherit: false,
                },
                expected: Some(format!(" {owner} is in {value}.\n")),
                provenance: "Authored owner-first request, exact field labels used offline only"
                    .into(),
            });
        }
        for (style, instruction) in [
            "",
            "Explain in a sentence.",
            "Explain the stop in a sentence.",
        ]
        .into_iter()
        .enumerate()
        {
            let expected = match style {
                0 => format!(" {value}.\n"),
                1 => format!(" {value} is the place.\n"),
                _ => format!(" {value} is the stop.\n"),
            };
            cases.push(Case {
                example: FieldCompositionExample {
                    id: format!("{split}-inherit/{index}/{layout}/{style}"),
                    prompt: format!("{fact} Where is {owner}? {instruction} Answer:"),
                    owner: owner.into(),
                    value: value.into(),
                    pieces: vec![],
                    inherit: true,
                },
                expected: Some(expected),
                provenance: "Matched retained value-first request; fit Base against actual parent"
                    .into(),
            });
        }
    }
    cases
}
fn prepare(input: &Path, out: &Path, fresh: bool) -> Result<()> {
    fs::create_dir(out)?;
    let groups: Vec<(String, String)> = if fresh {
        // Identities are drawn after design selection; forms remain authored.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        (0..3)
            .map(|i| {
                let h = blake3::hash(format!("{now}:{i}").as_bytes());
                let b = h.as_bytes();
                let owner: String = (0..5).map(|j| (b'a' + b[j] % 26) as char).collect();
                let value: String = (5..9).map(|j| (b'a' + b[j] % 26) as char).collect();
                (owner, format!("{} Bay", value))
            })
            .collect()
    } else {
        ["selvi", "serin", "navri", "pelvi"]
            .into_iter()
            .flat_map(|owner| {
                ["Dusk Ridge", "Copper Vale"]
                    .into_iter()
                    .map(move |value| (owner.to_string(), value.to_string()))
            })
            .collect()
    };
    let mut cases = Vec::new();
    for (i, (owner, value)) in groups.iter().enumerate() {
        cases.extend(group(
            owner,
            value,
            i,
            if fresh { "fresh" } else { "construction" },
        ));
    }
    save(&out.join("new-cases.json"), &cases)?;
    if !fresh {
        let old: Vec<Value> = serde_json::from_slice(&fs::read(input)?)?;
        for row in old {
            let id = row["id"].as_str().ok_or("prior id")?;
            let prompt = row["prompt"].as_str().ok_or("prior prompt")?;
            cases.push(Case {
                example: FieldCompositionExample {
                    id: format!("prior/{id}"),
                    prompt: prompt.into(),
                    owner: String::new(),
                    value: String::new(),
                    pieces: vec![],
                    inherit: true,
                },
                expected: None,
                provenance: input.display().to_string(),
            });
        }
    }
    if cases.len() > 1024 {
        return Err("population cap1024".into());
    }
    save(&out.join("cases.json"), &cases)?;
    let docs: Vec<_> = cases.iter().map(|c| &c.example).collect();
    save(&out.join("training.json"), &docs)?;
    save(
        &out.join("receipt.json"),
        &json!({"at":format!("{:?}",std::time::SystemTime::now()),"cases":cases.len(),"groups":groups,"fresh_draw":fresh,"forms":"fixed authored owner-first and retained value-first", "source":input,"no_runtime_labels":true}),
    )?;
    Ok(())
}
fn generate(model: &Model, prompt: &str, control: Control, checkpoints: bool) -> Result<Value> {
    let mut s = model.session(control)?;
    s.observe(model, BOS)?;
    for t in model.encode(prompt)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    let initial: Value = serde_json::from_slice(&s.checkpoint()?)?;
    let mut output = Vec::new();
    let mut trace = Vec::new();
    let mut eos = false;
    let mut restored_positions = 0;
    for position in 0..96 {
        let mut restored = if checkpoints {
            Some(model.restore_session(&s.checkpoint()?)?)
        } else {
            None
        };
        let p = s.predict(model)?;
        if checkpoints && s.predict(model)? != p {
            return Err("repeated prediction differs".into());
        }
        if let Some(r) = &mut restored {
            if r.predict(model)? != p {
                return Err("restored prediction differs".into());
            }
        }
        let field_decision = s.field_composition_decision();
        let before: Value = serde_json::from_slice(&s.checkpoint()?)?;
        s.observe(model, p.token)?;
        let after: Value = serde_json::from_slice(&s.checkpoint()?)?;
        if let Some(r) = &mut restored {
            r.observe(model, p.token)?;
            let rs: Value = serde_json::from_slice(&r.checkpoint()?)?;
            for key in [
                "values",
                "word_copy",
                "response_entry",
                "completion",
                "field_composition",
            ] {
                if after[key] != rs[key] {
                    return Err(format!("restored {key} differs").into());
                }
            }
            restored_positions += 1;
        }
        trace.push(json!({"position":position,"token":p.token,"field_decision":field_decision,"field_before":before["field_composition"],"field_after":after["field_composition"],"word_copy":after["word_copy"]}));
        if p.token == EOS {
            eos = true;
            break;
        }
        output.push(p.token);
    }
    let final_state: Value = serde_json::from_slice(&s.checkpoint()?)?;
    Ok(
        json!({"text":String::from_utf8(model.decode(&output)?)?,"tokens":output,"eos":eos,"trace":trace,"checkpoint_positions":restored_positions,"initial_relations":initial["values"]["relations"],"final_relations":final_state["values"]["relations"],"field_state":final_state["field_composition"],"work":s.work.word_copy}),
    )
}
fn evaluate(
    model: &Model,
    parent: Option<&Model>,
    cases: &[Case],
    out: &Path,
    controls: bool,
) -> Result<()> {
    fs::create_dir_all(out)?;
    let mut rows = Vec::new();
    let mut exact = 0;
    let mut labelled = 0;
    let mut preserved = 0;
    let mut inherited = 0;
    let mut disabled_equal = 0;
    for case in cases {
        let actual = generate(
            model,
            &case.example.prompt,
            Control::Full,
            !case.example.inherit,
        )?;
        let correct = case
            .expected
            .as_ref()
            .map(|e| actual["text"] == *e && actual["eos"] == true);
        if let Some(ok) = correct {
            labelled += 1;
            exact += usize::from(ok);
        }
        let reference = parent
            .map(|p| generate(p, &case.example.prompt, Control::Full, false))
            .transpose()?;
        let same = reference.as_ref().map(|r| {
            actual["text"] == r["text"]
                && actual["eos"] == r["eos"]
                && actual["initial_relations"] == r["initial_relations"]
                && actual["final_relations"] == r["final_relations"]
        });
        if case.example.inherit && reference.is_some() {
            inherited += 1;
            preserved += usize::from(same == Some(true));
        }
        let mut interventions = Vec::new();
        if controls && !case.example.inherit {
            for control in [
                Control::FieldCompositionDisabled,
                Control::FieldCompositionContextDisabled,
                Control::FieldCompositionGeometryDisabled,
                Control::FieldCompositionReadDisabled,
            ] {
                let result = generate(model, &case.example.prompt, control, false)?;
                if control == Control::FieldCompositionDisabled
                    && reference
                        .as_ref()
                        .is_some_and(|r| result["text"] == r["text"] && result["eos"] == r["eos"])
                {
                    disabled_equal += 1;
                }
                interventions.push(json!({"control":control,"result":result}));
            }
        }
        rows.push(json!({"id":case.example.id,"prompt":case.example.prompt,"expected":case.expected,"inherit":case.example.inherit,"actual":actual,"correct":correct,"parent":reference,"parent_equal":same,"controls":interventions}));
    }
    save(
        &out.join("result.json"),
        &json!({"artifact":model.artifact_cid(),"total":cases.len(),"labelled":labelled,"exact":exact,"inherited":inherited,"preserved":preserved,"disabled_parent_equal":disabled_equal,"rows":rows,"scope":"Typed field free generation; inherited rows compare actual parent output, not independent language correctness"}),
    )?;
    println!(
        "{}",
        json!({"total":cases.len(),"labelled":labelled,"exact":exact,"inherited":inherited,"preserved":preserved,"disabled_parent_equal":disabled_equal})
    );
    Ok(())
}

fn stress(out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    for (i, (fact, owner, value)) in [
        ("Record: Dusk Ridge holds selvi.", "selvi", "Dusk Ridge"),
        (
            "Record: selvi in Dusk Ridge. Record: tilva in Dusk Ridge.",
            "selvi",
            "Dusk Ridge",
        ),
        (
            "Record: selvi in Dusk Ridge. Record: tilva in Dusk Ridge.",
            "tilva",
            "Dusk Ridge",
        ),
        (
            "Record: selvi in Dusk Ridge. selvi now in Copper Vale.",
            "selvi",
            "Copper Vale",
        ),
        ("Record: selvi in Rome.", "selvi", "Rome"),
        ("Record: a in sentence.", "a", "sentence"),
    ]
    .into_iter()
    .enumerate()
    {
        for (j, padding) in [String::new(), "oak ash elm ".repeat(40)]
            .into_iter()
            .enumerate()
        {
            cases.push(Case{example:FieldCompositionExample{id:format!("stress/{i}/{j}"),prompt:format!("{fact} {padding}Where is {owner}? Name the owner first. Answer:"),owner:owner.into(),value:value.into(),pieces:vec![FieldPiece::Bytes(" ".into()),FieldPiece::Owner,FieldPiece::Bytes(" is in ".into()),FieldPiece::Value,FieldPiece::Bytes(".\n".into())],inherit:false},expected:Some(format!(" {owner} is in {value}.\n")),provenance:"Open structural contrasts: reverse relation, same-spelling distinct occurrence, revision, width, persistent-source eviction".into()});
        }
    }
    for (i, prompt) in [
        "Where is selvi? Name the owner first. Answer:",
        "Record: tilva in Oak Bay. Where is selvi? Name the owner first. Answer:",
    ]
    .into_iter()
    .enumerate()
    {
        cases.push(Case {
            example: FieldCompositionExample {
                id: format!("stress/unknown/{i}"),
                prompt: prompt.into(),
                owner: String::new(),
                value: String::new(),
                pieces: vec![],
                inherit: true,
            },
            expected: Some(" Unknown.\n".into()),
            provenance: "Unsupported-owner parent preservation".into(),
        });
    }
    save(&out.join("cases.json"), &cases)?;
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 3 && a[1] == "stress" {
        return stress(Path::new(&a[2]));
    }
    if a.len() == 4 && a[1] == "prepare" {
        return prepare(Path::new(&a[2]), Path::new(&a[3]), false);
    }
    if a.len() == 3 && a[1] == "fresh" {
        return prepare(Path::new("post-selection draw"), Path::new(&a[2]), true);
    }
    if a.len() != 5 {
        return Err("usage: native_field_composition prepare PRIOR OUT | fresh OUT | fit/baseline/evaluate/controls MODEL CASES OUT".into());
    }
    let bytes = fs::read(&a[2])?;
    let model = Model::from_bytes(&bytes)?;
    let out = Path::new(&a[4]);
    fs::create_dir_all(out)?;
    if a[1] == "fit" {
        let docs: Vec<FieldCompositionExample> = serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = SourceRoutingConfig {
            learned_features: 4096,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: 973,
            role_context_only: false,
        };
        let (candidate, report) = model.fit_field_composition(&docs, config)?;
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &report)?;
        println!("{}", report);
        return Ok(());
    }
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    if a[1] == "baseline" {
        return evaluate(&model, None, &cases, out, false);
    }
    let parent = model.without_field_composition()?;
    save(
        &out.join("lineage.json"),
        &json!({"parent":parent.artifact_cid(),"candidate":model.artifact_cid(),"roundtrip":model.to_bytes()?==bytes,"parent_bytes_blake3":blake3::hash(&parent.to_bytes()?).to_hex().to_string()}),
    )?;
    evaluate(&model, Some(&parent), &cases, out, a[1] == "controls")
}
