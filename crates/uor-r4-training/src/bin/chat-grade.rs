//! The scale ladder's chat-quality instrument (#820): a fixed panel of
//! everyday requests, replies from any saved stack model, and grades from a
//! local offline teacher.
//!
//! ```text
//! chat-grade extract out=PANEL.json heldout=SPLIT_DIR tokenizer=T.json \
//!   [count=48] [max_words=24] [seed=1]
//! chat-grade grade out=NEW_REPORT_ROOT model=ROOT/model tokenizer=T.json \
//!   requests=PANEL.json[,MORE.json] [protocol=2] [max_new_tokens=64] \
//!   [grader=qwen2.5:1.5b] [ollama_url=http://127.0.0.1:11434]
//! chat-grade reply out=NEW_REPORT_ROOT model=ROOT/model tokenizer=T.json \
//!   requests=PANEL.json[,MORE.json] [protocol=2] [max_new_tokens=64]
//! chat-grade grade-replies out=NEW_REPORT_ROOT replies=REPLIES.json \
//!   [grader=qwen2.5:1.5b] [ollama_url=http://127.0.0.1:11434] [ill_posed=IDS.txt]
//! chat-grade check requests=PANEL.json[,MORE.json] tokenizer=T.json \
//!   [protocol=2] [context=384] [max_new_tokens=64] [exclude=IDS.txt]
//! chat-grade filter out=NEW_PANEL.json requests=PANEL.json[,MORE.json] \
//!   (exclude=IDS.txt | only=IDS.txt)
//! chat-grade tiers report=REPORT.json [ill_posed=IDS.txt] [out=NEW_REPORT_ROOT]
//! chat-grade compare a=REPORT.json b=REPORT.json [tier=all|C|K-clean|...] \
//!   [ill_posed=IDS.txt] [out=NEW_REPORT_ROOT]
//! ```
//!
//! `grade` and `reply` also take `exclude=IDS.txt` (drop those request ids
//! before answering) and `grade`/`grade-replies` take `ill_posed=IDS.txt`.
//!
//! **Tiers.** Every graded row belongs to one evaluation tier, from its id and
//! category alone: `C` (the conversational panel, ids `conv-*`), `K-clean` and
//! `K-ill-posed` (category `heldout_first_turn`, split by the ill-posed id list:
//! requests that reference material they do not contain), `stretch` (category
//! `stretch_heldout`) and `everyday` (every other row, i.e. everyday-32). The
//! ill-posed list defaults to `data/panels/heldout-ill-posed-ids.txt`, embedded
//! in the executable; `ill_posed=` overrides it. Graded reports append
//! `per_tier` (actual, control and paired-against-control per tier, and per
//! category within it). `tiers` recomputes that from an existing report's rows
//! without regrading; `compare` pairs two graded reports' rows by id within each
//! tier and gives the two-sided exact McNemar for acceptable, fluent and relevant.
//! `check` runs the panel's context/turn check against a tokenizer with no model;
//! `filter` writes a new panel file and never edits its inputs.
//!
//! `extract` decodes a prepared protocol-2 held-out chat split (documents from
//! BOS; the first `User:` turn up to the next newline) and keeps, by a seeded
//! shuffle, `count` single-turn requests of at most `max_words` words. The
//! held-out split was never trained on.
//!
//! `grade` answers every request greedily (`stack_dialogue::reply_panel`)
//! and asks the grader, a local Ollama model used as an offline judge only,
//! to score each reply's fluency and relevance from 1 to 5 against the
//! conversation so far. As a validity control it also grades each reply
//! against the *next* request's conversation (a derangement): relevance must
//! fall there, fluency need not. The report root is claimed before the model
//! loads and sealed at the end.
//!
//! `reply` writes the same greedy replies without a grader (`replies.json`:
//! the `reply_panel` record with each reply's generated ids, seconds and ids
//! per second), so a served integer artifact's replies (`geometric-stack
//! lut-chat`) can be compared with the float model's id for id.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::sha256_file;
use uor_r4_training::stack_dialogue::{
    annotate_turn_costs, check_panel, greedy_reply, load_requests, reply_panel, Request, TurnCost,
};
use uor_r4_training::stack_tracking::Rng;

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chat-grade: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], keys: &[&str]) -> Result<Self, Error> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got {argument}"))?;
            if !keys.contains(&key) || pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("unknown or repeated argument {key}").into());
            }
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String, Error> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| format!("missing {key}=").into())
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, Error> {
        match self.0.get(key) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("invalid {key}={v}").into()),
        }
    }
}

fn run() -> Result<(), Error> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("extract") => extract(&arguments[1..]),
        Some("grade") => grade(&arguments[1..]),
        Some("reply") => reply(&arguments[1..]),
        Some("grade-replies") => grade_replies(&arguments[1..]),
        Some("check") => check(&arguments[1..]),
        Some("filter") => filter(&arguments[1..]),
        Some("tiers") => tiers(&arguments[1..]),
        Some("compare") => compare(&arguments[1..]),
        _ => Err(
            "usage: chat-grade extract|grade|reply|grade-replies|check|filter|tiers|compare \
             key=value..."
                .into(),
        ),
    }
}

fn load_tokenizer(path: &Path) -> Result<ByteBpeTokenizer, Error> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
        .ok_or_else(|| format!("{} is not a supported tokenizer.json", path.display()).into())
}

/// The first user turn of a decoded protocol-2 document, if it has one.
fn first_user_turn(text: &str) -> Option<String> {
    let start = text.find("User:")? + "User:".len();
    let rest = &text[start..];
    let line = rest.split('\n').next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

fn extract(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(
        arguments,
        &["out", "heldout", "tokenizer", "count", "max_words", "seed"],
    )?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    let heldout = PathBuf::from(args.required("heldout")?);
    let tokenizer = load_tokenizer(Path::new(&args.required("tokenizer")?))?;
    let (count, max_words, seed): (usize, usize, u64) = (
        args.number("count", 48)?,
        args.number("max_words", 24)?,
        args.number("seed", 1)?,
    );
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
    let reader = MmapCorpusReader::open(heldout.join("tokens.u16"))?;
    let tokens = reader.as_slice();
    let mut candidates: Vec<String> = Vec::new();
    let mut start = None;
    for (i, &t) in tokens.iter().enumerate().chain([(tokens.len(), &0u16)]) {
        let boundary = i == tokens.len() || u32::from(t) == protocol.bos_id;
        if boundary {
            if let Some(s) = start {
                let ids: Vec<u32> = tokens[s..i]
                    .iter()
                    .map(|&id| u32::from(id))
                    .filter(|&id| id != protocol.eos_id)
                    .collect();
                if let Some(turn) = first_user_turn(&tokenizer.decode(&ids)) {
                    let words = turn.split_whitespace().count();
                    if (3..=max_words).contains(&words) && !candidates.contains(&turn) {
                        candidates.push(turn);
                    }
                }
            }
            start = (i < tokens.len()).then_some(i + 1);
        }
    }
    let available = candidates.len();
    let mut rng = Rng::new(seed);
    for i in (1..candidates.len()).rev() {
        candidates.swap(i, rng.below(i + 1));
    }
    candidates.truncate(count);
    let requests: Vec<Request> = candidates
        .into_iter()
        .enumerate()
        .map(|(i, turn)| Request {
            id: format!("heldout-{i:03}"),
            category: "heldout_first_turn".into(),
            user_turns: vec![turn],
        })
        .collect();
    fs::write(&out, serde_json::to_vec_pretty(&requests)?)?;
    println!(
        "{} requests from {available} eligible first turns; heldout tokens sha256 {}",
        requests.len(),
        sha256_file(&heldout.join("tokens.u16"))?
    );
    Ok(())
}

/// Grades from the local Ollama judge.
struct Grader {
    url: String,
    model: String,
}

impl Grader {
    fn ask(&self, prompt: &str) -> Result<String, Error> {
        use std::io::Write;
        let body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": prompt}],
            "stream": false,
            "options": {"temperature": 0.0, "seed": 1, "num_predict": 4},
        });
        let mut child = std::process::Command::new("curl")
            .args(["-sS", "--fail", "-H", "Content-Type: application/json"])
            .args(["--data-binary", "@-"])
            .arg(format!("{}/api/chat", self.url))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .ok_or("curl has no stdin")?
            .write_all(body.to_string().as_bytes())?;
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(format!("ollama failed: {}", output.status).into());
        }
        let reply: Value = serde_json::from_slice(&output.stdout)?;
        Ok(reply["message"]["content"]
            .as_str()
            .ok_or("ollama returned no message")?
            .to_owned())
    }

    fn digest(&self) -> Result<String, Error> {
        let output = std::process::Command::new("curl")
            .args(["-sS", "--fail"])
            .arg(format!("{}/api/tags", self.url))
            .output()?;
        let tags: Value = serde_json::from_slice(&output.stdout)?;
        tags["models"]
            .as_array()
            .and_then(|m| m.iter().find(|m| m["name"] == self.model.as_str()))
            .and_then(|m| m["digest"].as_str())
            .map(str::to_owned)
            .ok_or_else(|| format!("ollama has no model {}", self.model).into())
    }

    /// One yes/no judgement of the last reply of `text`, with worked
    /// examples on topics outside the panel. Returns the verdict (or `None`
    /// when the answer has neither word) and the raw answer.
    fn judge(
        &self,
        text: &str,
        question: &str,
        examples: &str,
    ) -> Result<(Option<bool>, String), Error> {
        let prompt = format!(
            "You check replies written by a small chat assistant. Answer with one word: yes or no.\n\n\
             {examples}\nNow this conversation:\n{text}\nQuestion: {question}\nAnswer:"
        );
        let answer = self.ask(&prompt)?;
        Ok((parse_yes_no(&answer), answer))
    }

    /// Fluency and relevance verdicts for the final reply of `conversation`,
    /// with the raw answers.
    fn grade(&self, conversation: &[(String, String)]) -> Result<Grades, Error> {
        let mut text = String::new();
        for (user, assistant) in conversation {
            text.push_str(&format!("User: {user}\nAssistant: {assistant}\n"));
        }
        let (fluent, fluent_raw) = self.judge(&text, FLUENT_QUESTION, FLUENT_EXAMPLES)?;
        let (relevant, relevant_raw) = self.judge(&text, RELEVANT_QUESTION, RELEVANT_EXAMPLES)?;
        Ok(Grades {
            fluent,
            relevant,
            raw: [fluent_raw, relevant_raw],
        })
    }
}

const FLUENT_QUESTION: &str =
    "Is the last Assistant reply written in clear, correct, sensible English sentences?";
const FLUENT_EXAMPLES: &str = "Example:\nUser: Do you like trains?\nAssistant: Yes, I think trains are fun to ride.\n\
     Question: Is the last Assistant reply written in clear, correct, sensible English sentences?\nAnswer: yes\n\n\
     Example:\nUser: Do you like trains?\nAssistant: The train train is of the a ride ride.\n\
     Question: Is the last Assistant reply written in clear, correct, sensible English sentences?\nAnswer: no\n";
const RELEVANT_QUESTION: &str =
    "Does the last Assistant reply respond sensibly to the user's last message?";
const RELEVANT_EXAMPLES: &str = "Example:\nUser: What color is grass?\nAssistant: Grass is usually green.\n\
     Question: Does the last Assistant reply respond sensibly to the user's last message?\nAnswer: yes\n\n\
     Example:\nUser: What color is grass?\nAssistant: I had pancakes for breakfast today.\n\
     Question: Does the last Assistant reply respond sensibly to the user's last message?\nAnswer: no\n";

/// Two yes/no verdicts and the grader's raw answers.
struct Grades {
    fluent: Option<bool>,
    relevant: Option<bool>,
    raw: [String; 2],
}

impl Grades {
    fn record(&self) -> Value {
        json!({"fluent": self.fluent, "relevant": self.relevant, "raw": self.raw})
    }
}

/// The verdict of a yes/no answer: its first word, `yes` or `no`.
fn parse_yes_no(answer: &str) -> Option<bool> {
    let first: String = answer
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase();
    match first.as_str() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

impl Grades {
    fn acceptable(&self) -> bool {
        self.fluent == Some(true) && self.relevant == Some(true)
    }
}

/// Paired verdicts of each request's actual reply and its derangement
/// control: the discordant counts and the two-sided exact McNemar test.
#[derive(Default)]
struct Paired {
    /// Actual yes, control no.
    actual_only: u64,
    /// Control yes, actual no.
    control_only: u64,
    both: u64,
    neither: u64,
}

impl Paired {
    fn add(&mut self, actual: bool, control: bool) {
        match (actual, control) {
            (true, false) => self.actual_only += 1,
            (false, true) => self.control_only += 1,
            (true, true) => self.both += 1,
            (false, false) => self.neither += 1,
        }
    }

    fn record(&self) -> Value {
        let p = mcnemar_exact(self.actual_only, self.control_only);
        json!({
            "actual_only": self.actual_only, "control_only": self.control_only,
            "both": self.both, "neither": self.neither,
            "mcnemar_exact_p": p,
            "discriminates": self.actual_only > self.control_only && p < 0.05,
        })
    }
}

/// Two-sided exact McNemar p-value for discordant counts `b` and `c`: twice
/// the binomial(b + c, 1/2) tail at min(b, c), capped at 1.
fn mcnemar_exact(b: u64, c: u64) -> f64 {
    let n = b + c;
    if n == 0 {
        return 1.0;
    }
    let k = b.min(c);
    // log C(n, i) - n log 2, summed in probability space.
    let mut log_choose = 0f64;
    let mut tail = 0f64;
    for i in 0..=k {
        if i > 0 {
            log_choose += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        tail += (log_choose - n as f64 * std::f64::consts::LN_2).exp();
    }
    (2.0 * tail).min(1.0)
}

#[derive(Default)]
struct Tally {
    replies: usize,
    fluent: usize,
    relevant: usize,
    acceptable: usize,
    unparsed: usize,
}

impl Tally {
    fn add(&mut self, grades: &Grades) {
        self.replies += 1;
        self.fluent += usize::from(grades.fluent == Some(true));
        self.relevant += usize::from(grades.relevant == Some(true));
        self.acceptable +=
            usize::from(grades.fluent == Some(true) && grades.relevant == Some(true));
        self.unparsed +=
            usize::from(grades.fluent.is_none()) + usize::from(grades.relevant.is_none());
    }

    fn record(&self) -> Value {
        json!({
            "replies": self.replies, "fluent": self.fluent, "relevant": self.relevant,
            "acceptable": self.acceptable, "unparsed_answers": self.unparsed,
            "acceptable_rule": "judged both fluent and relevant (an unparsed answer counts as no)",
        })
    }
}

fn grade(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &[
            "out",
            "model",
            "tokenizer",
            "requests",
            "protocol",
            "max_new_tokens",
            "grader",
            "ollama_url",
            "exclude",
            "ill_posed",
        ],
    )?;
    let out = PathBuf::from(args.required("out")?);
    let exclude = optional_id_list(&args, "exclude")?;
    let ill = ill_posed(&args)?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let request_paths: Vec<PathBuf> = args
        .required("requests")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let version: u8 = args.number("protocol", 2)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 64)?;
    let grader = Grader {
        url: args
            .0
            .get("ollama_url")
            .cloned()
            .unwrap_or_else(|| "http://127.0.0.1:11434".into()),
        model: args
            .0
            .get("grader")
            .cloned()
            .unwrap_or_else(|| "qwen2.5:1.5b".into()),
    };
    report_output::claim(&out)?;
    let result = grade_into(
        &out,
        &model_dir,
        &tokenizer_path,
        &request_paths,
        version,
        max_new_tokens,
        &grader,
        exclude.as_ref(),
        &ill,
        started,
    );
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

/// A panel answered greedily by a saved float model.
struct Answered {
    protocol: DialogueProtocol,
    requests: Vec<Request>,
    model: StackModel,
    /// The `reply_panel` record with each reply's cost.
    panel: Value,
    generation_seconds: f64,
}

/// Answer every request greedily (`reply_panel` over `greedy_reply`) and
/// record each reply's generated ids and seconds.
fn answer(
    model_dir: &Path,
    tokenizer_path: &Path,
    request_paths: &[PathBuf],
    version: u8,
    max_new_tokens: usize,
    exclude: Option<&IdList>,
) -> Result<Answered, Error> {
    let tokenizer = load_tokenizer(tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)?;
    let encoder = protocol.bind(&tokenizer)?;
    let requests = load_panels(request_paths, exclude)?.0;
    let model = StackModel::load(model_dir, &candle_core::Device::Cpu)?;
    let mut costs = Vec::new();
    let clock = Instant::now();
    let mut panel = reply_panel(
        &encoder,
        &protocol,
        &requests,
        model.config.context,
        max_new_tokens,
        &|ids| tokenizer.decode(ids),
        &mut |history, cap| {
            let started = Instant::now();
            let reply = greedy_reply(&model, history, cap, protocol.eos_id)?;
            costs.push(TurnCost {
                ids: reply.ids.len(),
                seconds: started.elapsed().as_secs_f64(),
            });
            Ok(reply)
        },
    )?;
    let generation_seconds = clock.elapsed().as_secs_f64();
    annotate_turn_costs(&mut panel, &costs)?;
    Ok(Answered {
        protocol,
        requests,
        model,
        panel,
        generation_seconds,
    })
}

/// `reply`: the greedy replies alone, with their costs; no grader.
fn reply(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &[
            "out",
            "model",
            "tokenizer",
            "requests",
            "protocol",
            "max_new_tokens",
            "exclude",
        ],
    )?;
    let out = PathBuf::from(args.required("out")?);
    let exclude = optional_id_list(&args, "exclude")?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let request_paths: Vec<PathBuf> = args
        .required("requests")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let version: u8 = args.number("protocol", 2)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 64)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<(), Error> {
        let answered = answer(
            &model_dir,
            &tokenizer_path,
            &request_paths,
            version,
            max_new_tokens,
            exclude.as_ref(),
        )?;
        let report = json!({
            "schema": "uor-r4.chat-grade-reply/1",
            "model": model_dir.display().to_string(),
            "model_sha256": sha256_file(&model_dir.join("model.safetensors")).ok(),
            "parameters": answered.model.parameter_count(),
            "tokenizer_sha256": sha256_file(&tokenizer_path)?,
            "protocol": answered.protocol.schema,
            "requests": request_paths.iter().map(|p| json!({"path": p.display().to_string(), "sha256": sha256_file(p).ok()})).collect::<Vec<_>>(),
            "max_new_tokens": max_new_tokens,
            "decoding": "greedy over the float model's next-token scores (a pointer model's mixture), ties to the lower id; each step recomputes the whole window",
            "panel": answered.panel,
            "generation_seconds": answered.generation_seconds,
            "excluded": exclude.as_ref().map(|e| e.source.clone()),
            "executable_sha256": sha256_file(&std::env::current_exe()?)?,
            "wall_seconds": started.elapsed().as_secs_f64(),
        });
        fs::write(
            out.join("replies.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!(
            "{} replies, {}",
            answered.requests.len(),
            report["panel"]["cost"]
        );
        Ok(())
    })();
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

#[allow(clippy::too_many_arguments)]
fn grade_into(
    out: &Path,
    model_dir: &Path,
    tokenizer_path: &Path,
    request_paths: &[PathBuf],
    version: u8,
    max_new_tokens: usize,
    grader: &Grader,
    exclude: Option<&IdList>,
    ill: &IdList,
    started: Instant,
) -> Result<(), Error> {
    let Answered {
        protocol,
        requests: _,
        model,
        panel,
        generation_seconds,
    } = answer(
        model_dir,
        tokenizer_path,
        request_paths,
        version,
        max_new_tokens,
        exclude,
    )?;
    let judged = judge_panel(&panel, grader)?;
    let per_tier = tier_summary(judged_rows(&judged)?, &ill.ids)?;
    let report = json!({
        "schema": "uor-r4.chat-grade/1",
        "model": model_dir.display().to_string(),
        "model_sha256": sha256_file(&model_dir.join("model.safetensors")).ok(),
        "parameters": model.parameter_count(),
        "tokenizer_sha256": sha256_file(tokenizer_path)?,
        "protocol": protocol.schema,
        "requests": request_paths.iter().map(|p| json!({"path": p.display().to_string(), "sha256": sha256_file(p).ok()})).collect::<Vec<_>>(),
        "max_new_tokens": max_new_tokens,
        "decoding": "greedy",
        "grader": {
            "engine": "ollama (local)", "model": grader.model, "digest": grader.digest()?,
            "role": "offline judge only; it never serves", "temperature": 0, "seed": 1,
            "questions": [FLUENT_QUESTION, RELEVANT_QUESTION],
        },
        "actual": judged["actual"],
        "control_derangement": judged["control_derangement"],
        "paired_against_control": judged["paired_against_control"],
        "control_rule": judged["control_rule"],
        "per_category": judged["per_category"],
        "rows": judged["rows"],
        "generation_seconds": generation_seconds,
        "excluded": exclude.map(|e| e.source.clone()),
        "per_tier": per_tier,
        "tiers": tiers_record(ill),
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    println!(
        "actual {} | control {}",
        report["actual"], report["control_derangement"]
    );
    Ok(())
}

/// Each panel row's id, category and conversation as (user, reply) pairs,
/// from a `reply_panel` record (`rows[].turns[].{user, reply}`).
fn panel_conversations(
    panel: &Value,
) -> Result<Vec<(String, String, Vec<(String, String)>)>, Error> {
    let rows = panel["rows"].as_array().ok_or("panel without rows")?;
    rows.iter()
        .map(|row| {
            let field = |key: &str| -> Result<String, Error> {
                Ok(row[key]
                    .as_str()
                    .ok_or_else(|| format!("a panel row without {key}"))?
                    .to_owned())
            };
            let turns = row["turns"]
                .as_array()
                .ok_or("a panel row without turns")?
                .iter()
                .map(|t| -> Result<(String, String), Error> {
                    Ok((
                        t["user"].as_str().ok_or("a turn without user")?.to_owned(),
                        t["reply"]
                            .as_str()
                            .ok_or("a turn without reply")?
                            .trim()
                            .to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            if turns.is_empty() {
                return Err("a panel row with no turns".into());
            }
            Ok((field("id")?, field("category")?, turns))
        })
        .collect()
}

/// Grade every conversation of a `reply_panel` record and its derangement
/// control (each reply graded as the answer to the next request's last
/// turn), with the paired exact McNemar of actual against control. Returns
/// the report fields `actual`, `control_derangement`,
/// `paired_against_control`, `control_rule`, `per_category` and `rows`.
fn judge_panel(panel: &Value, grader: &Grader) -> Result<Value, Error> {
    let conversations = panel_conversations(panel)?;
    let (mut actual, mut control) = (Tally::default(), Tally::default());
    let (mut acceptable_pairs, mut relevant_pairs) = (Paired::default(), Paired::default());
    let mut per_category: BTreeMap<String, Tally> = BTreeMap::new();
    let mut graded_rows = Vec::new();
    let n = conversations.len();
    for (i, (id, category, conversation)) in conversations.iter().enumerate() {
        let grades = grader.grade(conversation)?;
        actual.add(&grades);
        per_category
            .entry(category.clone())
            .or_default()
            .add(&grades);
        let mut swapped = conversations[(i + 1) % n].2.clone();
        let reply = conversation
            .last()
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        if let Some(last) = swapped.last_mut() {
            last.1 = reply;
        }
        let control_grades = grader.grade(&swapped)?;
        control.add(&control_grades);
        acceptable_pairs.add(grades.acceptable(), control_grades.acceptable());
        relevant_pairs.add(
            grades.relevant == Some(true),
            control_grades.relevant == Some(true),
        );
        graded_rows.push(json!({
            "id": id, "category": category,
            "conversation": conversation.iter().map(|(u, a)| json!({"user": u, "assistant": a})).collect::<Vec<_>>(),
            "grades": grades.record(),
            "control_grades": control_grades.record(),
        }));
    }
    Ok(json!({
        "actual": actual.record(),
        "control_derangement": control.record(),
        "paired_against_control": {
            "acceptable": acceptable_pairs.record(),
            "relevant": relevant_pairs.record(),
            "rule": "a chat reading counts only if the actual replies beat the derangement control on the same requests with a two-sided exact McNemar p < 0.05",
        },
        "control_rule": "each reply graded as the answer to the next request's last turn; relevance must fall for the grader to be measuring relevance",
        "per_category": per_category.iter().map(|(k, t)| (k.clone(), t.record())).collect::<BTreeMap<_, _>>(),
        "rows": graded_rows,
    }))
}

/// `grade-replies`: grade replies another tool already produced -- a
/// `chat-grade reply` `replies.json` (`panel`) or a `geometric-stack lut-chat`
/// `chat.json` (`record`, an integer engine's replies) -- with the same
/// grader, derangement control and paired test as `grade`, so a served
/// integer artifact and its float model are judged identically.
fn grade_replies(arguments: &[String]) -> Result<(), Error> {
    let started = Instant::now();
    let args = Args::parse(
        arguments,
        &["out", "replies", "grader", "ollama_url", "ill_posed"],
    )?;
    let out = PathBuf::from(args.required("out")?);
    let ill = ill_posed(&args)?;
    let replies_path = PathBuf::from(args.required("replies")?);
    let grader = Grader {
        url: args
            .0
            .get("ollama_url")
            .cloned()
            .unwrap_or_else(|| "http://127.0.0.1:11434".into()),
        model: args
            .0
            .get("grader")
            .cloned()
            .unwrap_or_else(|| "qwen2.5:1.5b".into()),
    };
    let bytes = fs::read(&replies_path)?;
    let source: Value = serde_json::from_slice(&bytes)?;
    let panel = if source.get("panel").is_some() {
        &source["panel"]
    } else if source.get("record").is_some() {
        &source["record"]
    } else {
        return Err("replies must hold a reply_panel record under `panel` or `record`".into());
    };
    report_output::claim(&out)?;
    let result = (|| -> Result<(), Error> {
        let judged = judge_panel(panel, &grader)?;
        let per_tier = tier_summary(judged_rows(&judged)?, &ill.ids)?;
        let report = json!({
            "schema": "uor-r4.chat-grade/1",
            "replies": {
                "path": replies_path.display().to_string(),
                "sha256": uor_r4_training::sha256_bytes(&bytes),
                "schema": source["schema"],
                "model": source.get("model").or_else(|| source.get("artifact")).cloned(),
                "engine": source.get("engine").cloned(),
            },
            "decoding": "as recorded in the replies file",
            "grader": {
                "engine": "ollama (local)", "model": grader.model, "digest": grader.digest()?,
                "role": "offline judge only; it never serves", "temperature": 0, "seed": 1,
                "questions": [FLUENT_QUESTION, RELEVANT_QUESTION],
            },
            "actual": judged["actual"],
            "control_derangement": judged["control_derangement"],
            "paired_against_control": judged["paired_against_control"],
            "control_rule": judged["control_rule"],
            "per_category": judged["per_category"],
            "rows": judged["rows"],
            "per_tier": per_tier,
            "tiers": tiers_record(&ill),
            "executable_sha256": sha256_file(&std::env::current_exe()?)?,
            "wall_seconds": started.elapsed().as_secs_f64(),
        });
        fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
        println!(
            "actual {} | control {}",
            report["actual"], report["control_derangement"]
        );
        Ok(())
    })();
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

/// The ill-posed heldout-200 ids (tier `K-ill-posed`), embedded so a graded
/// report's tiers are bound to the executable unless `ill_posed=` overrides.
const EMBEDDED_ILL_POSED: &str = include_str!("../../../../data/panels/heldout-ill-posed-ids.txt");
const EMBEDDED_ILL_POSED_PATH: &str = "data/panels/heldout-ill-posed-ids.txt";

const TIER_RULE: &str = "tier from id and category only: C = ids conv-*; K-clean / K-ill-posed = \
     category heldout_first_turn, split by the ill-posed id list; stretch = category \
     stretch_heldout; everyday = every other row. A row's control is the derangement grade of \
     its own reply, so per-tier control and paired counts use the same rows as the tier's actual";

/// The tier names `tier_of` can return.
const TIERS: [&str; 5] = ["C", "K-clean", "K-ill-posed", "stretch", "everyday"];

/// An id list and where it came from (path or embedded, with its sha256).
struct IdList {
    ids: BTreeSet<String>,
    source: Value,
}

/// An id list file: one id per line; blank lines and `#` comments ignored; a
/// repeated id is an error.
fn parse_id_list(text: &str) -> Result<BTreeSet<String>, Error> {
    let mut ids = BTreeSet::new();
    for line in text.lines() {
        let id = line.trim();
        if id.is_empty() || id.starts_with('#') {
            continue;
        }
        if !ids.insert(id.to_owned()) {
            return Err(format!("id list repeats {id}").into());
        }
    }
    Ok(ids)
}

fn load_id_list(path: &Path) -> Result<IdList, Error> {
    let bytes = fs::read(path)?;
    let ids = parse_id_list(std::str::from_utf8(&bytes)?)?;
    Ok(IdList {
        source: json!({
            "path": path.display().to_string(),
            "sha256": uor_r4_training::sha256_bytes(&bytes),
            "ids": ids.len(),
        }),
        ids,
    })
}

fn optional_id_list(args: &Args, key: &str) -> Result<Option<IdList>, Error> {
    args.0
        .get(key)
        .map(|path| load_id_list(Path::new(path)))
        .transpose()
}

/// `ill_posed=` if given, else the embedded list.
fn ill_posed(args: &Args) -> Result<IdList, Error> {
    if let Some(list) = optional_id_list(args, "ill_posed")? {
        return Ok(list);
    }
    let ids = parse_id_list(EMBEDDED_ILL_POSED)?;
    Ok(IdList {
        source: json!({
            "embedded": EMBEDDED_ILL_POSED_PATH,
            "sha256": uor_r4_training::sha256_bytes(EMBEDDED_ILL_POSED.as_bytes()),
            "ids": ids.len(),
        }),
        ids,
    })
}

fn tiers_record(ill: &IdList) -> Value {
    json!({"rule": TIER_RULE, "ill_posed": ill.source})
}

/// The evaluation tier of a panel row (see [`TIER_RULE`]).
fn tier_of(id: &str, category: &str, ill: &BTreeSet<String>) -> &'static str {
    if id.starts_with("conv-") {
        "C"
    } else if category == "heldout_first_turn" {
        if ill.contains(id) {
            "K-ill-posed"
        } else {
            "K-clean"
        }
    } else if category == "stretch_heldout" {
        "stretch"
    } else {
        "everyday"
    }
}

/// Keep only (`keep = true`) or drop (`keep = false`) the listed request ids.
/// Returns the remaining requests and how many listed ids were present.
fn filter_requests(
    requests: Vec<Request>,
    ids: &BTreeSet<String>,
    keep: bool,
) -> (Vec<Request>, usize) {
    let matched = requests.iter().filter(|r| ids.contains(&r.id)).count();
    let kept = requests
        .into_iter()
        .filter(|r| ids.contains(&r.id) == keep)
        .collect();
    (kept, matched)
}

/// Load panel files (each within `load_requests`' limits), reject an id that
/// repeats across files, and drop `exclude`'s ids. Returns the requests and
/// the number of excluded ids that were present.
fn load_panels(
    paths: &[PathBuf],
    exclude: Option<&IdList>,
) -> Result<(Vec<Request>, usize), Error> {
    let mut requests: Vec<Request> = Vec::new();
    for path in paths {
        requests.extend(load_requests(path)?);
    }
    let distinct: BTreeSet<&str> = requests.iter().map(|r| r.id.as_str()).collect();
    if distinct.len() != requests.len() {
        return Err("a request id repeats across the panel files".into());
    }
    let (requests, excluded) = match exclude {
        Some(list) => filter_requests(requests, &list.ids, false),
        None => (requests, 0),
    };
    if requests.is_empty() {
        return Err("no requests remain after the exclusion".into());
    }
    Ok((requests, excluded))
}

fn split_paths(value: &str) -> Vec<PathBuf> {
    value.split(',').map(PathBuf::from).collect()
}

/// `check`: the panel's turn and context check (`check_panel`) against a
/// tokenizer, with no model; prints the worst-case history.
fn check(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(
        arguments,
        &[
            "requests",
            "tokenizer",
            "protocol",
            "context",
            "max_new_tokens",
            "exclude",
        ],
    )?;
    let paths = split_paths(&args.required("requests")?);
    let exclude = optional_id_list(&args, "exclude")?;
    let tokenizer = load_tokenizer(Path::new(&args.required("tokenizer")?))?;
    let protocol =
        DialogueProtocol::literal_roles_version(&tokenizer, args.number("protocol", 2)?)?;
    let encoder = protocol.bind(&tokenizer)?;
    let (context, max_new_tokens): (usize, usize) = (
        args.number("context", 384)?,
        args.number("max_new_tokens", 64)?,
    );
    let (requests, excluded) = load_panels(&paths, exclude.as_ref())?;
    check_panel(&encoder, &requests, context, max_new_tokens)?;
    let mut worst = (0usize, String::new());
    let mut categories: BTreeMap<&str, usize> = BTreeMap::new();
    for request in &requests {
        *categories.entry(request.category.as_str()).or_default() += 1;
        let mut longest = 1usize;
        for (turn, user) in request.user_turns.iter().enumerate() {
            longest += encoder.encode_user_prefix(user, turn != 0).tokens.len() + max_new_tokens;
            if turn + 1 < request.user_turns.len() {
                longest += 1;
            }
        }
        if longest > worst.0 {
            worst = (longest, request.id.clone());
        }
    }
    println!(
        "{}",
        json!({
            "requests": requests.len(), "excluded": excluded,
            "multi_turn": requests.iter().filter(|r| r.user_turns.len() > 1).count(),
            "categories": categories, "context": context, "max_new_tokens": max_new_tokens,
            "worst_case_history": {"positions": worst.0, "id": worst.1},
            "check_panel": "pass",
        })
    );
    Ok(())
}

/// `filter`: write a new panel file holding the requests of `requests` minus
/// `exclude` (or only `only`). The inputs are never modified.
fn filter(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["out", "requests", "exclude", "only"])?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    let paths = split_paths(&args.required("requests")?);
    let (list, keep) = match (
        optional_id_list(&args, "exclude")?,
        optional_id_list(&args, "only")?,
    ) {
        (Some(list), None) => (list, false),
        (None, Some(list)) => (list, true),
        _ => return Err("give exactly one of exclude= or only=".into()),
    };
    let (requests, matched) = filter_requests(load_panels(&paths, None)?.0, &list.ids, keep);
    if requests.is_empty() {
        return Err("the filter leaves no requests".into());
    }
    fs::write(&out, serde_json::to_vec_pretty(&requests)?)?;
    println!(
        "{} requests written ({} of {} listed ids present); {} sha256 {}",
        requests.len(),
        matched,
        list.ids.len(),
        out.display(),
        sha256_file(&out)?
    );
    Ok(())
}

/// The graded rows of a `judge_panel` record or a report.
fn judged_rows(record: &Value) -> Result<&[Value], Error> {
    Ok(record["rows"]
        .as_array()
        .ok_or("a graded record without rows")?)
}

/// The fluent/relevant verdicts of a graded row's `grades` or `control_grades`.
fn row_grades(record: &Value) -> Result<Grades, Error> {
    let verdict = |key: &str| -> Result<Option<bool>, Error> {
        match record.get(key) {
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(Value::Null) => Ok(None),
            _ => Err(format!("a graded row without a {key} verdict").into()),
        }
    };
    Ok(Grades {
        fluent: verdict("fluent")?,
        relevant: verdict("relevant")?,
        raw: [String::new(), String::new()],
    })
}

fn row_field<'a>(row: &'a Value, key: &str) -> Result<&'a str, Error> {
    row[key]
        .as_str()
        .ok_or_else(|| format!("a graded row without {key}").into())
}

#[derive(Default)]
struct TierAccumulator {
    actual: Tally,
    control: Tally,
    acceptable: Paired,
    relevant: Paired,
    per_category: BTreeMap<String, Tally>,
}

/// Per tier: the actual and control tallies, the paired-against-control
/// counts (acceptable and relevant, exact McNemar) and the per-category
/// tallies, from graded rows.
fn tier_summary(rows: &[Value], ill: &BTreeSet<String>) -> Result<Value, Error> {
    let mut tiers: BTreeMap<&'static str, TierAccumulator> = BTreeMap::new();
    for row in rows {
        let (id, category) = (row_field(row, "id")?, row_field(row, "category")?);
        let grades = row_grades(&row["grades"])?;
        let control = row_grades(&row["control_grades"])?;
        let tier = tiers.entry(tier_of(id, category, ill)).or_default();
        tier.actual.add(&grades);
        tier.control.add(&control);
        tier.acceptable
            .add(grades.acceptable(), control.acceptable());
        tier.relevant.add(
            grades.relevant == Some(true),
            control.relevant == Some(true),
        );
        tier.per_category
            .entry(category.to_owned())
            .or_default()
            .add(&grades);
    }
    let mut record = serde_json::Map::new();
    for (name, t) in tiers {
        let per_category: BTreeMap<String, Value> = t
            .per_category
            .iter()
            .map(|(k, v)| (k.clone(), v.record()))
            .collect();
        record.insert(
            name.to_owned(),
            json!({
                "actual": t.actual.record(),
                "control_derangement": t.control.record(),
                "paired_against_control": {
                    "acceptable": t.acceptable.record(),
                    "relevant": t.relevant.record(),
                },
                "per_category": per_category,
            }),
        );
    }
    Ok(Value::Object(record))
}

fn read_report(path: &Path) -> Result<(Value, String), Error> {
    let bytes = fs::read(path)?;
    let report: Value = serde_json::from_slice(&bytes)?;
    if report["schema"] != "uor-r4.chat-grade/1" {
        return Err(format!("{} is not a chat-grade report", path.display()).into());
    }
    Ok((report, uor_r4_training::sha256_bytes(&bytes)))
}

/// Write `name` into a newly claimed, then sealed and verified, report root.
fn write_sealed(out: &Path, name: &str, record: &Value) -> Result<(), Error> {
    report_output::claim(out)?;
    let written = fs::write(out.join(name), serde_json::to_vec_pretty(record)?);
    report_output::seal(out)?;
    report_output::verify(out)?;
    Ok(written?)
}

/// `tiers`: the per-tier summary of an existing graded report, without
/// regrading.
fn tiers(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["report", "ill_posed", "out"])?;
    let path = PathBuf::from(args.required("report")?);
    let ill = ill_posed(&args)?;
    let (report, sha256) = read_report(&path)?;
    let record = json!({
        "schema": "uor-r4.chat-grade-tiers/1",
        "report": {"path": path.display().to_string(), "sha256": sha256, "grader": report["grader"]},
        "tiers": tiers_record(&ill),
        "per_tier": tier_summary(judged_rows(&report)?, &ill.ids)?,
    });
    if let Some(out) = args.0.get("out") {
        write_sealed(Path::new(out), "tiers.json", &record)?;
    }
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

/// Paired verdicts of report `a` against report `b` on the same requests
/// (`Paired::actual_only` is `a` only, `control_only` is `b` only).
#[derive(Default)]
struct Compared {
    acceptable: Paired,
    fluent: Paired,
    relevant: Paired,
    /// Per category: acceptable in `a`, acceptable in `b`, rows.
    per_category: BTreeMap<String, [u64; 3]>,
}

impl Compared {
    fn add(&mut self, category: &str, a: &Grades, b: &Grades) {
        self.acceptable.add(a.acceptable(), b.acceptable());
        self.fluent
            .add(a.fluent == Some(true), b.fluent == Some(true));
        self.relevant
            .add(a.relevant == Some(true), b.relevant == Some(true));
        let entry = self.per_category.entry(category.to_owned()).or_default();
        entry[0] += u64::from(a.acceptable());
        entry[1] += u64::from(b.acceptable());
        entry[2] += 1;
    }

    fn rows(&self) -> u64 {
        let p = &self.acceptable;
        p.actual_only + p.control_only + p.both + p.neither
    }

    fn record(&self) -> Value {
        let pair = |p: &Paired| {
            json!({
                "rows": p.actual_only + p.control_only + p.both + p.neither,
                "a": p.actual_only + p.both, "b": p.control_only + p.both,
                "a_only": p.actual_only, "b_only": p.control_only,
                "both": p.both, "neither": p.neither,
                "mcnemar_exact_p": mcnemar_exact(p.actual_only, p.control_only),
            })
        };
        let per_category: BTreeMap<String, Value> = self
            .per_category
            .iter()
            .map(|(k, [a, b, n])| (k.clone(), json!({"a": a, "b": b, "rows": n})))
            .collect();
        json!({
            "acceptable": pair(&self.acceptable),
            "fluent": pair(&self.fluent),
            "relevant": pair(&self.relevant),
            "per_category_acceptable": per_category,
        })
    }
}

/// The user turns of a graded row (its conversation's `user` fields).
fn row_users(row: &Value) -> Result<Vec<&str>, Error> {
    row["conversation"]
        .as_array()
        .ok_or("a graded row without conversation")?
        .iter()
        .map(|t| {
            t["user"]
                .as_str()
                .ok_or_else(|| "a turn without user".into())
        })
        .collect()
}

fn index_rows(rows: &[Value]) -> Result<BTreeMap<&str, &Value>, Error> {
    let mut index = BTreeMap::new();
    for row in rows {
        let id = row_field(row, "id")?;
        if index.insert(id, row).is_some() {
            return Err(format!("a report repeats row {id}").into());
        }
    }
    Ok(index)
}

/// Pair the rows of two graded reports by id within each tier (or only
/// `tier`): an id in both must carry the same category and user turns. Rows in
/// only one report are counted as unpaired and left out of the test.
fn compare_rows(
    a: &[Value],
    b: &[Value],
    ill: &BTreeSet<String>,
    tier: Option<&str>,
) -> Result<Value, Error> {
    let (a, b) = (index_rows(a)?, index_rows(b)?);
    let mut tiers: BTreeMap<&'static str, Compared> = BTreeMap::new();
    let mut all = Compared::default();
    let mut unpaired: BTreeMap<&'static str, [u64; 2]> = BTreeMap::new();
    for (side, rows, other) in [(0usize, &a, &b), (1, &b, &a)] {
        for (id, row) in rows {
            let category = row_field(row, "category")?;
            let name = tier_of(id, category, ill);
            if tier.is_some_and(|t| t != name) {
                continue;
            }
            let Some(other_row) = other.get(id) else {
                unpaired.entry(name).or_default()[side] += 1;
                continue;
            };
            if side == 1 {
                continue;
            }
            if row_field(other_row, "category")? != category
                || row_users(row)? != row_users(other_row)?
            {
                return Err(format!(
                    "row {id} has a different category or user turns in the two reports"
                )
                .into());
            }
            let (ga, gb) = (
                row_grades(&row["grades"])?,
                row_grades(&other_row["grades"])?,
            );
            tiers.entry(name).or_default().add(category, &ga, &gb);
            all.add(category, &ga, &gb);
        }
    }
    if all.rows() == 0 {
        return Err("the two reports share no rows in the selected tier".into());
    }
    let per_tier: BTreeMap<String, Value> = tiers
        .iter()
        .map(|(k, c)| (k.to_string(), c.record()))
        .collect();
    let unpaired: BTreeMap<String, Value> = unpaired
        .iter()
        .map(|(k, [x, y])| (k.to_string(), json!({"a_rows": x, "b_rows": y})))
        .collect();
    Ok(json!({
        "per_tier": per_tier,
        "all_selected": all.record(),
        "unpaired": unpaired,
        "rule": "rows paired by id within a tier; a_only = acceptable (or fluent, relevant) in a \
                 but not in b; two-sided exact McNemar on the discordant counts; rows present in \
                 only one report are excluded and counted under unpaired",
    }))
}

/// `compare`: paired McNemar of two graded reports, per tier.
fn compare(arguments: &[String]) -> Result<(), Error> {
    let args = Args::parse(arguments, &["a", "b", "tier", "ill_posed", "out"])?;
    let path_a = PathBuf::from(args.required("a")?);
    let path_b = PathBuf::from(args.required("b")?);
    let ill = ill_posed(&args)?;
    let tier = args
        .0
        .get("tier")
        .map(String::as_str)
        .filter(|t| *t != "all");
    if let Some(t) = tier {
        if !TIERS.contains(&t) {
            return Err(format!("unknown tier {t}; one of all, {}", TIERS.join(", ")).into());
        }
    }
    let (a, sha_a) = read_report(&path_a)?;
    let (b, sha_b) = read_report(&path_b)?;
    let compared = compare_rows(judged_rows(&a)?, judged_rows(&b)?, &ill.ids, tier)?;
    let same_grader = a["grader"]["model"] == b["grader"]["model"]
        && a["grader"]["digest"] == b["grader"]["digest"];
    let source = |r: &Value| r.get("model").or_else(|| r.get("replies")).cloned();
    let record = json!({
        "schema": "uor-r4.chat-grade-compare/1",
        "a": {"path": path_a.display().to_string(), "sha256": sha_a, "grader": a["grader"], "source": source(&a)},
        "b": {"path": path_b.display().to_string(), "sha256": sha_b, "grader": b["grader"], "source": source(&b)},
        "same_grader": same_grader,
        "tier": tier.unwrap_or("all"),
        "tiers": tiers_record(&ill),
        "compared": compared,
    });
    if !same_grader {
        eprintln!("chat-grade: warning: the two reports were graded by different graders");
    }
    if let Some(out) = args.0.get("out") {
        write_sealed(Path::new(out), "compare.json", &record)?;
    }
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_and_turns_parse() {
        assert_eq!(parse_yes_no("Yes."), Some(true));
        assert_eq!(parse_yes_no(" no, it does not"), Some(false));
        assert_eq!(parse_yes_no("Maybe"), None);
        // Exact McNemar: 8 vs 0 discordant gives 2 / 2^8; balanced gives 1.
        assert!((mcnemar_exact(8, 0) - 2.0 / 256.0).abs() < 1e-12);
        assert!((mcnemar_exact(3, 3) - 1.0).abs() < 1e-12);
        assert!((mcnemar_exact(0, 0) - 1.0).abs() < 1e-12);
        assert!(mcnemar_exact(10, 2) < 0.05 && mcnemar_exact(6, 2) > 0.05);
        assert_eq!(
            first_user_turn("User: Hello there\nAssistant: Hi"),
            Some("Hello there".into())
        );
        assert_eq!(first_user_turn("Assistant: Hi"), None);
    }

    fn request(id: &str, category: &str) -> Request {
        Request {
            id: id.into(),
            category: category.into(),
            user_turns: vec![format!("turn of {id}")],
        }
    }

    /// A graded row: `[fluent, relevant]` for the reply and its control.
    fn row(id: &str, category: &str, actual: [bool; 2], control: [bool; 2]) -> Value {
        json!({
            "id": id, "category": category,
            "conversation": [{"user": format!("turn of {id}"), "assistant": "a reply"}],
            "grades": {"fluent": actual[0], "relevant": actual[1], "raw": ["", ""]},
            "control_grades": {"fluent": control[0], "relevant": control[1], "raw": ["", ""]},
        })
    }

    #[test]
    fn id_lists_filter_and_tiers() {
        let ids = parse_id_list("# comment\nheldout-002\n\n  heldout-005  \n").unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("heldout-005"));
        assert!(parse_id_list("a\na\n").is_err());
        let embedded = parse_id_list(EMBEDDED_ILL_POSED).unwrap();
        assert_eq!(embedded.len(), 71);
        assert!(embedded.contains("heldout-008") && !embedded.contains("heldout-000"));

        let panel = vec![
            request("talk-01", "smalltalk"),
            request("heldout-002", "heldout_first_turn"),
            request("heldout-003", "heldout_first_turn"),
            request("heldout-005", "heldout_first_turn"),
        ];
        let (kept, matched) = filter_requests(panel.clone(), &ids, false);
        assert_eq!(matched, 2);
        let kept: Vec<&str> = kept.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(kept, ["talk-01", "heldout-003"]);
        let (only, _) = filter_requests(panel, &ids, true);
        assert_eq!(only.len(), 2);

        assert_eq!(tier_of("conv-mem-01", "multi_turn_memory", &ids), "C");
        assert_eq!(
            tier_of("heldout-002", "heldout_first_turn", &ids),
            "K-ill-posed"
        );
        assert_eq!(
            tier_of("heldout-003", "heldout_first_turn", &ids),
            "K-clean"
        );
        assert_eq!(tier_of("stretch-001", "stretch_heldout", &ids), "stretch");
        assert_eq!(tier_of("follow-01", "follow_up", &ids), "everyday");
    }

    #[test]
    fn tier_summary_counts_per_tier_and_control() {
        let ill: BTreeSet<String> = ["heldout-009".to_owned()].into();
        let rows = vec![
            row(
                "conv-talk-01",
                "smalltalk_feelings",
                [true, true],
                [true, false],
            ),
            row(
                "conv-mem-01",
                "multi_turn_memory",
                [true, true],
                [true, false],
            ),
            row(
                "conv-do-01",
                "self_contained_instruction",
                [true, false],
                [true, true],
            ),
            row(
                "heldout-001",
                "heldout_first_turn",
                [true, true],
                [true, true],
            ),
            row(
                "heldout-009",
                "heldout_first_turn",
                [false, false],
                [true, false],
            ),
            row("talk-01", "smalltalk", [true, true], [true, false]),
        ];
        let summary = tier_summary(&rows, &ill).unwrap();
        let c = &summary["C"];
        assert_eq!(c["actual"]["replies"], 3);
        assert_eq!(c["actual"]["acceptable"], 2);
        assert_eq!(c["actual"]["relevant"], 2);
        assert_eq!(c["control_derangement"]["acceptable"], 1);
        let paired = &c["paired_against_control"]["acceptable"];
        assert_eq!(paired["actual_only"], 2);
        assert_eq!(paired["control_only"], 1);
        assert_eq!(c["per_category"]["multi_turn_memory"]["acceptable"], 1);
        assert_eq!(summary["K-clean"]["actual"]["acceptable"], 1);
        assert_eq!(summary["K-ill-posed"]["actual"]["replies"], 1);
        assert_eq!(summary["K-ill-posed"]["actual"]["fluent"], 0);
        assert_eq!(summary["everyday"]["actual"]["replies"], 1);
        assert!(summary.get("stretch").is_none());
        // An unparsed verdict (null) counts as no; a missing one is an error.
        let mut unparsed = row("conv-x", "c", [true, true], [true, true]);
        unparsed["grades"]["relevant"] = Value::Null;
        let s = tier_summary(std::slice::from_ref(&unparsed), &ill).unwrap();
        assert_eq!(s["C"]["actual"]["acceptable"], 0);
        assert_eq!(s["C"]["actual"]["unparsed_answers"], 1);
        unparsed["grades"].as_object_mut().unwrap().remove("fluent");
        assert!(tier_summary(&[unparsed], &ill).is_err());
    }

    #[test]
    fn compare_pairs_by_id_within_tier() {
        let ill: BTreeSet<String> = ["heldout-009".to_owned()].into();
        let (ok, bad) = ([true, true], [true, false]);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        // C: a acceptable on 9 rows that b fails, b on 1 that a fails, 2 both.
        for i in 0..12 {
            let id = format!("conv-do-{i:02}");
            let (x, y) = match i {
                0..=8 => (ok, bad),
                9 => (bad, ok),
                _ => (ok, ok),
            };
            a.push(row(&id, "self_contained_instruction", x, ok));
            b.push(row(&id, "self_contained_instruction", y, ok));
        }
        a.push(row("heldout-001", "heldout_first_turn", ok, ok));
        b.push(row("heldout-001", "heldout_first_turn", bad, ok));
        a.push(row("heldout-009", "heldout_first_turn", bad, ok));
        // Only in b: unpaired.
        b.push(row("heldout-002", "heldout_first_turn", ok, ok));

        let all = compare_rows(&a, &b, &ill, None).unwrap();
        let c = &all["per_tier"]["C"]["acceptable"];
        assert_eq!(c["rows"], 12);
        assert_eq!(c["a_only"], 9);
        assert_eq!(c["b_only"], 1);
        assert_eq!(c["both"], 2);
        let p = c["mcnemar_exact_p"].as_f64().unwrap();
        assert!((p - mcnemar_exact(9, 1)).abs() < 1e-15 && p < 0.05);
        assert_eq!(all["per_tier"]["C"]["fluent"]["a_only"], 0);
        assert_eq!(all["per_tier"]["K-clean"]["acceptable"]["a_only"], 1);
        assert_eq!(all["unpaired"]["K-ill-posed"]["a_rows"], 1);
        assert_eq!(all["unpaired"]["K-clean"]["b_rows"], 1);
        assert_eq!(all["all_selected"]["acceptable"]["rows"], 13);
        assert_eq!(
            all["per_tier"]["C"]["per_category_acceptable"]["self_contained_instruction"]["a"],
            11
        );

        let only_c = compare_rows(&a, &b, &ill, Some("C")).unwrap();
        assert_eq!(only_c["all_selected"]["acceptable"]["rows"], 12);
        assert!(only_c["per_tier"].get("K-clean").is_none());
        assert!(compare_rows(&a, &b, &ill, Some("stretch")).is_err());

        // The same id with different user turns cannot be paired.
        let mut changed = b.clone();
        changed[0]["conversation"][0]["user"] = json!("another turn");
        assert!(compare_rows(&a, &changed, &ill, None).is_err());
        // A repeated id in one report is an error.
        let mut repeated = a.clone();
        repeated.push(a[0].clone());
        assert!(compare_rows(&repeated, &b, &ill, None).is_err());
    }
}
