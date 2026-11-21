//! Write the conjunction memory from TRAIN and read it beside the model on
//! DEV (`uor_r4_discrete::conjunction`). Its own binary, so the driver source
//! pin (which covers `main.rs`) and every existing checkpoint stay valid.
//!
//! Usage: conjunction-probe CONFIG_JSON CHECKPOINT_JSON|- OUT_JSON
//! MODEL_TARGETS BULK_TARGETS SLOTS unordered|ordered HOLDOUT
//! OUT_JSON must not exist. `-` reads a new model from the config.

use std::process::ExitCode;

#[cfg(feature = "cuda")]
fn run(args: &[String]) -> Result<(), String> {
    use std::path::Path;
    use uor_r4_discrete::conjunction::{self, Options};
    use uor_r4_discrete::training::DriverConfig;

    if args.first().map(String::as_str) == Some("replies") {
        // conjunction-probe replies CONFIG_JSON PANEL_JSON OUT_JSON SLOTS
        // unordered|ordered WINDOW LIMIT EOS ORDERS_CSV|-
        if args.len() != 10 {
            return Err(
                "usage: conjunction-probe replies CONFIG_JSON PANEL_JSON OUT_JSON SLOTS unordered|ordered WINDOW LIMIT EOS ORDERS_CSV|-"
                    .into(),
            );
        }
        let order = match args[5].as_str() {
            "unordered" => conjunction::Order::Unordered,
            "ordered" => conjunction::Order::Ordered,
            other => return Err(format!("order {other}: unordered or ordered")),
        };
        let bytes = std::fs::read(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
        let config: DriverConfig = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let orders: Vec<usize> = if args[9] == "-" {
            Vec::new()
        } else {
            args[9]
                .split(',')
                .map(|text| text.parse::<usize>().map_err(|e| format!("{text}: {e}")))
                .collect::<Result<_, _>>()?
        };
        let number = |text: &String| text.parse::<u32>().map_err(|e| format!("{text}: {e}"));
        let summary = conjunction::run_replies(
            config,
            Path::new(&args[2]),
            Path::new(&args[3]),
            number(&args[4])? as usize,
            order,
            number(&args[6])? as usize,
            number(&args[7])? as usize,
            number(&args[8])?,
            &orders,
        )
        .map_err(|e| e.to_string())?;
        println!("{summary}");
        return Ok(());
    }
    if args.len() != 8 {
        return Err(
            "usage: conjunction-probe CONFIG_JSON CHECKPOINT_JSON|- OUT_JSON MODEL_TARGETS BULK_TARGETS SLOTS unordered|ordered HOLDOUT"
                .into(),
        );
    }
    let order = match args[6].as_str() {
        "unordered" => conjunction::Order::Unordered,
        "ordered" => conjunction::Order::Ordered,
        other => return Err(format!("order {other}: unordered or ordered")),
    };
    let bytes = std::fs::read(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?;
    let config: DriverConfig = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let checkpoint = (args[1] != "-").then(|| Path::new(&args[1]));
    let number = |text: &String| text.parse::<u64>().map_err(|e| format!("{text}: {e}"));
    let options = Options {
        model_targets: number(&args[3])?,
        bulk_targets: number(&args[4])?,
        slots: number(&args[5])? as usize,
        order,
        holdout: number(&args[7])?,
    };
    let report = conjunction::run(config, checkpoint, Path::new(&args[2]), options)
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "empty": report["dev_empty_memory"],
            "model_written": report["model_written"]["dev"],
            "bulk_written": report["bulk_written"]["dev"],
            "tail": report["bulk_written"]["tail"],
        })
    );
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn run(_: &[String]) -> Result<(), String> {
    Err("conjunction-probe needs the cuda feature".into())
}

/// The token pieces of the M2 reply panel's public wording
/// (`m2-reply512`: two current-role statements and one question a case).
#[derive(serde::Deserialize)]
struct M2Grammar {
    bos: u32,
    eos: u32,
    period: u32,
    newline: u32,
    /// `User:` and `\nAssistant:`.
    user: Vec<u32>,
    assistant: Vec<u32>,
    /// Statement openings a role, the value and a period follow:
    /// index 0 job, 1 home.
    statements: [Vec<Vec<u32>>; 2],
    queries: [Vec<Vec<u32>>; 2],
    /// Reply openings a role, the value and a period follow.
    replies: [Vec<u32>; 2],
    /// Ids a TRAIN value is drawn from. None stands in a development value.
    pool: Vec<u32>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind")]
enum M2Segment {
    Context { token_ids: Vec<u32> },
    Source { relation: u32 },
}

#[derive(serde::Deserialize)]
struct M2Case {
    id: String,
    segments: Vec<serde_json::Value>,
    query_ids: Vec<u32>,
}

#[derive(serde::Deserialize)]
struct M2Inputs {
    cases: Vec<M2Case>,
}

impl M2Grammar {
    /// One case as the reader's prompt: each statement its own user turn
    /// with an empty assistant turn, then the question.
    fn prompt(&self, statements: [&[u32]; 2], query: &[u32]) -> Vec<u32> {
        let mut tokens = vec![self.bos];
        for statement in statements {
            tokens.extend(&self.user);
            tokens.extend(statement);
            tokens.extend(&self.assistant);
            tokens.extend([self.eos, self.newline]);
        }
        tokens.extend(&self.user);
        tokens.extend(query);
        tokens.extend(&self.assistant);
        tokens
    }
    fn reply(&self, role: usize, value: &[u32]) -> Vec<u32> {
        let mut tokens = self.replies[role].clone();
        tokens.extend(value);
        tokens.extend([self.period, self.eos]);
        tokens
    }
}

/// A fixed integer sequence (splitmix64) for the TRAIN draw.
struct Draw {
    state: u64,
}

impl Draw {
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut word = self.state;
        word = (word ^ (word >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        word = (word ^ (word >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        word ^ (word >> 31)
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// conjunction-probe m2-format GRAMMAR_JSON INPUTS_JSON OUT_DIR EXAMPLES SEED
///
/// Writes `OUT_DIR/panel.json` (the M2 development cases as prompts and
/// references) and a TRAIN of EXAMPLES cases of the same wording with values
/// drawn from the grammar's pool (`tokens.u16`, `response_mask.u8`).
fn m2_format(args: &[String]) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use std::path::Path;

    if args.len() != 6 {
        return Err(
            "usage: conjunction-probe m2-format GRAMMAR_JSON INPUTS_JSON OUT_DIR EXAMPLES SEED"
                .into(),
        );
    }
    let read = |path: &String| std::fs::read(path).map_err(|e| format!("{path}: {e}"));
    let grammar: M2Grammar = serde_json::from_slice(&read(&args[1])?).map_err(|e| e.to_string())?;
    let inputs: M2Inputs = serde_json::from_slice(&read(&args[2])?).map_err(|e| e.to_string())?;
    let out = Path::new(&args[3]);
    let examples: usize = args[4].parse().map_err(|e| format!("{}: {e}", args[4]))?;
    let seed: u64 = args[5].parse().map_err(|e| format!("{}: {e}", args[5]))?;
    if grammar.pool.is_empty() || grammar.queries.iter().any(Vec::is_empty) {
        return Err("the grammar needs a value pool and questions for both roles".into());
    }
    std::fs::create_dir(out).map_err(|e| format!("{}: {e}", out.display()))?;

    let mut panel = Vec::new();
    for case in &inputs.cases {
        let mut contexts: Vec<Vec<u32>> = Vec::new();
        let mut roles = Vec::new();
        for segment in &case.segments {
            match serde_json::from_value::<M2Segment>(strip(segment)) {
                Ok(M2Segment::Context { token_ids }) => contexts.push(token_ids),
                Ok(M2Segment::Source { relation }) => roles.push(relation as usize),
                Err(e) => return Err(format!("{}: {e}", case.id)),
            }
        }
        if contexts.len() != 2 || roles.len() != 2 {
            return Err(format!("{}: two statements expected", case.id));
        }
        let role = (0..2)
            .find(|&role| grammar.queries[role].contains(&case.query_ids))
            .ok_or_else(|| format!("{}: question not in the grammar", case.id))?;
        let at = roles
            .iter()
            .position(|&relation| relation == role + 1)
            .ok_or_else(|| format!("{}: no statement for the asked role", case.id))?;
        let value = grammar.statements[role]
            .iter()
            .find_map(|opening| contexts[at].strip_prefix(opening.as_slice()))
            .and_then(|rest| rest.strip_suffix(&[grammar.period]))
            .ok_or_else(|| format!("{}: statement not in the grammar", case.id))?;
        panel.push(serde_json::json!({
            "id": case.id,
            "prompt": grammar.prompt([&contexts[0], &contexts[1]], &case.query_ids),
            "reference": grammar.reply(role, value),
        }));
    }

    let mut draw = Draw { state: seed };
    let mut tokens: Vec<u32> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for _ in 0..examples {
        let mut values = Vec::new();
        let mut statements = Vec::new();
        for role in 0..2 {
            let value: Vec<u32> = (0..1 + draw.below(12))
                .map(|_| grammar.pool[draw.below(grammar.pool.len())])
                .collect();
            let openings = &grammar.statements[role];
            let mut statement = openings[draw.below(openings.len())].clone();
            statement.extend(&value);
            statement.push(grammar.period);
            values.push(value);
            statements.push(statement);
        }
        let first = draw.below(2);
        let role = draw.below(2);
        let queries = &grammar.queries[role];
        let prompt = grammar.prompt(
            [&statements[first], &statements[1 - first]],
            &queries[draw.below(queries.len())],
        );
        let reply = grammar.reply(role, &values[role]);
        mask.extend(std::iter::repeat(0u8).take(prompt.len()));
        mask.extend(std::iter::repeat(1u8).take(reply.len()));
        tokens.extend(prompt);
        tokens.extend(reply);
    }
    let bytes =
        uor_r4_discrete::carrier_fresh::uort_bytes(&tokens, 4096).map_err(|e| e.to_string())?;
    let write = |name: &str, bytes: &[u8]| {
        std::fs::write(out.join(name), bytes).map_err(|e| format!("{name}: {e}"))
    };
    let sha = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    write("tokens.u16", &bytes)?;
    write("response_mask.u8", &mask)?;
    write(
        "panel.json",
        &serde_json::to_vec(&panel).map_err(|e| e.to_string())?,
    )?;
    println!(
        "{}",
        serde_json::json!({
            "panel_rows": panel.len(),
            "train_examples": examples,
            "train_tokens": tokens.len(),
            "train_eligible": mask.iter().filter(|&&bit| bit == 1).count(),
            "tokens_sha256": sha(&bytes),
            "mask_sha256": sha(&mask),
        })
    );
    Ok(())
}

/// A segment with only the fields the format reads.
fn strip(segment: &serde_json::Value) -> serde_json::Value {
    match segment["kind"].as_str() {
        Some("Context") => {
            serde_json::json!({"kind": "Context", "token_ids": segment["token_ids"]})
        }
        _ => serde_json::json!({"kind": "Source", "relation": segment["relation"]}),
    }
}

/// `reader-save` trains the reader from a TRAIN stream and writes it as one
/// integer file; `reader-replies` answers a panel from such a file alone.
/// Neither needs a model, a driver config or the cuda feature.
fn reader(args: &[String]) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use std::time::Instant;
    use uor_r4_discrete::conjunction::{self, saved::Reader};

    let sha = |bytes: &[u8]| {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let read = |path: &String| std::fs::read(path).map_err(|e| format!("{path}: {e}"));
    let number = |text: &String| text.parse::<u32>().map_err(|e| format!("{text}: {e}"));
    let fresh = |path: &String| {
        if std::path::Path::new(path).exists() {
            return Err(format!("{path} already exists"));
        }
        Ok(())
    };
    if args[0] == "bind-census" {
        if args.len() != 2 {
            return Err("usage: conjunction-probe bind-census CODEBOOK_TABLE_JSON".into());
        }
        let bytes = read(&args[1])?;
        let file: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", args[1]))?;
        let size = |name: &str| {
            file["spec"][name]
                .as_u64()
                .ok_or(format!("the codebook table has no spec.{name}"))
        };
        let table: Vec<i32> = serde_json::from_value(file["table"].clone())
            .map_err(|e| format!("{} table: {e}", args[1]))?;
        let census = uor_r4_discrete::conjunction::bind::census(
            &table,
            size("vocabulary")? as usize,
            size("blocks")? as usize,
        )
        .map_err(|e| e.to_string())?;
        println!(
            "{}",
            serde_json::json!({ "codebook_sha256": sha(&bytes), "census": census })
        );
        return Ok(());
    }
    if args[0] == "reader-save" {
        if !(12..=16).contains(&args.len()) {
            return Err(
                "usage: conjunction-probe reader-save TOKENS_U16 MASK_U8 OUT_FILE VOCABULARY WINDOW LIMIT BOS EOS SLOTS ORDERS_CSV|- LADDER_LEAST [CODEBOOK_TABLE_JSON|- [latest|pair|octonion:CODEBOOK_TABLE_JSON [open|- [READS_CSV(precise,explain,tail)]]]]"
                    .into(),
            );
        }
        fresh(&args[3])?;
        let stream = read(&args[1])?;
        let mask = read(&args[2])?;
        let tokens: Vec<u32> = uor_r4_discrete::w2_capacity::parse_uort_tokens(&stream)?
            .into_iter()
            .map(u32::from)
            .collect();
        if mask.len() != tokens.len() || mask.iter().any(|&bit| bit > 1) {
            return Err("the mask must hold one 0 or 1 for each token".into());
        }
        let eligible: Vec<usize> = (0..mask.len()).filter(|&place| mask[place] == 1).collect();
        let orders: Vec<usize> = if args[10] == "-" {
            Vec::new()
        } else {
            args[10]
                .split(',')
                .map(|text| text.parse::<usize>().map_err(|e| format!("{text}: {e}")))
                .collect::<Result<_, _>>()?
        };
        // Class codes: each token's writer cell in an address-cells table.
        let mut codebook_sha256 = None;
        use uor_r4_discrete::conjunction::{bind::BindTable, StandBy};
        let (by, bind) = match args.get(13).map(String::as_str) {
            None | Some("latest") => (StandBy::Latest, BindTable::default()),
            Some("pair") => (StandBy::Pair, BindTable::default()),
            Some(other) => {
                let path = other
                    .strip_prefix("octonion:")
                    .ok_or(format!("unknown stand choice {other}"))?
                    .to_string();
                let file: serde_json::Value =
                    serde_json::from_slice(&read(&path)?).map_err(|e| format!("{path}: {e}"))?;
                let blocks = file["spec"]["blocks"]
                    .as_u64()
                    .ok_or("the codebook table has no spec.blocks")?;
                let table: Vec<i32> = serde_json::from_value(file["table"].clone())
                    .map_err(|e| format!("{path} table: {e}"))?;
                let bind =
                    BindTable::from_codebook(&table, number(&args[4])? as usize, blocks as usize)
                        .map_err(|e| e.to_string())?;
                (StandBy::Octonion, bind)
            }
        };
        let stand = format!("{by:?}");
        let open = match args.get(14).map(String::as_str) {
            None => false,
            Some("-") => false,
            Some("open") => true,
            Some(other) => return Err(format!("unknown form choice {other}")),
        };
        let codes: Vec<u16> = match args.get(12).filter(|path| path.as_str() != "-") {
            None => Vec::new(),
            Some(path) => {
                let bytes = read(path)?;
                codebook_sha256 = Some(sha(&bytes));
                let file: serde_json::Value =
                    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
                let blocks = file["spec"]["blocks"]
                    .as_u64()
                    .ok_or("the codebook table has no spec.blocks")?;
                let table: Vec<i32> = serde_json::from_value(file["table"].clone())
                    .map_err(|e| format!("{path} table: {e}"))?;
                uor_r4_discrete::conjunction::saved::cell_codes(
                    &table,
                    number(&args[4])? as usize,
                    blocks as usize,
                )
                .map_err(|e| e.to_string())?
            }
        };
        let mut reads = uor_r4_discrete::conjunction::Reads::default();
        for choice in args.get(15).map(String::as_str).unwrap_or("").split(',') {
            match choice {
                "" => {}
                "precise" => reads.precise = true,
                "explain" => reads.explain = true,
                "tail" => reads.tail_row = true,
                other => return Err(format!("unknown read choice {other}")),
            }
        }
        let started = Instant::now();
        let (reader, trained) = Reader::train(
            &tokens,
            &eligible,
            number(&args[4])?,
            number(&args[5])? as usize,
            number(&args[7])?,
            number(&args[8])?,
            number(&args[9])? as usize,
            &orders,
            number(&args[6])? as usize,
            &codes,
            by,
            bind,
            open,
            reads,
        )
        .map_err(|e| e.to_string())?;
        let train_ms = started.elapsed().as_millis() as u64;
        let started = Instant::now();
        let (bytes, saved) = reader.to_bytes(number(&args[11])?);
        // The file must read back to the same bytes before it is kept.
        let loaded = Reader::from_bytes(&bytes).map_err(|e| e.to_string())?;
        let (again, resaved) = loaded.to_bytes(1);
        if again != bytes || resaved != saved {
            return Err("the saved reader does not read back to its own bytes".into());
        }
        std::fs::write(&args[3], &bytes).map_err(|e| format!("{}: {e}", args[3]))?;
        println!(
            "{}",
            serde_json::json!({
                "tokens_sha256": sha(&stream),
                "mask_sha256": sha(&mask),
                "codebook_sha256": codebook_sha256,
                "class_codes": codes.iter().collect::<std::collections::BTreeSet<_>>().len(),
                "stand": stand,
                "open_forms": open,
                "reads": reads,
                "trained": trained,
                "saved": saved,
                "reader_sha256": sha(&bytes),
                "train_ms": train_ms,
                "save_ms": started.elapsed().as_millis() as u64,
            })
        );
        return Ok(());
    }
    let sharper = args.get(5).map(String::as_str) == Some("sharper");
    if args.len() != 5 && !(args.len() == 6 && sharper) {
        return Err(
            "usage: conjunction-probe reader-replies READER_FILE PANEL_JSON OUT_JSON LIMIT [sharper]"
                .into(),
        );
    }
    fresh(&args[3])?;
    let bytes = read(&args[1])?;
    let panel_bytes = read(&args[2])?;
    let limit = number(&args[4])? as usize;
    let started = Instant::now();
    let mut reader = Reader::from_bytes(&bytes).map_err(|e| e.to_string())?;
    reader.asked.read_sharper_classes(sharper);
    let load_ms = started.elapsed().as_millis() as u64;
    let panel: Vec<conjunction::PanelRow> =
        serde_json::from_slice(&panel_bytes).map_err(|e| e.to_string())?;
    if panel
        .iter()
        .flat_map(|row| row.prompt.iter())
        .any(|&id| id >= reader.vocabulary)
    {
        return Err("a panel id is past the vocabulary".into());
    }
    let started = Instant::now();
    let (count, replies) = conjunction::count_replies_by(&panel, reader.asked.eos(), &|prompt| {
        reader.reply(prompt, limit)
    });
    let reply_us = (started.elapsed().as_micros() as u64).max(1);
    let ids: usize = replies.iter().map(Vec::len).sum();
    // Where class forms could matter: prompts with no token form, with any
    // class form, and with a class form of a shorter gap than the token form.
    let mut no_token_form = 0u64;
    let mut no_token_form_turns: Vec<usize> = Vec::new();
    let mut any_class_form = 0u64;
    let mut class_sharper = 0u64;
    let mut class_sharper_exact = 0u64;
    for (row, reply) in panel.iter().zip(&replies) {
        let (turn, [tokens, classes]) = reader.asked.sharpest(&row.prompt);
        if tokens.is_none() {
            no_token_form += 1;
            no_token_form_turns.push(turn);
        }
        any_class_form += u64::from(classes.is_some());
        if let (Some(tokens), Some(classes)) = (tokens, classes) {
            if classes < tokens {
                class_sharper += 1;
                class_sharper_exact += u64::from(reply == &row.reference);
            }
        }
    }
    no_token_form_turns.sort_unstable();
    // Replies read under class forms alone, and how many of those are exact.
    let mut class_read = 0u64;
    let mut class_read_exact = 0u64;
    let mut open_read = 0u64;
    let mut open_read_exact = 0u64;
    for (row, reply) in panel.iter().zip(&replies) {
        if reader.asked.reads_classes(&row.prompt) {
            class_read += 1;
            class_read_exact += u64::from(reply == &row.reference);
        }
        if reader.asked.reads_open(&row.prompt) {
            open_read += 1;
            open_read_exact += u64::from(reply == &row.reference);
        }
    }
    let summary = serde_json::json!({
        "schema": "uor-r4.discrete-reader-replies/1",
        "reader_sha256": sha(&bytes),
        "reader_bytes": bytes.len(),
        "panel_sha256": sha(&panel_bytes),
        "reply_limit": limit,
        "sharper_class_read": sharper,
        "count": count,
        "no_token_form": no_token_form,
        "no_token_form_turn_lengths": no_token_form_turns,
        "any_class_form": any_class_form,
        "class_sharper": class_sharper,
        "class_sharper_exact": class_sharper_exact,
        "class_read": class_read,
        "class_read_exact": class_read_exact,
        "open_read": open_read,
        "open_read_exact": open_read_exact,
        "load_ms": load_ms,
        "reply_us": reply_us,
        "reply_ids": ids,
        "ids_per_second": ids as u64 * 1_000_000 / reply_us,
    });
    let mut report = summary.clone();
    report["replies"] = serde_json::json!(replies);
    let traces: Vec<Vec<String>> = panel
        .iter()
        .map(|row| {
            let (_, trace) = reader.reply_traced(&row.prompt, limit);
            trace
        })
        .collect();
    report["trace"] = serde_json::json!(traces);
    std::fs::write(
        &args[3],
        serde_json::to_vec(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", args[3]))?;
    println!("{summary}");
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let done = if args.first().map(String::as_str) == Some("m2-format") {
        m2_format(&args)
    } else if matches!(
        args.first().map(String::as_str),
        Some("reader-save" | "reader-replies" | "bind-census")
    ) {
        reader(&args)
    } else {
        run(&args)
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("conjunction-probe: {message}");
            ExitCode::FAILURE
        }
    }
}
