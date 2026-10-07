//! Feasibility gate for a dedicated learned entry scorer.
//!
//! The entry position is the one position with an empty answer prefix, where the
//! target is absent from the physical Copy candidates in every retained row and
//! the exported emission ranks it near the middle of the vocabulary. Before
//! building a scorer, ask the data question: at the boundary, is the first
//! answer token determined by features the native path actually exposes -- the
//! query ids and the physical source records -- or does it need composition?
//!
//! This reads the retained panel directly. No model, no GPU, no fit. It reports,
//! per candidate feature signature, whether the answer is a function of it, and
//! how a count table fitted on half the panel predicts the other half.
//!
//! usage: entry-scorer-feasibility INPUTS LABELS TOKENIZER_JSON

use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    schema: String,
    cases: Vec<Packet>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    #[allow(dead_code)]
    actual_prefix_ids: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        #[allow(dead_code)]
        event: u64,
        record: u64,
        commit: u64,
        #[allow(dead_code)]
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        #[allow(dead_code)]
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    #[allow(dead_code)]
    schema: String,
    #[allow(dead_code)]
    protocol: String,
    #[allow(dead_code)]
    membership_only: bool,
    cases: Vec<Label>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
}

/// One row: the boundary features that the native read exposes, and the first
/// answer token the canonical tokenization law produces.
struct Row {
    id: String,
    answer: u32,
    query: Vec<u32>,
    /// (record, commit, relation, view, entity) per physical source, in order.
    source_identity: Vec<(u64, u64, u32, u32, Vec<u32>)>,
    /// Surface tokens of each physical source, in order.
    source_tokens: Vec<Vec<u32>>,
    /// Context roles and their token counts, in order.
    context: Vec<(u32, usize)>,
}

fn signature(parts: &[String]) -> String {
    parts.join("|")
}

impl Row {
    fn sig_query(&self) -> String {
        signature(&[format!("q{:?}", self.query)])
    }
    fn sig_query_source_tokens(&self) -> String {
        signature(&[
            format!("q{:?}", self.query),
            format!("s{:?}", self.source_tokens),
        ])
    }
    fn sig_query_source_identity(&self) -> String {
        signature(&[
            format!("q{:?}", self.query),
            format!("i{:?}", self.source_identity),
        ])
    }
    fn sig_all(&self) -> String {
        signature(&[
            format!("q{:?}", self.query),
            format!("s{:?}", self.source_tokens),
            format!("i{:?}", self.source_identity),
            format!("c{:?}", self.context),
        ])
    }
    /// The value-form hypothesis: one physical source value token plus the query
    /// and view determines the answer token.
    fn sig_value_form(&self) -> Option<String> {
        let mut values = self
            .source_tokens
            .iter()
            .filter_map(|tokens| tokens.last().copied())
            .collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        (values.len() == 1).then(|| {
            signature(&[
                format!("v{}", values[0]),
                format!("q{:?}", self.query),
                format!("c{:?}", self.context),
            ])
        })
    }
}

fn determinism(rows: &[Row], key: impl Fn(&Row) -> Option<String>) -> (usize, usize, usize) {
    let mut seen = BTreeMap::<String, BTreeSet<u32>>::new();
    let mut covered = 0usize;
    for row in rows {
        if let Some(key) = key(row) {
            covered += 1;
            seen.entry(key).or_default().insert(row.answer);
        }
    }
    let ambiguous = seen.values().filter(|answers| answers.len() > 1).count();
    (seen.len(), ambiguous, covered)
}

fn held_out(
    rows: &[Row],
    key: impl Fn(&Row) -> Option<String>,
    parity: usize,
) -> (usize, usize, usize) {
    let mut table = BTreeMap::<String, BTreeMap<u32, usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        if index % 2 == parity {
            continue;
        }
        if let Some(key) = key(row) {
            *table.entry(key).or_default().entry(row.answer).or_default() += 1;
        }
    }
    let mut seen = 0usize;
    let mut correct = 0usize;
    let mut test = 0usize;
    for (index, row) in rows.iter().enumerate() {
        if index % 2 != parity {
            continue;
        }
        test += 1;
        let Some(key) = key(row) else { continue };
        let Some(counts) = table.get(&key) else {
            continue;
        };
        seen += 1;
        let best = counts
            .iter()
            .max_by(|x, y| x.1.cmp(y.1).then_with(|| y.0.cmp(x.0)))
            .map(|(token, _)| *token);
        if best == Some(row.answer) {
            correct += 1;
        }
    }
    (test, seen, correct)
}

fn report(
    rows: &[Row],
    name: &str,
    key: impl Fn(&Row) -> Option<String> + Copy,
) -> serde_json::Value {
    let (groups, ambiguous, covered) = determinism(rows, key);
    let (test, seen, correct) = held_out(rows, key, 1);
    json!({"signature":name,"rows":rows.len(),"covered_rows":covered,
        "distinct_signatures":groups,"signatures_with_multiple_answers":ambiguous,
        "held_out_test_rows":test,"held_out_signature_seen":seen,"held_out_correct":correct,
        "held_out_accuracy_of_seen":if seen==0 {0.0} else {correct as f64/seen as f64}})
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let inputs_path = args.next().ok_or("INPUTS required")?;
    let labels_path = args.next().ok_or("LABELS required")?;
    let tokenizer_path = args.next().ok_or("TOKENIZER_JSON required")?;
    if args.next().is_some() {
        return Err("three arguments only".into());
    }
    let inputs: Inputs = serde_json::from_slice(&std::fs::read(&inputs_path)?)?;
    let labels: Labels = serde_json::from_slice(&std::fs::read(&labels_path)?)?;
    if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || inputs.cases.len() != labels.cases.len()
    {
        return Err("panel schema or count mismatch".into());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&std::fs::read(&tokenizer_path)?)
        .ok_or("tokenizer unavailable")?;

    let mut rows = Vec::new();
    for (packet, label) in inputs.cases.iter().zip(&labels.cases) {
        if packet.id != label.id {
            return Err("panel id mismatch".into());
        }
        let target = tokenizer.encode(&label.answers.accepted[0]);
        let answer = *target.first().ok_or("empty canonical target")?;
        let (mut source_identity, mut source_tokens, mut context) =
            (Vec::new(), Vec::new(), Vec::new());
        for segment in &packet.segments {
            match segment {
                Segment::Source {
                    record,
                    commit,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                    ..
                } => {
                    source_identity.push((*record, *commit, *relation, *view, entity.clone()));
                    source_tokens.push(original_source_ids.clone());
                }
                Segment::Context {
                    role, token_ids, ..
                } => context.push((*role, token_ids.len())),
            }
        }
        rows.push(Row {
            id: packet.id.clone(),
            answer,
            query: packet.query_ids.clone(),
            source_identity,
            source_tokens,
            context,
        });
    }

    let mut answers = BTreeMap::<u32, usize>::new();
    for row in &rows {
        *answers.entry(row.answer).or_default() += 1;
    }
    let answer_in_source = rows
        .iter()
        .filter(|row| {
            row.source_tokens
                .iter()
                .any(|tokens| tokens.contains(&row.answer))
        })
        .count();
    let value_form = report(&rows, "value_token_query_view", Row::sig_value_form);
    let out = json!({
        "schema": "uor-r4.entry-scorer-feasibility/1",
        "inputs_sha256": uor_r4_training::sha256_file(std::path::Path::new(&inputs_path))?,
        "labels_sha256": uor_r4_training::sha256_file(std::path::Path::new(&labels_path))?,
        "rows": rows.len(),
        "distinct_answers": answers.len(),
        "top_answers": answers.iter().rev().take(8).collect::<BTreeMap<_,_>>(),
        "rows_whose_answer_token_is_in_a_source": answer_in_source,
        "signatures": [
            report(&rows, "query_only", |row| Some(row.sig_query())),
            report(&rows, "query_plus_source_tokens", |row| Some(row.sig_query_source_tokens())),
            report(&rows, "query_plus_source_identity", |row| Some(row.sig_query_source_identity())),
            report(&rows, "all_boundary_features", |row| Some(row.sig_all())),
            value_form,
        ],
        "scope": "offline panel feasibility read for an entry scorer; no model, no fit, no serving claim; train and development are the identical exposed 512-row panel",
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
