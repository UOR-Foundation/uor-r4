//! Bounded source-only learned native turn compiler and exact-store integration.
//! Exact addressed admission and all-bank native reads are reported separately.
#[path = "geometric_native_compiler/reader.rs"]
mod reader;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_turn_compiler::{fit_examples, FitConfig, NativeTurnCompiler},
    relation_compiler::{Example, NONE},
    sha256_bytes,
    stack_grounded_session::{CompiledAction, SourceSpan, TurnCompiler},
    stack_store::{HistoryView, StackStore, Update},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fail(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
struct Args {
    native: PathBuf,
    binding: PathBuf,
    tokenizer: PathBuf,
    output: PathBuf,
    binding_sha: Option<String>,
    cue: PathBuf,
    prefix: PathBuf,
    end: PathBuf,
}
fn args() -> Result<Args> {
    let mut flags = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| fail("every flag needs a value"))?;
        if flags.insert(key, value).is_some() {
            return Err(fail("duplicate flag").into());
        }
    }
    let mut get = |key: &str| {
        flags
            .remove(key)
            .map(PathBuf::from)
            .ok_or_else(|| fail(&format!("missing {key}")))
    };
    let (native, binding, tokenizer, output) = (
        get("--native-artifact")?,
        get("--binding")?,
        get("--tokenizer")?,
        get("--output")?,
    );
    let (cue, prefix, end) = (get("--cue")?, get("--prefix")?, get("--source-end")?);
    let binding_sha = flags.remove("--binding-sha");
    if !flags.is_empty() {
        return Err(fail("unknown arguments").into());
    }
    Ok(Args {
        native,
        binding,
        tokenizer,
        output,
        binding_sha,
        cue,
        prefix,
        end,
    })
}
fn example(relation: &str, act: &'static str, template: &str, value: &str) -> Example {
    Example {
        text: template.replace("{v}", value),
        relation: relation.into(),
        act,
        template: Some(template.into()),
    }
}
fn pools(
    split: &str,
) -> (
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
) {
    match split {
        "training" => (
            vec![
                "engineer",
                "green valley",
                "librarian",
                "red meadow",
                "architect",
                "blue harbor",
                "singer",
                "silver hill",
            ],
            vec![
                "My {r} is {v}.",
                "For my {r}, record {v}.",
                "I have {v} as my {r}.",
                "The value of my {r} is {v}.",
            ],
            vec![
                "My {r} is now {v}.",
                "Correct my {r} to {v}.",
                "Update my {r}: {v}.",
                "Replace my {r} with {v}.",
            ],
            vec![
                "What is my {r}?",
                "Tell me my {r}.",
                "Recall my {r}.",
                "Which {r} did I state?",
            ],
        ),
        "development" => (
            vec!["potter", "amber lake", "dancer", "cedar grove"],
            vec![
                "Regarding my {r}, it is {v}.",
                "Please remember {v} for my {r}.",
            ],
            vec![
                "Revise the saved {r} to {v}.",
                "The old {r} should become {v}.",
            ],
            vec![
                "Can you recall the saved {r}?",
                "What did I give as my {r}?",
            ],
        ),
        _ => (
            vec!["weaver", "orchid bay", "painter", "birch ridge"],
            vec![
                "The {r} I want remembered is {v}.",
                "Keep this as my {r}: {v}.",
            ],
            vec![
                "Change the remembered {r} into {v}.",
                "Use {v} instead for my {r}.",
            ],
            vec![
                "Give back the {r} I recorded.",
                "Which value is saved for my {r}?",
            ],
        ),
    }
}
fn panel(split: &str) -> Vec<Example> {
    let (values, asserts, updates, queries) = pools(split);
    let mut rows = Vec::new();
    for relation in ["job", "home"] {
        for (act, templates) in [("assert", &asserts), ("update", &updates)] {
            for template in templates {
                let t = template.replace("{r}", relation);
                for value in &values {
                    rows.push(example(relation, act, &t, value));
                }
            }
        }
        for template in &queries {
            rows.push(example(
                relation,
                "query",
                &template.replace("{r}", relation),
                "",
            ));
        }
    }
    let negatives = match split {
        "training" => vec![
            "Hello there.",
            "The weather seems pleasant.",
            "I enjoy stories.",
            "Let us think carefully.",
            "A job can require patience.",
            "A home can be welcoming.",
            "What a lovely evening.",
            "Thanks for listening.",
            "Words have many meanings.",
            "We should be curious.",
            "The river flows quietly.",
            "I like reading fiction.",
            "It might rain tomorrow.",
            "Geometry is interesting.",
            "Good morning everyone.",
            "Please continue talking.",
        ],
        "development" => vec![
            "I am happy to chat.",
            "The garden looks beautiful.",
            "Learning takes time.",
            "That was an interesting thought.",
            "People discuss job satisfaction.",
            "A home needs care.",
            "The sunset was bright.",
            "Thank you very much.",
        ],
        _ => vec![
            "We can discuss ideas.",
            "A quiet walk is pleasant.",
            "The moon is visible.",
            "I appreciate your help.",
            "Job interviews can be difficult.",
            "Home design is fascinating.",
            "That story made me smile.",
            "Good afternoon to you.",
        ],
    };
    for text in negatives {
        rows.push(Example {
            text: text.into(),
            relation: NONE.into(),
            act: NONE,
            template: None,
        });
    }
    rows
}
fn input_rows(rows: &[Example]) -> Value {
    json!(rows.iter().map(|e|json!({"text":e.text,"relation":e.relation,"act":e.act,"template":e.template,"gold_span":e.slot_span()})).collect::<Vec<_>>())
}
fn action_fields(action: &CompiledAction) -> (&str, Option<u32>, Option<SourceSpan>) {
    match action {
        CompiledAction::Assert { relation, span } => ("assert", Some(*relation), Some(*span)),
        CompiledAction::Correct { relation, span } => ("update", Some(*relation), Some(*span)),
        CompiledAction::QueryCurrent { relation } => ("query", Some(*relation), None),
        CompiledAction::Query { relation, .. } => ("query", Some(*relation), None),
        CompiledAction::Unresolved { .. } => (NONE, None, None),
    }
}
fn evaluate(compiler: &NativeTurnCompiler<'_>, rows: &[Example]) -> Result<Value> {
    let mut records = Vec::new();
    let (
        mut act_correct,
        mut relation_correct,
        mut span_correct,
        mut exact,
        mut prose,
        mut false_writes,
    ) = (0, 0, 0, 0, 0, 0);
    for e in rows {
        let action = compiler.compile(&e.text)?;
        let (act, relation, span) = action_fields(&action);
        let gold_relation = match e.relation.as_str() {
            "job" => Some(1),
            "home" => Some(2),
            _ => None,
        };
        let gold_span = e.slot_span().map(|(start, end)| SourceSpan { start, end });
        let ac = act == e.act;
        let rc = relation == gold_relation;
        let sc = span == gold_span;
        act_correct += usize::from(ac);
        relation_correct += usize::from(rc);
        span_correct += usize::from(sc);
        exact += usize::from(ac && rc && sc);
        if e.act == NONE {
            prose += 1;
            false_writes += usize::from(matches!(
                &action,
                CompiledAction::Assert { .. } | CompiledAction::Correct { .. }
            ));
        }
        records.push(json!({"source":e.text,"gold_act":e.act,"gold_relation":gold_relation,"gold_span":gold_span,"predicted":action,"act_correct":ac,"relation_correct":rc,"span_exact":sc,"complete_exact":ac&&rc&&sc}));
    }
    Ok(
        json!({"rows":records,"examples":rows.len(),"act_correct":act_correct,"relation_correct":relation_correct,"span_exact":span_correct,"complete_exact":exact,"prose_rows":prose,"prose_false_writes":false_writes}),
    )
}
fn store_episodes(
    compiler: &NativeTurnCompiler<'_>,
    tokenizer: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
    split: &str,
    a: &Args,
) -> Result<Value> {
    let (values, asserts, updates, queries) = pools(split);
    let mut rows = Vec::new();
    let (mut correct, mut queries_total, mut selected_complete, mut bank_complete) = (0, 0, 0, 0);
    for (episode, relation) in ["job", "home"].iter().enumerate() {
        let mut store = StackStore::new(100 + episode as u64, 16)?;
        let mut journal = BTreeMap::<u64, (u64, Vec<u32>)>::new();
        let mut steps = Vec::new();
        let distractor = if *relation == "job" { "home" } else { "job" };
        let turns = vec![
            example(
                distractor,
                "assert",
                &asserts[0].replace("{r}", distractor),
                values[2],
            ),
            example(
                relation,
                "assert",
                &asserts[0].replace("{r}", relation),
                values[0],
            ),
            Example {
                text: "I appreciate our conversation.".into(),
                relation: NONE.into(),
                act: NONE,
                template: None,
            },
            example(relation, "query", &queries[0].replace("{r}", relation), ""),
            example(
                relation,
                "update",
                &updates[0].replace("{r}", relation),
                values[1],
            ),
            example(relation, "query", &queries[1].replace("{r}", relation), ""),
        ];
        for (index, e) in turns.iter().enumerate() {
            let expected = if index < 4 { values[0] } else { values[1] };
            let answers = FrozenAnswers {
                intent: RecordedValueIntent::Current,
                accepted: vec![expected.to_owned(), format!("{expected}.")],
            };
            answers.validate()?;
            let action = compiler.compile(&e.text)?;
            let mut read: Option<Value> = None;
            let mut write = None;
            match &action {
                CompiledAction::Assert { relation, span }
                | CompiledAction::Correct { relation, span } => {
                    let text = e
                        .text
                        .get(span.start..span.end)
                        .ok_or_else(|| fail("predicted span is not UTF8 source range"))?;
                    let tokens = tokenizer.encode(text);
                    let update = if matches!(&action, CompiledAction::Assert { .. }) {
                        Update::Assert
                    } else {
                        Update::Correct
                    };
                    let written = store.write_from(
                        b"compiler-probe",
                        &[1],
                        *relation,
                        &tokens,
                        update,
                        index as u64,
                    )?;
                    journal.insert(written.id, (index as u64, tokenizer.encode(&e.text)));
                    write = Some(written);
                    let bytes = store.to_bytes()?;
                    let history = store.history_sha256()?;
                    fs::write(
                        a.output
                            .join(format!("{split}-episode-{episode}-step-{index}-store.bin")),
                        &bytes,
                    )?;
                    store = StackStore::from_bytes(&bytes, 100 + episode as u64)?;
                    if store.history_sha256()? != history {
                        return Err(fail("store reload changed history").into());
                    }
                }
                CompiledAction::QueryCurrent {
                    relation: predicted_relation,
                } => {
                    let result = store.read(
                        b"compiler-probe",
                        &[1],
                        *predicted_relation,
                        HistoryView::Current,
                    )?;
                    let tokens = result.value().map(|v| v.tokens.clone());
                    let expected = if index < 4 { values[0] } else { values[1] };
                    let hit = tokens
                        .as_ref()
                        .is_some_and(|t| *t == tokenizer.encode(expected));
                    let mut generated = Vec::new();
                    // Admission uses actual predicted store addresses, never the expected label.
                    let mut bank = Vec::new();
                    for address in store.addresses()? {
                        if address.scope != b"compiler-probe" || address.entity != vec![1] {
                            continue;
                        }
                        let actual = store.read(
                            &address.scope,
                            &address.entity,
                            address.relation,
                            HistoryView::Current,
                        )?;
                        if let Some(v) = actual.value() {
                            let (event, cue) = journal.get(&v.record).ok_or_else(|| {
                                fail("actual record has no original statement cue")
                            })?;
                            bank.push(reader::Record {
                                event: *event,
                                record: v.record,
                                commit: v.commit,
                                relation: address.relation,
                                cue: cue.clone(),
                                value: v.tokens.clone(),
                            });
                        }
                    }
                    bank.sort_by_key(|r| (r.event, r.commit, r.record));
                    for (lane, records) in [
                        (
                            "selected-record",
                            bank.iter()
                                .filter(|r| Some(r.record) == result.value().map(|v| v.record))
                                .collect::<Vec<_>>(),
                        ),
                        ("all-bank", bank.iter().collect::<Vec<_>>()),
                    ] {
                        let owned = records
                            .into_iter()
                            .map(|r| reader::Record {
                                event: r.event,
                                record: r.record,
                                commit: r.commit,
                                relation: r.relation,
                                cue: r.cue.clone(),
                                value: r.value.clone(),
                            })
                            .collect::<Vec<_>>();
                        if owned.is_empty() {
                            generated
                                .push(json!({"lane":lane,"status":"no actual admitted records"}));
                            continue;
                        }
                        let mut output = reader::generate(
                            native,
                            &a.cue,
                            &a.prefix,
                            &a.end,
                            &tokenizer.encode(&e.text),
                            &owned,
                            32,
                        )?;
                        let ids: Vec<u32> =
                            serde_json::from_value(output["generated_ids_including_eos"].clone())?;
                        let eos = output["eos"] == true;
                        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
                        let bytes = tokenizer.decode_bytes(plain);
                        let raw = String::from_utf8_lossy(&bytes);
                        let text = raw.strip_prefix(' ').unwrap_or(&raw);
                        let complete = eos
                            && String::from_utf8(bytes.clone()).is_ok()
                            && answers.accepts(text);
                        if e.act == "query" {
                            if lane == "selected-record" {
                                selected_complete += usize::from(complete);
                            } else {
                                bank_complete += usize::from(complete);
                            }
                        }
                        output["lane"] = json!(lane);
                        output["reply_text"] = json!(text);
                        output["complete_answer"] = json!(complete);
                        output["frozen_answers_evaluation_only"] = json!(answers);
                        generated.push(output);
                    }
                    if e.act == "query" {
                        queries_total += 1;
                        correct += usize::from(hit);
                    }
                    read = Some(
                        json!({"store_read":result,"decoded":tokens.map(|t|tokenizer.decode(&t)),"expected_evaluation_only":expected,"exact_store_answer":hit,"native_generation":generated}),
                    );
                }
                _ => {}
            }
            if e.act == "query" && !matches!(&action, CompiledAction::QueryCurrent { .. }) {
                queries_total += 1;
            }
            steps.push(json!({"source":e.text,"predicted":action,"write":write,"read":read}));
        }
        rows.push(json!({"episode":episode,"relation_evaluation_label":relation,"steps":steps,"final_store_history_sha256":store.history_sha256()?}));
    }
    Ok(
        json!({"episodes":rows,"query_rows":queries_total,"exact_store_answers":correct,"selected_record_complete":selected_complete,"all_bank_complete":bank_complete,"policy":"predicted-address-and-source-span-only;serialize-reload-after-writes;original-statement-cues;gold-used-only-to-score"}),
    )
}
fn run(a: &Args) -> Result<()> {
    let started = Instant::now();
    let (training, development, fresh) = (panel("training"), panel("development"), panel("fresh"));
    if training.len() > 512 || development.len() > 128 || fresh.len() > 64 {
        return Err(fail("panel cap exceeded").into());
    }
    let config = FitConfig {
        steps: 64,
        learning_rate: 0.03,
        max_tokens: 128,
        max_words: 64,
        max_value_words: 8,
    };
    let inputs = json!({"schema":"native-turn-frozen-inputs/1","training":input_rows(&training),"development":input_rows(&development),"fresh":input_rows(&fresh),"config":config,"limitations":"authored supervised panels;no general-language qualification"});
    let input_bytes = serde_json::to_vec_pretty(&inputs)?;
    fs::write(a.output.join("frozen-inputs.json"), &input_bytes)?;
    let binding_bytes = fs::read(&a.binding)?;
    let binding_sha = sha256_bytes(&binding_bytes);
    if a.binding_sha
        .as_ref()
        .is_some_and(|sha| sha != &binding_sha)
    {
        return Err(fail("trusted binding receipt mismatch").into());
    }
    let binding: NativeArtifactBinding = serde_json::from_slice(&binding_bytes)?;
    let native = NativeSourceRealizer::load_native(&a.native, &binding)?;
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    let tokenizer_sha = sha256_bytes(&tokenizer_bytes);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| fail("invalid tokenizer"))?;
    let fit = fit_examples(
        &native,
        &tokenizer,
        &tokenizer_bytes,
        &["job".into(), "home".into()],
        &training,
        &development,
        config,
    )?;
    for checkpoint in &fit.checkpoint_artifacts {
        let dir = a.output.join(format!("checkpoint-{:04}", checkpoint.step));
        fs::create_dir(&dir)?;
        fs::write(
            dir.join("native-compiler.json"),
            &checkpoint.native_artifact,
        )?;
        fs::write(
            dir.join("source-parameters.json"),
            &checkpoint.source_parameters,
        )?;
    }
    fs::write(
        a.output.join("selected-native-compiler.json"),
        &fit.native_artifact,
    )?;
    fs::write(
        a.output.join("source-parameters.json"),
        &fit.source_parameters,
    )?;
    let selected = fs::read(a.output.join("selected-native-compiler.json"))?;
    let selected_sha = sha256_bytes(&selected);
    let compiler = NativeTurnCompiler::load(
        &selected,
        &selected_sha,
        &native,
        &tokenizer,
        &tokenizer_bytes,
    )?;
    let dev = evaluate(&compiler, &development)?;
    let fresh_eval = evaluate(&compiler, &fresh)?;
    let dev_store = store_episodes(&compiler, &tokenizer, &native, "development", &a)?;
    let fresh_store = store_episodes(&compiler, &tokenizer, &native, "fresh", &a)?;
    write_json(
        &a.output.join("report.json"),
        &json!({"schema":"geometric-native-compiler-report/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"host":std::env::consts::ARCH,"elapsed_seconds":started.elapsed().as_secs_f64(),"binding_receipt_sha256":binding_sha,"tokenizer_sha256":tokenizer_sha,"binding_trust":if a.binding_sha.is_some(){"explicit-pinned-sha"}else{"caller-supplied-trusted-receipt"},"frozen_inputs_sha256":sha256_bytes(&input_bytes),"selected_step":fit.selected_step,"selected_artifact_sha256":selected_sha,"compiler_identity":compiler.identity(),"checkpoints":fit.checkpoints,"diagnostics":fit.diagnostics,"development":dev,"fresh":fresh_eval,"development_store":dev_store,"fresh_store":fresh_store,"geometric_reader":"actual-selected-record-and-all-bank","native_emission":"own-prefix-native-source-end","claims":"authored native compiler/store/reader episodes;encoder frozen;authored panels;no general prose/reasoning/energy qualification"}),
    )?;
    report_output::seal(&a.output)?;
    report_output::verify(&a.output)?;
    println!("sealed {}", a.output.display());
    Ok(())
}

fn main() -> Result<()> {
    let a = args()?;
    if !a.output.is_absolute() {
        return Err(fail("output must be absolute").into());
    }
    let parent = a
        .output
        .parent()
        .ok_or_else(|| fail("output has no parent"))?;
    let output = fs::canonicalize(parent)?.join(
        a.output
            .file_name()
            .ok_or_else(|| fail("output needs basename"))?,
    );
    for input in [
        &a.native,
        &a.binding,
        &a.tokenizer,
        &a.cue,
        &a.prefix,
        &a.end,
    ] {
        let input = fs::canonicalize(input)?;
        if output.starts_with(&input) {
            return Err(fail("output is beneath input").into());
        }
        for ancestor in input.ancestors() {
            if ancestor.join(report_output::MANIFEST_FILE).is_file() && output.starts_with(ancestor)
            {
                return Err(fail("output is beneath sealed input").into());
            }
        }
    }
    report_output::claim(&a.output)?;
    match run(&a) {
        Ok(()) => Ok(()),
        Err(error) => {
            write_json(
                &a.output.join("failure.json"),
                &json!({"status":"execution-failure","error":error.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"model_quality_evidence":false}),
            )?;
            report_output::seal(&a.output)?;
            report_output::verify(&a.output)?;
            Err(error)
        }
    }
}
