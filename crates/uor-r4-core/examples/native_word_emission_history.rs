//! Exact retained-history replay with open source-first sentence comparisons.
//! Labels only judge predictions. No history response or final answer is injected.
//! Compact reports retain every bounded token projection, including failures.
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_core::native_geometric::{
    Control, Model, ResponseEntryAction, Session, WordCopyAction, BOS, EOS,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn save(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec(value)?)?;
    Ok(())
}
fn checkpoint(session: &Session) -> Result<Value> {
    Ok(serde_json::from_slice(&session.checkpoint()?)?)
}
fn ingest(model: &Model, session: &mut Session, text: &str) -> Result<()> {
    for token in model.encode(text)? {
        session.observe(model, token)?;
    }
    Ok(())
}
fn records(state: &Value) -> Result<Value> {
    let rows = state["values"]["relations"]["records"]
        .as_array()
        .ok_or("relation records metadata absent")?;
    Ok(json!(rows
        .iter()
        .filter(|r| r["id"].as_u64().is_some_and(|id| id != 0))
        .cloned()
        .collect::<Vec<_>>()))
}
fn versions(rows: &Value) -> Result<Value> {
    Ok(json!(rows.as_array().ok_or("version rows absent")?.iter()
        .map(|r| json!({"id":r["id"],"previous":r["previous"],"action":r["action"],"conflict":r["conflict"]}))
        .collect::<Vec<_>>()))
}
fn modified_query(query: &str, style: &str) -> Result<String> {
    Ok(match style {
        "plain" => query.to_owned(),
        "place-prefix" => format!("Explain in a sentence. {query}"),
        "place-suffix" => format!(
            "{}Explain in a sentence. Answer:",
            query
                .strip_suffix("Answer:")
                .ok_or("query Answer boundary absent")?
        ),
        _ => return Err("unknown style".into()),
    })
}

fn generate(model: &Model, session: &mut Session, expected: &str) -> Result<Value> {
    session.begin_response(model)?;
    let initial = checkpoint(session)?;
    let initial_records = records(&initial)?;
    let mut trace = Vec::new();
    let mut tokens = Vec::new();
    let mut eos = false;
    let mut learned_stop = false;
    let mut base_eos = false;
    let mut verification_error = None;
    let mut commit = Value::Null;
    let mut commit_stable = true;
    let mut suffix_steps = 0;
    let mut no_read_seen = false;
    let mut numeric_completion_seen = false;
    let outcome = (|| -> Result<()> {
        for position in 0..96 {
            let prior = checkpoint(session)?;
            let mut restored = model.restore_session(&session.checkpoint()?)?;
            let prediction = session.predict(model)?;
            let word = session.word_copy_decision();
            let entry = session.response_entry_decision();
            let completion = session.completion_decision();
            let value = session.value_decision();
            let equal = restored.predict(model)? == prediction
                && restored.word_copy_decision() == word
                && restored.response_entry_decision() == entry
                && restored.completion_decision() == completion
                && restored.value_decision() == value;
            session.observe(model, prediction.token)?;
            restored.observe(model, prediction.token)?;
            let state = checkpoint(session)?;
            let restored_state = checkpoint(&restored)?;
            let committed_equal = ["values", "word_copy", "response_entry", "completion"]
                .iter()
                .all(|key| state[*key] == restored_state[*key]);
            let active_commit = &state["word_copy"]["read_commit"];
            if active_commit["source"].as_u64().is_some() && commit.is_null() {
                commit = active_commit.clone();
            }
            let suffix = word
                .is_some_and(|d| matches!(d.action, WordCopyAction::Emit | WordCopyAction::Stop));
            if suffix && !commit.is_null() {
                suffix_steps += 1;
                commit_stable &= if prediction.token == EOS {
                    prior["word_copy"]["read_commit"] == commit && active_commit.is_null()
                } else {
                    active_commit == &commit
                };
            }
            no_read_seen |= word.is_some_and(|d| d.action == WordCopyAction::NoRead);
            numeric_completion_seen |=
                state["completion"]["active"] == true || state["completion"]["anchor"].is_object();
            let unchanged_records = records(&state)? == initial_records;
            let unchanged_directory = state["values"]["relations"]["directory"]
                == initial["values"]["relations"]["directory"];
            trace.push(json!({"position":position,"token":prediction.token,
                "word_copy_decision":word,"response_entry_decision":entry,
                "completion_decision":completion,"value_decision":value,
                "prior_word_progress":prior["word_copy"]["progress"],
                "word_progress":state["word_copy"]["progress"],
                "read_commit":active_commit,"origin":state["word_copy"]["origin"],
                "span_words":state["word_copy"]["span_words"],
                "completion_active":state["completion"]["active"],
                "completion_anchor":state["completion"]["anchor"],
                "checkpoint_prediction_equal":equal,"checkpoint_committed_equal":committed_equal,
                "records_unchanged":unchanged_records,"directory_unchanged":unchanged_directory,
                "suffix_step":suffix}));
            if prediction.token == EOS {
                eos = true;
                learned_stop = word.is_some_and(|d| d.action == WordCopyAction::Stop)
                    || entry.is_some_and(|d| d.action == ResponseEntryAction::Stop);
                base_eos = commit.is_null() && !learned_stop;
            } else {
                tokens.push(prediction.token);
            }
            if !equal || !committed_equal || !unchanged_records || !unchanged_directory {
                return Err("checkpoint parity or retained relation state changed".into());
            }
            if eos {
                break;
            }
        }
        if !eos {
            return Err("96 prediction output bound reached without EOS".into());
        }
        if !commit.is_null() && !learned_stop {
            return Err("EOS lacks observed learned Stop decision".into());
        }
        Ok(())
    })();
    if let Err(error) = outcome {
        verification_error = Some(error.to_string());
    }
    let final_state = checkpoint(session)?;
    let bytes = model.decode(&tokens)?;
    let text = String::from_utf8(bytes.clone());
    if let Err(error) = &text {
        verification_error = Some(error.to_string());
    }
    let actual = text.ok();
    let source_record = initial_records
        .as_array()
        .and_then(|rows| {
            rows.iter().find(|r| {
                commit["relation_id"]
                    .as_u64()
                    .is_some_and(|id| r["id"] == id)
            })
        })
        .cloned();
    Ok(
        json!({"expected":expected,"text":actual,"decoded_bytes":bytes,"tokens":tokens,
        "eos":eos,"learned_stop":learned_stop,"base_eos":base_eos,
        "learned_stop_required":!commit.is_null(),
        "exact":actual.as_deref()==Some(expected) && eos && (commit.is_null() || learned_stop) && verification_error.is_none(),
        "verification_error":verification_error,"trace":trace,
        "source_commit":commit,"source_record":source_record,
        "suffix_commit_stable":commit_stable,"suffix_steps":suffix_steps,
        "no_read_seen":no_read_seen,"numeric_completion_seen":numeric_completion_seen,
        "initial_records":initial_records,"final_records":records(&final_state)?,
        "initial_directory":initial["values"]["relations"]["directory"],
        "final_directory":final_state["values"]["relations"]["directory"],
        "final_next_id":final_state["values"]["relations"]["next_id"],
        "initial_checkpoint_blake3":blake3::hash(&serde_json::to_vec(&initial)?).to_hex().to_string(),
        "final_checkpoint_blake3":blake3::hash(&serde_json::to_vec(&final_state)?).to_hex().to_string()}),
    )
}

fn run_case(model: &Model, report: &Value, file: &str, end: usize, style: &str) -> Result<Value> {
    let retained = report["turns"].as_array().ok_or("retained turns absent")?;
    let mut session = model.session(Control::Full)?;
    session.observe(model, BOS)?;
    let padding = "quiet sky. ".repeat(96);
    let padding_tokens = model.encode(&padding)?.len();
    if padding_tokens != 1056 {
        return Err("retained padding token count changed".into());
    }
    let mut turns = Vec::new();
    let mut histories_exact = true;
    let mut final_exact = false;
    let mut identity_exact = false;
    for (index, source) in retained.iter().enumerate().take(end + 1) {
        session.end_response(model)?;
        let input = source["input"].as_str().ok_or("retained input absent")?;
        let original_query = source["query"].as_str().ok_or("retained query absent")?;
        let original_expected = source["expected"].as_str().ok_or("retained label absent")?;
        let final_turn = index == end;
        let query = if final_turn {
            modified_query(original_query, style)?
        } else {
            original_query.to_owned()
        };
        let expected = if final_turn && end == 1 && style != "plain" {
            format!(
                "{} is the place.\n",
                original_expected
                    .strip_suffix(".\n")
                    .ok_or("plain source label boundary absent")?
            )
        } else {
            original_expected.to_owned()
        };
        ingest(model, &mut session, input)?;
        ingest(model, &mut session, &padding)?;
        ingest(model, &mut session, &query)?;
        let mut actual = generate(model, &mut session, &expected)?;
        let version_exact = versions(&actual["final_records"])? == versions(&source["records"])?
            && actual["final_next_id"]
                == source["expected_versions"]
                    .as_u64()
                    .ok_or("expected versions absent")?
                    + 1;
        actual["versions_match_retained"] = json!(version_exact);
        let exact = actual["exact"] == true && version_exact;
        if final_turn {
            final_exact = exact;
            identity_exact = if end == 1 {
                actual["source_commit"]["relation_id"] == 3
                    && actual["source_record"]["previous"] == 1
                    && actual["source_record"]["conflict"] == false
                    && actual["suffix_commit_stable"] == true
                    && actual["suffix_steps"].as_u64().is_some_and(|n| n > 0)
                    && actual["final_directory"]
                        .as_array()
                        .is_some_and(|d| d.contains(&json!(3)))
            } else {
                actual["source_commit"].is_null()
                    && actual["no_read_seen"] == true
                    && actual["numeric_completion_seen"] == false
                    && actual["final_directory"]
                        .as_array()
                        .is_some_and(|d| d.contains(&json!(5)))
                    && actual["trace"].as_array().is_some_and(|steps| {
                        !steps.is_empty()
                            && steps.iter().all(|step| {
                                step["prior_word_progress"]
                                    .as_str()
                                    .is_some_and(|progress| progress != "complete")
                            })
                    })
                    && actual["final_records"].as_array().is_some_and(|rows| {
                        rows.iter()
                            .any(|r| r["id"] == 5 && r["previous"] == 4 && r["conflict"] == true)
                    })
            };
        } else {
            histories_exact &= exact;
        }
        turns.push(json!({"source_id":format!("{file}#/turns/{index}"),"input":input,"query":query,"actual":actual}));
        if !exact && !final_turn {
            break;
        }
    }
    Ok(
        json!({"id":format!("{file}/turn-{end}/{style}"),"source_report":file,"end_turn":end,"style":style,
        "histories_exact":histories_exact,"final_exact":final_exact,"identity_exact":identity_exact,
        "exact":histories_exact && final_exact && identity_exact,"padding_tokens_per_turn":padding_tokens,"turns":turns}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: native_word_emission_history MODEL PRESERVATION_REPORT_DIR OUT".into());
    }
    let out = Path::new(&args[3]);
    fs::create_dir(out)?;
    let model = Model::from_bytes(&fs::read(&args[1])?)?;
    let mut rows = Vec::new();
    let mut receipts = Vec::new();
    let mut parent = None;
    for file in ["forward-open.json", "reverse-open.json"] {
        let path = Path::new(&args[2]).join(file);
        let bytes = fs::read(&path)?;
        let report: Value = serde_json::from_slice(&bytes)?;
        let artifact = report["artifact"]
            .as_str()
            .ok_or("retained artifact identity absent")?;
        if parent.as_ref().is_some_and(|old: &String| old != artifact) {
            return Err("retained report parent identities differ".into());
        }
        parent = Some(artifact.to_owned());
        let turns = report["turns"].as_array().ok_or("retained turns absent")?;
        if turns.len() != 6
            || report["exact"] != 6
            || report["total"] != 6
            || turns.iter().any(|r| {
                r["exact"] != true
                    || r["expected"] != r["text"]
                    || r["checkpoint_predictions_equal"] != true
                    || !(r["writes_exact"] == true || r["version_count_exact"] == true)
            })
        {
            return Err("retained report is incomplete or unqualified".into());
        }
        receipts.push(json!({"path":path,"blake3":blake3::hash(&bytes).to_hex().to_string(),"artifact":artifact}));
        for end in [1, 4] {
            for style in ["plain", "place-prefix", "place-suffix"] {
                let row = match run_case(&model, &report, file, end, style) {
                    Ok(row) => row,
                    Err(error) => {
                        json!({"id":format!("{file}/turn-{end}/{style}"),"exact":false,"metadata_or_execution_error":error.to_string()})
                    }
                };
                save(&out.join(format!("{file}-turn-{end}-{style}.json")), &row)?;
                rows.push(row);
                save(
                    &out.join("full.json"),
                    &json!({"artifact":model.artifact_cid(),"retained_parent":parent,
                    "source_receipts":receipts,"total":12,"completed":rows.len(),
                    "exact":rows.iter().filter(|r|r["exact"]==true).count(),"cases":rows,
                    "scope":"Open bounded version/conflict history replay. Every history and final response freely generated; sentence target is source-first. Retained context unchanged; 1056 padding tokens ingested each turn. Output bound96 includes EOS. Metadata absence is an error, not identity success. Stop is an observed routing decision, not proof of predictive geometry advantage."}),
                )?;
            }
        }
    }
    Ok(())
}
