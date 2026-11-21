mod device;

use device::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, io::Write};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Index {
    artifact: String,
    train_tokens: String,
    train_mask: String,
    selected: usize,
    labeled: usize,
    prototypes: Vec<Prototype>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Prototype {
    state: Vec<i32>,
    forms: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prompt {
    id: String,
    prompt: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    id: String,
    reference: Vec<u32>,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn save(path: &str, value: &impl Serialize) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn read_model(artifact: &str) -> Result<IntegerDiscreteModel> {
    let model = IntegerDiscreteModel::from_bytes(&fs::read(artifact)?)?;
    if model.question_reader().is_none() || !model.is_trunk_d11() {
        return Err("requires frozen reader carried by a native D11 trunk".into());
    }
    Ok(model)
}

fn state(model: &IntegerDiscreteModel, prompt: &[u32]) -> Result<Vec<i32>> {
    let limit = model.shape().context as usize;
    Ok(model.question_causal_state(&prompt[prompt.len().saturating_sub(limit)..])?)
}

fn nearest<'a>(query: &[i32], prototypes: &'a [Prototype]) -> (Option<u64>, &'a [u32]) {
    let mut distance = None;
    let mut forms = &[][..];
    for key in prototypes {
        let cost = query
            .iter()
            .zip(&key.state)
            .map(|(&a, &b)| u64::from(a.abs_diff(b)))
            .sum();
        if distance.is_none_or(|best| cost < best) {
            distance = Some(cost);
            forms = &key.forms;
        } else if distance == Some(cost) && forms != key.forms {
            forms = &[];
        }
    }
    (distance, forms)
}

fn train_onsets(tokens: &[u8], mask: &[u8], vocabulary: u32) -> Result<(Vec<u32>, Vec<usize>)> {
    let ids: Vec<_> = uor_r4_discrete::w2_capacity::parse_uort_tokens(tokens)?
        .into_iter()
        .map(u32::from)
        .collect();
    if vocabulary != 4096
        || ids.len() != mask.len()
        || ids.first() != Some(&0)
        || ids.iter().any(|&id| id >= vocabulary)
        || mask.first() != Some(&0)
        || mask.iter().any(|&b| b > 1)
    {
        return Err("UORT vocabulary/BOS or binary response-mask alignment differs".into());
    }
    let onsets = (1..mask.len())
        .filter(|&i| mask[i] == 1 && mask[i - 1] == 0)
        .collect();
    Ok((ids, onsets))
}

fn uort_finite(model: &IntegerDiscreteModel) -> Result<()> {
    let payload = [0u16, 100, 101, 1, 0, 102, 103, 1];
    let mask = [0, 0, 1, 1, 0, 0, 0, 1];
    let mut stream = vec![0u8; 64];
    stream[..4].copy_from_slice(b"UORT");
    stream[4..8].copy_from_slice(&1u32.to_le_bytes());
    stream[8..16].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    stream[16..20].copy_from_slice(&4096u32.to_le_bytes());
    stream.extend(payload.into_iter().flat_map(u16::to_le_bytes));
    let (ids, onsets) = train_onsets(&stream, &mask, 4096)?;
    let reader = model.question_reader().ok_or("missing reader")?;
    if onsets != [2, 7]
        || reader.asked.train_prompt(&ids, onsets[0]) != [0, 100]
        || reader.asked.train_prompt(&ids, onsets[1]) != [0, 102, 103]
        || train_onsets(&stream, &mask[..7], 4096).is_ok()
        || train_onsets(&stream[64..], &mask, 4096).is_ok()
    {
        return Err("planted UORT first/last onset alignment control failed".into());
    }
    Ok(())
}

fn fit(
    engine: &Engine,
    model: &IntegerDiscreteModel,
    tokens: &[u8],
    mask: &[u8],
    cap: usize,
) -> Result<Index> {
    let reader = model.question_reader().ok_or("missing reader")?;
    if !(1..=4096).contains(&cap) {
        return Err("cap must be in 1..=4096".into());
    }
    let (ids, onsets) = train_onsets(tokens, mask, reader.vocabulary)?;
    let selected = cap.min(onsets.len());
    let mut grouped: BTreeMap<Vec<i32>, BTreeMap<Vec<u32>, usize>> = BTreeMap::new();
    let mut labeled = Vec::new();
    for i in 0..selected {
        let prompt = reader
            .asked
            .train_prompt(&ids, onsets[i * onsets.len() / selected]);
        let forms = reader.asked.form_ids(prompt);
        if !prompt.is_empty() && !forms.is_empty() {
            labeled.push((prompt, forms));
        }
    }
    let captured = engine.states(&labeled.iter().map(|(p, _)| *p).collect::<Vec<_>>(), false)?;
    for ((_, forms), state) in labeled.iter().zip(captured) {
        *grouped
            .entry(state)
            .or_default()
            .entry(forms.clone())
            .or_default() += 1;
    }
    let prototypes: Vec<_> = grouped
        .into_iter()
        .filter_map(|(state, counts)| {
            let votes = counts.values().copied().max()?;
            let mut winners = counts.into_iter().filter(|(_, n)| *n == votes);
            let (forms, _) = winners.next()?;
            winners
                .next()
                .is_none()
                .then_some(Prototype { state, forms })
        })
        .collect();
    if prototypes.is_empty() {
        return Err("no nonambiguous TRAIN prototype from frozen reader labels".into());
    }
    let index = Index {
        artifact: model.artifact_sha256().into(),
        train_tokens: digest(tokens),
        train_mask: digest(mask),
        selected,
        labeled: labeled.len(),
        prototypes,
    };
    let exact: Vec<_> = index
        .prototypes
        .iter()
        .map(|row| row.state.clone())
        .collect();
    if engine
        .matches(&exact, &index)?
        .into_iter()
        .enumerate()
        .any(|(row, (distance, matched))| distance != Some(0) || row != matched)
    {
        return Err("TRAIN prototype index failed GPU exact replay".into());
    }
    Ok(index)
}

fn predict(
    engine: &Engine,
    model: &IntegerDiscreteModel,
    index: &Index,
    panel: &[Prompt],
    limit: usize,
) -> Result<Vec<Value>> {
    let reader = model.question_reader().ok_or("missing reader")?;
    let forms = reader.asked.forms_by_id();
    let words = model.shape().blocks as usize * 8;
    if index.artifact != model.artifact_sha256()
        || index.prototypes.is_empty()
        || index.prototypes.iter().any(|p| {
            p.state.len() != words
                || p.forms.is_empty()
                || p.forms.iter().any(|id| !forms.contains_key(id))
        })
        || panel.len() < 2
        || panel.len() % 2 != 0
        || panel.len() > 4096
        || limit == 0
        || panel.iter().any(|row| {
            row.prompt.is_empty() || row.prompt.iter().any(|&id| id >= reader.vocabulary)
        })
        || panel
            .iter()
            .map(|row| &row.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != panel.len()
    {
        return Err("index binding, row, prompt, panel size, or reply limit invalid".into());
    }
    let prompts: Vec<_> = panel.iter().map(|row| row.prompt.as_slice()).collect();
    let live = engine.states(&prompts, false)?;
    let knockout = engine.states(&prompts, true)?;
    let live_matches = engine.matches(&live, index)?;
    let knockout_matches = engine.matches(&knockout, index)?;
    panel.iter().enumerate().map(|(i, row)| {
        let lexical = reader.asked.form_ids(&row.prompt);
        // Panels supply matched role pairs in adjacent rows.
        let donor = i ^ 1;
        let arm = |forms: &[u32], distance: Option<u64>, prototype: Option<usize>| -> Result<Value> {
            let (reply, under_form, writer_trace) =
                reader.asked.reply_for_forms(&reader.ladder, &row.prompt, limit, forms);
            let (replayed, writer_words, empty_read_retry) =
                reader.asked.reply_words_for_forms(&reader.ladder, &row.prompt, limit, forms);
            if replayed != reply {
                return Err("explicit-form writer words diverged from original-ID trajectory".into());
            }
            Ok(json!({"forms": forms, "distance": distance, "prototype": prototype, "reply": reply,
                "under_form": under_form, "abstained": forms.is_empty(), "writer_trace": writer_trace,
                "writer_step_words": writer_words, "empty_read_retry": empty_read_retry,
                "ended": reply.last() == Some(&1), "reached_limit": reply.len() == limit}))
        };
        let selected = |(distance, at): (Option<u64>, usize)| {
            let matched = index.prototypes.get(at);
            arm(matched.map_or(&[], |p| p.forms.as_slice()), distance, matched.map(|_| at))
        };
        let lexical_arm = arm(&lexical, None, None)?;
        if lexical_arm["reply"] != json!(model.question_reply(&row.prompt, limit)?) {
            return Err("ordinary lexical writer parity failed".into());
        }
        let raw_hash = |v: &[i32]| digest(&v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>());
        Ok(json!({
            "id": row.id, "prompt": row.prompt,
            "lexical": lexical_arm, "transported": selected(live_matches[i])?,
            "knockout": selected(knockout_matches[i])?, "permuted": selected(live_matches[donor])?,
            "fallback": arm(&[], None, None)?,
            "trunk_state": live[i], "knockout_state": knockout[i],
            "donor_id": panel[donor].id, "donor_state": live[donor],
            "trunk_state_sha256": raw_hash(&live[i]),
            "knockout_state_sha256": raw_hash(&knockout[i]),
            "donor_state_sha256": raw_hash(&live[donor]),
        }))
    }).collect()
}

fn first_wrong(reference: &[u32], reply: &[u32]) -> Option<Value> {
    (0..reference.len().max(reply.len()))
        .find(|&i| reference.get(i) != reply.get(i))
        .map(|i| json!({"index": i, "expected": reference.get(i), "actual": reply.get(i)}))
}

fn adjudicate(predictions: Vec<Value>, references: Vec<Reference>) -> Result<Vec<Value>> {
    if predictions.len() != references.len() {
        return Err("prediction/reference count differs".into());
    }
    predictions
        .into_iter()
        .zip(references)
        .map(|(row, gold)| {
            if row["id"].as_str() != Some(&gold.id) || gold.reference.is_empty() {
                return Err("prediction/reference ID mismatch or empty reference".into());
            }
            let mut result = json!({"id": gold.id, "reference": gold.reference});
            for arm in ["lexical", "transported", "knockout", "permuted", "fallback"] {
                let reply: Vec<u32> = serde_json::from_value(row[arm]["reply"].clone())?;
                result[arm] =
                    json!({"first_wrong": first_wrong(&gold.reference, &reply), "reply": reply});
            }
            Ok(result)
        })
        .collect()
}

fn finite(model: &IntegerDiscreteModel, engine: &Engine) -> Result<Value> {
    uort_finite(model)?;
    let ablated = model.with_state_transport_knockout();
    // A fixed paired family keeps the two-token suffix and offsets constant.
    let synthetic: Vec<[u32; 4]> = (2..=17).map(|first| [0, first, 18, 19]).collect();
    let prompts: Vec<&[u32]> = synthetic.iter().map(|row| row.as_slice()).collect();
    let gpu = engine.states(&prompts, false)?;
    let gpu_ablation = engine.states(&prompts, true)?;
    for (i, prefix) in prompts.iter().enumerate() {
        if state(model, prefix)? != gpu[i] || state(&ablated, prefix)? != gpu_ablation[i] {
            return Err(format!("CPU/GPU pre-gate raw-word mismatch at synthetic row {i}").into());
        }
        if ablated.evaluate_prefix(prefix)?.read_vector != gpu_ablation[i] {
            return Err("ablation disagrees with ordinary production read".into());
        }
    }
    let donor = gpu
        .iter()
        .position(|query| query != &gpu[0])
        .ok_or("matched same-suffix synthetic inputs never change the transported state")?;
    let changed_under_ablation = gpu
        .iter()
        .zip(&gpu_ablation)
        .filter(|(a, b)| a != b)
        .count();
    if changed_under_ablation == 0 {
        return Err("knockout changes no state on the synthetic panel".into());
    }
    let mut index = Index {
        artifact: model.artifact_sha256().into(),
        train_tokens: String::new(),
        train_mask: String::new(),
        selected: 0,
        labeled: 0,
        prototypes: vec![
            Prototype {
                state: gpu[0].clone(),
                forms: vec![123],
            },
            Prototype {
                state: gpu[donor].clone(),
                forms: vec![456],
            },
        ],
    };
    let queries = [gpu[0].clone(), gpu[donor].clone()];
    let device_matches = engine.matches(&queries, &index)?;
    if device_matches != [(Some(0), 0), (Some(0), 1)] {
        return Err("device nearest did not switch on actual donor state".into());
    }
    index.prototypes[1].state.clone_from(&queries[0]);
    if engine.matches(&queries[..1], &index)? != [(Some(0), u32::MAX as usize)]
        || nearest(&queries[0], &index.prototypes) != (Some(0), &[][..])
    {
        return Err("equal-distance label conflict must abstain".into());
    }
    Ok(json!({
        "artifact": model.artifact_sha256(),
        "same_suffix_probes": prompts.len(),
        "uort_first_last_onsets_alignment": true,
        "all_raw_pre_gate_reads_match_host": true,
        "all_knockout_reads_match_ordinary_evaluation": true,
        "changed_under_ablation": changed_under_ablation,
        "matched_prefixes": [prompts[0], prompts[donor]],
        "transported_states": [gpu[0], gpu[donor]],
        "knockout_states": [gpu_ablation[0], gpu_ablation[donor]],
        "different_labels_on_state_donation": true,
        "conflicting_label_abstention": true,
        "labels_are_synthetic": true,
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, artifact, tokens, mask, cap, out] if mode == "fit" => {
            let model = read_model(artifact)?;
            let engine = Engine::load(&fs::read(artifact)?)?;
            let index = fit(&engine, &model, &fs::read(tokens)?, &fs::read(mask)?, cap.parse()?)?;
            save(out, &index)
        }
        [mode, artifact, index, panel, limit, out] if mode == "predict" => {
            let model = read_model(artifact)?;
            let index: Index = serde_json::from_slice(&fs::read(index)?)?;
            let panel: Vec<Prompt> = serde_json::from_slice(&fs::read(panel)?)?;
            let engine = Engine::load(&fs::read(artifact)?)?;
            save(out, &predict(&engine, &model, &index, &panel, limit.parse()?)?)
        }
        [mode, predictions, references, out] if mode == "adjudicate" => {
            let predictions = serde_json::from_slice(&fs::read(predictions)?)?;
            let references = serde_json::from_slice(&fs::read(references)?)?;
            save(out, &adjudicate(predictions, references)?)
        }
        [mode, artifact] if mode == "uort-finite" => uort_finite(&read_model(artifact)?),
        [mode, artifact, out] if mode == "finite" => save(out, &finite(&read_model(artifact)?, &Engine::load(&fs::read(artifact)?)?)?),
        _ => Err("usage: question-causal-state fit ARTIFACT TOKENS_U16 MASK_U8 CAP OUT | predict ARTIFACT INDEX PROMPTS LIMIT OUT | adjudicate PREDICTIONS REFERENCES OUT | finite ARTIFACT OUT".into()),
    }
}
