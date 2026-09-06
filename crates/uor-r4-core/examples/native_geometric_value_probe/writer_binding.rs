//! Construction-only write labels and direct generated/state checks. No fixture
//! parser or expected answers are linked into native inference.
use super::*;
use uor_r4_core::native_geometric::{DependentReadExample, RelationExample, RelationLabel};

// The supplied dependent fixture declares simple leading assertions. This is
// an offline label adapter, deliberately rejecting any other construction form.
fn labeled(d: ValueExample) -> ProbeResult<RelationExample> {
    let end = d
        .prompt
        .find("Question:")
        .or_else(|| d.prompt.find("\nfn "))
        .unwrap_or(d.prompt.len());
    let mut writes = Vec::new();
    let mut offset = 0;
    for part in d.prompt[..end].split_inclusive('.') {
        if !part.ends_with('.') {
            break;
        }
        let words: Vec<_> = part
            .trim_matches(|c: char| c.is_whitespace() || c == '/' || c == '.')
            .split_whitespace()
            .collect();
        let (owner, value, action) = match words.as_slice() {
            [o, "in", v] => (*o, *v, 1),
            ["Now" | "now", o, "in", v] | [o, "now", "in", v] => (*o, *v, 2),
            [o, "not", "in", v] => (*o, *v, 3),
            _ => return Err(format!("undeclared construction statement: {part}").into()),
        };
        let oi = part.find(owner).ok_or("owner label absent")?;
        let vi = part.rfind(value).ok_or("value label absent")?;
        writes.push(RelationLabel {
            owner_end_byte: (offset + oi + owner.len() - 1) as u64,
            value_end_byte: (offset + vi + value.len() - 1) as u64,
            action,
        });
        offset += part.len();
    }
    Ok(RelationExample {
        id: d.id,
        prompt: d.prompt,
        response: d.response,
        writes,
    })
}
fn variant(d: &RelationExample) -> ProbeResult<RelationExample> {
    let mut next = ValueExample {
        id: format!("{}/owner-now", d.id),
        prompt: d.prompt.clone(),
        response: d.response.clone(),
    };
    let at = next.prompt.find("Now ").ok_or("revision cue absent")?;
    let owner = next.prompt[at + 4..]
        .split_whitespace()
        .next()
        .ok_or("owner absent")?
        .to_string();
    next.prompt = next
        .prompt
        .replacen(&format!("Now {owner} in"), &format!("{owner} now in"), 1);
    labeled(next)
}
fn evaluate(model: &Model, docs: &[RelationExample]) -> ProbeResult<Value> {
    let start = Instant::now();
    let mut rows = Vec::new();
    for d in docs {
        let generation = model.generate(&d.prompt, 32, Control::Full)?;
        let mut session = model.session(Control::Full)?;
        session.observe(model, 0)?;
        for t in model.encode(&d.prompt)? {
            session.observe(model, t)?;
        }
        session.begin_response(model)?;
        let wire: Value = serde_json::from_slice(&session.checkpoint()?)?;
        let records = wire["values"]["relations"]["records"]
            .as_array()
            .ok_or("relation state absent")?;
        let written: Vec<_> = records
            .iter()
            .filter(|r| r["id"].as_u64().unwrap_or(0) > 0)
            .collect();
        let writes_exact = written.len() == d.writes.len()
            && d.writes.iter().all(|l| {
                written.iter().any(|r| {
                    r["owner"]["byte_end"] == l.owner_end_byte
                        && r["value"]["byte_end"] == l.value_end_byte
                        && r["action"] == l.action
                })
            });
        rows.push(json!({"id":d.id,"prompt":d.prompt,"expected":d.response,"exact":generation.text==d.response,"writes_exact":writes_exact,"expected_writes":d.writes,"relations":wire["values"]["relations"],"generation":generation}));
    }
    Ok(
        json!({"artifact":model.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"writes_exact":rows.iter().filter(|r|r["writes_exact"]==true).count(),"total":rows.len(),"elapsed_ms":start.elapsed().as_millis(),"cases":rows}),
    )
}
fn responses(model: &Model, docs: &[ValueExample]) -> ProbeResult<Value> {
    let mut rows = Vec::new();
    for d in docs {
        let generation = model.generate(&d.prompt, 32, Control::Full)?;
        rows.push(json!({"id":d.id,"exact":generation.text==d.response,"expected":d.response,"generation":generation}));
    }
    Ok(
        json!({"artifact":model.artifact_cid(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"cases":rows}),
    )
}
pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    let mode = args
        .first()
        .map(String::as_str)
        .ok_or("writer-binding mode absent")?;
    if mode == "admission" || mode == "admission-verify" {
        if args.len() != if mode == "admission-verify" { 7 } else { 5 } {
            return Err("writer-binding admission MODEL SOURCE NEW_MODEL NEW_REPORT".into());
        }
        let model = Model::from_bytes(&fs::read(&args[1])?)?;
        let source: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
        let fit: Vec<RelationExample> = serde_json::from_value(source["fit"].clone())?;
        let (cached, compilation) = model.compile_relation_admission(&fit)?;
        let bytes = cached.to_bytes()?;
        let mut parent: Value = serde_json::from_slice(&model.to_bytes()?)?;
        let mut stripped: Value = serde_json::from_slice(&bytes)?;
        let admission = stripped["relation_writer"]
            .as_object_mut()
            .ok_or("replacement writer absent")?
            .remove("admission")
            .ok_or("replacement writer cache absent")?;
        for key in ["artifact_cid", "uor_model_address"] {
            parent
                .as_object_mut()
                .ok_or("parent object absent")?
                .remove(key);
            stripped
                .as_object_mut()
                .ok_or("candidate object absent")?
                .remove(key);
        }
        if parent != stripped || admission["parent"] != model.artifact_cid() {
            return Err("NoWrite compilation changed parent parameters or binding".into());
        }
        write_new(Path::new(&args[3]), &bytes)?;
        let report = json!({"parent":model.artifact_cid(),"artifact":cached.artifact_cid(),
            "compilation":compilation,"parameters_unchanged":true,"serialized_bytes":bytes.len(),
            "scope":"Construction prompts only; writer-scoped exact NoWrite metadata. All learned parameters, tokenizer, reader and inherited cache unchanged. No fitting or new reserve."});
        write_json(Path::new(&args[4]), &report)?;
        println!("{}", report);
        if mode == "admission-verify" {
            let out = Path::new(&args[6]);
            fs::create_dir(out)?;
            for (field, label) in [("development", "development"), ("fresh", "exposed-names")] {
                let docs: Vec<RelationExample> = serde_json::from_value(source[field].clone())?;
                write_json(
                    &out.join(format!("{label}.json")),
                    &evaluate(&cached, &docs)?,
                )?;
            }
            for label in ["preservation", "prior"] {
                let docs: Vec<ValueExample> = serde_json::from_value(source[label].clone())?;
                write_json(
                    &out.join(format!("{label}.json")),
                    &responses(&cached, &docs)?,
                )?;
            }
            let long: Value = serde_json::from_slice(&fs::read(&args[5])?)?;
            let docs: Vec<RelationExample> = serde_json::from_value(long["first_use"].clone())?;
            write_json(&out.join("long.json"), &evaluate(&cached, &docs)?)?;
            relation_memory::verify_model(cached, &out.join("sessions.json"))?;
        }
        return Ok(());
    }
    if mode == "continue-source" {
        if args.len() != 3 {
            return Err("writer-binding continue-source SOURCE NEW_SOURCE".into());
        }
        let mut source: Value = serde_json::from_slice(&fs::read(&args[1])?)?;
        let fit: Vec<RelationExample> = serde_json::from_value(source["fit"].clone())?;
        let mut next = fit.clone();
        // Generated bytes are excluded from the writer's observed-word stream.
        // The previous question therefore directly precedes the next assertion.
        for (i, d) in fit.iter().take(28).enumerate() {
            let prefix = if i % 2 == 0 {
                "Where is cyra? Answer: "
            } else {
                "Question: Where is the location of crate? Answer: "
            };
            let mut d = d.clone();
            d.id = format!("{}/after-question", d.id);
            d.prompt = format!("{prefix}{}", d.prompt);
            for w in &mut d.writes {
                w.owner_end_byte += prefix.len() as u64;
                w.value_end_byte += prefix.len() as u64;
            }
            next.push(d);
        }
        source["fit"] = serde_json::to_value(next)?;
        source["opened_fresh"] = source["fresh"].clone();
        let mut fresh: Vec<RelationExample> = serde_json::from_value(source["fresh"].clone())?;
        let previous = serde_json::to_string(&source)?;
        for (old, new) in [
            ("valdrin", "vornel"),
            ("wenrik", "ibren"),
            ("Tersul", "Ordel"),
            ("Poldex", "Quendal"),
            ("zorvek", "azveth"),
            ("ulveth", "dornek"),
            ("Nerdux", "Velmur"),
            ("Felsar", "Jorvax"),
        ] {
            if previous.contains(new) {
                return Err(format!("new reserve overlaps supplied source: {new}").into());
            }
            for d in &mut fresh {
                d.prompt = d.prompt.replace(old, new);
                d.response = d.response.replace(old, new);
            }
        }
        for d in &mut fresh {
            d.id = d.id.replace("fresh", "reserved");
        }
        // Renamed byte lengths differ; regenerate only the offline source labels.
        source["fresh"] = serde_json::to_value(
            fresh
                .into_iter()
                .map(|d| {
                    labeled(ValueExample {
                        id: d.id,
                        prompt: d.prompt,
                        response: d.response,
                    })
                })
                .collect::<ProbeResult<Vec<_>>>()?,
        )?;
        source["fresh_scope"] = json!("Replacement reserve after accidental opening of prior fresh set; familiar constructions, disjoint supplied vocabulary. Do not evaluate before final design selection.");
        write_json(Path::new(&args[2]), &source)?;
        println!("{}", json!({"fit":200,"new_reserved":28,"opened_fresh":28}));
        return Ok(());
    }
    if mode == "prepare" {
        if args.len() != 4 {
            return Err("writer-binding prepare RELATION_SOURCE DEPENDENT_SOURCE OUTPUT".into());
        }
        let old: Value = serde_json::from_slice(&fs::read(&args[1])?)?;
        let dep: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
        let mut fit: Vec<RelationExample> = serde_json::from_value(old["fit"].clone())?;
        let prior_fit: Vec<DependentReadExample> = serde_json::from_value(dep["fit"].clone())?;
        for d in prior_fit
            .into_iter()
            .filter(|d| d.example.id.starts_with("dependent/fit/"))
        {
            fit.push(labeled(d.example)?);
        }
        let variants: Vec<_> = fit
            .iter()
            .filter(|d| d.prompt.contains("Now "))
            .map(variant)
            .collect::<ProbeResult<_>>()?;
        fit.extend(variants);
        for (i, prompt) in [
            "Question: Where is the location of crate? Answer:",
            "Question: Where is ada? Answer:",
            "Question: Which city is ada in? Answer:",
            "Question: What city holds ada? Answer:",
        ]
        .iter()
        .enumerate()
        {
            fit.push(RelationExample {
                id: format!("writer/question/{i}"),
                prompt: prompt.to_string(),
                response: " Unknown.\n".into(),
                writes: vec![],
            });
        }
        let cases: Vec<DependentReadExample> = serde_json::from_value(dep["development"].clone())?;
        let development: Vec<_> = cases
            .into_iter()
            .map(|d| labeled(d.example))
            .collect::<ProbeResult<_>>()?;
        let mut fresh = Vec::new();
        let prior_text = serde_json::to_string(&old)? + &serde_json::to_string(&dep)?;
        for (old, new) in [
            ("casket", "valdrin"),
            ("elvin", "wenrik"),
            ("Bremen", "Tersul"),
            ("Zurich", "Poldex"),
            ("basket", "zorvek"),
            ("freya", "ulveth"),
            ("Turin", "Nerdux"),
            ("Lagos", "Felsar"),
        ] {
            if prior_text.contains(new) {
                return Err(format!("fresh name overlap: {new}").into());
            }
            let _ = old;
        }
        for d in development.iter().take(24) {
            let mut p = d.prompt.clone();
            let mut response = d.response.clone();
            for (old, new) in [
                ("casket", "valdrin"),
                ("elvin", "wenrik"),
                ("Bremen", "Tersul"),
                ("Zurich", "Poldex"),
                ("basket", "zorvek"),
                ("freya", "ulveth"),
                ("Turin", "Nerdux"),
                ("Lagos", "Felsar"),
            ] {
                p = p.replace(old, new);
                response = response.replace(old, new);
            }
            let d = labeled(ValueExample {
                id: d.id.replace("development", "fresh"),
                prompt: p,
                response,
            })?;
            if d.prompt.contains("Now ") {
                fresh.push(variant(&d)?);
            }
            fresh.push(d);
        }
        write_json(
            Path::new(&args[3]),
            &json!({"fit":fit,"development":development,"fresh":fresh,"preservation":dep["preservation"],"prior":dep["prior"],"acceptance":"Repair all8 OPEN revision failures while preserving40correct dependent,62prior,24transfer,28long-context exact answers/writes and5sessionturns. Require exact writes, not answers alone. Fresh28 familiar forms with names absent from supplied sources, run only after design selection; no general-language qualification."}),
        )?;
        println!(
            "{}",
            json!({"fit":fit.len(),"development":development.len(),"fresh":fresh.len()})
        );
        return Ok(());
    }
    if mode == "fit" {
        if args.len() != 5 {
            return Err("writer-binding fit MODEL SOURCE NEW_MODEL REPORT".into());
        }
        let model = Model::from_bytes(&fs::read(&args[1])?)?;
        let source: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
        let docs: Vec<RelationExample> = serde_json::from_value(source["fit"].clone())?;
        let (next, report) = model.refit_relation_writer(&docs, 64)?;
        write_new(Path::new(&args[3]), &next.to_bytes()?)?;
        write_json(Path::new(&args[4]), &report)?;
        println!("{report}");
        return Ok(());
    }
    if mode == "evaluate" || mode == "preserve" || mode == "fresh" {
        if args.len() != 4 {
            return Err(
                "writer-binding evaluate|preserve|fresh MODEL SOURCE NEW_OUTPUT_DIR".into(),
            );
        }
        let model = Model::from_bytes(&fs::read(&args[1])?)?;
        let source: Value = serde_json::from_slice(&fs::read(&args[2])?)?;
        let out = Path::new(&args[3]);
        fs::create_dir(out)?;
        let split = if mode == "fresh" {
            "fresh"
        } else {
            "development"
        };
        let docs: Vec<RelationExample> = serde_json::from_value(source[split].clone())?;
        let result = evaluate(&model, &docs)?;
        write_json(&out.join(format!("{split}.json")), &result)?;
        println!(
            "{}",
            json!({"split":split,"exact":result["exact"],"writes_exact":result["writes_exact"],"total":result["total"]})
        );
        if mode == "evaluate" || mode == "preserve" {
            for label in ["preservation", "prior"] {
                let docs: Vec<ValueExample> = serde_json::from_value(source[label].clone())?;
                let r = responses(&model, &docs)?;
                write_json(&out.join(format!("{label}.json")), &r)?;
                println!(
                    "{}",
                    json!({"split":label,"exact":r["exact"],"total":r["total"]})
                );
            }
        }
        if mode == "preserve" {
            let docs: Vec<RelationExample> = serde_json::from_value(source["fresh"].clone())?;
            let result = evaluate(&model, &docs)?;
            write_json(&out.join("exposed-names.json"), &result)?;
            println!(
                "{}",
                json!({"split":"exposed-names","exact":result["exact"],"writes_exact":result["writes_exact"],"total":result["total"]})
            );
        }
        if mode == "preserve" {
            relation_memory::verify_model(model, &out.join("sessions.json"))?;
        }
        return Ok(());
    }
    Err("unknown writer-binding mode".into())
}
