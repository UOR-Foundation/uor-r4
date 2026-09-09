//! Open instruction/fact diagnostic. Labels never enter serving observations.
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{Control, Model, BOS, EOS};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checkpoint(s: &uor_r4_core::native_geometric::Session) -> Result<Value> {
    Ok(serde_json::from_slice(&s.checkpoint()?)?)
}
fn records(v: &Value) -> Vec<Value> {
    v["values"]["relations"]["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r["id"].as_u64().is_some_and(|n| n != 0))
        .cloned()
        .collect()
}
fn text(v: &Value) -> Option<String> {
    let n = v["len"].as_u64()? as usize;
    let b = v["bytes"]
        .as_array()?
        .iter()
        .take(n)
        .map(|x| x.as_u64().and_then(|x| u8::try_from(x).ok()))
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(b).ok()
}
fn run(model: &Model, prompt: &str) -> Result<Value> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    let mut writes = Vec::new();
    let mut known = Vec::new();
    for (position, t) in model.encode(prompt)?.into_iter().enumerate() {
        s.observe(model, t)?;
        let state = checkpoint(&s)?;
        let now = records(&state);
        for r in now.iter().filter(|r| !known.contains(&r["id"])) {
            writes.push(json!({"position":position,"token":t,"record":r,"owner_text":text(&r["owner"]),"value_text":text(&r["value"]),"span_text":text(&r["span"])}));
        }
        known = now.iter().map(|r| r["id"].clone()).collect();
    }
    s.begin_response(model)?;
    let initial = checkpoint(&s)?;
    let mut output = Vec::new();
    let mut decisions = Vec::new();
    let mut eos = false;
    for position in 0..96 {
        let p = s.predict(model)?;
        if position == 0 {
            decisions.push(json!({"word":s.word_copy_decision(),"entry":s.response_entry_decision(),"numeric":s.value_decision()}));
        }
        s.observe(model, p.token)?;
        if p.token == EOS {
            eos = true;
            break;
        }
        output.push(p.token);
    }
    Ok(
        json!({"text":String::from_utf8(model.decode(&output)?)?,"eos":eos,"initial_records":records(&initial),"writes":writes,"decisions":decisions,"source_trace":model.source_routing_trace(prompt)?}),
    )
}
fn diagnostic_main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: native_instruction_fact MODEL PREVIOUS_ABSTENTION_REPORT OUT".into());
    }
    let model = Model::from_bytes(&fs::read(&args[1])?)?;
    let raw = fs::read(&args[2])?;
    let parent: Value = serde_json::from_slice(&raw)?;
    let rows = parent["rows"].as_array().ok_or("rows absent")?;
    let out = Path::new(&args[3]); // args[4] is a declared diagnostic label.
    fs::create_dir(out)?;
    let mut result = Vec::new();
    for r in rows {
        let prompt = r["case"]["prompt"].as_str().ok_or("prompt absent")?;
        for present in [false, true] {
            let p = if present {
                prompt.replace("Record: tovin", "Record: velra")
            } else {
                prompt.into()
            };
            let actual = run(&model, &p)?;
            result.push(json!({"id":format!("{}/{}",r["id"].as_str().ok_or("id absent")?,if present{"matched-present"}else{"absent"}),"prompt":p,"present":present,"previous":if present{Value::Null}else{r["text"].clone()},"actual":actual}));
            fs::write(
                out.join("result.json"),
                serde_json::to_vec(
                    &json!({"artifact":model.artifact_cid(),"source_blake3":blake3::hash(&raw).to_hex().to_string(),"label":args[4],"completed":result.len(),"total":rows.len()*2,"rows":result,"scope":"Open diagnostic; exact source candidates, dispatch, learned feature scores and observed relation writes; matched owner identity only is changed."}),
                )?,
            )?;
        }
    }
    Ok(())
}
fn prepare(preservation: &Path, word_data: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases = Vec::new();
    let forms = [
        ("plain", "", false),
        ("place-prefix", "Explain in a sentence. ", true),
        ("place-suffix", "Explain in a sentence. ", false),
        ("stop-prefix", "Explain the stop in a sentence. ", true),
        ("stop-suffix", "Explain the stop in a sentence. ", false),
        (
            "neutral6-prefix",
            "farnel vopra duskin zemer pelka norvi. ",
            true,
        ),
        (
            "neutral6-suffix",
            "farnel vopra duskin zemer pelka norvi. ",
            false,
        ),
        ("neutral4-suffix", "farnel vopra duskin zemer. ", false),
        (
            "neutral8-suffix",
            "farnel vopra duskin zemer pelka norvi caspel ulven. ",
            false,
        ),
        ("neutral2-suffix", "farnel vopra. ", false),
    ];
    for (vi, value) in ["Cedar", "Amber"].iter().enumerate() {
        for (style, instruction, before) in forms {
            for present in [false, true] {
                let owner = if present { "velra" } else { "tovin" };
                let question = "Where is velra? ";
                let prompt = format!(
                    "Record: {owner} in {value}. {}{}Answer:",
                    if before { instruction } else { question },
                    if before { question } else { instruction }
                );
                let ending = if style.starts_with("place") {
                    " is the place.\n"
                } else if style.starts_with("stop") {
                    " is the stop.\n"
                } else {
                    ".\n"
                };
                let expected = if present {
                    format!(" {value}{ending}")
                } else {
                    " Unknown.\n".into()
                };
                cases.push(json!({"id":format!("position/{vi}/{style}/{}",if present{"present"}else{"absent"}),"source_report":"construction positions","source_id":style,"owner":"velra","value":if present{*value}else{"Unknown"},"placement":if present{style}else{"plain"},"prompt":prompt,"expected":expected,"retained_plain_expected":if present{format!(" {value}.\n")}else{" Unknown.\n".into()},"present":present}));
            }
        }
    }
    let mut train = Vec::new();
    let mut sources = Vec::new();
    for path in [
        preservation.join("retained-construction.json"),
        word_data.join("construction.json"),
        word_data.join("open.json"),
    ] {
        let raw = fs::read(&path)?;
        let v: Value = serde_json::from_slice(&raw)?;
        let rows = v
            .as_array()
            .or_else(|| v["cases"].as_array())
            .ok_or("preservation cases absent")?;
        for (i, r) in rows.iter().enumerate() {
            train.push(json!({"id":format!("preserve/{}/{i}",sources.len()),"prompt":r["prompt"],"target":"preserve"}));
        }
        sources.push(
            json!({"path":path,"blake3":blake3::hash(&raw).to_hex().to_string(),"rows":rows.len()}),
        );
    }
    for c in &cases {
        train.push(json!({"id":c["id"],"prompt":c["prompt"],"target":if c["present"]==true{"preserve"}else{"no_read"}}));
    }
    if train.len() > 768 {
        return Err("training case cap768".into());
    }
    // Extra present is diagnostic metadata, omitted from the existing Case wire.
    let wire: Vec<_> = cases
        .iter()
        .cloned()
        .map(|mut c| {
            c.as_object_mut().unwrap().remove("present");
            c
        })
        .collect();
    fs::write(out.join("cases.json"), serde_json::to_vec(&wire)?)?;
    fs::write(out.join("training.json"), serde_json::to_vec(&train)?)?;
    fs::write(
        out.join("receipt.json"),
        serde_json::to_vec(
            &json!({"sources":sources,"training":train.len(),"positions":cases.len(),"no_read":cases.iter().filter(|c|c["present"]==false).count(),"scope":"Open construction. Preserve exact parent decisions for retained cases; NoRead labels for absent-owner position contrasts. No output text observed by serving."}),
        )?,
    )?;
    Ok(())
}
fn fresh(out: &Path) -> Result<()> {
    let elapsed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let seed = elapsed.as_nanos() as u64;
    let mut rng = if seed == 0 { 1 } else { seed };
    let mut used = std::collections::BTreeSet::new();
    let mut draw = |title: bool| -> Result<String> {
        for _ in 0..1024 {
            let mut word = String::new();
            for _ in 0..4 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                word.push((b'a' + (rng % 26) as u8) as char);
            }
            if !matches!(word.as_str(), "stop" | "city" | "live" | "what" | "does")
                && used.insert(word.clone())
            {
                if title {
                    word[..1].make_ascii_uppercase();
                }
                return Ok(word);
            }
        }
        Err("fresh owner/value draw exhausted".into())
    };
    let forms = [
        ("plain", "", false),
        ("stop-prefix", "Explain the stop in a sentence. ", true),
        ("stop-suffix", "Explain the stop in a sentence. ", false),
        (
            "neutral6-suffix",
            "farnel vopra duskin zemer pelka norvi. ",
            false,
        ),
    ];
    let mut cases = Vec::new();
    let mut groups = Vec::new();
    for group in 0..4 {
        let owner = draw(false)?;
        let other_owner = draw(false)?;
        let value = draw(true)?;
        groups.push(json!({"group":group,"queried_owner":owner,"absent_fact_owner":other_owner,"value":value}));
        for (style, instruction, before) in forms {
            for present in [false, true] {
                let fact_owner = if present { &owner } else { &other_owner };
                let question = format!("Where is {owner}? ");
                let prompt = format!(
                    "Record: {fact_owner} in {value}. {}{}Answer:",
                    if before { instruction } else { &question },
                    if before { &question } else { instruction }
                );
                let ending = if style.starts_with("stop") {
                    " is the stop.\n"
                } else {
                    ".\n"
                };
                let expected = if present {
                    format!(" {value}{ending}")
                } else {
                    " Unknown.\n".into()
                };
                cases.push(json!({"id":format!("fresh-position/{group}/{style}/{}",if present{"present"}else{"absent"}),
                    "source_report":"freshdraw/receipt.json","source_id":format!("groups/{group}/{style}"),
                    "owner":owner,"value":if present{value.as_str()}else{"Unknown"},
                    "placement":if present{style}else{"plain"},"prompt":prompt,"expected":expected,
                    "retained_plain_expected":if present{format!(" {value}.\n")}else{" Unknown.\n".into()}}));
            }
        }
    }
    if cases.len() != 32 {
        return Err("fresh position count must be32".into());
    }
    fs::create_dir(out)?;
    let bytes = serde_json::to_vec(&cases)?;
    fs::write(out.join("cases.json"), &bytes)?;
    fs::write(
        out.join("receipt.json"),
        serde_json::to_vec(&json!({
        "schema":"uor-r4.instruction-fact-fresh/1","seed":seed,
        "draw_unix_seconds":elapsed.as_secs(),"draw_unix_subsecond_nanos":elapsed.subsec_nanos(),
        "rng":"xorshift64; four ASCII letters per draw; unique case-folded payloads within this draw",
        "groups":groups,"forms":forms.iter().map(|f|f.0).collect::<Vec<_>>(),
        "cases":32,"present":16,"absent":16,
        "cases_blake3":blake3::hash(&bytes).to_hex().to_string(),
        "scope":"Post-selection owner/value transfer under four fixed retained prompt forms only. Model selection is externally bound; no model execution or fitting occurs in this preparation mode. New random draw does not establish global nonoccurrence in historical material. No general instruction understanding or general prose qualification."}))?,
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 3 && a[1] == "fresh" {
        return fresh(Path::new(&a[2]));
    }
    if a.len() == 5 && a[1] == "fit" {
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        let docs: Vec<uor_r4_core::native_geometric::SourceRoleRefinementExample> =
            serde_json::from_slice(&fs::read(&a[3])?)?;
        let config = uor_r4_core::native_geometric::SourceRoutingConfig {
            learned_features: 256,
            passes: 2,
            proposals: 120,
            max_seconds: 120,
            mode: uor_r4_core::native_geometric::RoutingMode::Angular,
            seed: 973,
            role_context_only: true,
        };
        let (candidate, report) = model.fit_source_role_refinement(&docs, config)?;
        fs::create_dir(&a[4])?;
        fs::write(Path::new(&a[4]).join("model.json"), candidate.to_bytes()?)?;
        fs::write(
            Path::new(&a[4]).join("fit.json"),
            serde_json::to_vec(&report)?,
        )?;
        return Ok(());
    }
    if a.len() == 5 && a[1] == "prepare" {
        return prepare(Path::new(&a[2]), Path::new(&a[3]), Path::new(&a[4]));
    }
    if a.len() == 4 && a[1] == "writer" {
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        fs::create_dir(&a[3])?;
        for (name, prompt) in [
            (
                "instruction",
                "Where is selvi? Explain in a sentence. Answer:",
            ),
            ("fact", "Where is selvi? velra in Dusk Ridge. Answer:"),
        ] {
            fs::write(
                Path::new(&a[3]).join(format!("{name}.json")),
                serde_json::to_vec(&model.relation_writer_trace(prompt)?)?,
            )?;
        }
        return Ok(());
    }
    if a.len() == 5 && a[1] == "positions" {
        let model = Model::from_bytes(&fs::read(&a[2])?)?;
        let cases: Vec<Value> = serde_json::from_slice(&fs::read(&a[3])?)?;
        fs::create_dir(&a[4])?;
        let mut rows = Vec::new();
        for c in cases {
            let prompt = c["prompt"].as_str().ok_or("prompt")?;
            rows.push(json!({"case":c,"actual":run(&model,prompt)?}));
        }
        fs::write(
            Path::new(&a[4]).join("result.json"),
            serde_json::to_vec(
                &json!({"artifact":model.artifact_cid(),"rows":rows,"scope":"Actual output plus exact source scores and observed writes"}),
            )?,
        )?;
        return Ok(());
    }
    diagnostic_main()
}
