//! One bounded learned-routing comparison. Inputs are deliberately small,
//! authored prose/Rust development fixtures, not broad language qualification.
//! Usage: native_geometric_routing_probe PARENT_MODEL NEW_OUTPUT_DIRECTORY
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, fs, path::Path, time::Instant};
use uor_r4_core::native_geometric::{
    Control, Document, Model, RoutingFitConfig, RoutingMode, ValueExample, BOS,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn write(path: &Path, value: &impl Serialize) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn documents(prefix: &str, texts: &[&str]) -> Vec<Document> {
    texts
        .iter()
        .enumerate()
        .map(|(i, text)| Document {
            id: format!("routing-v1/{prefix}/{i}"),
            text: (*text).into(),
        })
        .collect()
}

fn source() -> (Vec<Document>, Vec<Document>, Vec<Document>) {
    let mut fit=documents("fit/prose", &[
        "A map records the paths between towns. A traveler follows a path to reach another town. The shortest path may change when a bridge closes.",
        "Alice put the red book beside the blue cup. Bob moved the cup to the shelf. The book stayed on the table while the cup changed places.",
        "Memory retains information from earlier events. A useful answer selects the relevant information and combines it with the current question.",
        "The first box contains three stones. The second box contains five stones. Moving two stones from the first box to the second leaves one and seven.",
        "A function accepts an input and returns an output. Calling the function twice can produce a different result when the function changes stored state.",
        "Learning adjusts a model using examples. Evaluation uses separate examples to measure whether the learned rule applies beyond the training data.",
        "The river flows from the mountain through the village. After heavy rain the water rises. The villagers close the bridge until the river falls.",
        "A sequence preserves order. Reversing two instructions can change the result. First add four to three, then double the sum to obtain fourteen.",
        "The green key opens the small door. The blue key opens the large door. To enter the small room, select the green key and turn it in the lock.",
        "A local model reads text and predicts the next token. Its prediction depends on retained context, learned parameters and the available computation.",
        "A tree has a root and branches. Each branch can contain smaller branches. Searching one relevant branch avoids visiting every leaf.",
        "The program stores a value under a name. A later instruction reads that value, adds a number and writes the result under a different name.",
    ]);
    fit.extend(documents("fit/rust", &[
        "fn add(left: i32, right: i32) -> i32 { left + right }\nfn main() { assert_eq!(add(3, 4), 7); }",
        "fn twice(value: i32) -> i32 { value + value }\nfn main() { let sum = 3 + 4; assert_eq!(twice(sum), 14); }",
        "fn first(values: &[i32]) -> Option<i32> { values.first().copied() }\nfn main() { assert_eq!(first(&[2, 5]), Some(2)); }",
        "fn count(values: &[i32]) -> usize { values.len() }\nfn main() { assert_eq!(count(&[1, 2, 3]), 3); }",
        "fn largest(left: i32, right: i32) -> i32 { if left > right { left } else { right } }",
        "fn total(values: &[i32]) -> i32 { let mut sum = 0; for value in values { sum += value; } sum }",
        "fn next(value: Option<i32>) -> Option<i32> { value.map(|number| number + 1) }",
        "fn main() { let mut left = 3; let mut right = 5; left -= 2; right += 2; assert_eq!((left, right), (1, 7)); }",
        "fn contains(values: &[i32], target: i32) -> bool { values.iter().any(|value| *value == target) }",
        "fn main() { let name = String::from(\"alice\"); let copy = name.clone(); assert_eq!(name, copy); }",
        "fn positive(value: i32) -> bool { value > 0 }\nfn main() { assert!(positive(4)); }",
        "fn combine(left: &str, right: &str) -> String { format!(\"{} {}\", left, right) }",
    ]));
    let prose=documents("development/prose", &[
        "A traveler reads a map before crossing the river. When the bridge is closed, the traveler must find another path to the village.",
        "Alice moved the blue book from the shelf to the table. Bob left the red cup on the shelf. The two objects are now in different places.",
        "The first instruction adds two to five. The second instruction doubles the result. Applying the instructions in order gives fourteen.",
        "A program can keep a value in memory and read it later. To answer a question, the model must select the relevant value and use it correctly.",
    ]);
    let rust=documents("development/rust", &[
        "fn sum(a: i32, b: i32) -> i32 { a + b }\nfn main() { assert_eq!(sum(5, 2), 7); }",
        "fn double(x: i32) -> i32 { x + x }\nfn main() { let result = double(5 + 2); assert_eq!(result, 14); }",
        "fn last(items: &[i32]) -> Option<i32> { items.last().copied() }\nfn main() { assert_eq!(last(&[5, 2]), Some(2)); }",
        "fn main() { let mut count = 2; count += 5; let result = count + count; assert_eq!(result, 14); }",
    ]);
    (fit, prose, rust)
}

const PROMPTS: [(&str,&str,&str); 8] = [
    ("prose-continuation", "A useful answer selects the relevant information and", " combines"),
    ("rust-continuation", "fn sum(a: i32, b: i32) -> i32 {", " a + b }"),
    ("prose-two-operations", "Start with five. Add two, then double the result. Answer:", "14"),
    ("rust-two-operations", "fn main() { let x = 5 + 2; let y = x + x; assert_eq!(y,", "14); }"),
    ("prose-binding", "The red box belongs to Alice. Alice lives in Rome. Where does the owner of the red box live? Answer:", "Rome"),
    ("rust-binding", "let first = 5; let second = first + 2; let third = second + 3; // third =", "10"),
    ("known-copy-shape", "the orb is red. the cube is green. Now the orb is blue. Question: What color is the orb? Answer:", "blue"),
    ("known-number-shape", "left = 13; right = 4; total:", "17"),
];

fn evaluate(
    model: &Model,
    control: Control,
    prose: &[Document],
    rust: &[Document],
) -> Result<Value> {
    let start = Instant::now();
    let prose_score = model.evaluate(prose, control)?;
    let rust_score = model.evaluate(rust, control)?;
    let next_token_ms = start.elapsed().as_secs_f64() * 1000.0;
    let generation_start = Instant::now();
    let mut generated = Vec::new();
    for (id, prompt, expected) in PROMPTS {
        let output = model.generate(prompt, 24, control)?;
        generated.push(json!({"id":id,"prompt":prompt,"expected":expected,
            "exact":output.text.trim()==expected.trim(),"expected_prefix":output.text.trim_start().starts_with(expected.trim_start()),"generation":output}));
    }
    let mut route_samples = Vec::new();
    for document in [&prose[0], &rust[0]] {
        let mut session = model.session(control)?;
        session.observe(model, BOS)?;
        for token in model.encode(&document.text)?.into_iter().take(16) {
            let prediction = session.predict(model)?;
            route_samples.push(json!({"id":document.id,"prediction":prediction,
                "route":session.routing_decision(),"actual_next_token":token}));
            session.observe(model, token)?;
        }
    }
    Ok(
        json!({"control":control,"artifact_cid":model.artifact_cid(),"prose":prose_score,"rust":rust_score,
        "next_token_ms":next_token_ms,"generation_and_trace_ms":generation_start.elapsed().as_secs_f64()*1000.0,
        "generation":generated,"route_samples":route_samples}),
    )
}

fn token_exposure(model: &Model, documents: &[Document]) -> Result<Value> {
    let mut byte_tokens = 0;
    let mut lexical_tokens = 0;
    for document in documents {
        for token in model.encode(&document.text)? {
            if (2..258).contains(&token) {
                byte_tokens += 1;
            } else {
                lexical_tokens += 1;
            }
        }
    }
    Ok(
        json!({"byte_fallback_tokens":byte_tokens,"lexical_tokens":lexical_tokens,"document_end_targets":documents.len()}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 5 && matches!(args[2].as_str(), "--sources" | "--sources-role-only") {
        return source_routing(
            &args[0],
            &args[1],
            &args[3],
            &args[4],
            args[2] == "--sources-role-only",
        );
    }
    if args.len() == 4 && args[2] == "--recurrent" {
        return recurrent(&args[0], &args[1], &args[3]);
    }
    if args.len() != 2 {
        return Err(
            "usage: native_geometric_routing_probe PARENT_MODEL NEW_OUTPUT_DIRECTORY".into(),
        );
    }
    let output = Path::new(&args[1]);
    fs::create_dir(output)?;
    let start = Instant::now();
    let parent = Model::from_bytes(&fs::read(&args[0])?)?;
    let parent_read_and_validate_ms = start.elapsed().as_secs_f64() * 1000.0;
    let (fit, prose, rust) = source();
    let source = json!({"schema":"uor-r4.routing-development-source/1","scope":"Authored synthetic raw-text development; labels are next tokens. New text is separated from this fit. This is not sealed or broad natural-language qualification.",
        "fit":fit,"development_prose":prose,"development_rust":rust,"prompts":PROMPTS});
    write(&output.join("source.json"), &source)?;
    write(
        &output.join("token-exposure.json"),
        &json!({"vocabulary":parent.vocabulary_size(),
        "fit":token_exposure(&parent,&fit)?,"prose":token_exposure(&parent,&prose)?,"rust":token_exposure(&parent,&rust)?}),
    )?;
    let source_cid = blake3::hash(&serde_json::to_vec(&source)?)
        .to_hex()
        .to_string();
    let parent_result = evaluate(&parent, Control::Full, &prose, &rust)?;
    write(&output.join("parent.json"), &parent_result)?;
    let mut arms = Vec::new();
    for (name, mode, learn_placement) in [
        ("angular", RoutingMode::Angular, true),
        ("equality", RoutingMode::Equality, true),
        ("fixed-placement", RoutingMode::Angular, false),
    ] {
        let config = RoutingFitConfig {
            mode,
            learn_placement,
            ..RoutingFitConfig::default()
        };
        let (model, fit_report) = parent.fit_routing_block(&fit, config)?;
        let bytes = model.to_bytes()?;
        use std::io::Write;
        let mut artifact = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(format!("{name}-model.json")))?;
        artifact.write_all(&bytes)?;
        write(&output.join(format!("{name}-fit.json")), &fit_report)?;
        let load_start = Instant::now();
        let loaded = Model::from_bytes(&bytes)?;
        let deserialize_and_validate_ms = load_start.elapsed().as_secs_f64() * 1000.0;
        let full = evaluate(&loaded, Control::Full, &prose, &rust)?;
        write(&output.join(format!("{name}-full.json")), &full)?;
        let mut controls = Vec::new();
        if name == "angular" {
            for (label, control) in [
                ("disabled", Control::LearnedRoutingDisabled),
                (
                    "selection-disabled",
                    Control::LearnedRoutingSelectionDisabled,
                ),
                (
                    "transform-disabled",
                    Control::LearnedRoutingTransformDisabled,
                ),
            ] {
                let result = evaluate(&model, control, &prose, &rust)?;
                write(&output.join(format!("angular-{label}.json")), &result)?;
                controls.push(result);
            }
            let prompt = PROMPTS[1].1;
            let replay = model.generate(prompt, 24, Control::Full)?
                == loaded.generate(prompt, 24, Control::Full)?;
            if !replay {
                return Err("artifact replay mismatch".into());
            }
            write(
                &output.join("reload.json"),
                &json!({"exact_generation_work_state":replay}),
            )?;
        }
        arms.push(
            json!({"name":name,"artifact_cid":model.artifact_cid(),"artifact_bytes":bytes.len(),"deserialize_and_validate_ms":deserialize_and_validate_ms,
            "fit":fit_report,"full":full,"controls":controls}),
        );
        println!(
            "{}",
            json!({"completed":name,"elapsed_ms":start.elapsed().as_millis()})
        );
    }
    write(
        &output.join("result.json"),
        &json!({"schema":"uor-r4.routing-development-result/1",
        "source_cid":source_cid,"parent_cid":parent.artifact_cid(),"parent_read_and_validate_ms":parent_read_and_validate_ms,"parent":parent_result,"arms":arms,
        "elapsed_ms":start.elapsed().as_millis(),"rust_compile_and_execute":"NOT_RUN; exact generated text is reported",
        "decision":"DEVELOPMENT_ONLY; inspect quality, controls and whole-path cost before adoption"}),
    )?;
    Ok(())
}

fn response_examples(value: &Value, key: &str) -> Result<Vec<ValueExample>> {
    value[key]
        .as_array()
        .ok_or("missing response examples")?
        .iter()
        .map(|row| {
            Ok(ValueExample {
                id: row["id"].as_str().ok_or("missing id")?.into(),
                prompt: row["prompt"].as_str().ok_or("missing prompt")?.into(),
                response: row["response"].as_str().ok_or("missing response")?.into(),
            })
        })
        .collect()
}

fn complete_responses(model: &Model, cases: &[ValueExample], control: Control) -> Result<Value> {
    let started = Instant::now();
    let mut rows = Vec::new();
    let mut exact = 0;
    for case in cases {
        let generation = model.generate(&case.prompt, 64, control)?;
        let equal = generation.text == case.response;
        exact += usize::from(equal);
        rows.push(
            json!({"id":case.id,"prompt":case.prompt,"expected":case.response,
            "exact":equal,"generation":generation}),
        );
    }
    Ok(json!({"exact":exact,"total":cases.len(),"rows":rows,
        "elapsed_ms":started.elapsed().as_secs_f64()*1000.0}))
}

fn recurrent(parent_path: &str, output_path: &str, preservation_source: &str) -> Result<()> {
    let output = Path::new(output_path);
    fs::create_dir(output)?;
    let started = Instant::now();
    let parent = Model::from_bytes(&fs::read(parent_path)?)?;
    let prior_source: Value = serde_json::from_slice(&fs::read(preservation_source)?)?;
    let preservation = response_examples(&prior_source, "development")?;
    // Construction examples only. Earlier OPEN development remains evaluation.
    let mut responses: Vec<_> = response_examples(&prior_source, "fit")?
        .into_iter()
        .step_by(16)
        .collect();
    let (fit, prose, rust) = source();
    for (i,(prompt,response)) in [
        ("A traveler follows a", " path."),
        ("A map records the paths between", " towns."),
        ("Memory retains information from earlier", " events."),
        ("A sequence preserves", " order."),
        ("The river flows through the", " village."),
        ("A tree has a root and", " branches."),
        ("A local model reads text and predicts the next", " token."),
        ("The program stores a value under a", " name."),
        ("fn add(left: i32, right: i32) -> i32 {", " left + right }"),
        ("fn twice(value: i32) -> i32 {", " value + value }"),
        ("fn count(values: &[i32]) -> usize {", " values.len() }"),
        ("fn positive(value: i32) -> bool {", " value > 0 }"),
        ("fn main() { let left = 3; let right = 4; let sum =", " left + right; }"),
        ("fn next(value: Option<i32>) -> Option<i32> {", " value.map(|number| number + 1) }"),
        ("fn first(values: &[i32]) -> Option<i32> {", " values.first().copied() }"),
        ("fn largest(left: i32, right: i32) -> i32 {", " if left > right { left } else { right } }"),
        ("Start with three. Add four, then double the result. Answer:", "14"),
        ("Start with two. Add three, then double the result. Answer:", "10"),
        ("Start with four. Add two, then double the result. Answer:", "12"),
        ("Start with six. Add two, then double the result. Answer:", "16"),
        ("The red box belongs to Alice. Alice lives in Paris. Where does the owner of the red box live? Answer:", "Paris"),
        ("The blue box belongs to Bob. Bob lives in Lima. Where does the owner of the blue box live? Answer:", "Lima"),
        ("let first = 2; let second = first + 3; let third = second + 2; // third =", "7"),
        ("let first = 4; let second = first + 1; let third = second + 3; // third =", "8"),
    ].into_iter().enumerate() {
        responses.push(ValueExample { id:format!("routing-v2/fit/response/{i}"),
            prompt:prompt.into(),response:response.into() });
    }
    let source = json!({"schema":"uor-r4.recurrent-routing-source/1",
        "scope":"Authored OPEN development. Construction-only prior memory examples plus raw text and response targets. Prior 62-case development and eight continuation prompts are reused OPEN evaluation, never sealed or used for fitting this revision.",
        "fit_documents":fit,"fit_responses":responses,"prose":prose,"rust":rust,
        "preservation":preservation,"prompts":PROMPTS});
    write(&output.join("source.json"), &source)?;
    write(
        &output.join("parent.json"),
        &evaluate(&parent, Control::Full, &prose, &rust)?,
    )?;
    let parent_preservation = complete_responses(&parent, &preservation, Control::Full)?;
    write(
        &output.join("parent-preservation.json"),
        &parent_preservation,
    )?;
    let mut arms = Vec::new();
    for (name, mode) in [
        ("angular", RoutingMode::Angular),
        ("equality", RoutingMode::Equality),
    ] {
        let config = RoutingFitConfig {
            max_positions: 1024,
            learned_tokens: 12,
            passes: 1,
            max_seconds: 30,
            mode,
            ..RoutingFitConfig::default()
        };
        let (model, report) = parent.fit_recurrent_routing(&fit, &responses, config)?;
        write(&output.join(format!("{name}-fit.json")), &report)?;
        let bytes = model.to_bytes()?;
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(format!("{name}-model.json")))?
            .write_all(&bytes)?;
        let loaded = Model::from_bytes(&bytes)?;
        let full = evaluate(&loaded, Control::Full, &prose, &rust)?;
        write(&output.join(format!("{name}-full.json")), &full)?;
        let kept = complete_responses(&loaded, &preservation, Control::Full)?;
        write(&output.join(format!("{name}-preservation.json")), &kept)?;
        let fitted_responses = complete_responses(&loaded, &responses, Control::Full)?;
        write(
            &output.join(format!("{name}-fit-responses.json")),
            &fitted_responses,
        )?;
        if name == "angular" {
            for (label, control) in [
                ("chain-disabled", Control::LearnedRoutingChainDisabled),
                (
                    "transform-disabled",
                    Control::LearnedRoutingTransformDisabled,
                ),
                ("disabled", Control::LearnedRoutingDisabled),
            ] {
                write(
                    &output.join(format!("angular-{label}.json")),
                    &evaluate(&loaded, control, &prose, &rust)?,
                )?;
            }
            let disabled =
                complete_responses(&loaded, &preservation, Control::LearnedRoutingDisabled)?;
            write(
                &output.join("angular-disabled-preservation.json"),
                &disabled,
            )?;
            // Compare each complete Generation object with integer-safe Rust Values.
            let exact_parent_objects = disabled["rows"]
                .as_array()
                .ok_or("rows")?
                .iter()
                .zip(
                    parent_preservation["rows"]
                        .as_array()
                        .ok_or("parent rows")?,
                )
                .filter(|(a, b)| {
                    let mut generation = a["generation"].clone();
                    if generation["state"]["control"] != json!("learned_routing_disabled") {
                        return false;
                    }
                    generation["state"]["control"] = json!("full");
                    generation == b["generation"]
                })
                .count();
            write(
                &output.join("parent-replay.json"),
                &json!({"exact_except_declared_control_label":exact_parent_objects,
                "normalized_field":"generation.state.control", "total":preservation.len()}),
            )?;
            if exact_parent_objects != preservation.len() {
                return Err("disabled parent replay mismatch".into());
            }
            if model.generate(PROMPTS[1].1, 24, Control::Full)?
                != loaded.generate(PROMPTS[1].1, 24, Control::Full)?
            {
                return Err("recurrent artifact replay mismatch".into());
            }
        }
        arms.push(
            json!({"name":name,"artifact_cid":loaded.artifact_cid(),"artifact_bytes":bytes.len(),
            "fit":report,"preserved":kept["exact"],"fit_responses_exact":fitted_responses["exact"],
            "prose_correct":full["prose"]["correct"],"rust_correct":full["rust"]["correct"]}),
        );
        println!(
            "{}",
            json!({"completed":name,"elapsed_ms":started.elapsed().as_millis()})
        );
    }
    write(
        &output.join("result.json"),
        &json!({"schema":"uor-r4.recurrent-routing-result/1",
        "source_cid":blake3::hash(&serde_json::to_vec(&source)?).to_hex().to_string(),
        "arms":arms,"elapsed_ms":started.elapsed().as_millis(),
        "decision":"DEVELOPMENT_ONLY_PENDING_BEHAVIOR_REVIEW"}),
    )?;
    Ok(())
}

fn source_routing(
    parent_path: &str,
    output_path: &str,
    development_path: &str,
    first_use_path: &str,
    role_context_only: bool,
) -> Result<()> {
    use uor_r4_core::native_geometric::{SourceRoutingConfig, ValueExample};
    let output = Path::new(output_path);
    fs::create_dir(output)?;
    let start = Instant::now();
    let parent = Model::from_bytes(&fs::read(parent_path)?)?;
    let dev: Value = serde_json::from_slice(&fs::read(development_path)?)?;
    let first: Value = serde_json::from_slice(&fs::read(first_use_path)?)?;
    let fit = response_examples(&dev, "fit")?;
    let preservation = response_examples(&dev, "development")?;
    let prior = response_examples(&first, "development")?;
    let changes = [
        ("velra", "firden"),
        ("tovin", "ashwinx"),
        ("neril", "oakmar"),
        ("sovek", "elmoss"),
        ("Lodov", "Brelan"),
        ("Merok", "Cusven"),
        ("Vesul", "Darnoc"),
    ];
    for (_, name) in changes {
        if fit
            .iter()
            .any(|d| d.prompt.contains(name) || d.response.contains(name))
        {
            return Err("fresh name already in fit".into());
        }
    }
    let fresh: Vec<_> = prior
        .iter()
        .filter_map(|d| {
            let mut prompt = d.prompt.clone();
            let mut response = d.response.clone();
            for (old, new) in changes {
                prompt = prompt.replace(old, new);
                response = response.replace(old, new);
            }
            (prompt != d.prompt).then(|| ValueExample {
                id: format!("source-routing/fresh/{}", d.id),
                prompt,
                response,
            })
        })
        .collect();
    write(
        &output.join("source.json"),
        &json!({"scope":"480 existing construction cases; reused OPEN preservation/first-use evaluation; additional exact name/value substitutions absent from fit. Known grammar, not sealed general language evaluation.","fit":fit,"preservation":preservation,"prior_first_use":prior,"fresh":fresh,"substitutions":changes}),
    )?;
    let baseline = complete_responses(&parent, &preservation, Control::Full)?;
    write(&output.join("parent-preservation.json"), &baseline)?;
    write(
        &output.join("parent-prior.json"),
        &complete_responses(&parent, &prior, Control::Full)?,
    )?;
    write(
        &output.join("parent-fresh.json"),
        &complete_responses(&parent, &fresh, Control::Full)?,
    )?;
    let mut arms = Vec::new();
    for (name, mode) in [
        ("angular", RoutingMode::Angular),
        ("equality", RoutingMode::Equality),
    ] {
        let (model, report) = parent.fit_source_routing(
            &fit,
            SourceRoutingConfig {
                mode,
                role_context_only,
                ..SourceRoutingConfig::default()
            },
        )?;
        write(&output.join(format!("{name}-fit.json")), &report)?;
        let bytes = model.to_bytes()?;
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(format!("{name}-model.json")))?
            .write_all(&bytes)?;
        let loaded = Model::from_bytes(&bytes)?;
        for (label, cases) in [
            ("preservation", &preservation),
            ("prior", &prior),
            ("fresh", &fresh),
        ] {
            let measured = complete_responses(&loaded, cases, Control::Full)?;
            write(&output.join(format!("{name}-{label}.json")), &measured)?;
            arms.push(json!({"name":name,"population":label,"exact":measured["exact"],"total":cases.len(),"elapsed_ms":measured["elapsed_ms"]}));
        }
        let replay = loaded.generate(&prior[0].prompt, 64, Control::Full)?;
        if replay != model.generate(&prior[0].prompt, 64, Control::Full)? {
            return Err("source artifact reload mismatch".into());
        }
        if name == "angular" {
            let disabled =
                complete_responses(&loaded, &preservation, Control::LearnedRoutingDisabled)?;
            let same = disabled["rows"]
                .as_array()
                .ok_or("rows")?
                .iter()
                .zip(baseline["rows"].as_array().ok_or("parent rows")?)
                .filter(|(a, b)| {
                    let mut g = a["generation"].clone();
                    g["state"]["control"] = json!("full");
                    g == b["generation"]
                })
                .count();
            write(&output.join("disabled-preservation.json"), &disabled)?;
            write(
                &output.join("parent-replay.json"),
                &json!({"complete_objects_except_control_label":same,"total":preservation.len()}),
            )?;
            if same != preservation.len() {
                return Err("source-disabled parent replay differs".into());
            }
            write(
                &output.join("codes-disabled-fresh.json"),
                &complete_responses(&loaded, &fresh, Control::LearnedRoutingTransformDisabled)?,
            )?;
        }
        println!(
            "{}",
            json!({"completed":name,"elapsed_ms":start.elapsed().as_millis()})
        );
    }
    write(
        &output.join("result.json"),
        &json!({"schema":"uor-r4.source-routing-comparison/1","arms":arms,"elapsed_ms":start.elapsed().as_millis(),"decision":"DEVELOPMENT_PENDING_REVIEW"}),
    )?;
    Ok(())
}
