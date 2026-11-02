//! CPU evidence for distinguishing referenced TRAIN material from new model behaviour.
//!
//! This instrument evaluates the discrete integer artifact through the same
//! `IntegerDiscreteModel` used by native serving. It reports next-token
//! admission/top-1/top-8, checks the independent CPU writer on the serving
//! query, and stratifies panel rows by whole-run overlap with a supplied TRAIN
//! split. It does not call generated text "complete" and does not infer
//! generated-reply novelty when no generated-reply file is supplied.

use std::collections::BTreeMap;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

type Result<T> = std::result::Result<T, String>;

const SCHEMA: &str = "uor-r4.m2-evidence/1";
const OPENING_KEY_TOKENS: usize = 8;

#[derive(Clone, Debug, Serialize, Default, PartialEq, Eq)]
pub struct Reading {
    pub targets: u64,
    pub admitted: u64,
    pub top1: u64,
    pub top8: u64,
}

impl Reading {
    fn observe(&mut self, admitted: bool, top1: bool, top8: bool) {
        self.targets += 1;
        self.admitted += u64::from(admitted);
        self.top1 += u64::from(top1);
        self.top8 += u64::from(top8);
    }

    fn merge(&mut self, other: &Self) {
        self.targets += other.targets;
        self.admitted += other.admitted;
        self.top1 += other.top1;
        self.top8 += other.top8;
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub path: String,
    pub sha256: String,
    pub model_sha256: String,
    pub vocabulary: u32,
    pub context: u32,
    pub blocks: u32,
    pub kept_cells: u32,
    pub read_mode: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CorpusIdentity {
    pub path: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub documents: usize,
    pub tokens: usize,
    pub response_targets: usize,
    pub prompt_runs: usize,
    pub reply_runs: usize,
    pub max_tokens: usize,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReplyOverlap {
    pub reference_replies: u64,
    pub reference_replies_seen_in_train: u64,
    pub reference_prompts_seen_in_train: u64,
    pub reference_prompts_keyed_in_train: u64,
    pub reference_reply_new_4grams: u64,
    pub reference_reply_total_4grams: u64,
    pub reference_reply_new_8grams: u64,
    pub reference_reply_total_8grams: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct EvidenceReport {
    pub schema: String,
    pub artifact: ArtifactIdentity,
    pub panel: CorpusIdentity,
    pub train: Option<CorpusIdentity>,
    pub measured_targets: usize,
    pub overall: Reading,
    pub strata: BTreeMap<String, Reading>,
    pub reply_overlap: Option<ReplyOverlap>,
    pub unavailable: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct Run {
    ordinal: usize,
    document: usize,
    mask: u8,
    range: Range<usize>,
}

#[derive(Clone, Debug)]
struct Corpus {
    path: PathBuf,
    tokens: Vec<u32>,
    mask: Vec<u8>,
    documents: Vec<Range<usize>>,
    runs: Vec<Run>,
    tokens_sha256: String,
    mask_sha256: String,
    max_tokens: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Metadata {
    prompt_seen: Option<bool>,
    keyed: Option<bool>,
    reply_seen: Option<bool>,
    opening: bool,
}

#[derive(Clone, Debug)]
struct Target {
    prefix: Vec<u32>,
    target: u32,
    metadata: Metadata,
}

/// A compact index of variable-length token runs. Records point into one
/// contiguous arena and are sorted by the run's token sequence.
struct RunIndex {
    data: Vec<u32>,
    records: Vec<(usize, usize)>,
}

impl RunIndex {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            records: Vec::new(),
        }
    }

    fn push(&mut self, tokens: &[u32]) -> usize {
        let ordinal = self.records.len();
        let offset = self.data.len();
        self.data.extend_from_slice(tokens);
        self.records.push((offset, tokens.len()));
        ordinal
    }

    fn finish(&mut self) {
        let data = &self.data;
        self.records.sort_unstable_by(|left, right| {
            let left = &data[left.0..left.0 + left.1];
            let right = &data[right.0..right.0 + right.1];
            left.cmp(right)
        });
    }

    fn contains(&self, tokens: &[u32]) -> bool {
        self.records
            .binary_search_by(|(offset, len)| self.data[*offset..*offset + *len].cmp(tokens))
            .is_ok()
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[derive(Deserialize)]
struct ManifestToken {
    vocab_size: usize,
    bos_id: u32,
}

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    mask_schema: String,
    tokens: usize,
    tokens_bytes: usize,
    tokens_sha256: String,
    mask_bytes: usize,
    mask_sha256: String,
    response_tokens: usize,
    max_tokens: usize,
    tokenizer: ManifestToken,
}

fn parse_uort(token_bytes: &[u8], mask: &[u8]) -> Result<Vec<u32>> {
    if token_bytes.len() < 64 || &token_bytes[..4] != b"UORT" {
        return Err("tokens.u16 is not a canonical UORT file".to_owned());
    }
    let version = u32::from_le_bytes(
        token_bytes[4..8]
            .try_into()
            .map_err(|_| "UORT version differs".to_owned())?,
    );
    let count = u64::from_le_bytes(
        token_bytes[8..16]
            .try_into()
            .map_err(|_| "UORT count differs".to_owned())?,
    );
    let vocabulary = u32::from_le_bytes(
        token_bytes[16..20]
            .try_into()
            .map_err(|_| "UORT vocabulary differs".to_owned())?,
    );
    if version != 1
        || token_bytes[20..64].iter().any(|&byte| byte != 0)
        || usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .and_then(|bytes| bytes.checked_add(64))
            != Some(token_bytes.len())
        || usize::try_from(count).ok() != Some(mask.len())
        || vocabulary == 0
    {
        return Err("UORT header or payload length differs".to_owned());
    }
    Ok(token_bytes[64..]
        .chunks_exact(2)
        .map(|bytes| u32::from(u16::from_le_bytes([bytes[0], bytes[1]])))
        .collect())
}

fn read_corpus(path: &Path, context: usize, vocabulary: usize, bos: u32) -> Result<Corpus> {
    let token_bytes = fs::read(path.join("tokens.u16"))
        .map_err(|e| format!("{}: {e}", path.join("tokens.u16").display()))?;
    let mask = fs::read(path.join("response_mask.u8"))
        .map_err(|e| format!("{}: {e}", path.join("response_mask.u8").display()))?;
    let manifest = fs::read(path.join("manifest.json"))
        .map_err(|e| format!("{}: {e}", path.join("manifest.json").display()))?;
    let identity: Manifest =
        serde_json::from_slice(&manifest).map_err(|error| format!("manifest: {error}"))?;
    if identity.schema != "uor-r4-chat-corpus/v1"
        || identity.mask_schema != "uor-r4-response-mask/u8/v1"
        || identity.tokenizer.vocab_size != vocabulary
        || identity.tokenizer.bos_id != bos
        || identity.tokens_bytes != token_bytes.len()
        || identity.mask_bytes != mask.len()
        || identity.response_tokens != mask.iter().filter(|&&m| m == 1).count()
        || identity.tokens_sha256 != sha256(&token_bytes)
        || identity.mask_sha256 != sha256(&mask)
    {
        return Err(format!(
            "{}: corpus manifest identity or artifact vocabulary/BOS differs",
            path.display()
        ));
    }
    let tokens = parse_uort(&token_bytes, &mask)?;
    if identity.tokens != tokens.len()
        || identity.max_tokens < context
        || mask.iter().any(|&value| value > 1)
        || tokens.iter().any(|&token| token as usize >= vocabulary)
    {
        return Err("corpus manifest inventory differs from UORT payload".to_owned());
    }
    let starts: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, &token)| (token == bos).then_some(index))
        .collect();
    let documents: Vec<Range<usize>> = starts
        .iter()
        .enumerate()
        .map(|(index, &start)| start..starts.get(index + 1).copied().unwrap_or(tokens.len()))
        .collect();
    let runs = collect_runs(&mask, &documents);
    Ok(Corpus {
        path: path.to_path_buf(),
        tokens,
        mask,
        documents,
        runs,
        tokens_sha256: identity.tokens_sha256,
        mask_sha256: identity.mask_sha256,
        max_tokens: identity.max_tokens,
    })
}

fn collect_runs(mask: &[u8], documents: &[Range<usize>]) -> Vec<Run> {
    let mut runs = Vec::new();
    for (document, span) in documents.iter().enumerate() {
        let mut start = span.start;
        while start < span.end {
            let value = mask[start];
            let mut end = start + 1;
            while end < span.end && mask[end] == value {
                end += 1;
            }
            runs.push(Run {
                ordinal: runs.len(),
                document,
                mask: value,
                range: start..end,
            });
            start = end;
        }
    }
    runs
}

fn corpus_identity(corpus: &Corpus) -> CorpusIdentity {
    CorpusIdentity {
        path: corpus.path.display().to_string(),
        tokens_sha256: corpus.tokens_sha256.clone(),
        mask_sha256: corpus.mask_sha256.clone(),
        documents: corpus.documents.len(),
        tokens: corpus.tokens.len(),
        response_targets: corpus.mask.iter().filter(|&&m| m == 1).count(),
        prompt_runs: corpus.runs.iter().filter(|run| run.mask == 0).count(),
        reply_runs: corpus.runs.iter().filter(|run| run.mask == 1).count(),
        max_tokens: corpus.max_tokens,
    }
}

fn prompt_run_before(corpus: &Corpus, reply: usize) -> Option<&Run> {
    let reply = corpus.runs.get(reply)?;
    corpus.runs[..reply.ordinal]
        .iter()
        .rev()
        .find(|run| run.document == reply.document && run.mask == 0)
}

fn run_tokens<'a>(corpus: &'a Corpus, run: &Run) -> &'a [u32] {
    &corpus.tokens[run.range.clone()]
}

fn run_index(corpus: &Corpus, mask: u8) -> RunIndex {
    let mut index = RunIndex::new();
    for run in corpus.runs.iter().filter(|run| run.mask == mask) {
        index.push(run_tokens(corpus, run));
    }
    index.finish();
    index
}

fn metadata_for(
    panel: &Corpus,
    reply: &Run,
    prompts: Option<&RunIndex>,
    keys: Option<&RunIndex>,
    replies: Option<&RunIndex>,
) -> Metadata {
    let prompt = prompt_run_before(panel, reply.ordinal);
    let prompt_tokens = prompt.map(|run| run_tokens(panel, run));
    let key_len = prompt_tokens
        .map(|tokens| tokens.len().min(OPENING_KEY_TOKENS))
        .unwrap_or(0);
    Metadata {
        prompt_seen: prompts
            .zip(prompt_tokens)
            .map(|(index, tokens)| index.contains(tokens)),
        keyed: keys
            .zip(prompt_tokens)
            .map(|(index, tokens)| index.contains(&tokens[..key_len])),
        reply_seen: replies.map(|index| index.contains(run_tokens(panel, reply))),
        opening: true,
    }
}

fn targets_in(corpus: &Corpus, context: usize, reply_metadata: &[Metadata]) -> Vec<Target> {
    let mut targets = Vec::with_capacity(corpus.mask.iter().filter(|&&mask| mask == 1).count());
    for (reply_ordinal, run) in corpus.runs.iter().filter(|run| run.mask == 1).enumerate() {
        let document = &corpus.documents[run.document];
        for target_position in run.range.clone() {
            if corpus.tokens[target_position] == 0 {
                continue;
            }
            let mut metadata = reply_metadata
                .get(reply_ordinal)
                .copied()
                .unwrap_or_default();
            metadata.opening = target_position == run.range.start;
            let prefix_start = target_position.saturating_sub(context).max(document.start);
            targets.push(Target {
                prefix: corpus.tokens[prefix_start..target_position].to_vec(),
                target: corpus.tokens[target_position],
                metadata,
            });
        }
    }
    targets
}

fn evenly_cap<T: Clone>(values: Vec<T>, maximum: Option<usize>) -> Vec<T> {
    let Some(maximum) = maximum else {
        return values;
    };
    if values.len() <= maximum || maximum == 0 {
        return values.into_iter().take(maximum).collect();
    }
    (0..maximum)
        .map(|index| values[index * values.len() / maximum].clone())
        .collect()
}

fn add_stratum(strata: &mut BTreeMap<String, Reading>, name: impl Into<String>, reading: &Reading) {
    strata.entry(name.into()).or_default().merge(reading);
}

fn observe_targets<F>(
    artifact: &ArtifactIdentity,
    targets: &[Target],
    mut predict: F,
) -> Result<(Reading, BTreeMap<String, Reading>)>
where
    F: FnMut(&[u32], u32) -> Result<(bool, bool, bool)>,
{
    let mut overall = Reading::default();
    let mut strata = BTreeMap::new();
    for (index, sample) in targets.iter().enumerate() {
        let (admitted, top1, top8) = predict(&sample.prefix, sample.target)?;
        let mut row = Reading::default();
        row.observe(admitted, top1, top8);
        overall.merge(&row);
        add_stratum(
            &mut strata,
            if sample.metadata.opening {
                "reply-position:opening"
            } else {
                "reply-position:continuation"
            },
            &row,
        );
        for (name, value) in [
            ("prompt-overlap", sample.metadata.prompt_seen),
            ("memory-key", sample.metadata.keyed),
            ("reference-reply-overlap", sample.metadata.reply_seen),
        ] {
            let value = match value {
                Some(true) => "present",
                Some(false) => "absent",
                None => "unavailable",
            };
            add_stratum(&mut strata, format!("{name}:{value}"), &row);
        }
        if (index + 1) % 1000 == 0 {
            eprintln!(
                "{index} of {} targets evaluated for {}",
                targets.len(),
                artifact.sha256
            );
        }
    }
    Ok((overall, strata))
}

fn evaluate_model(
    artifact_bytes: &[u8],
    artifact: &ArtifactIdentity,
    corpus: &Corpus,
    train: Option<&Corpus>,
    max_targets: Option<usize>,
) -> Result<EvidenceReport> {
    let model = IntegerDiscreteModel::from_bytes(artifact_bytes).map_err(|e| e.to_string())?;
    let shape = model.shape();
    let prompt_index = train.map(|train| run_index(train, 0));
    let reply_index = train.map(|train| run_index(train, 1));
    let key_index = train.map(|train| {
        let mut index = RunIndex::new();
        for run in train.runs.iter().filter(|run| run.mask == 0) {
            let tokens = run_tokens(train, run);
            index.push(&tokens[..tokens.len().min(OPENING_KEY_TOKENS)]);
        }
        index.finish();
        index
    });

    let reply_runs: Vec<&Run> = corpus.runs.iter().filter(|run| run.mask == 1).collect();
    let metadata: Vec<Metadata> = reply_runs
        .iter()
        .map(|run| {
            metadata_for(
                corpus,
                run,
                prompt_index.as_ref(),
                key_index.as_ref(),
                reply_index.as_ref(),
            )
        })
        .collect();
    let targets = evenly_cap(
        targets_in(corpus, shape.context as usize, &metadata),
        max_targets,
    );
    let (overall, strata) = observe_targets(artifact, &targets, |prefix, target| {
        let evaluation = model.evaluate_prefix(prefix).map_err(|e| e.to_string())?;
        let admitted = evaluation
            .admitted_cells
            .contains(&model.cells()[target as usize]);
        Ok((
            admitted,
            evaluation.prediction == target,
            evaluation.top8.contains(&target),
        ))
    })?;

    let mut unavailable = BTreeMap::new();
    unavailable.insert(
        "generated_complete_reply".to_owned(),
        "requires a sealed/generated reply file; corpus rows are teacher-forced targets".to_owned(),
    );
    unavailable.insert(
        "generated_reply_novelty".to_owned(),
        "requires model-generated replies; the report measures reference and panel overlap only"
            .to_owned(),
    );
    unavailable.insert(
        "value_masked_prompt_template".to_owned(),
        "requires the generator's value spans or a precomputed masked-template census".to_owned(),
    );
    unavailable.insert(
        "independent_cpu_read".to_owned(),
        "this instrument calls the native integer serving engine directly; independent CPU-trunk parity belongs to the isolated CPU-crate lane"
            .to_owned(),
    );
    let reply_overlap = train.map(|train| reply_overlap(corpus, train));
    Ok(EvidenceReport {
        schema: SCHEMA.to_owned(),
        artifact: artifact.clone(),
        panel: corpus_identity(corpus),
        train: train.map(corpus_identity),
        measured_targets: targets.len(),
        overall,
        strata,
        reply_overlap,
        unavailable,
    })
}

/// Evaluate a panel. `artifact` is a discrete UORLUT01 file; `panel` has
/// canonical `tokens.u16`, `response_mask.u8` and `manifest.json`. The current
/// implementation deliberately leaves TRAIN overlap unavailable until the
/// `train` index is wired, instead of reporting it as zero.
pub fn evaluate(
    artifact_path: &Path,
    panel_path: &Path,
    train_path: Option<&Path>,
    max_targets: Option<usize>,
) -> Result<EvidenceReport> {
    let artifact_bytes =
        fs::read(artifact_path).map_err(|e| format!("{}: {e}", artifact_path.display()))?;
    let model = IntegerDiscreteModel::from_bytes(&artifact_bytes).map_err(|e| e.to_string())?;
    let shape = model.shape();
    let manifest = fs::read(panel_path.join("manifest.json"))
        .map_err(|e| format!("{}: {e}", panel_path.join("manifest.json").display()))?;
    let parsed: serde_json::Value =
        serde_json::from_slice(&manifest).map_err(|e| format!("manifest: {e}"))?;
    let bos = parsed["tokenizer"]["bos_id"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| "manifest tokenizer.bos_id differs".to_owned())?;
    let corpus = read_corpus(
        panel_path,
        shape.context as usize,
        shape.vocabulary as usize,
        bos,
    )?;
    let identity = ArtifactIdentity {
        path: artifact_path.display().to_string(),
        sha256: sha256(&artifact_bytes),
        model_sha256: model.artifact_sha256().to_owned(),
        vocabulary: shape.vocabulary,
        context: shape.context,
        blocks: shape.blocks,
        kept_cells: shape.kept_cells,
        read_mode: shape.read_mode.clone(),
    };

    let train = train_path
        .map(|path| read_corpus(path, shape.context as usize, shape.vocabulary as usize, bos))
        .transpose()?;
    let report = evaluate_model(
        &artifact_bytes,
        &identity,
        &corpus,
        train.as_ref(),
        max_targets,
    )?;
    Ok(report)
}

fn ngram_index(corpus: &Corpus, n: usize) -> RunIndex {
    let mut index = RunIndex::new();
    for run in corpus.runs.iter().filter(|run| run.mask == 1) {
        let tokens = run_tokens(corpus, run);
        for window in tokens.windows(n) {
            index.push(window);
        }
    }
    index.finish();
    index
}

fn reply_overlap(panel: &Corpus, train: &Corpus) -> ReplyOverlap {
    let prompts = run_index(train, 0);
    let replies = run_index(train, 1);
    let mut keys = RunIndex::new();
    for run in train.runs.iter().filter(|run| run.mask == 0) {
        let tokens = run_tokens(train, run);
        keys.push(&tokens[..tokens.len().min(OPENING_KEY_TOKENS)]);
    }
    keys.finish();
    let n4 = ngram_index(train, 4);
    let n8 = ngram_index(train, 8);
    let mut reading = ReplyOverlap {
        reference_replies: 0,
        reference_replies_seen_in_train: 0,
        reference_prompts_seen_in_train: 0,
        reference_prompts_keyed_in_train: 0,
        reference_reply_new_4grams: 0,
        reference_reply_total_4grams: 0,
        reference_reply_new_8grams: 0,
        reference_reply_total_8grams: 0,
    };
    for reply in panel.runs.iter().filter(|run| run.mask == 1) {
        let reply_tokens = run_tokens(panel, reply);
        reading.reference_replies += 1;
        reading.reference_replies_seen_in_train += u64::from(replies.contains(reply_tokens));
        if let Some(prompt) = prompt_run_before(panel, reply.ordinal) {
            let prompt_tokens = run_tokens(panel, prompt);
            reading.reference_prompts_seen_in_train += u64::from(prompts.contains(prompt_tokens));
            reading.reference_prompts_keyed_in_train += u64::from(
                keys.contains(&prompt_tokens[..prompt_tokens.len().min(OPENING_KEY_TOKENS)]),
            );
        }
        for window in reply_tokens.windows(4) {
            reading.reference_reply_total_4grams += 1;
            reading.reference_reply_new_4grams += u64::from(!n4.contains(window));
        }
        for window in reply_tokens.windows(8) {
            reading.reference_reply_total_8grams += 1;
            reading.reference_reply_new_8grams += u64::from(!n8.contains(window));
        }
    }
    reading
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus(tokens: &[u32], mask: &[u8]) -> Corpus {
        let documents: Vec<Range<usize>> = tokens
            .iter()
            .enumerate()
            .filter_map(|(index, &token)| (token == 0).then_some(index))
            .enumerate()
            .map(|(index, start)| {
                let starts: Vec<_> = tokens
                    .iter()
                    .enumerate()
                    .filter_map(|(index, &token)| (token == 0).then_some(index))
                    .collect();
                start..starts.get(index + 1).copied().unwrap_or(tokens.len())
            })
            .collect();
        let runs = collect_runs(mask, &documents);
        Corpus {
            path: PathBuf::from("fixture"),
            tokens: tokens.to_vec(),
            mask: mask.to_vec(),
            documents,
            runs,
            tokens_sha256: "00".repeat(32),
            mask_sha256: "11".repeat(32),
            max_tokens: 32,
        }
    }

    #[test]
    fn run_indices_are_exact_and_lexicographic() {
        let mut index = RunIndex::new();
        index.push(&[3, 4]);
        index.push(&[1, 2, 3]);
        index.push(&[1, 2]);
        index.finish();
        assert!(index.contains(&[1, 2]));
        assert!(index.contains(&[1, 2, 3]));
        assert!(index.contains(&[3, 4]));
        assert!(!index.contains(&[2]));
        assert!(!index.contains(&[3]));
    }

    #[test]
    fn reply_and_prompt_metadata_use_the_immediately_prior_prompt() {
        let panel = corpus(
            &[0, 10, 11, 12, 20, 21, 22, 0, 30, 31, 32],
            &[0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
        );
        let prompts = run_index(&panel, 0);
        let replies = run_index(&panel, 1);
        let reply = panel.runs.iter().find(|run| run.mask == 1).unwrap();
        let metadata = metadata_for(
            &panel,
            reply,
            Some(&prompts),
            Some(&prompts),
            Some(&replies),
        );
        assert_eq!(metadata.prompt_seen, Some(true));
        assert_eq!(metadata.keyed, Some(true));
        assert_eq!(metadata.reply_seen, Some(true));
    }

    #[test]
    fn target_cap_is_even_and_deterministic() {
        let values = (0..10).collect::<Vec<_>>();
        assert_eq!(evenly_cap(values.clone(), Some(4)), vec![0, 2, 5, 7]);
        assert_eq!(evenly_cap(values.clone(), Some(0)), Vec::<i32>::new());
        assert_eq!(evenly_cap(values, None).len(), 10);
    }

    #[test]
    fn reply_overlap_separates_seen_and_novel_windows() {
        let train = corpus(
            &[0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
            &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        );
        let panel = corpus(
            &[0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
            &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        );
        let reading = reply_overlap(&panel, &train);
        assert_eq!(reading.reference_replies_seen_in_train, 1);
        assert_eq!(reading.reference_reply_new_4grams, 0);
        assert_eq!(reading.reference_reply_new_8grams, 0);
        assert_eq!(reading.reference_prompts_keyed_in_train, 1);
    }
}
