//! Dialogue learning and replies for the geometric stack
//! ([`crate::geometric_stack`]).
//!
//! The stack learns the prepared literal-role corpus
//! (`uor_r4_tokenizer::dialogue`, schema `uor-r4.literal-role-dialogue/1`)
//! through the retained dialogue study's episodes
//! ([`crate::dialogue_episodes`]): the same contract, eligibility,
//! response-uniform sampler and source-stratified development panel
//! ([`crate::dialogue_development::select`]). The contract is built the way
//! that study builds it: UNK pads, and the assistant marker is the empty
//! assistant prefix after BOS. Replies to a request panel also follow the
//! study: greedy, stopped at EOS or a token cap, and an unfinished reply is
//! closed with EOS by the caller before the next user turn.
//!
//! A prepared split is a UORT token store ([`MmapCorpusReader`]), one
//! response-mask byte per token, and the split manifest, whose `files` give
//! each source's label and token count in store order.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::dialogue_episodes::{
    EpisodeBatch, EpisodeContract, EpisodeIndex, PrefixPolicy, SourceSpan, EPISODE_CONTEXT,
};
use crate::geometric_stack::StackModel;
use crate::{invalid, Result};

/// One prepared split of the dialogue corpus.
pub struct DialogueSplit {
    reader: MmapCorpusReader,
    mask: Vec<u8>,
    sources: Vec<SourceSpan>,
}

impl DialogueSplit {
    /// Read a split's token store, response mask and manifest. The manifest
    /// must record no literal special-token text, overall and per source, as
    /// the retained study requires.
    pub fn load(tokens: &Path, mask: &Path, manifest: &Path) -> Result<Self> {
        let reader = MmapCorpusReader::open(tokens)
            .map_err(|e| invalid(format!("dialogue token store: {e}")))?;
        let mask = fs::read(mask)?;
        let manifest: Value = serde_json::from_slice(&fs::read(manifest)?)?;
        if reader.as_slice().len() != mask.len() || !cfg!(target_endian = "little") {
            return Err(invalid("dialogue tokens and mask differ in length"));
        }
        if manifest["drops"]["special_token_occurrences"] != 0 {
            return Err(invalid("the split holds literal special-token text"));
        }
        let files = manifest["files"]
            .as_array()
            .ok_or_else(|| invalid("the split manifest has no files"))?;
        let mut sources = Vec::with_capacity(files.len());
        let mut start = 0usize;
        for file in files {
            let tokens = file["tokens"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| invalid("a source without a token count"))?;
            if file["special_token_occurrences"] != 0 {
                return Err(invalid("a source holds literal special-token text"));
            }
            let label = file["label"]
                .as_str()
                .ok_or_else(|| invalid("a source without a label"))?;
            let end = start
                .checked_add(tokens)
                .ok_or_else(|| invalid("source lengths overflow"))?;
            sources.push(SourceSpan {
                label: label.to_owned(),
                start,
                end,
            });
            start = end;
        }
        if start != mask.len() {
            return Err(invalid("the sources do not cover the token store"));
        }
        Ok(Self {
            reader,
            mask,
            sources,
        })
    }

    /// The store's declared vocabulary size.
    pub fn vocab_size(&self) -> usize {
        self.reader.vocab_size() as usize
    }

    /// Every episode of the split under `contract`.
    pub fn index(&self, contract: EpisodeContract) -> Result<EpisodeIndex<'_>> {
        EpisodeIndex::new(self.reader.as_slice(), &self.mask, contract, &self.sources)
    }
}

/// The literal-role protocol of `tokenizer` and the episode contract the
/// retained study builds from it.
pub fn episode_contract(
    tokenizer: &ByteBpeTokenizer,
    vocab_size: usize,
) -> Result<(DialogueProtocol, EpisodeContract)> {
    let protocol =
        DialogueProtocol::literal_roles_v1(tokenizer).map_err(|e| invalid(e.to_string()))?;
    let encoder = protocol
        .bind(tokenizer)
        .map_err(|e| invalid(e.to_string()))?;
    let marker = encoder.encode_assistant_prefix(&[]);
    if marker.tokens.first() != Some(&protocol.bos_id) {
        return Err(invalid(
            "the empty assistant prefix does not start with BOS",
        ));
    }
    let contract = EpisodeContract {
        context: EPISODE_CONTEXT,
        vocab_size,
        bos_id: protocol.bos_id,
        eos_id: protocol.eos_id,
        unk_id: protocol.unk_id,
        padding_id: protocol.unk_id,
        assistant_marker_ids: marker.tokens[1..].to_vec(),
    };
    Ok((protocol, contract))
}

/// An episode batch without the columns after its longest episode: inputs,
/// targets, weights and the trimmed time. Padding follows each episode's last
/// target and the stack is causal, so no weighted target changes; only the
/// wasted positions go.
pub struct Trimmed {
    pub inputs: Vec<u32>,
    pub targets: Vec<u32>,
    pub weights: Vec<f32>,
    pub time: usize,
}

pub fn trim(batch: &EpisodeBatch) -> Trimmed {
    let time = batch
        .rows
        .iter()
        .map(|row| row.counts.real_input_positions)
        .max()
        .unwrap_or(1)
        .max(1);
    let keep = |values: &[u32]| -> Vec<u32> {
        values
            .chunks(batch.time)
            .flat_map(|row| row[..time].iter().copied())
            .collect()
    };
    Trimmed {
        inputs: keep(&batch.inputs),
        targets: keep(&batch.targets),
        weights: batch
            .weights
            .chunks(batch.time)
            .flat_map(|row| row[..time].iter().copied())
            .collect(),
        time,
    }
}

#[derive(Clone, Default)]
struct Totals {
    responses: usize,
    targets: usize,
    nll: f64,
    first_targets: usize,
    first_nll: f64,
    eos_targets: usize,
}

impl Totals {
    fn merge(&mut self, other: &Self) {
        self.responses += other.responses;
        self.targets += other.targets;
        self.nll += other.nll;
        self.first_targets += other.first_targets;
        self.first_nll += other.first_nll;
        self.eos_targets += other.eos_targets;
    }

    fn report(&self) -> Value {
        json!({
            "selected_responses": self.responses,
            "supervised_targets": self.targets,
            "eos_targets": self.eos_targets,
            "response_mean_nll": (self.targets != 0).then(|| self.nll / self.targets as f64),
            "first_four_targets": self.first_targets,
            "first_four_response_targets_mean_nll":
                (self.first_targets != 0).then(|| self.first_nll / self.first_targets as f64),
        })
    }
}

/// Score a stack on the development responses `ids` under their full original
/// prefixes: the token-mean response NLL and the NLL of each response's first
/// four targets, pooled and per source. The fields are those of the retained
/// study's panel report (`dialogue_development::evaluate`); here every
/// target's NLL is computed in f64 and summed in f64.
pub fn development(
    model: &StackModel,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch: usize,
) -> Result<Value> {
    let contract = index.contract();
    if !(1..=64).contains(&batch)
        || ids.is_empty()
        || model.config.context != contract.context
        || model.config.vocab_size != contract.vocab_size
    {
        return Err(invalid(
            "development needs responses, a batch of 1..64 and the model's context and vocabulary",
        ));
    }
    let unique: BTreeSet<_> = ids.iter().collect();
    if unique.len() != ids.len() {
        return Err(invalid("duplicate development response"));
    }
    let mut totals = vec![Totals::default(); index.sources().len()];
    let mut selected = vec![Vec::new(); index.sources().len()];
    for chunk in ids.chunks(batch) {
        let episodes = index.materialize(chunk, PrefixPolicy::FullPrefix)?;
        let trimmed = trim(&episodes);
        let time = trimmed.time;
        let nll = model.target_nll(&trimmed.inputs, &trimmed.targets, episodes.batch, time)?;
        for (lane, row) in episodes.rows.iter().enumerate() {
            let total = &mut totals[row.source_index];
            selected[row.source_index].push(row.response_id);
            total.responses += 1;
            total.eos_targets += row.counts.eos_targets;
            let start = lane * time;
            let mut first = 0usize;
            for (weight, value) in trimmed.weights[start..start + time]
                .iter()
                .zip(&nll[start..start + time])
            {
                if *weight == 1.0 {
                    total.targets += 1;
                    total.nll += value;
                    if first < 4 {
                        total.first_targets += 1;
                        total.first_nll += value;
                        first += 1;
                    }
                }
            }
        }
    }
    let mut pooled = Totals::default();
    let mut sources = Vec::with_capacity(totals.len());
    for (source_index, total) in totals.iter().enumerate() {
        pooled.merge(total);
        let mut report = total.report();
        report["source_index"] = json!(source_index);
        report["label"] = json!(index.sources()[source_index].label);
        report["eligible_responses"] =
            json!(index.population().sources[source_index].eligible_responses);
        report["response_ids"] = json!(selected[source_index]);
        sources.push(report);
    }
    let mut report = pooled.report();
    report["schema"] = json!("uor-r4.stack-dialogue-development/1");
    report["response_ids"] = json!(ids);
    report["per_source"] = json!(sources);
    report["conditioning"] = json!("full_original_prefix");
    report["scope"] = json!(
        "Token means over the selected development responses, not the full corpus or an equal-source mean."
    );
    Ok(report)
}

/// One request of a reply panel: user turns answered in order, the format of
/// the retained study's request files.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub id: String,
    pub category: String,
    pub user_turns: Vec<String>,
}

/// Read a request panel.
pub fn load_requests(path: &Path) -> Result<Vec<Request>> {
    let requests: Vec<Request> = serde_json::from_slice(&fs::read(path)?)?;
    if requests.is_empty()
        || requests
            .iter()
            .any(|r| r.user_turns.is_empty() || r.user_turns.iter().any(|t| t.trim().is_empty()))
    {
        return Err(invalid(
            "a request panel needs requests with nonblank turns",
        ));
    }
    Ok(requests)
}

/// A generated reply: its ids (ending in EOS when the model ended it) and
/// whether the model ended it.
pub struct Reply {
    pub ids: Vec<u32>,
    pub eos: bool,
}

/// Answer every request. For each user turn, the turn's prefix is appended
/// to the history and `reply(history, cap)` generates up to `cap` ids. A reply
/// the model did not end is closed with EOS before the next user turn. The
/// whole history must fit the context with every reply at its cap; nothing is
/// truncated.
pub fn reply_panel(
    encoder: &DialogueEncoder<'_>,
    protocol: &DialogueProtocol,
    requests: &[Request],
    context: usize,
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    reply: &mut dyn FnMut(&[u32], usize) -> Result<Reply>,
) -> Result<Value> {
    if max_new_tokens == 0 {
        return Err(invalid("replies need a positive token cap"));
    }
    let mut rows = Vec::with_capacity(requests.len());
    for request in requests {
        let mut history = vec![protocol.bos_id];
        let mut turns = Vec::with_capacity(request.user_turns.len());
        for (turn, user) in request.user_turns.iter().enumerate() {
            let prefix = encoder.encode_user_prefix(user, turn != 0);
            if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                return Err(invalid(format!("request {}: turn {turn}", request.id)));
            }
            history.extend(&prefix.tokens);
            if history.len() + max_new_tokens + 1 > context {
                return Err(invalid(format!(
                    "request {}: the history and a capped reply exceed the context",
                    request.id
                )));
            }
            let generated = reply(&history, max_new_tokens)?;
            history.extend(&generated.ids);
            let closed = !generated.eos && turn + 1 < request.user_turns.len();
            if closed {
                history.push(protocol.eos_id);
            }
            let text_ids: Vec<u32> = generated
                .ids
                .iter()
                .copied()
                .filter(|&id| id != protocol.eos_id)
                .collect();
            turns.push(json!({
                "turn": turn + 1, "user": user, "reply": decode(&text_ids),
                "reply_ids": generated.ids, "model_eos": generated.eos,
                "caller_eos_inserted_before_next_request": closed,
            }));
        }
        rows.push(json!({
            "id": request.id, "category": request.category, "turns": turns,
            "history_ids": history,
        }));
    }
    Ok(json!({"rows": rows, "max_new_tokens": max_new_tokens}))
}

/// The float stack's greedy reply to `history`: the highest logit (ties to the
/// lower id) until EOS or `cap` ids. Each step recomputes the whole window.
pub fn greedy_reply(model: &StackModel, history: &[u32], cap: usize, eos: u32) -> Result<Reply> {
    let mut window = history.to_vec();
    let mut ids = Vec::with_capacity(cap);
    for _ in 0..cap {
        if window.len() > model.config.context {
            return Err(invalid("the reply outgrew the context"));
        }
        let logits = model.forward(&window, 1, window.len())?.detach();
        let last = logits.get(window.len() - 1)?.to_vec1::<f32>()?;
        let mut best = 0usize;
        for (i, v) in last.iter().enumerate() {
            if *v > last[best] {
                best = i;
            }
        }
        let next = best as u32;
        ids.push(next);
        window.push(next);
        if next == eos {
            return Ok(Reply { ids, eos: true });
        }
    }
    Ok(Reply { ids, eos: false })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue_development;
    use crate::geometric_stack::{ReadScore, StackAdamW, StackArch, StackConfig};
    use candle_core::Device;
    use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
    use uor_r4_tokenizer::dialogue::Message;

    /// GPT-2's byte-to-character alphabet.
    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut table = vec!['\0'; 256];
        let mut extra = 0;
        for byte in 0u32..256 {
            table[byte as usize] = if printable.contains(&byte) {
                char::from_u32(byte).unwrap()
            } else {
                extra += 1;
                char::from_u32(255 + extra).unwrap()
            };
        }
        table
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2.
    fn tokenizer() -> ByteBpeTokenizer {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        for (byte, ch) in alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = ["<|bos|>", "<|eos|>", "<|unk|>"]
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    const VOCAB: usize = 288;

    /// Two sources of one- and two-turn conversations whose answers follow
    /// from the questions, written as a prepared split.
    fn split(directory: &Path, tokenizer: &ByteBpeTokenizer) -> DialogueSplit {
        let protocol = DialogueProtocol::literal_roles_v1(tokenizer).unwrap();
        let encoder = protocol.bind(tokenizer).unwrap();
        let mut tokens = Vec::new();
        let mut mask = Vec::new();
        let mut files = Vec::new();
        for (label, pairs) in [
            ("colors", [("sky?", "blue"), ("grass?", "green")]),
            ("sounds", [("cat?", "meow"), ("dog?", "woof")]),
        ] {
            let start = tokens.len();
            let turn = |(question, answer): (&'static str, &'static str)| {
                [
                    Message {
                        role: "user",
                        content: question,
                    },
                    Message {
                        role: "assistant",
                        content: answer,
                    },
                ]
            };
            let (a, b) = (turn(pairs[0]), turn(pairs[1]));
            let documents = [a.to_vec(), b.to_vec(), [a, b].concat(), [b, a].concat()];
            for _ in 0..3 {
                for messages in &documents {
                    let document = encoder.encode_document(messages);
                    tokens.extend(document.tokens.iter().map(|&id| id as u16));
                    mask.extend(document.response_mask);
                }
            }
            files.push(json!({"label": label, "tokens": tokens.len() - start,
                "special_token_occurrences": 0}));
        }
        let paths = [
            directory.join("tokens.uort"),
            directory.join("mask.bin"),
            directory.join("manifest.json"),
        ];
        CorpusWriter::write_file(&paths[0], VOCAB as u32, &tokens).unwrap();
        fs::write(&paths[1], &mask).unwrap();
        fs::write(
            &paths[2],
            json!({"files": files, "drops": {"special_token_occurrences": 0}}).to_string(),
        )
        .unwrap();
        DialogueSplit::load(&paths[0], &paths[1], &paths[2]).unwrap()
    }

    fn stack() -> StackModel {
        let mut config = StackConfig::transformer_control(5);
        config.arch = StackArch::Geometric;
        config.vocab_size = VOCAB;
        config.width = 32;
        config.heads = 2;
        config.mlp_hidden = 64;
        config.pattern = "ra".to_owned();
        config.read = ReadScore::Lorentz;
        StackModel::new(config, &Device::Cpu).unwrap()
    }

    #[test]
    fn the_stack_learns_responses_of_a_prepared_split_and_answers_requests() {
        let directory = std::env::temp_dir().join(format!(
            "uor-r4-stack-dialogue-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let tokenizer = tokenizer();
        let split = split(&directory, &tokenizer);
        let (protocol, contract) = episode_contract(&tokenizer, split.vocab_size()).unwrap();
        assert_eq!(contract.padding_id, protocol.unk_id);
        assert_eq!(
            tokenizer.decode(&contract.assistant_marker_ids),
            "Assistant: "
        );
        let index = split.index(contract).unwrap();
        // Per source, three copies of two one-turn and two two-turn documents.
        assert_eq!(index.episodes().len(), 2 * 3 * (1 + 1 + 2 + 2));
        let panel = dialogue_development::select(&index, 7, 2).unwrap();
        assert_eq!(panel.len(), 4);

        let model = stack();
        let before = development(&model, &index, &panel, 4).unwrap();
        // Trimming the padding columns leaves the weighted loss unchanged.
        let ids = index.sample_ids(3, 0, 8).unwrap();
        let batch = index.materialize(&ids, PrefixPolicy::FullPrefix).unwrap();
        let trimmed = trim(&batch);
        assert!(trimmed.time < 64);
        let full = model
            .weighted_loss(
                &batch.inputs,
                &batch.targets,
                &batch.weights,
                batch.batch,
                batch.time,
            )
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        let short = model
            .weighted_loss(
                &trimmed.inputs,
                &trimmed.targets,
                &trimmed.weights,
                batch.batch,
                trimmed.time,
            )
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        assert!(
            (full - short).abs() < 1e-5,
            "trimmed loss {short} against {full}"
        );
        let mut optimizer = StackAdamW::new(&model, 0.0, 1.0).unwrap();
        for step in 0..240u64 {
            let ids = index.sample_ids(3, step, 8).unwrap();
            let batch = index.materialize(&ids, PrefixPolicy::FullPrefix).unwrap();
            let trimmed = trim(&batch);
            // Only response targets carry weight.
            for ((&weight, &target), &input) in trimmed
                .weights
                .iter()
                .zip(&trimmed.targets)
                .zip(&trimmed.inputs)
            {
                if weight != 0.0 {
                    assert_ne!(target, protocol.bos_id);
                    assert_ne!(input, index.contract().padding_id);
                }
            }
            let loss = model
                .weighted_loss(
                    &trimmed.inputs,
                    &trimmed.targets,
                    &trimmed.weights,
                    batch.batch,
                    trimmed.time,
                )
                .unwrap();
            let grads = loss.backward().unwrap();
            optimizer.update(&model, &grads, 0.01).unwrap();
        }
        let after = development(&model, &index, &panel, 4).unwrap();
        let nll = |report: &Value| report["response_mean_nll"].as_f64().unwrap();
        assert!(
            nll(&after) < 0.5 * nll(&before),
            "response NLL {} -> {}",
            nll(&before),
            nll(&after)
        );
        assert_eq!(after["per_source"].as_array().unwrap().len(), 2);
        assert_eq!(after["selected_responses"], 4);

        let requests = vec![Request {
            id: "r1".into(),
            category: "test".into(),
            user_turns: vec!["sky?".into(), "grass?".into()],
        }];
        let encoder = protocol.bind(&tokenizer).unwrap();
        let panel = reply_panel(
            &encoder,
            &protocol,
            &requests,
            model.config.context,
            8,
            &|ids| tokenizer.decode(ids),
            &mut |history, cap| greedy_reply(&model, history, cap, protocol.eos_id),
        )
        .unwrap();
        let turns = panel["rows"][0]["turns"].as_array().unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0]["reply"], "blue");
        assert_eq!(turns[0]["model_eos"], true);
        assert_eq!(turns[1]["reply"], "green");
        let history: Vec<u32> =
            serde_json::from_value(panel["rows"][0]["history_ids"].clone()).unwrap();
        assert_eq!(
            tokenizer.decode(&history),
            "<|bos|>User: sky?\nAssistant: blue<|eos|>\nUser: grass?\nAssistant: green<|eos|>"
        );
        let _ = fs::remove_dir_all(&directory);
    }
}
