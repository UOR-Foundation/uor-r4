//! Open diagnostic of retained nonnumeric facts under sentence instructions.
//! Authored targets are comparisons only; every output token comes from predict.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, RoutingMode, Session, SourceRoutingConfig, WordEmissionExample, BOS, EOS,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Serialize, Deserialize)]
struct Case {
    id: String,
    source_report: String,
    source_id: String,
    owner: String,
    value: String,
    placement: String,
    prompt: String,
    expected: String,
    retained_plain_expected: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    copy_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    inherit_suffix: bool,
}
fn save(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn checkpoint(s: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&s.checkpoint()?)?)
}
fn text_bytes(v: &Value) -> Option<String> {
    let n = v["len"].as_u64()? as usize;
    let bytes = v["bytes"].as_array()?;
    if n > bytes.len() {
        return None;
    }
    let b = bytes[..n]
        .iter()
        .map(|b| b.as_u64().and_then(|b| u8::try_from(b).ok()))
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(b).ok()
}
fn sources(state: &Value) -> Value {
    let relations = &state["values"]["relations"];
    let directory = &relations["directory"];
    let records: Vec<_> = relations["records"].as_array().into_iter().flatten()
        .filter(|r| r["id"].as_u64().is_some_and(|id|id != 0))
        .map(|r|json!({"id":r["id"],"current":directory.as_array().is_some_and(|d|d.contains(&r["id"])),
            "owner_text":text_bytes(&r["owner"]),"anchor_text":text_bytes(&r["value"]),
            "span_text":text_bytes(&r["span"]),"owner":r["owner"],"value":r["value"],"span":r["span"],
            "previous":r["previous"],"action":r["action"],"conflict":r["conflict"]})).collect();
    let words = &state["values"]["lexemes"];
    let count = words["query_len"].as_u64().unwrap_or(0) as usize;
    let queries: Vec<_> = words["queries"]
        .as_array()
        .into_iter()
        .flatten()
        .take(count)
        .map(|w| json!({"text":text_bytes(w),"atom":w}))
        .collect();
    json!({"relations":records,"directory":directory,"query_words":queries,"numeric_records":state["values"]["records"]})
}
fn selected_source(state: &Value, decision: &Value) -> Value {
    let commit = &state["word_copy"]["read_commit"];
    let relation_id = commit["relation_id"].as_u64();
    let relations: Vec<_> = state["values"]["relations"]["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| {
            r["id"].as_u64().is_some_and(|id| id != 0)
                && (relation_id == r["id"].as_u64()
                    || decision["dependency"]
                        .as_array()
                        .is_some_and(|d| d.contains(&r["id"])))
        })
        .cloned()
        .collect();
    let at = decision["source_end"].as_u64();
    let byte_at = decision["source_byte_end"].as_u64();
    let words: Vec<_> = state["values"]["lexemes"]["queries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|w| {
            w["len"].as_u64().is_some_and(|n| n != 0)
                && at == w["end"].as_u64()
                && byte_at == w["byte_end"].as_u64()
        })
        .cloned()
        .collect();
    json!({"read_commit":commit,"origin":state["word_copy"]["origin"],
        "span_words":state["word_copy"]["span_words"],"decision":decision,
        "matched_relation_records":relations,"matched_query_occurrences":words,
        "identity_resolved":!relations.is_empty() || !words.is_empty()})
}
fn generate(model: &Model, case: &Case, control: Control) -> Result<Value> {
    let mut s = model.session(Control::Full)?;
    s.observe(model, BOS)?;
    for t in model.encode(&case.prompt)? {
        s.observe(model, t)?;
    }
    s.begin_response(model)?;
    if control != Control::Full {
        let mut state = checkpoint(&s)?;
        state["control"] = json!(control);
        s = model.restore_session(&serde_json::to_vec(&state)?)?;
    }
    let initial = checkpoint(&s)?;
    let before_copy_work = serde_json::to_value(&s.work.word_copy)?;
    let initial_sources = sources(&initial);
    let labelled_relations: Vec<_> = initial_sources["relations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| {
            r["owner_text"] == case.owner
                && (r["span_text"] == case.value
                    || (r["span_text"].is_null() && r["anchor_text"] == case.value))
        })
        .cloned()
        .collect();
    let mut tokens = Vec::new();
    let mut trace = Vec::new();
    let mut selections = Vec::new();
    let mut eos = false;
    let mut completion_active = false;
    let mut numeric_anchor_seen = false;
    let mut lexical_read_seen = false;
    let mut preceding_state = initial.clone();
    let outcome = (|| -> Result<()> {
        for position in 0..96 {
            let mut restored = model.restore_session(&s.checkpoint()?)?;
            let routing_before = serde_json::to_value(&s.work.word_copy.routing)?;
            let p = s.predict(model)?;
            let word = s.word_copy_decision();
            let completion = s.completion_decision();
            let value = s.value_decision();
            let entry = s.response_entry_decision();
            if restored.predict(model)? != p
                || restored.word_copy_decision() != word
                || restored.completion_decision() != completion
                || restored.value_decision() != value
                || restored.response_entry_decision() != entry
                || s.predict(model)? != p
                || s.word_copy_decision() != word
                || s.completion_decision() != completion
                || s.value_decision() != value
            {
                return Err("checkpoint/repeated prediction differs".into());
            }
            s.observe(model, p.token)?;
            restored.observe(model, p.token)?;
            let state = checkpoint(&s)?;
            let restored_state = checkpoint(&restored)?;
            for key in ["values", "word_copy", "response_entry", "completion"] {
                if state[key] != restored_state[key] {
                    return Err(format!("checkpoint committed {key} differs").into());
                }
            }
            let c = &state["completion"];
            completion_active |= c["active"] == true;
            numeric_anchor_seen |= c["anchor"].is_object();
            lexical_read_seen |= c["lexical_read"].is_object();
            if let Some(decision) = word {
                let serialized = serde_json::to_value(decision)?;
                if matches!(
                    decision.action,
                    uor_r4_core::native_geometric::WordCopyAction::Read
                        | uor_r4_core::native_geometric::WordCopyAction::Prepare
                        | uor_r4_core::native_geometric::WordCopyAction::Start
                ) {
                    selections.push(selected_source(&state, &serialized));
                }
            }
            trace.push(json!({"position":position,"token":p.token,"word_copy_decision":word,"prior_word_progress":preceding_state["word_copy"]["progress"],"word_routing_before":routing_before,"word_routing_after":s.work.word_copy.routing,
                "completion_decision":completion,"value_decision":value,"response_entry_decision":entry,
                "word_copy":state["word_copy"],"completion_active":c["active"],"completion_anchor":c["anchor"],"lexical_read":c["lexical_read"]}));
            preceding_state = state.clone();
            if p.token == EOS {
                eos = true;
                break;
            }
            tokens.push(p.token);
        }
        if !eos {
            return Err("response exceeded96 tokens".into());
        }
        Ok(())
    })();
    let final_state = checkpoint(&s)?;
    let decoded = model.decode(&tokens)?;
    let text = String::from_utf8(decoded.clone());
    let error = outcome
        .err()
        .map(|e| e.to_string())
        .or_else(|| text.as_ref().err().map(|e| e.to_string()));
    let actual_text = text.ok();
    Ok(
        json!({"id":case.id,"case":case,"text":actual_text,"decoded_bytes":decoded,"tokens":tokens,"eos":eos,
        "exact":actual_text.as_deref()==Some(case.expected.as_str()) && eos && error.is_none(),
        "retained_plain_text_equal":actual_text.as_deref()==Some(case.retained_plain_expected.as_str()),
        "verification_error":error,"trace":trace,"selections":selections,
        "initial_sources":initial_sources,"final_sources":sources(&final_state),"labelled_relation_candidates":labelled_relations,"labelled_relation_found":!labelled_relations.is_empty(),
        "initial_checkpoint_blake3":blake3::hash(&serde_json::to_vec(&initial)?).to_hex().to_string(),
        "final_checkpoint_blake3":blake3::hash(&serde_json::to_vec(&final_state)?).to_hex().to_string(),
        "numeric_completion_active_seen":completion_active,"numeric_completion_anchor_seen":numeric_anchor_seen,
        "shared_lexical_operand_read_seen":lexical_read_seen,"word_copy_work_before":before_copy_work,"word_copy_work_after":s.work.word_copy,
        "completion_work":s.work.completion,"source_identity_scope":"Observed ReadCommit relation/version or exact query end/byte_end; never inferred from response text",
        "shared_emitter_scope":"Completion active/anchor and lexical-read observations only; absence does not by itself prove every internal candidate call was skipped"}),
    )
}
fn diagnostic_main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "diagnostic" {
        return Err(
            "usage: native_word_sentence diagnostic MODEL PRESERVATION_REPORT_DIR OUT".into(),
        );
    }
    let root = Path::new(&args[3]);
    let out = Path::new(&args[4]);
    fs::create_dir_all(out)?;
    let specifications = [
        ("prior.json", "role-read/first-use/0/0/0", "velra", "Lodov"),
        ("prior.json", "role-read/first-use/0/3/0", "velra", "Merok"),
        (
            "prior-span-development.json",
            "extent-context-open/0/stop",
            "tilva",
            "Ash",
        ),
        (
            "prior-span-development.json",
            "extent-context-open/0/place",
            "tilva",
            "Ash Court",
        ),
        (
            "prior-span-development.json",
            "extent-context-open/1/place",
            "savin",
            "Mist Vale",
        ),
        (
            "prior-span-fresh.json",
            "extent-context-fresh/1/place",
            "tulvi",
            "Quiet River Bank",
        ),
    ];
    let model = Model::from_bytes(&fs::read(&args[2])?)?;
    let mut cases = Vec::new();
    let mut receipts = Vec::new();
    for (file, id, owner, value) in specifications {
        let path = root.join(file);
        let bytes = fs::read(&path)?;
        let report: Value = serde_json::from_slice(&bytes)?;
        if report["artifact"] != model.artifact_cid() {
            return Err("retained source report artifact differs".into());
        }
        let mut matches = report["cases"]
            .as_array()
            .ok_or("source cases absent")?
            .iter()
            .filter(|r| r["id"] == id);
        let row = matches.next().ok_or("retained source case absent")?;
        if matches.next().is_some()
            || row["exact"] != true
            || row["generation"]["stop"] != "end_of_document"
            || row["generation"]["text"] != row["expected"]
        {
            return Err("source case is not uniquely qualified".into());
        }
        let plain = format!(" {value}.\n");
        if row["expected"] != plain {
            return Err("declared source value differs from retained label".into());
        }
        let prompt = row["prompt"].as_str().ok_or("source prompt absent")?;
        let question = prompt
            .rfind("Where is ")
            .ok_or("declared diagnostic question boundary absent")?;
        let marker = if prompt.ends_with("Assistant:") {
            "Assistant:"
        } else {
            "Answer:"
        };
        let tail = prompt
            .strip_suffix(marker)
            .ok_or("response boundary absent")?;
        for placement in ["plain", "prefix", "suffix"] {
            let input = match placement {
                "prefix" => format!(
                    "{}Explain in a sentence. {}",
                    &prompt[..question],
                    &prompt[question..]
                ),
                "suffix" => format!("{tail}Explain in a sentence. {marker}"),
                _ => prompt.into(),
            };
            cases.push(Case {
                id: format!("word-sentence/{id}/{placement}"),
                source_report: file.into(),
                source_id: id.into(),
                owner: owner.into(),
                value: value.into(),
                placement: placement.into(),
                prompt: input,
                expected: if placement == "plain" {
                    plain.clone()
                } else {
                    format!("{owner} is in {value}.\n")
                },
                retained_plain_expected: plain.clone(),
                copy_prefix: None,
                inherit_suffix: false,
            });
        }
        receipts.push(json!({"path":path,"source_id":id,"artifact":report["artifact"],"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string()}));
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("source.receipt.json"),
        &json!({"sources":receipts,"cases":cases.len(),"scope":"Six already-open retained sources crossed with three authored placements. Sentence targets are explicit owner/location labels, not inferred from generated output. No fitting."}),
    )?;
    for (label, control) in [
        ("full", Control::Full),
        ("word-copy-disabled", Control::WordCopyDisabled),
    ] {
        let mut rows = Vec::new();
        for case in &cases {
            rows.push(match generate(&model, case, control) {
                Ok(r) => r,
                Err(e) => json!({"id":case.id,"exact":false,"error":e.to_string()}),
            });
            save(
                &out.join(format!("{label}.json")),
                &json!({"artifact":model.artifact_cid(),"control":control,"total":cases.len(),"completed":rows.len(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"rows":rows,
                "scope":"Open diagnostic; WordCopyDisabled is a path intervention, not source erasure. Complete authored sentence correctness and retained source-path observations are separate."}),
            )?;
        }
    }
    Ok(())
}

fn authored_case(id: String, owner: &str, value: &str, prompt: String, style: &str) -> Case {
    let plain = format!(" {value}.\n");
    let suffix = if style == "plain" {
        ".\n"
    } else if style.contains("stop") {
        " is the stop.\n"
    } else {
        " is the place.\n"
    };
    Case {
        id,
        source_report: "authored-construction".into(),
        source_id: String::new(),
        owner: owner.into(),
        value: value.into(),
        placement: style.into(),
        prompt,
        expected: format!(" {value}{suffix}"),
        retained_plain_expected: plain,
        copy_prefix: None,
        inherit_suffix: false,
    }
}
fn variants(id: &str, owner: &str, value: &str, plain: &str) -> Result<Vec<Case>> {
    let q = plain.rfind("Where is ").ok_or("question absent")?;
    let marker = if plain.ends_with("Assistant:") {
        "Assistant:"
    } else {
        "Answer:"
    };
    let tail = plain.strip_suffix(marker).ok_or("boundary absent")?;
    let mut out = Vec::new();
    for (style, instruction, front) in [
        ("plain", "", false),
        ("place-prefix", "Explain in a sentence. ", true),
        ("place-suffix", "Explain in a sentence. ", false),
        ("stop-prefix", "Explain the stop in a sentence. ", true),
        ("stop-suffix", "Explain the stop in a sentence. ", false),
    ] {
        let prompt = if instruction.is_empty() {
            plain.to_string()
        } else if front {
            format!("{}{instruction}{}", &plain[..q], &plain[q..])
        } else {
            format!("{tail}{instruction}{marker}")
        };
        out.push(authored_case(
            format!("{id}/{style}"),
            owner,
            value,
            prompt,
            style,
        ));
    }
    Ok(out)
}
fn prepare_cases(input: &Path, out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let original: Vec<Case> = serde_json::from_slice(&fs::read(input)?)?;
    let mut open = Vec::new();
    let mut over_limit = Vec::new();
    for (i, c) in original
        .iter()
        .filter(|c| c.placement == "plain")
        .enumerate()
    {
        for mut x in variants(&format!("open/{i}"), &c.owner, &c.value, &c.prompt)? {
            x.source_report = c.source_report.clone();
            x.source_id = c.source_id.clone();
            if x.expected.len() + 1 > 32 {
                over_limit.push(x)
            } else {
                open.push(x)
            }
        }
    }
    let mut construction = Vec::new();
    for (i, owner, value, old) in [
        (0, "navri", "Cedar Bay", "Birch"),
        (1, "pelvi", "Amber Hill", "Cloud"),
    ] {
        let single = value.split_whitespace().next().ok_or("value")?;
        for (form, v, p) in [
            (
                "single",
                single,
                format!("Record: {owner} in {single}. Where is {owner}? Answer:"),
            ),
            (
                "updated",
                single,
                format!(
                    "Record: {owner} in {old}. {owner} now in {single}. Where is {owner}? Answer:"
                ),
            ),
            (
                "span",
                value,
                format!("User: {owner} lives in {value}.\nUser: Where is {owner}?\nAssistant:"),
            ),
        ] {
            construction.extend(variants(&format!("construction/{i}/{form}"), owner, v, &p)?);
        }
    }
    let mut abstention = Vec::new();
    for (i, prompt) in [
        "Record: tovin in Lodov. Where is velra? Answer:",
        "Record: tovin in Talven. Where is velra? Answer:",
    ]
    .into_iter()
    .enumerate()
    {
        for mut c in variants(&format!("absent/{i}"), "velra", "Unknown", prompt)? {
            c.expected = " Unknown.\n".into();
            c.source_report = "retained-prior-absent-controls".into();
            abstention.push(c);
        }
    }
    save(&out.join("abstention.json"), &abstention)?;
    save(&out.join("construction.json"), &construction)?;
    save(&out.join("open.json"), &open)?;
    save(&out.join("over-limit.json"), &over_limit)?;
    save(
        &out.join("target-policy.json"),
        &json!({"original_diagnostic":"Owner-first labels remain frozen and failed12/12 sentence rows. Source-first is a narrower new construction target, not a regrade.","source_first":"Exact copied payload followed by learned contextual suffix; no owner reread or arbitrary sentence reordering claim.","max_total_steps":32,"construction":construction.len(),"open":open.len(),"over_limit":over_limit.len()}),
    )?;
    Ok(())
}
fn evaluate_cases(model: &Model, cases: &[Case], out: &Path, controls: bool) -> Result<()> {
    fs::create_dir_all(out)?;
    let all = [
        ("full", Control::Full),
        ("previous-emitter", Control::WordEmissionDisabled),
        ("context-disabled", Control::WordEmissionContextDisabled),
        ("geometry-disabled", Control::WordEmissionGeometryDisabled),
        (
            "prefix-context-disabled",
            Control::WordEmissionPrefixContextDisabled,
        ),
    ];
    for (label, control) in all.into_iter().take(if controls { 5 } else { 1 }) {
        let mut rows = Vec::new();
        for c in cases {
            rows.push(generate(model, c, control)?);
            save(
                &out.join(format!("{label}.json")),
                &json!({"artifact":model.artifact_cid(),"control":control,"total":cases.len(),"completed":rows.len(),"exact":rows.iter().filter(|r|r["exact"]==true).count(),"rows":rows}),
            )?;
        }
    }
    Ok(())
}
fn fresh_cases(out: &Path) -> Result<()> {
    fs::create_dir_all(out)?;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos() as u64;
    let mut rng = seed;
    let mut word = |title: bool| {
        let mut x = String::new();
        for i in 0..4 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let b = b'a' + (rng % 26) as u8;
            x.push(if title && i == 0 {
                b.to_ascii_uppercase() as char
            } else {
                b as char
            });
        }
        x
    };
    let owner = word(false);
    let single = word(true);
    let old = word(true);
    let two = format!("{} {}", word(true), word(true));
    let three = format!("{} {} {}", word(true), word(true), word(true));
    let mut cases = Vec::new();
    for (form, v, p) in [
        (
            "single",
            single.as_str(),
            format!("Record: {owner} in {single}. Where is {owner}? Answer:"),
        ),
        (
            "updated",
            single.as_str(),
            format!("Record: {owner} in {old}. {owner} now in {single}. Where is {owner}? Answer:"),
        ),
        (
            "two",
            two.as_str(),
            format!("User: {owner} lives in {two}.\nUser: Where is {owner}?\nAssistant:"),
        ),
        (
            "three",
            three.as_str(),
            format!("User: {owner} lives in {three}.\nUser: Where is {owner}?\nAssistant:"),
        ),
    ] {
        cases.extend(variants(&format!("fresh/{form}"), &owner, v, &p)?);
    }
    // Draw only after the original twenty fact cases. This evaluation-only
    // extension preserves their exact RNG sequence and adds no fit labels.
    let mut rust_identifiers = Vec::new();
    for index in 0..4 {
        let mut identifier = None;
        for _ in 0..1024 {
            let draw = word(false);
            if !matches!(
                draw.as_str(),
                "else" | "enum" | "impl" | "loop" | "move" | "priv" | "self" | "true" | "type"
            ) && !rust_identifiers.contains(&draw)
                && ![owner.as_str(), "item", "navi"].contains(&draw.as_str())
            {
                identifier = Some(draw);
                break;
            }
        }
        let identifier = identifier.ok_or("fresh Rust identifier draw exhausted")?;
        let expected = format!("{identifier}\n}}\n");
        cases.push(Case {
            id: format!("fresh/rust-identity/{index}"),
            source_report: "freshdraw/draw.json".into(),
            source_id: format!("rust_identifiers/{index}"),
            owner: String::new(),
            value: identifier.clone(),
            placement: "plain".into(),
            prompt: format!(
                "// ada has 11 coins; cyra has 301.\n// Return the input unchanged.\nfn identity({identifier}: i32) -> i32 {{\n    "
            ),
            expected: expected.clone(),
            retained_plain_expected: expected,
            copy_prefix: Some(identifier.clone()),
            inherit_suffix: false,
        });
        rust_identifiers.push(identifier);
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("draw.json"),
        &json!({"seed":seed,"owner":owner,"single":single,"old":old,"two":two,"three":three,"rust_identifiers":rust_identifiers,"total":cases.len(),"rust_prompt_source":"retained-construction.json#/cases: source-joint/literal-admission/word-copy/fit/context-0/name-0","scope":"Post-selection name/value and Rust identifier transfer under fixed retained forms only; twenty fact cases followed by four identifier-return cases. Evaluation only, no fitting after draw; not unseen-language or general Rust qualification"}),
    )?;
    Ok(())
}
fn prepare_preserving(
    construction: &Path,
    parent: &Path,
    rejected: &Path,
    out: &Path,
) -> Result<()> {
    fs::create_dir(out)?;
    let mut cases: Vec<Case> = serde_json::from_slice(&fs::read(construction)?)?;
    for c in &mut cases {
        c.inherit_suffix = c.placement == "plain";
    }
    let mut receipts = Vec::new();
    for file in ["retained-construction.json", "prior-start-open.json"] {
        let before = fs::read(parent.join(file))?;
        let after = fs::read(rejected.join(file))?;
        let good: Value = serde_json::from_slice(&before)?;
        let bad: Value = serde_json::from_slice(&after)?;
        let rows = |r: &Value| {
            r["cases"]
                .as_array()
                .or_else(|| r["rows"].as_array())
                .cloned()
        };
        let prior = rows(&good).ok_or("parent cases absent")?;
        let failed = rows(&bad).ok_or("rejected cases absent")?;
        let mut added = 0;
        for r in failed.iter().filter(|r| r["exact"] == false) {
            let p = prior
                .iter()
                .find(|p| p["id"] == r["id"])
                .ok_or("parent case absent")?;
            if p["exact"] != true || p["expected"] != r["expected"] || p["prompt"] != r["prompt"] {
                return Err("regression source differs from qualified parent".into());
            }
            let expected = p["expected"].as_str().ok_or("expected absent")?;
            // Offline target partition only; runtime verifies the actual copied
            // prefix and freely generated frozen-parent continuation.
            let prefix = expected
                .strip_suffix("\n}\n")
                .or_else(|| expected.strip_suffix(".\n"))
                .ok_or("unsupported retained suffix partition")?;
            cases.push(Case {
                id: format!("preserve/{file}/{}", p["id"].as_str().ok_or("id absent")?),
                source_report: parent.join(file).display().to_string(),
                source_id: p["id"].as_str().ok_or("id absent")?.into(),
                owner: String::new(),
                value: prefix.trim_start().into(),
                placement: "plain".into(),
                prompt: p["prompt"].as_str().ok_or("prompt absent")?.into(),
                expected: expected.into(),
                retained_plain_expected: expected.into(),
                copy_prefix: Some(prefix.into()),
                inherit_suffix: true,
            });
            added += 1;
        }
        receipts.push(json!({"report":file,"parent_artifact":good["artifact"],"rejected_artifact":bad["artifact"],"parent_blake3":blake3::hash(&before).to_hex().to_string(),"rejected_blake3":blake3::hash(&after).to_hex().to_string(),"added":added}));
    }
    if cases.len() > 256 {
        return Err("preservation case cap256".into());
    }
    save(&out.join("cases.json"), &cases)?;
    save(
        &out.join("receipt.json"),
        &json!({"cases":cases.len(),"inherit":cases.iter().filter(|c|c.inherit_suffix).count(),"sources":receipts,"scope":"All observed regressions in named prior populations become open training controls; no held-out claim. Plain construction also learns Base/defer. Candidate alphabet and query vocabulary remain frozen to the rejected seed."}),
    )?;
    Ok(())
}
fn fresh_rust_source(input: &Path, out: &Path) -> Result<()> {
    let bytes = fs::read(input)?;
    let report: Value = serde_json::from_slice(&bytes)?;
    let rows = report["rows"]
        .as_array()
        .ok_or("fresh result rows absent")?;
    if report["completed"].as_u64() != Some(rows.len() as u64)
        || report["total"].as_u64() != Some(rows.len() as u64)
    {
        return Err("fresh report is incomplete".into());
    }
    let selected: Vec<_> = rows
        .iter()
        .filter(|row| {
            row["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("fresh/rust-identity/"))
        })
        .collect();
    if selected.len() != 4 {
        return Err("expected four fresh Rust identity results".into());
    }
    let mut program = String::from("fn main() {\n");
    let mut receipt = Vec::new();
    for index in 0..4 {
        let id = format!("fresh/rust-identity/{index}");
        let matches: Vec<_> = selected.iter().filter(|row| row["id"] == id).collect();
        if matches.len() != 1 {
            return Err(format!("missing or duplicated result {id}").into());
        }
        let row = matches[0];
        let case: Case = serde_json::from_value(row["case"].clone())?;
        let actual = row["text"].as_str().ok_or("actual Rust response absent")?;
        if row["exact"] != true
            || row["eos"] != true
            || row.get("verification_error") != Some(&Value::Null)
            || actual != case.expected
            || case.id != id
        {
            return Err(format!("unqualified actual Rust output {id}").into());
        }
        // Pin the declared fixed form, then compile the actual supplied prompt
        // and generated response. No expected answer is substituted into source.
        let identifier = case
            .copy_prefix
            .as_deref()
            .ok_or("Rust copy prefix absent")?;
        if identifier.len() != 4 || !identifier.bytes().all(|b| b.is_ascii_lowercase())
            || case.expected != format!("{identifier}\n}}\n")
            || case.prompt != format!("// ada has 11 coins; cyra has 301.\n// Return the input unchanged.\nfn identity({identifier}: i32) -> i32 {{\n    ")
            || case.inherit_suffix || case.placement != "plain"
        {
            return Err(format!("fresh Rust fixed form differs {id}").into());
        }
        program.push_str("{\n");
        program.push_str(&case.prompt);
        program.push_str(actual);
        program.push_str("assert_eq!(identity(19), 19);\nassert_eq!(identity(-7), -7);\n}\n");
        receipt.push(json!({"id":id,"identifier":identifier,"prompt":case.prompt,
            "actual_text":actual,"eos":row["eos"],"checks":[[19,19],[-7,-7]]}));
    }
    program.push_str("}\n");
    fs::create_dir(out)?;
    fs::write(out.join("generated_checks.rs"), &program)?;
    save(
        &out.join("receipt.json"),
        &json!({"source_report":input,
        "report_blake3":blake3::hash(&bytes).to_hex().to_string(),"artifact":report["artifact"],
        "source_blake3":blake3::hash(program.as_bytes()).to_hex().to_string(),
        "cases":4,"checks":8,"rows":receipt,
        "scope":"Rust source assembled from four exact freely generated identifier-return completions under fixed retained prompts. Eight runtime checks prepared; compilation and execution are separate results."}),
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 4 && a[1] == "rust-source" {
        return fresh_rust_source(Path::new(&a[2]), Path::new(&a[3]));
    }
    if a.len() == 6 && a[1] == "prepare-preserving" {
        return prepare_preserving(
            Path::new(&a[2]),
            Path::new(&a[3]),
            Path::new(&a[4]),
            Path::new(&a[5]),
        );
    }
    if a.len() == 4 && a[1] == "identity" {
        let bytes = fs::read(&a[2])?;
        let model = Model::from_bytes(&bytes)?;
        let encoded = model.to_bytes()?;
        if bytes != encoded {
            return Err("artifact byte roundtrip differs".into());
        }
        save(
            Path::new(&a[3]),
            &json!({"artifact":model.artifact_cid(),"path":a[2],"bytes":bytes.len(),"blake3":blake3::hash(&bytes).to_hex().to_string(),"serialization_byte_exact":true}),
        )?;
        return Ok(());
    }
    if a.len() == 3 && a[1] == "fresh" {
        return fresh_cases(Path::new(&a[2]));
    }
    if a.get(1).is_some_and(|x| x == "diagnostic") {
        return diagnostic_main();
    }
    if a.len() == 4 && a[1] == "prepare" {
        return prepare_cases(Path::new(&a[2]), Path::new(&a[3]));
    }
    if a.len() != 5 {
        return Err("usage: native_word_sentence prepare DIAGNOSTIC_CASES OUT | fit/fit-balanced/evaluate/controls MODEL CASES OUT | fresh OUT | rust-source FULL_REPORT_JSON OUT_DIR".into());
    }
    if !matches!(
        a[1].as_str(),
        "fit"
            | "fit-balanced"
            | "fit-contextual"
            | "fit-pairs"
            | "fit-preserving"
            | "evaluate"
            | "controls"
    ) {
        return Err("unknown word sentence mode".into());
    }
    let parent = Model::from_bytes(&fs::read(&a[2])?)?;
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(&a[3])?)?;
    let out = Path::new(&a[4]);
    fs::create_dir_all(out)?;
    if cases.is_empty() || cases.len() > 256 {
        return Err("case limit".into());
    }
    save(&out.join("cases.json"), &cases)?;
    let model = if matches!(
        a[1].as_str(),
        "fit" | "fit-balanced" | "fit-contextual" | "fit-pairs" | "fit-preserving"
    ) {
        let docs: Vec<_> = cases
            .iter()
            .map(|c| {
                let prefix = c
                    .copy_prefix
                    .clone()
                    .unwrap_or_else(|| format!(" {}", c.value));
                Ok(WordEmissionExample {
                    id: c.id.clone(),
                    prompt: c.prompt.clone(),
                    suffix: c
                        .expected
                        .strip_prefix(&prefix)
                        .ok_or("prefix label mismatch")?
                        .into(),
                    prefix,
                    inherit_suffix: c.inherit_suffix,
                })
            })
            .collect::<Result<_>>()?;
        save(&out.join("training.json"), &docs)?;
        let config = SourceRoutingConfig {
            learned_features: 4096,
            passes: 8,
            proposals: 120,
            max_seconds: 120,
            mode: RoutingMode::Angular,
            seed: 973,
            role_context_only: false,
        };
        let (candidate, report) = if a[1] == "fit-preserving" {
            parent.fit_word_emission_preserving(&docs, config)?
        } else if a[1] == "fit-pairs" {
            parent.fit_word_emission_pairs(&docs, config)?
        } else if a[1] == "fit-contextual" {
            parent.fit_word_emission_contextual(&docs, config)?
        } else if a[1] == "fit-balanced" {
            parent.fit_word_emission_balanced(&docs, config)?
        } else {
            parent.fit_word_emission(&docs, config)?
        };
        fs::write(out.join("model.json"), candidate.to_bytes()?)?;
        save(&out.join("fit.json"), &report)?;
        candidate
    } else {
        parent
    };
    evaluate_cases(&model, &cases, out, a[1] == "controls")
}
