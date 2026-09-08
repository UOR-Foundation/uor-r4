//! Offline writer labels and direct complete-output/write checks. Runtime gets
//! raw prompts only; no authored phrase parser or answer enters serving.
use super::*;
use uor_r4_core::native_geometric::{RelationExample, RelationLabel};

fn labeled_reverse(doc: ValueExample) -> ProbeResult<RelationExample> {
    let at = doc
        .prompt
        .find(" holds ")
        .ok_or("authored reverse linker absent")?;
    if doc.prompt[at + 7..].contains(" holds ") {
        return Err("multiple assertions in single-reverse source adapter".into());
    }
    let owner_at = at + 7;
    let owner_len = doc.prompt[owner_at..]
        .find('.')
        .ok_or("authored owner end absent")?;
    if at == 0 || owner_len == 0 || doc.prompt[owner_at..owner_at + owner_len].contains(' ') {
        return Err("authored reverse source shape differs".into());
    }
    Ok(RelationExample {
        id: doc.id,
        prompt: doc.prompt,
        response: doc.response,
        writes: vec![RelationLabel {
            owner_end_byte: (owner_at + owner_len - 1) as u64,
            value_end_byte: (at - 1) as u64,
            action: 1,
        }],
    })
}

fn authored(
    tag: &str,
    worlds: &[(&str, &str, &str, &str)],
    long: bool,
) -> ProbeResult<Vec<RelationExample>> {
    let mut docs = Vec::new();
    for (world, &(intro, owner, full, single)) in worlds.iter().enumerate() {
        let padding = if long {
            "quiet sky. ".repeat(96)
        } else {
            String::new()
        };
        for (variant, (prefix, value)) in [(intro, full), ("", full), (intro, single), ("", single)]
            .into_iter()
            .enumerate()
        {
            docs.push(labeled_reverse(ValueExample {
                id: format!("{tag}/{world}/{variant}"),
                prompt: format!(
                    "{prefix}{value} holds {owner}. {padding}Where is {owner}? Answer:"
                ),
                response: format!(" {value}.\n"),
            })?);
        }
        for (variant, prefix) in [intro, ""].into_iter().enumerate() {
            docs.push(RelationExample {
                id: format!("{tag}/{world}/nonasserting-{variant}"),
                prompt: format!("{prefix}{full}. {padding}Where is {owner}? Answer:"),
                response: " Unknown.\n".into(),
                writes: Vec::new(),
            });
        }
    }
    Ok(docs)
}

fn evaluate(model: &Model, docs: &[RelationExample]) -> ProbeResult<Value> {
    let began = Instant::now();
    let mut cases = Vec::new();
    for doc in docs {
        let start = Instant::now();
        let generation = model.generate(&doc.prompt, 32, Control::Full)?;
        let generation_ns = start.elapsed().as_nanos();
        let inspection_start = Instant::now();
        let mut session = model.session(Control::Full)?;
        session.observe(model, 0)?;
        for token in model.encode(&doc.prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let input_record_writes = session.work.values.relations.record_writes;
        let output_record_writes = generation
            .work
            .values
            .relations
            .record_writes
            .checked_sub(input_record_writes)
            .ok_or("generation writes precede matching input writes")?;
        let wire: Value = serde_json::from_slice(&session.checkpoint()?)?;
        let records = wire["values"]["relations"]["records"]
            .as_array()
            .ok_or("relation state absent")?;
        let actual: Vec<_> = records
            .iter()
            .filter(|record| record["id"].as_u64().is_some_and(|id| id > 0))
            .map(|record| {
                json!({"owner_end_byte":record["owner"]["byte_end"],
                "value_end_byte":record["value"]["byte_end"],"action":record["action"]})
            })
            .collect();
        let writes_exact = input_record_writes == doc.writes.len() as u64
            && actual.len() == doc.writes.len()
            && doc.writes.iter().all(|label| {
                actual.iter().any(|record| {
                    record["owner_end_byte"] == label.owner_end_byte
                        && record["value_end_byte"] == label.value_end_byte
                        && record["action"] == label.action
                })
            });
        cases.push(json!({"id":doc.id,"prompt":doc.prompt,"expected":doc.response,"expected_writes":doc.writes,
            "actual_writes":actual,"writes_exact":writes_exact,"write_count":actual.len(),
            "input_record_writes":input_record_writes,"output_record_writes":output_record_writes,
            "nonasserting":doc.writes.is_empty(),
            "exact":generation.bytes == doc.response.as_bytes() && generation.stop == "end_of_document",
            "generation":{"text":generation.text,"bytes":generation.bytes,"stop":generation.stop,"token_ids":generation.token_ids},
            "generation_elapsed_ns":generation_ns,"inspection_elapsed_ns":inspection_start.elapsed().as_nanos(),"work":generation.work}));
    }
    Ok(json!({"artifact":model.artifact_cid(),"total":cases.len(),
        "exact":cases.iter().filter(|case|case["exact"] == true).count(),
        "writes_exact":cases.iter().filter(|case|case["writes_exact"] == true).count(),
        "generated_output_write_cases":cases.iter().filter(|case|case["output_record_writes"] != 0).count(),
        "nonasserting_total":cases.iter().filter(|case|case["nonasserting"] == true).count(),
        "nonasserting_false_positive_cases":cases.iter().filter(|case|case["nonasserting"] == true && case["write_count"] != 0).count(),
        "cases":cases,"elapsed_ms":began.elapsed().as_millis(),
        "scope":"Complete generated bytes plus EOS; all retained writer owner/endpoint/action records compared with offline labels, including zero-write controls. Second prompt ingestion obtains record evidence; its time is explicitly separate and charged in total elapsed. No full state or per-token traces saved."}))
}

fn save(out: &Path, name: &str, model: &Model, docs: &[RelationExample]) -> ProbeResult<Value> {
    let report = evaluate(model, docs)?;
    write_json(&out.join(format!("{name}.json")), &report)?;
    Ok(report)
}

fn passed(report: &Value) -> bool {
    report["total"].as_u64().is_some_and(|total| total > 0)
        && report["exact"] == report["total"]
        && report["writes_exact"] == report["total"]
        && report["generated_output_write_cases"] == 0
}

fn cache_equal(left: &Value, right: &Value) -> ProbeResult<bool> {
    let left = left["cases"].as_array().ok_or("uncached cases absent")?;
    let right = right["cases"].as_array().ok_or("cached cases absent")?;
    Ok(left.len() == right.len()
        && left.iter().zip(right).all(|(a, b)| {
            a["id"] == b["id"]
                && a["generation"] == b["generation"]
                && a["actual_writes"] == b["actual_writes"]
                && a["output_record_writes"] == b["output_record_writes"]
        }))
}

fn prior_phrases(directory: &Path) -> ProbeResult<Vec<RelationExample>> {
    let source: Value = serde_json::from_slice(&fs::read(directory.join("source.json"))?)?;
    let fresh: Value = serde_json::from_slice(&fs::read(directory.join("fresh-source.json"))?)?;
    let mut docs = Vec::new();
    for (label, raw, count) in [
        ("supplied", &source["supplied_fit"], 52),
        ("open", &source["development"], 12),
        ("exposed-fresh", &fresh["fresh"], 12),
    ] {
        let rows: Vec<ValueExample> = serde_json::from_value(raw.clone())?;
        if rows.len() != count {
            return Err(format!("prior phrase {label} count differs").into());
        }
        for mut row in rows {
            row.id = format!("writer-preserve/{label}/{}", row.id);
            docs.push(labeled_reverse(row)?);
        }
    }
    Ok(docs)
}

fn preserve(model: &Model, root: &Path, out: &Path, contextual: &Path) -> ProbeResult<bool> {
    let mut preserved = reverse_span::preserve(model, root, out)?;
    let source: Value = serde_json::from_slice(&fs::read(contextual.join("source.json"))?)?;
    let prior: Vec<ValueExample> = serde_json::from_value(source["prior_development"].clone())?;
    let prior: Vec<_> = prior
        .into_iter()
        .map(labeled_reverse)
        .collect::<ProbeResult<_>>()?;
    let report = save(&out.join("preservation"), "prior-start-open", model, &prior)?;
    preserved &= passed(&report);
    for (name, names, values, prefix) in [
        (
            "reverse-construction",
            ["nelra", "vesk"],
            ["Orin Grove", "Amber Grove", "Silver Cape"],
            "",
        ),
        (
            "reverse-open",
            ["serin", "mavra"],
            ["Copper Vale", "Cobalt Vale", "Ivory Pier"],
            "Record: ",
        ),
        (
            "reverse-prior-fresh",
            ["belvi", "norvi"],
            ["Quiet River Bend", "Silver River Bend", "Violet Quay"],
            "",
        ),
    ] {
        let report = reverse_span::panel(model, names, values, prefix)?;
        preserved &= report["exact"] == report["total"]
            && report["version_counts_exact"] == true
            && report["isolation"] == true;
        write_json(
            &out.join("preservation").join(format!("{name}.json")),
            &report,
        )?;
    }
    let short = reverse_span::short(model)?;
    preserved &= short["exact"] == short["total"];
    write_json(&out.join("preservation/reverse-short.json"), &short)?;
    Ok(preserved)
}

pub(super) fn run(args: &[String]) -> ProbeResult<()> {
    if args.first().map(String::as_str) == Some("preserve") {
        if args.len() != 5 {
            return Err(
                "writer-refinement preserve MODEL ROOT PRIOR_CONTEXT_DIR NEW_DIRECTORY".into(),
            );
        }
        OUTPUT_BYTES_REMAINING.store(64 * 1024 * 1024, Ordering::Relaxed);
        let model = Model::from_bytes(&fs::read(&args[1])?)?;
        let root = Path::new(&args[2]);
        let out = Path::new(&args[4]);
        fs::create_dir(out)?;
        let passed = preserve(&model, root, out, Path::new(&args[3]))?;
        write_json(
            &out.join("result.json"),
            &json!({"artifact":model.artifact_cid(),"preserved":passed,"scope":"Previously opened retained populations; no new held-out evaluation"}),
        )?;
        if !passed {
            return Err("retained population regression".into());
        }
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("inspect") {
        return inspect(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("boundary") {
        return run_fit(&args[1..], true);
    }
    run_fit(args, false)
}

fn boundary_construction() -> Vec<RelationExample> {
    let mut docs = Vec::new();
    for (world, (owner, material)) in [
        ("nevri", "fabric"),
        ("palri", "leather"),
        ("torvi", "foam"),
        ("zilra", "cork"),
    ]
    .into_iter()
    .enumerate()
    {
        for (variant, separator) in [".\n", ". "].into_iter().enumerate() {
            docs.push(RelationExample {
                id: format!("writer-boundary-fit/{world}/{variant}"),
                prompt: format!("User: {owner} carries {material} holds{separator}User: What does {owner} carry?\nAssistant:"),
                response: format!(" {material} holds.\n"), writes: Vec::new(),
            });
        }
    }
    docs
}

fn boundary_fresh() -> Vec<RelationExample> {
    let mut docs = Vec::new();
    for (world, (owner, material)) in [("fesri", "cotton"), ("lomri", "bamboo")]
        .into_iter()
        .enumerate()
    {
        for (variant, separator) in [".\n", ". "].into_iter().enumerate() {
            docs.push(RelationExample {
                id: format!("writer-boundary-fresh/{world}/{variant}"),
                prompt: format!("User: {owner} carries {material} holds{separator}User: What does {owner} carry?\nAssistant:"),
                response: format!(" {material} holds.\n"), writes: Vec::new(),
            });
        }
    }
    docs
}

fn report_subset(report: &Value, docs: &[RelationExample]) -> ProbeResult<Value> {
    let ids: BTreeSet<_> = docs.iter().map(|doc| doc.id.as_str()).collect();
    let rows: Vec<_> = report["cases"]
        .as_array()
        .ok_or("construction cases absent")?
        .iter()
        .filter(|row| row["id"].as_str().is_some_and(|id| ids.contains(id)))
        .cloned()
        .collect();
    if rows.len() != docs.len() {
        return Err("construction subset identities differ".into());
    }
    Ok(json!({"artifact":report["artifact"],"total":rows.len(),
        "exact":rows.iter().filter(|row|row["exact"] == true).count(),
        "writes_exact":rows.iter().filter(|row|row["writes_exact"] == true).count(),
        "generated_output_write_cases":rows.iter().filter(|row|row["output_record_writes"] != 0).count(),
        "cases":rows,"scope":"Subset of the same completed construction execution; no additional generation."}))
}

fn run_fit(args: &[String], boundary: bool) -> ProbeResult<()> {
    if args.len() != 5 {
        return Err("writer-refinement PARENT ROOT PRIOR_CONTEXT_DIR NEW_DIRECTORY EPOCHS".into());
    }
    let began = Instant::now();
    OUTPUT_BYTES_REMAINING.store(64 * 1024 * 1024, Ordering::Relaxed);
    let root = Path::new(&args[1]);
    let contextual = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    let epochs: usize = args[4].parse()?;
    fs::create_dir(out)?;
    let parent = Model::from_bytes(&fs::read(&args[0])?)?;
    let source: Value = serde_json::from_slice(&fs::read(
        root.join("writer-binding-continuation-source.json"),
    )?)?;
    let mut fit: Vec<RelationExample> = serde_json::from_value(source["fit"].clone())?;
    if fit.len() != 200 {
        return Err("expected exactly original200 writer anchors".into());
    }
    let anchors = fit.clone();
    let mut added = authored(
        "writer-refine-fit",
        &[
            ("notes say ", "ranvi", "quiet harbor", "quiet"),
            ("log tells ", "dorvi", "quiet meadow", "quiet"),
            ("report notes ", "verli", "sky bank", "sky"),
            ("file says ", "senvri", "city brook", "city"),
        ],
        false,
    )?;
    let boundary_docs = if boundary {
        boundary_construction()
    } else {
        Vec::new()
    };
    added.extend(boundary_docs.iter().cloned());
    fit.extend(added.iter().cloned());
    let open = authored(
        "writer-refine-open",
        &[
            ("notes say ", "tirvi", "quiet shore", "quiet"),
            ("report notes ", "velni", "sky grove", "sky"),
        ],
        false,
    )?;
    let prior = prior_phrases(contextual)?;
    let fit_count = if boundary { 232 } else { 224 };
    if fit.len() != fit_count || open.len() != 12 || prior.len() != 76 {
        return Err("writer source counts changed".into());
    }
    let boundary_scope = if boundary {
        json!({"mode":"explicit-boundary-refinement","original_anchor_count":200,"new_construction_count":32,"writer_label_count":232,
            "construction":"Original200 and preceding24 unchanged, plus8 NoWrite object-phrase contrasts: four new materials/owners with period-newline versus period-space after terminal holds and fixed User headers. This varies the line/sentence crossing at the observed alias. No runtime fixture parser.",
            "criterion_change":"The preceding matched inspection established parent and candidate identical on200/200 actual complete outputs and writes, while both satisfy159/200 historical answer labels. Preserve all200 original writer labels and expected responses in reports. In this explicitly separate mode, require all232 writer labels, actual parent-output equality for original200, exact new32 answers, OPEN12, prior76 and inherited preservation; do not silently discard or relabel the41 legacy response failures.",
            "fresh":"The original twelve evicted writer-phrase cases are unchanged. Four authored short first-use boundary NoWrite cases (fesri/cotton and lomri/bamboo, each period-newline/period-space) retain raw source for object copying. All sixteen remain unopened until this selection passes."})
    } else {
        Value::Null
    };
    write_json(
        &out.join("source.json"),
        &json!({"fit":fit,"added_construction":added,"development":open,"prior_phrases":prior,"boundary_construction":boundary_docs,"boundary_scope":boundary_scope,
        "known_writer_cues":"quiet, sky and city are known in the frozen writer dictionary; notes and says are unknown there. Writer dictionary and contextual-start registry are distinct namespaces.",
        "scope":if boundary {
            "Exact original200 labeled writer anchors plus the unchanged24 context-word/value contrasts and eight new boundary NoWrite object cases. All232 writer labels remain required; original200 actual parent-output equality and new32 complete-answer correctness are distinct gates. Twelve OPEN cases and prior76 unchanged; old quiet river/selra two are OPEN repair cases, not fit or fresh. Old shape-only fresh eight remain unopened."
        } else {
            "Exact original200 labeled writer anchors plus24 authored context-word/value and nonassertion contrasts. Twelve OPEN cases. Prior52 phrase construction,12 OPEN and12 already exposed fresh cases retained unchanged except ID prefix; old quiet river/selra two are OPEN repair cases, not fit or fresh. Source labels are offline only. Old shape-only relation-start fresh eight remain unopened."
        },"epochs":epochs}),
    )?;
    let parent_added = save(out, "parent-added-construction", &parent, &added)?;
    let parent_open = save(out, "parent-development", &parent, &open)?;
    let parent_prior = save(out, "parent-prior-phrases", &parent, &prior)?;
    let parent_anchors = if boundary {
        save(out, "parent-anchors", &parent, &anchors)?
    } else {
        Value::Null
    };
    let fit_result = if boundary {
        parent.refine_relation_writer_with_boundaries(&fit, epochs)
    } else {
        parent.refine_relation_writer(&fit, epochs)
    };
    let (uncached, fit_report) = match fit_result {
        Ok(result) => result,
        Err(error) => {
            write_json(
                &out.join("fit-error.json"),
                &json!({"parent":parent.artifact_cid(),"error":error.to_string(),"status":"FIT_FAILED","fresh":"NOT_RUN","elapsed_ms":began.elapsed().as_millis()}),
            )?;
            return Err(error.into());
        }
    };
    write_json(&out.join("fit.json"), &fit_report)?;
    let control = uncached.without_relation_writer_refinement()?;
    if control.to_bytes()? != parent.to_bytes()? {
        return Err("restored writer parent differs".into());
    }
    let mut old: Value = serde_json::from_slice(&parent.to_bytes()?)?;
    let mut next: Value = serde_json::from_slice(&uncached.to_bytes()?)?;
    for model in [&mut old, &mut next] {
        let object = model.as_object_mut().ok_or("model shape")?;
        for key in [
            "artifact_cid",
            "uor_model_address",
            "relation_writer",
            "relation_writer_refinement",
        ] {
            object.remove(key);
        }
    }
    if old != next {
        return Err("refinement changed inherited model fields".into());
    }
    let (model, compilation) = uncached.compile_relation_admission(&fit)?;
    let mut uncached_wire: Value = serde_json::from_slice(&uncached.to_bytes()?)?;
    let mut cached_wire: Value = serde_json::from_slice(&model.to_bytes()?)?;
    let admission = cached_wire["relation_writer"]
        .as_object_mut()
        .ok_or("cached writer absent")?
        .remove("admission")
        .ok_or("compiled admission absent")?;
    for value in [&mut uncached_wire, &mut cached_wire] {
        let object = value.as_object_mut().ok_or("cache model shape")?;
        object.remove("artifact_cid");
        object.remove("uor_model_address");
    }
    if uncached_wire != cached_wire || admission["parent"] != uncached.artifact_cid() {
        return Err("NoWrite compilation changed learned parameters or cache lineage".into());
    }
    write_json(&out.join("admission.json"), &compilation)?;
    write_new(&out.join("uncached-model.json"), &uncached.to_bytes()?)?;
    write_new(&out.join("model.json"), &model.to_bytes()?)?;
    let reloaded = Model::from_bytes(&fs::read(out.join("model.json"))?)?;
    if model.to_bytes()? != reloaded.to_bytes()? {
        return Err("writer artifact roundtrip differs".into());
    }
    let mut cached_reports = Vec::new();
    let mut matched = true;
    for (label, docs) in [
        ("construction", &fit),
        ("development", &open),
        ("prior-phrases", &prior),
    ] {
        let uncached_report = save(out, &format!("uncached-{label}"), &uncached, docs)?;
        let cached_report = save(out, label, &model, docs)?;
        matched &= cache_equal(&uncached_report, &cached_report)?;
        cached_reports.push(cached_report);
    }
    let preserved = preserve(&model, root, out, contextual)?;
    let boundary_gate = if boundary {
        let candidate_anchors = report_subset(&cached_reports[0], &anchors)?;
        let candidate_added = report_subset(&cached_reports[0], &added)?;
        write_json(&out.join("candidate-anchors.json"), &candidate_anchors)?;
        write_json(&out.join("added-construction.json"), &candidate_added)?;
        let anchor_equal = cache_equal(&parent_anchors, &candidate_anchors)?;
        let labels_exact = cached_reports[0]["writes_exact"] == cached_reports[0]["total"];
        let no_output_writes = cached_reports[0]["generated_output_write_cases"] == 0;
        let gate = json!({"scope":boundary_scope,"parent_anchor_oracle_exact":parent_anchors["exact"],
            "candidate_anchor_oracle_exact":candidate_anchors["exact"],"anchor_total":200,"actual_parent_output_and_writes_equal":anchor_equal,
            "all_writer_labels_exact":labels_exact,"writer_label_count":232,"no_output_writes":no_output_writes,
            "added_exact":candidate_added["exact"],"added_writes_exact":candidate_added["writes_exact"],"added_total":32,
            "passed":anchor_equal && labels_exact && no_output_writes && passed(&candidate_added)});
        write_json(&out.join("boundary-construction-gate.json"), &gate)?;
        gate
    } else {
        Value::Null
    };
    let construction_passed = if boundary {
        boundary_gate["passed"] == true
    } else {
        passed(&cached_reports[0])
    };
    let selected =
        matched && preserved && construction_passed && cached_reports[1..].iter().all(passed);
    write_json(
        &out.join("selection.json"),
        &json!({"parent":parent.artifact_cid(),"artifact":model.artifact_cid(),
        "parent_control_bytes_equal":true,"other_model_fields_equal":true,"cache_parameters_and_lineage_equal":true,"cached_uncached_output_and_writes_equal":matched,
        "construction_exact":cached_reports[0]["exact"],"construction_writes_exact":cached_reports[0]["writes_exact"],"construction_total":fit_count,"boundary_gate":boundary_gate,
        "development_exact":cached_reports[1]["exact"],"development_writes_exact":cached_reports[1]["writes_exact"],"development_total":12,
        "prior_phrases_exact":cached_reports[2]["exact"],"prior_phrases_writes_exact":cached_reports[2]["writes_exact"],"prior_phrases_total":76,
        "preservation":preserved,"selected_before_fresh":selected,"fresh":"NOT_RUN_AT_SELECTION"}),
    )?;
    let fresh_report = if selected {
        let mut fresh = authored(
            "writer-refine-fresh",
            &[
                ("notes say ", "farvi", "quiet coast", "quiet"),
                ("report notes ", "jorli", "sky fen", "sky"),
            ],
            true,
        )?;
        if boundary {
            fresh.extend(boundary_fresh());
        }
        write_json(
            &out.join("fresh-source.json"),
            &json!({"fresh":fresh,"evicted_writer_phrase_cases":12,"short_boundary_nowrite_cases":if boundary {4} else {0},
            "scope":if boundary {
                "Sixteen first-use cases: original twelve evicted writer phrases unchanged, followed by four short boundary NoWrite object cases with new owners/materials and period-newline/period-space contrasts. The latter retain raw source because the object value currently uses source copying. All opened after selection only; no general prose claim."
            } else {
                "Twelve first-use familiar-form cases with new owners and multiword payloads; role-value words remain familiar. Source eviction padding precedes query. Opened after selection only; no general prose claim."
            }}),
        )?;
        save(out, "fresh", &model, &fresh)?
    } else {
        json!({"status":"NOT_RUN_SELECTION_FAILED"})
    };
    let report = json!({"parent":parent.artifact_cid(),"artifact":model.artifact_cid(),"selected":selected,"boundary_gate":boundary_gate,
        "parent_added_exact":parent_added["exact"],"parent_open_exact":parent_open["exact"],"parent_prior_exact":parent_prior["exact"],
        "cached_uncached_equal":matched,"preservation":preserved,"fresh_exact":fresh_report["exact"],"fresh_writes_exact":fresh_report["writes_exact"],
        "fresh_total":fresh_report["total"],"fresh_status":fresh_report["status"],"elapsed_ms":began.elapsed().as_millis()});
    write_json(&out.join("report.json"), &report)?;
    println!("{report}");
    Ok(())
}

fn atom_text(atom: &Value) -> String {
    let bytes: Vec<_> = atom["bytes"]
        .as_array()
        .into_iter()
        .flatten()
        .take(atom["len"].as_u64().unwrap_or(0) as usize)
        .filter_map(Value::as_u64)
        .map(|byte| byte as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn inspect_span(model: &Model, docs: &[ValueExample]) -> ProbeResult<Value> {
    let mut cases = Vec::new();
    for doc in docs {
        let start = Instant::now();
        let generation = model.generate(&doc.prompt, 32, Control::Full)?;
        let mut session = model.session(Control::Full)?;
        session.observe(model, 0)?;
        for token in model.encode(&doc.prompt)? {
            session.observe(model, token)?;
        }
        session.begin_response(model)?;
        let wire: Value = serde_json::from_slice(&session.checkpoint()?)?;
        let records = wire["values"]["relations"]["records"]
            .as_array()
            .ok_or("span diagnostic records absent")?;
        let records: Vec<_> = records
            .iter()
            .filter(|record| record["id"].as_u64().is_some_and(|id| id > 0))
            .map(|record| {
                json!({"id":record["id"],"owner":atom_text(&record["owner"]),
                "owner_end_byte":record["owner"]["byte_end"],"value":atom_text(&record["value"]),
                "value_end_byte":record["value"]["byte_end"],"action":record["action"],
                "span":record["span"],"previous":record["previous"],"conflict":record["conflict"]})
            })
            .collect();
        let source_trace = model.source_routing_trace(&doc.prompt)?;
        cases.push(json!({"id":doc.id,"prompt":doc.prompt,"expected":doc.response,
            "exact":generation.bytes == doc.response.as_bytes() && generation.stop == "end_of_document",
            "generation":{"text":generation.text,"bytes":generation.bytes,"stop":generation.stop,"token_ids":generation.token_ids},
            "actual_records":records,"retained_record_count":records.len(),
            "input_record_writes":session.work.values.relations.record_writes,"generation_record_writes":generation.work.values.relations.record_writes,
            "source_routing_trace":source_trace,"generation_work":generation.work,"elapsed_ms":start.elapsed().as_millis()}));
    }
    Ok(
        json!({"artifact":model.artifact_cid(),"exact":cases.iter().filter(|case|case["exact"] == true).count(),
        "total":cases.len(),"cases":cases,"scope":"Read-only diagnosis of the exact eight already observed span failures; no new labels, fitting or fresh evaluation. Minimal committed owner/value bytes/endpoints and source routing trace retained."}),
    )
}

fn inspect(args: &[String]) -> ProbeResult<()> {
    if args.len() != 4 {
        return Err("writer-refinement inspect PARENT CANDIDATE ROOT NEW_DIRECTORY".into());
    }
    OUTPUT_BYTES_REMAINING.store(64 * 1024 * 1024, Ordering::Relaxed);
    let began = Instant::now();
    let parent = Model::from_bytes(&fs::read(&args[0])?)?;
    let candidate = Model::from_bytes(&fs::read(&args[1])?)?;
    let root = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    fs::create_dir(out)?;
    let source: Value = serde_json::from_slice(&fs::read(
        root.join("writer-binding-continuation-source.json"),
    )?)?;
    let anchors: Vec<RelationExample> = serde_json::from_value(source["fit"].clone())?;
    if anchors.len() != 200 {
        return Err("diagnostic expected original200 anchors".into());
    }
    write_json(
        &out.join("anchor-source.json"),
        &json!({"anchors":anchors,"scope":"Original200 source and writer labels unchanged; generated-output correctness and parent equality are separate diagnostics."}),
    )?;
    let parent_report = save(out, "parent-anchors", &parent, &anchors)?;
    let candidate_report = save(out, "candidate-anchors", &candidate, &anchors)?;
    let parent_rows = parent_report["cases"]
        .as_array()
        .ok_or("parent anchor cases absent")?;
    let candidate_rows = candidate_report["cases"]
        .as_array()
        .ok_or("candidate anchor cases absent")?;
    let comparison: Vec<_> = parent_rows.iter().zip(candidate_rows).map(|(p,c)| json!({"id":p["id"],
        "identity_equal":p["id"] == c["id"],"parent_exact":p["exact"],"candidate_exact":c["exact"],
        "parent_writes_exact":p["writes_exact"],"candidate_writes_exact":c["writes_exact"],
        "output_equal":p["generation"] == c["generation"],"writes_equal":p["actual_writes"] == c["actual_writes"]})).collect();
    let failure_root = Path::new(&args[1])
        .parent()
        .ok_or("candidate directory absent")?
        .join("preservation");
    let mut spans = Vec::new();
    for name in [
        "prior-span-construction",
        "prior-span-development",
        "prior-span-fresh",
    ] {
        let report: Value =
            serde_json::from_slice(&fs::read(failure_root.join(format!("{name}.json")))?)?;
        for row in report["cases"]
            .as_array()
            .ok_or("prior span cases absent")?
            .iter()
            .filter(|row| row["exact"] == false)
        {
            spans.push(ValueExample {
                id: row["id"].as_str().ok_or("span id absent")?.into(),
                prompt: row["prompt"].as_str().ok_or("span prompt absent")?.into(),
                response: row["expected"]
                    .as_str()
                    .ok_or("span expected absent")?
                    .into(),
            });
        }
    }
    if spans.len() != 8 {
        return Err("diagnostic expected exact eight previously observed span failures".into());
    }
    write_json(
        &out.join("span-source.json"),
        &json!({"cases":spans,"previous_candidate":candidate.artifact_cid(),"scope":"Exact failure rows from the candidate's saved preservation reports; all are already exposed."}),
    )?;
    let parent_spans = inspect_span(&parent, &spans)?;
    let candidate_spans = inspect_span(&candidate, &spans)?;
    write_json(&out.join("parent-spans.json"), &parent_spans)?;
    write_json(&out.join("candidate-spans.json"), &candidate_spans)?;
    let report = json!({"parent":parent.artifact_cid(),"candidate":candidate.artifact_cid(),"anchors":200,
        "parent_anchor_exact":parent_report["exact"],"candidate_anchor_exact":candidate_report["exact"],
        "parent_anchor_writes_exact":parent_report["writes_exact"],"candidate_anchor_writes_exact":candidate_report["writes_exact"],
        "anchor_output_equal":comparison.iter().filter(|row|row["output_equal"] == true).count(),
        "anchor_writes_equal":comparison.iter().filter(|row|row["writes_equal"] == true).count(),
        "comparisons":comparison,"parent_span_exact":parent_spans["exact"],"candidate_span_exact":candidate_spans["exact"],
        "span_total":8,"elapsed_ms":began.elapsed().as_millis(),"scope":"Read-only matched parent diagnosis; does not alter original source labels, acceptance or fit. No fresh evaluation."});
    write_json(&out.join("report.json"), &report)?;
    println!("{report}");
    Ok(())
}
