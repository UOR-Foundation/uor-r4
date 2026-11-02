//! Lawful M-world v2 training-corpus writer for the oracle-memory comparison.
//!
//! The historical `examples/m-world.rs` remains source evidence and is not a
//! Cargo target after D29. This writer imports only the admitted M-world and
//! tokenizer modules. It emits the same prepared-store shape as
//! `m-world corpus world=v2`, without the historical evaluator, compiler or
//! model code:
//!
//! ```text
//! m-world-recall-corpus out=NEW_ROOT tokenizer=TOKENIZER.json \
//!   [world=v2|v2c] [conversations=5600] [seed=1] \
//!   [mqar_share=0.35] [copy_share=0.10] [relation_share=0.30] \
//!   [other_share=0.25] [closed_share=0.20] [recall=off|oracle] \
//!   [recall_at=reply|query] [context=384] [protocol=1|2] [exclude=PANEL.json]
//! ```
//!
//! `out/train/` is a `uor-r4-chat-corpus/v1` split. It can be passed directly
//! to `train-in-space` with its token and mask SHA-256 values, or composed with
//! the pinned chat corpus through `mix-chat-corpus`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::{normalized, Split};
use uor_r4_training::milestone_world_v2::{
    oracle_recall, recall_line, takes_recall, Conversation2, MWorld2, Mix, Turn2, Variant, CONTEXT,
    REVISION,
};

type Result<T> = std::result::Result<T, String>;

const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
const MASK_RULE: &str = "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.";
const TEMPLATE_RULE: &str = "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.";

const ALLOWED: &[&str] = &[
    "out",
    "tokenizer",
    "world",
    "conversations",
    "seed",
    "mqar_share",
    "copy_share",
    "relation_share",
    "other_share",
    "closed_share",
    "recall",
    "recall_at",
    "context",
    "protocol",
    "exclude",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Recall {
    Off,
    Oracle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecallAt {
    Reply,
    Query,
}

#[derive(Clone)]
struct Options {
    out: PathBuf,
    tokenizer: PathBuf,
    world: Variant,
    conversations: usize,
    seed: u64,
    mix: Mix,
    recall: Recall,
    recall_at: RecallAt,
    context: usize,
    protocol: u8,
    exclude: Option<PathBuf>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, found {argument:?}"))?;
            if !ALLOWED.contains(&key) {
                return Err(format!("unknown argument {key}="));
            }
            if pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("duplicate argument {key}="));
            }
        }
        let required = |key: &str| {
            pairs
                .get(key)
                .cloned()
                .ok_or_else(|| format!("missing {key}="))
        };
        let optional = |key: &str| pairs.get(key).map(String::as_str);
        let number = |key: &str, default: &str| -> Result<u64> {
            optional(key)
                .unwrap_or(default)
                .parse()
                .map_err(|_| format!("{key}= must be a nonnegative integer"))
        };
        let share = |key: &str, default: &str| -> Result<f64> {
            let value = optional(key)
                .unwrap_or(default)
                .parse::<f64>()
                .map_err(|_| format!("{key}= must be a number"))?;
            if !value.is_finite() || value < 0.0 || value > 1.0 {
                return Err(format!("{key}= must be in [0, 1]"));
            }
            Ok(value)
        };
        let world = match optional("world") {
            None | Some("v2") => Variant::V2,
            Some("v2c") => Variant::V2c,
            Some(other) => return Err(format!("unknown world={other}: v2 or v2c")),
        };
        let recall = match optional("recall") {
            None | Some("off") => Recall::Off,
            Some("oracle") => Recall::Oracle,
            Some(other) => return Err(format!("unknown recall={other}: off or oracle")),
        };
        let recall_at = match optional("recall_at") {
            None | Some("reply") => RecallAt::Reply,
            Some("query") => RecallAt::Query,
            Some(other) => return Err(format!("unknown recall_at={other}: reply or query")),
        };
        if recall == Recall::Off && optional("recall_at").is_some() {
            return Err("recall_at= needs recall=oracle".to_owned());
        }
        let protocol = number("protocol", "2")?;
        if !(1..=2).contains(&protocol) {
            return Err("protocol= must be 1 or 2".to_owned());
        }
        let context = usize::try_from(number("context", "384")?)
            .map_err(|_| "context= does not fit this machine".to_owned())?;
        if context < CONTEXT {
            return Err(format!("context= must be at least {CONTEXT}"));
        }
        let conversations = usize::try_from(number("conversations", "5600")?)
            .map_err(|_| "conversations= does not fit this machine".to_owned())?;
        if conversations == 0 {
            return Err("conversations= must be positive".to_owned());
        }
        let mix = Mix {
            mqar: share("mqar_share", "0.35")?,
            copy: share("copy_share", "0.10")?,
            relation: share("relation_share", "0.30")?,
            other: share("other_share", "0.25")?,
            closed: share("closed_share", "0.20")?,
        };
        if (mix.mqar + mix.copy + mix.relation + mix.other - 1.0).abs() > 1e-9 {
            return Err(
                "mqar_share, copy_share, relation_share and other_share must sum to 1".into(),
            );
        }
        Ok(Self {
            out: PathBuf::from(required("out")?),
            tokenizer: PathBuf::from(required("tokenizer")?),
            world,
            conversations,
            seed: number("seed", "1")?,
            mix,
            recall,
            recall_at,
            context,
            protocol: protocol as u8,
            exclude: optional("exclude").map(PathBuf::from),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Receipt {
    conversations: usize,
    turns: usize,
    tokens: usize,
    response_tokens: usize,
    recall_lines: usize,
    recall_value_lines: usize,
    recall_none_lines: usize,
    longest: usize,
}

fn load_tokenizer(path: &Path) -> Result<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path).map_err(|e| e.to_string())?)
        .ok_or_else(|| "tokenizer JSON is not a supported byte-level BPE".to_owned())
}

fn load_exclusions(path: &Path) -> Result<BTreeSet<String>> {
    let rows: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let rows = rows
        .as_array()
        .ok_or_else(|| format!("{}: panel must be a JSON array", path.display()))?;
    let mut turns = BTreeSet::new();
    for row in rows {
        let user_turns = row["user_turns"]
            .as_array()
            .ok_or_else(|| format!("{}: a row has no user_turns array", path.display()))?;
        for turn in user_turns {
            let turn = turn
                .as_str()
                .ok_or_else(|| format!("{}: a user turn is not a string", path.display()))?;
            turns.insert(normalized(turn));
        }
    }
    Ok(turns)
}

fn messages<'a>(turns: &'a [Turn2], lines: &'a [Option<String>], at: RecallAt) -> Vec<Message<'a>> {
    let mut messages = Vec::with_capacity(3 * turns.len());
    for (turn, line) in turns.iter().zip(lines) {
        let system = line.as_deref().map(|content| Message {
            role: "system",
            content,
        });
        if at == RecallAt::Query {
            messages.extend(system);
        }
        messages.push(Message {
            role: "user",
            content: &turn.user,
        });
        if at == RecallAt::Reply {
            messages.extend(system);
        }
        messages.push(Message {
            role: "assistant",
            content: &turn.reply,
        });
    }
    messages
}

fn lines_for(conversation: &Conversation2, recall: Recall) -> (Vec<Option<String>>, usize, usize) {
    let mut values = 0;
    let mut none = 0;
    let lines = conversation
        .turns
        .iter()
        .map(|turn| {
            if recall == Recall::Off || !takes_recall(turn) {
                return None;
            }
            let value = oracle_recall(turn);
            if value.is_some() {
                values += 1;
            } else {
                none += 1;
            }
            Some(recall_line(value))
        })
        .collect();
    (lines, values, none)
}

fn generate(options: &Options) -> Result<Receipt> {
    let tokenizer = load_tokenizer(&options.tokenizer)?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, options.protocol)
        .map_err(|e| format!("protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("protocol: {e}"))?;
    let count = |text: &str| tokenizer.encode(text).len();
    let mut world = MWorld2::new(&count, options.mix)
        .map_err(|e| e.to_string())?
        .with_variant(options.world);
    let mut rng = uor_r4_training::milestone_world::Rng::new(options.seed);
    let excluded = match &options.exclude {
        Some(path) => load_exclusions(path)?,
        None => BTreeSet::new(),
    };
    let vocab = u32::try_from(tokenizer.vocab_size())
        .map_err(|_| "tokenizer vocabulary does not fit the token store".to_owned())?;
    let mut token_ids = Vec::new();
    let mut response_mask = Vec::new();
    let mut rejected = 0usize;
    let mut turns = 0usize;
    let mut recall_lines = 0usize;
    let mut recall_value_lines = 0usize;
    let mut recall_none_lines = 0usize;
    let mut longest = 0usize;
    let mut by_kind: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut by_category: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();

    for index in 0..options.conversations {
        let conversation = world
            .conversation_excluding(&mut rng, Split::Train, &excluded, &mut rejected)
            .map_err(|e| e.to_string())?;
        let (lines, values, none) = lines_for(&conversation, options.recall);
        recall_lines += lines.iter().flatten().count();
        recall_value_lines += values;
        recall_none_lines += none;
        let encoded =
            encoder.encode_document(&messages(&conversation.turns, &lines, options.recall_at));
        if encoded.emitted_turns != conversation.turns.len() * 2 + lines.iter().flatten().count()
            || encoded.special_token_occurrences != 0
        {
            return Err(format!("M-world v2 conversation {index} did not encode"));
        }
        if encoded.tokens.len() > options.context {
            return Err(format!(
                "M-world v2 conversation {index} has {} tokens, over the {}-token context",
                encoded.tokens.len(),
                options.context
            ));
        }
        for &id in &encoded.tokens {
            if id >= vocab {
                return Err("an M-world token is outside the vocabulary".to_owned());
            }
            token_ids.push(id as u16);
        }
        let response = encoded.response_mask.iter().filter(|&&m| m == 1).count();
        response_mask.extend(&encoded.response_mask);
        longest = longest.max(encoded.tokens.len());
        let kind = by_kind
            .entry(format!("{:?}", conversation.kind))
            .or_default();
        *kind = (kind.0 + 1, kind.1 + encoded.tokens.len(), kind.2 + response);
        turns += conversation.turns.len();
        for turn in &conversation.turns {
            let category = by_category
                .entry(format!("{:?}", turn.category))
                .or_default();
            *category = (
                category.0 + 1,
                category.1 + count(&turn.user) + count(&turn.reply) + 1,
                category.2 + count(&turn.reply) + 1,
            );
        }
    }

    if options.recall == Recall::Oracle && recall_value_lines == 0 {
        return Err("recall=oracle emitted no value-bearing memory lines".to_owned());
    }

    let train = options.out.join("train");
    fs::create_dir_all(&train).map_err(|e| e.to_string())?;
    let tokens_path = train.join("tokens.u16");
    let mut writer =
        CorpusWriter::create(&tokens_path, vocab).map_err(|e| format!("token store: {e}"))?;
    writer
        .write_tokens(&token_ids)
        .map_err(|e| format!("token store: {e}"))?;
    let written = writer.finish().map_err(|e| format!("token store: {e}"))?;
    let mask_path = train.join("response_mask.u8");
    fs::write(&mask_path, &response_mask).map_err(|e| e.to_string())?;
    let response_tokens = response_mask.iter().filter(|&&mask| mask == 1).count();

    let tokenizer_sha256 =
        uor_r4_training::sha256_file(&options.tokenizer).map_err(|e| e.to_string())?;
    let table = |cells: &BTreeMap<String, (usize, usize, usize)>, unit: &str| {
        json!(cells
            .iter()
            .map(|(key, value)| (
                key,
                json!({unit: value.0, "tokens": value.1, "response_tokens": value.2})
            ))
            .collect::<BTreeMap<_, _>>())
    };
    let recall = (options.recall == Recall::Oracle).then(|| {
        json!({
            "mode": "oracle",
            "at": match options.recall_at {
                RecallAt::Reply => "reply",
                RecallAt::Query => "query",
            },
            "lines": recall_lines,
            "value_lines": recall_value_lines,
            "none_lines": recall_none_lines,
            "context": options.context,
            "rule": "one system turn per MQAR and relation query, \"Memory: {value}.\" or \"Memory: none.\"; the plain episode is the one MWorld2 meters",
        })
    });
    let excluded_record = options.exclude.as_ref().map(|path| {
        json!({
            "path": path,
            "sha256": uor_r4_training::sha256_file(path).ok(),
            "turns": excluded.len(),
            "rejected_draws": rejected,
        })
    });
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": MASK_SCHEMA,
        "split": "train",
        "mask_rule": MASK_RULE,
        "template_rule": TEMPLATE_RULE,
        "dialogue_protocol": protocol.schema,
        "tokenizer": {
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "unk_id": protocol.unk_id,
            "path": options.tokenizer,
            "sha256": tokenizer_sha256,
            "tokenizer_cid": protocol.tokenizer_cid,
            "vocab_size": tokenizer.vocab_size(),
        },
        "max_tokens": options.context,
        "inputs": [{"label": "m-world-v2.train", "path": "generated"}],
        "files": [{
            "label": "m-world-v2.train",
            "path": "generated: uor_r4_training::milestone_world_v2 (Split::Train)",
            "rows_total": options.conversations,
            "rows_used": options.conversations,
            "tokens": token_ids.len(),
            "response_tokens": response_tokens,
            "special_token_occurrences": 0,
        }],
        "drops": {
            "messages_skipped_empty": 0,
            "messages_skipped_unknown_role": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_malformed": 0,
            "rows_dropped_no_messages": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0,
        },
        "rows_used": options.conversations,
        "tokens": written,
        "response_tokens": response_tokens,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": response_mask.len(),
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        "composition": {
            "m_world": {
                "version": options.world.world_name(),
                "revision": REVISION,
                "world_digest": MWorld2::digest_for(options.world),
                "split": "train",
                "seed": options.seed,
                "mix": options.mix,
                "protocol": options.protocol,
                "conversations": options.conversations,
                "turns": turns,
                "tokens": token_ids.len(),
                "response_tokens": response_tokens,
                "episode_tokens_max": longest,
                "episodes_per_kind": table(&by_kind, "episodes"),
                "turns_per_category": table(&by_category, "turns"),
                "recall": recall,
                "exclude": excluded_record,
            }
        }
    });
    let manifest_path = train.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        options.out.join("corpus.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "uor-r4.m-world-recall-corpus/1",
            "executable_sha256": uor_r4_training::sha256_file(
                &std::env::current_exe().map_err(|e| e.to_string())?
            ).ok(),
            "train_manifest_sha256": uor_r4_training::sha256_file(&manifest_path)
                .map_err(|e| e.to_string())?,
            "receipt": {
                "conversations": options.conversations,
                "turns": turns,
                "tokens": written,
                "response_tokens": response_tokens,
                "recall_lines": recall_lines,
                "recall_value_lines": recall_value_lines,
                "recall_none_lines": recall_none_lines,
                "episode_tokens_max": longest,
            },
            "lawful_scope": "imports the admitted M-world v2, tokenizer and UORT writer only; no evaluator, compiler, model, probability operation or dense contraction",
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(Receipt {
        conversations: options.conversations,
        turns,
        tokens: written as usize,
        response_tokens,
        recall_lines,
        recall_value_lines,
        recall_none_lines,
        longest,
    })
}

fn execute(options: &Options) -> Result<Receipt> {
    report_output::claim(&options.out).map_err(|e| e.to_string())?;
    let result = generate(options);
    if let Err(error) = &result {
        fs::write(
            options.out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error})).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    report_output::seal(&options.out).map_err(|e| e.to_string())?;
    report_output::verify(&options.out).map_err(|e| e.to_string())?;
    result
}

fn run() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = Options::parse(&arguments)?;
    let receipt = execute(&options)?;
    println!(
        "{} conversations, {} turns, {} tokens, {} response tokens; recall lines {} ({} values, {} none); longest {}",
        receipt.conversations,
        receipt.turns,
        receipt.tokens,
        receipt.response_tokens,
        receipt.recall_lines,
        receipt.recall_value_lines,
        receipt.recall_none_lines,
        receipt.longest
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

    fn byte_tokenizer_json() -> Value {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, surface) in specials.iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let ch = if printable.contains(&byte) {
                char::from_u32(byte)
            } else {
                extra += 1;
                char::from_u32(255 + extra)
            }
            .expect("a valid char");
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": added,
            "model": {"type": "BPE", "vocab": vocab, "merges": []},
        })
    }

    fn byte_tokenizer() -> ByteBpeTokenizer {
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            serde_json::to_vec(&byte_tokenizer_json())
                .expect("JSON")
                .as_slice(),
        )
        .expect("the byte tokenizer parses")
    }

    fn byte_tokenizer_path() -> PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| {
            let path = std::env::temp_dir().join(format!(
                "m-world-recall-tokenizer-{}.json",
                std::process::id()
            ));
            fs::write(
                &path,
                serde_json::to_vec(&byte_tokenizer_json()).expect("JSON"),
            )
            .expect("tokenizer");
            path
        })
        .clone()
    }

    fn options(root: &Path, recall: Recall) -> Options {
        Options {
            out: root.to_owned(),
            tokenizer: byte_tokenizer_path(),
            world: Variant::V2,
            conversations: 24,
            seed: 73,
            mix: Mix {
                mqar: 0.0,
                copy: 0.0,
                relation: 0.75,
                other: 0.25,
                closed: 0.20,
            },
            recall,
            recall_at: RecallAt::Reply,
            context: 384,
            protocol: 2,
            exclude: None,
        }
    }

    fn clean(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "m-world-recall-corpus-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test root");
        root
    }

    fn unclaimed(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "m-world-recall-corpus-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn read(root: &Path) -> (Vec<u8>, Vec<u8>, Vec<u8>, String) {
        let train = root.join("train");
        let token_bytes = fs::read(train.join("tokens.u16")).expect("tokens");
        let mask_bytes = fs::read(train.join("response_mask.u8")).expect("mask");
        let manifest_bytes = fs::read(train.join("manifest.json")).expect("manifest");
        let reader = MmapCorpusReader::open(train.join("tokens.u16")).expect("token store");
        let ids: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        let tokenizer = byte_tokenizer();
        (
            token_bytes,
            mask_bytes,
            manifest_bytes,
            tokenizer.decode(&ids),
        )
    }

    #[test]
    fn oracle_lines_are_counted_and_off_is_a_matched_control() {
        let root = clean("oracle");
        let receipt = generate(&options(&root, Recall::Oracle)).expect("oracle corpus");
        assert_eq!(
            receipt,
            Receipt {
                conversations: 24,
                turns: 66,
                tokens: 4_793,
                response_tokens: 1_853,
                recall_lines: 10,
                recall_value_lines: 8,
                recall_none_lines: 2,
                longest: 281,
            }
        );
        assert!(receipt.recall_value_lines > 0);
        assert_eq!(
            receipt.recall_lines,
            receipt.recall_value_lines + receipt.recall_none_lines
        );
        let (_, mask, manifest, text) = read(&root);
        assert_eq!(mask.len(), receipt.tokens);
        assert_eq!(text.matches("Memory:").count(), receipt.recall_lines);
        assert!(text.contains("Memory:"));
        let manifest: Value = serde_json::from_slice(&manifest).expect("JSON");
        assert_eq!(
            manifest["composition"]["m_world"]["recall"]["value_lines"],
            json!(receipt.recall_value_lines)
        );
        let _ = fs::remove_dir_all(root);

        let root = clean("off");
        let control = generate(&options(&root, Recall::Off)).expect("control corpus");
        assert_eq!(
            (control.conversations, control.turns),
            (receipt.conversations, receipt.turns)
        );
        let (_, _, manifest, text) = read(&root);
        assert!(!text.contains("Memory:"));
        let manifest: Value = serde_json::from_slice(&manifest).expect("JSON");
        assert!(manifest["composition"]["m_world"]["recall"].is_null());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn oracle_corpora_are_byte_deterministic() {
        let first_root = clean("oracle-first");
        let second_root = clean("oracle-second");
        generate(&options(&first_root, Recall::Oracle)).expect("first corpus");
        generate(&options(&second_root, Recall::Oracle)).expect("second corpus");
        let first = read(&first_root);
        let second = read(&second_root);
        assert_eq!((first.0, first.1, first.2), (second.0, second.1, second.2));
        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }

    #[test]
    fn the_command_claims_seals_and_verifies_its_output_root() {
        let root = unclaimed("sealed");
        let mut args = options(&root, Recall::Off);
        args.out = root.clone();
        execute(&args).expect("sealed corpus");
        report_output::verify(&root).expect("verified root");
        assert!(root.join("manifest.json").is_file());
        assert!(root.join("train/manifest.json").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn arguments_are_strict_and_keep_off_as_the_default() {
        let parsed =
            Options::parse(&["out=test".to_owned(), "tokenizer=tokenizer.json".to_owned()])
                .expect("minimal arguments");
        assert_eq!(parsed.recall, Recall::Off);
        assert_eq!(parsed.protocol, 2);
        assert_eq!(parsed.context, 384);
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "recall=oracle".to_owned(),
            "recall_at=reply".to_owned(),
        ])
        .is_ok());
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "recall_at=reply".to_owned(),
        ])
        .is_err());
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "unknown=1".to_owned(),
        ])
        .is_err());
    }
}
