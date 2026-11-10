//! Cross-representation audit of one stack contract against its run and artifact.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use repo_model::stack::{Probe, StackContract};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

type Fail = Box<dyn Error>;
type Result<T> = std::result::Result<T, Fail>;

const SCHEMA: &str = "uor-r4.stack-audit/1";

/// Run `cargo xtask stack-audit` from already parsed command-line arguments.
pub fn run(root: &Path, args: &[String]) -> Result<()> {
    let options = Options::parse(root, args)?;
    let contract_text = fs::read_to_string(&options.contract)?;
    let contract: StackContract = toml::from_str(&contract_text)?;
    contract.check(root)?;

    let mut report = Report::new(&options, &contract);
    if let Some(path) = &options.config {
        audit_config(&mut report, path, &contract);
    }
    if let Some(path) = &options.checkpoint {
        audit_checkpoint(&mut report, path, &contract);
    }
    if let Some(path) = &options.artifact {
        audit_artifact(&mut report, path, &options.contract, &contract);
    }
    if let (Some(tokenizer), Some(artifact)) = (&options.tokenizer, &options.artifact) {
        audit_tokenizer(&mut report, tokenizer, artifact, &contract);
    }
    if let (Some(chat), Some(artifact), Some(tokenizer)) =
        (&options.chat_bin, &options.artifact, &options.tokenizer)
    {
        audit_chat(&mut report, chat, artifact, tokenizer, &contract)?;
    } else if options.artifact.is_some() {
        report.record(
            "integer-session",
            AuditStatus::Unavailable,
            Value::String("an executable uor-chat path".to_owned()),
            Value::Null,
            "stack-audit",
            "pass --chat-bin and --tokenizer to load the artifact through the actual integer product path",
        );
    }
    audit_probes(
        &mut report,
        options.probes.as_deref().unwrap_or(root),
        &contract,
        options.chat_bin.as_deref(),
        options.artifact.as_deref(),
        options.tokenizer.as_deref(),
    )?;
    report.finish();
    let json = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = &options.out {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &json)?;
    }
    print!("{json}");
    if report.overall == "MISMATCH" {
        return Err("stack audit found a cross-representation mismatch".into());
    }
    Ok(())
}

#[derive(Debug)]
struct Options {
    contract: PathBuf,
    artifact: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    config: Option<PathBuf>,
    checkpoint: Option<PathBuf>,
    chat_bin: Option<PathBuf>,
    probes: Option<PathBuf>,
    out: Option<PathBuf>,
}

impl Options {
    fn parse(root: &Path, args: &[String]) -> Result<Self> {
        let mut contract = None;
        let mut artifact = None;
        let mut tokenizer = None;
        let mut config = None;
        let mut checkpoint = None;
        let mut chat_bin = None;
        let mut probes = None;
        let mut out = None;
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{flag} needs a value"))?;
            let path = {
                let path = PathBuf::from(value);
                if path.is_absolute() {
                    path
                } else {
                    root.join(path)
                }
            };
            match flag {
                "--contract" => contract = Some(path),
                "--artifact" => artifact = Some(path),
                "--tokenizer" => tokenizer = Some(path),
                "--config" => config = Some(path),
                "--checkpoint" => checkpoint = Some(path),
                "--chat-bin" => chat_bin = Some(path),
                "--probes" => probes = Some(path),
                "--out" => out = Some(path),
                other => return Err(format!("unknown stack-audit option {other}").into()),
            }
            index += 2;
        }
        Ok(Self {
            contract: contract.ok_or("--contract <PATH> is required")?,
            artifact,
            tokenizer,
            config,
            checkpoint,
            chat_bin,
            probes,
            out,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum AuditStatus {
    #[serde(rename = "MATCH")]
    Match,
    #[serde(rename = "MISMATCH")]
    Mismatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

#[derive(Debug, Serialize)]
struct AuditCheck {
    id: String,
    status: AuditStatus,
    expected: Value,
    actual: Value,
    source: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema: &'static str,
    contract: String,
    contract_semantic_blake3: String,
    artifact: Option<String>,
    artifact_sha256: Option<String>,
    overall: String,
    checks: Vec<AuditCheck>,
}

impl Report {
    fn new(options: &Options, contract: &StackContract) -> Self {
        Self {
            schema: SCHEMA,
            contract: contract.id.clone(),
            contract_semantic_blake3: blake3::hash(
                &serde_json::to_vec(contract).expect("contract serializes"),
            )
            .to_hex()
            .to_string(),
            artifact: options
                .artifact
                .as_ref()
                .map(|path| path.display().to_string()),
            artifact_sha256: None,
            overall: "NOT_RUN".to_owned(),
            checks: Vec::new(),
        }
    }

    fn record(
        &mut self,
        id: impl Into<String>,
        status: AuditStatus,
        expected: Value,
        actual: Value,
        source: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.checks.push(AuditCheck {
            id: id.into(),
            status,
            expected,
            actual,
            source: source.into(),
            detail: detail.into(),
        });
    }

    fn compare(
        &mut self,
        id: impl Into<String>,
        expected: Value,
        actual: Value,
        source: &Path,
        detail: &str,
    ) {
        let status = if expected == actual {
            AuditStatus::Match
        } else {
            AuditStatus::Mismatch
        };
        self.record(
            id,
            status,
            expected,
            actual,
            source.display().to_string(),
            detail,
        );
    }

    fn finish(&mut self) {
        self.overall = if self
            .checks
            .iter()
            .any(|check| check.status == AuditStatus::Mismatch)
        {
            "MISMATCH"
        } else if self
            .checks
            .iter()
            .any(|check| check.status == AuditStatus::Unavailable)
        {
            "UNAVAILABLE"
        } else if self.checks.is_empty()
            || self
                .checks
                .iter()
                .all(|check| check.status == AuditStatus::NotRun)
        {
            "NOT_RUN"
        } else {
            "MATCH"
        }
        .to_owned();
    }
}

fn audit_config(report: &mut Report, path: &Path, contract: &StackContract) {
    let config = match read_json(path) {
        Ok(value) => value,
        Err(error) => {
            report.record(
                "run-config",
                AuditStatus::Unavailable,
                Value::Bool(true),
                Value::String(error.to_string()),
                path.display().to_string(),
                "run config is absent or unreadable",
            );
            return;
        }
    };
    compare_optional(
        report,
        "run-config-objective",
        contract.model.objective.clone(),
        &config,
        "objective",
        path,
    );
    compare_optional_usize(
        report,
        "run-config-width",
        contract.model.width,
        &config,
        "width",
        path,
    );
    compare_optional_usize(
        report,
        "run-config-context",
        contract
            .model
            .training_context
            .unwrap_or(contract.model.context),
        &config,
        "context",
        path,
    );
    let hamming = config
        .get("hamming_read")
        .and_then(Value::as_u64)
        .is_some_and(|heads| heads > 0);
    let hamming_mechanisms: Vec<_> = contract
        .mechanisms
        .iter()
        .filter(|mechanism| {
            mechanism
                .sections
                .iter()
                .any(|section| section == "hamming-reads")
                || mechanism
                    .sidecars
                    .iter()
                    .any(|sidecar| sidecar.contains("hamming"))
                || mechanism.id.contains("hamming")
        })
        .collect();
    if !hamming_mechanisms.is_empty() {
        report.compare(
            "run-config-hamming-presence",
            Value::Bool(true),
            Value::Bool(hamming),
            path,
            "the contract's Hamming mechanism is enabled in the run config",
        );
    }
    let memory_contract = contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("turn-memory") || mechanism.id == "memory");
    if memory_contract {
        let memory = config
            .get("turn_memory")
            .and_then(Value::as_u64)
            .is_some_and(|k| k > 0);
        report.compare(
            "run-config-turn-memory-presence",
            Value::Bool(true),
            Value::Bool(memory),
            path,
            "the contract's turn-memory mechanism is enabled in the run config",
        );
    }
}

fn compare_optional(
    report: &mut Report,
    id: &str,
    expected: String,
    object: &Value,
    key: &str,
    source: &Path,
) {
    match object.get(key).and_then(Value::as_str) {
        Some(actual) => report.compare(
            id,
            Value::String(expected),
            Value::String(actual.to_owned()),
            source,
            &format!("run config `{key}` agrees with the contract"),
        ),
        None => report.record(
            id,
            AuditStatus::Unavailable,
            Value::String(expected),
            Value::Null,
            source.display().to_string(),
            format!("run config does not record `{key}`"),
        ),
    }
}

fn compare_optional_usize(
    report: &mut Report,
    id: &str,
    expected: usize,
    object: &Value,
    key: &str,
    source: &Path,
) {
    match object.get(key).and_then(Value::as_u64) {
        Some(actual) => report.compare(
            id,
            Value::from(expected),
            Value::from(actual),
            source,
            &format!("run config `{key}` agrees with the contract"),
        ),
        None => report.record(
            id,
            AuditStatus::Unavailable,
            Value::from(expected),
            Value::Null,
            source.display().to_string(),
            format!("run config does not record `{key}`"),
        ),
    }
}

fn audit_checkpoint(report: &mut Report, path: &Path, contract: &StackContract) {
    if !path.is_dir() {
        report.record(
            "checkpoint",
            AuditStatus::Unavailable,
            Value::String("checkpoint directory".to_owned()),
            Value::Null,
            path.display().to_string(),
            "checkpoint is absent or is not a directory",
        );
        return;
    }
    let mut required: BTreeSet<String> = ["config.json", "model.safetensors"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    for mechanism in &contract.mechanisms {
        required.extend(
            mechanism
                .sidecars
                .iter()
                .filter(|name| !name.starts_with("artifact:"))
                .cloned(),
        );
    }
    for name in required {
        let target = path.join(&name);
        report.record(
            format!("checkpoint:{name}"),
            if target.is_file() {
                AuditStatus::Match
            } else {
                AuditStatus::Mismatch
            },
            Value::Bool(true),
            Value::Bool(target.is_file()),
            target.display().to_string(),
            "required checkpoint file or sidecar is present",
        );
    }
}

#[derive(Debug)]
struct ArtifactHeader {
    schema: String,
    shape: Value,
    source: Value,
    sections: BTreeSet<String>,
    has_hamming: bool,
    has_memory: bool,
    has_memory_chain: bool,
    has_point: bool,
}

fn parse_artifact(path: &Path) -> Result<(ArtifactHeader, String)> {
    let bytes = fs::read(path)?;
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err(format!("{}: missing UORLUT01 magic", path.display()).into());
    }
    let header_len = u64::from_le_bytes(bytes[8..16].try_into().expect("8 bytes")) as usize;
    let end = 16usize
        .checked_add(header_len)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| format!("{}: header length does not fit", path.display()))?;
    let header: Value = serde_json::from_slice(&bytes[16..end])?;
    let sections = header
        .get("matrices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            header
                .get("tables")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
        .filter_map(|entry| entry.get("name").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((
        ArtifactHeader {
            schema: header
                .get("schema")
                .and_then(Value::as_str)
                .ok_or("artifact header has no schema")?
                .to_owned(),
            shape: header
                .get("shape")
                .cloned()
                .ok_or("artifact has no shape")?,
            source: header.get("source").cloned().unwrap_or(Value::Null),
            sections,
            has_hamming: header.get("hamming").is_some(),
            has_memory: header.get("memory").is_some(),
            has_memory_chain: header.get("memory_chain").is_some(),
            has_point: header.get("point").is_some(),
        },
        sha256,
    ))
}

fn audit_artifact(
    report: &mut Report,
    path: &Path,
    contract_path: &Path,
    contract: &StackContract,
) {
    let (artifact, sha256) = match parse_artifact(path) {
        Ok(parsed) => parsed,
        Err(error) => {
            report.record(
                "artifact-parse",
                AuditStatus::Unavailable,
                Value::Bool(true),
                Value::String(error.to_string()),
                path.display().to_string(),
                "artifact is absent or its header cannot be read",
            );
            return;
        }
    };
    report.artifact_sha256 = Some(sha256);
    report.compare(
        "artifact-family",
        Value::String(contract.artifact.family.clone()),
        Value::String("uor-r4.lut-stack".to_owned()),
        path,
        "served container family",
    );
    if let Some(schema) = &contract.artifact.schema {
        report.compare(
            "artifact-schema",
            Value::String(schema.clone()),
            Value::String(artifact.schema.clone()),
            path,
            "exact artifact header schema",
        );
    }
    for (id, key, expected) in [
        ("artifact-vocab", "vocab", contract.model.vocab),
        ("artifact-width", "width", contract.model.width),
        ("artifact-context", "context", contract.model.context),
    ] {
        let actual = artifact
            .shape
            .get(key)
            .and_then(Value::as_u64)
            .map(Value::from)
            .unwrap_or(Value::Null);
        report.compare(id, Value::from(expected), actual, path, key);
    }
    let layers = artifact
        .shape
        .get("pattern")
        .and_then(Value::as_str)
        .map(|pattern| pattern.len())
        .unwrap_or(0);
    report.compare(
        "artifact-layers",
        Value::from(contract.model.layers),
        Value::from(layers),
        path,
        "layer pattern length",
    );
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("hamming"))
    {
        report.compare(
            "artifact-hamming-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_hamming),
            path,
            "the artifact carries the contract's Hamming reads",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("turn-memory") || mechanism.id == "memory")
    {
        report.compare(
            "artifact-turn-memory-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_memory),
            path,
            "the artifact carries the contract's turn memory",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("memory-chain"))
    {
        report.compare(
            "artifact-memory-chain-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_memory_chain),
            path,
            "the artifact carries the contract's memory chain",
        );
    }
    if contract
        .mechanisms
        .iter()
        .any(|mechanism| mechanism.id.contains("point-key"))
    {
        report.compare(
            "artifact-point-key-presence",
            Value::Bool(true),
            Value::Bool(artifact.has_point),
            path,
            "the artifact carries the contract's point key",
        );
    }
    for mechanism in &contract.mechanisms {
        for sidecar in &mechanism.sidecars {
            if sidecar.starts_with("artifact:") {
                let prefix = sidecar.trim_start_matches("artifact:");
                let present = artifact
                    .sections
                    .iter()
                    .any(|section| section == prefix || section.starts_with(&format!("{prefix}.")));
                report.compare(
                    format!("artifact-section:{}:{prefix}", mechanism.id),
                    Value::Bool(true),
                    Value::Bool(present),
                    path,
                    "mechanism sidecar naming an artifact section is present",
                );
            }
        }
    }
    report.record(
        "artifact-source",
        if artifact.source.is_null() {
            AuditStatus::Unavailable
        } else {
            AuditStatus::Match
        },
        Value::String("exporter provenance object".to_owned()),
        artifact.source.clone(),
        path.display().to_string(),
        "provenance is diagnostic; the serving arithmetic never reads it",
    );
    match artifact.source.get("stack_contract") {
        Some(provenance) => {
            let expected = fs::read(contract_path)
                .ok()
                .map(Sha256::digest)
                .map(|digest| hex_string(&digest))
                .unwrap_or_default();
            report.compare(
                "artifact-stack-contract-sha256",
                Value::String(expected),
                provenance.get("sha256").cloned().unwrap_or(Value::Null),
                path,
                "optional diagnostic provenance names this exact contract source",
            );
        }
        None => report.record(
            "artifact-stack-contract-sha256",
            AuditStatus::NotRun,
            Value::String("optional diagnostic provenance".to_owned()),
            Value::Null,
            path.display().to_string(),
            "the artifact was exported without --stack-contract; serving does not require it",
        ),
    }
    match artifact.source.get("tokenizer_sha256") {
        Some(tokenizer) => report.compare(
            "artifact-tokenizer-sha256",
            Value::String(contract.model.tokenizer_sha256.clone()),
            tokenizer.clone(),
            path,
            "optional diagnostic provenance binds the artifact to the contract tokenizer",
        ),
        None => report.record(
            "artifact-tokenizer-sha256",
            AuditStatus::NotRun,
            Value::String("optional diagnostic provenance".to_owned()),
            Value::Null,
            path.display().to_string(),
            "the artifact predates tokenizer provenance; its shape is still audited",
        ),
    }
}

fn audit_tokenizer(
    report: &mut Report,
    tokenizer: &Path,
    _artifact: &Path,
    contract: &StackContract,
) {
    match fs::read(tokenizer) {
        Ok(bytes) => {
            let actual: String = Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            report.compare(
                "tokenizer-sha256",
                Value::String(contract.model.tokenizer_sha256.clone()),
                Value::String(actual),
                tokenizer,
                "tokenizer bytes match the contract identity",
            );
        }
        Err(error) => report.record(
            "tokenizer-sha256",
            AuditStatus::Unavailable,
            Value::String(contract.model.tokenizer_sha256.clone()),
            Value::String(error.to_string()),
            tokenizer.display().to_string(),
            "tokenizer is unreadable",
        ),
    }
}

fn audit_chat(
    report: &mut Report,
    chat: &Path,
    artifact: &Path,
    tokenizer: &Path,
    contract: &StackContract,
) -> Result<()> {
    let scratch = std::env::temp_dir().join(format!(
        "uor-stack-audit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    fs::create_dir_all(&scratch)?;
    let script = scratch.join("chat-script.json");
    fs::write(
        &script,
        br#"{"schema":"uor-r4.chat-script/1","turns":[{"user":"Hello"}]}"#,
    )?;
    let command_args = vec![
        "--stack".to_owned(),
        artifact.display().to_string(),
        "--tokenizer".to_owned(),
        tokenizer.display().to_string(),
        "--script".to_owned(),
        script.display().to_string(),
        "--max-tokens".to_owned(),
        "1".to_owned(),
    ];
    let output = Command::new(chat).args(&command_args).output();
    let _ = fs::remove_dir_all(&scratch);
    match output {
        Ok(output) if output.status.success() => {
            let record: Value = match serde_json::from_slice(&output.stdout) {
                Ok(record) => record,
                Err(error) => {
                    report.record(
                        "chat-integer-session",
                        AuditStatus::Mismatch,
                        Value::String("uor-r4.uor-chat-stack-script/1".to_owned()),
                        Value::String(format!("invalid JSON: {error}")),
                        chat.display().to_string(),
                        "the product chat path did not emit its declared record",
                    );
                    return Ok(());
                }
            };
            report.compare(
                "chat-schema",
                Value::String("uor-r4.uor-chat-stack-script/1".to_owned()),
                record.get("schema").cloned().unwrap_or(Value::Null),
                chat,
                "the real chat path emitted the script record",
            );
            report.compare(
                "chat-artifact-sha256",
                Value::String(report.artifact_sha256.clone().unwrap_or_default()),
                record
                    .get("artifact_sha256")
                    .cloned()
                    .unwrap_or(Value::Null),
                chat,
                "chat loaded the audited artifact",
            );
            report.compare(
                "chat-context",
                Value::from(contract.model.context),
                record.get("context").cloned().unwrap_or(Value::Null),
                chat,
                "chat reports the contract context",
            );
            let nonempty = record
                .get("transcript")
                .and_then(Value::as_array)
                .is_some_and(|turns| {
                    turns.len() == 1
                        && turns[0]
                            .get("reply_ids")
                            .and_then(Value::as_array)
                            .is_some_and(|ids| !ids.is_empty())
                });
            report.record(
                "chat-generated-one-token",
                if nonempty {
                    AuditStatus::Match
                } else {
                    AuditStatus::Mismatch
                },
                Value::Bool(true),
                Value::Bool(nonempty),
                chat.display().to_string(),
                "the integer session generated at least one token through the product path",
            );
        }
        Ok(output) => report.record(
            "chat-integer-session",
            AuditStatus::Unavailable,
            Value::String("exit 0".to_owned()),
            Value::String(String::from_utf8_lossy(&output.stderr).into_owned()),
            chat.display().to_string(),
            "the chat path could not load this artifact and tokenizer",
        ),
        Err(error) => report.record(
            "chat-integer-session",
            AuditStatus::Unavailable,
            Value::String("executable".to_owned()),
            Value::String(error.to_string()),
            chat.display().to_string(),
            "the chat executable is absent",
        ),
    }
    Ok(())
}

fn audit_probes(
    report: &mut Report,
    directory: &Path,
    contract: &StackContract,
    chat: Option<&Path>,
    artifact: Option<&Path>,
    tokenizer: Option<&Path>,
) -> Result<()> {
    for probe in &contract.probes {
        let Some(fixture) = &probe.fixture else {
            continue;
        };
        let path = directory.join(fixture);
        match verify_fixture(&path, probe) {
            Ok(fixture) => {
                if let (Some(chat), Some(artifact), Some(tokenizer)) = (chat, artifact, tokenizer) {
                    run_fixture(report, &path, &fixture, probe, chat, artifact, tokenizer)?;
                } else {
                    report.record(
                        format!("probe-fixture:{}", probe.id),
                        AuditStatus::NotRun,
                        Value::String(probe.id.clone()),
                        Value::from(fixture.cases_len()),
                        path.display().to_string(),
                        "fixture is valid but no chat executable, artifact and tokenizer were supplied",
                    );
                }
            }
            Err(error) => report.record(
                format!("probe-fixture:{}", probe.id),
                AuditStatus::Unavailable,
                Value::String(probe.id.clone()),
                Value::String(error.to_string()),
                path.display().to_string(),
                "fixture is absent or malformed",
            ),
        }
    }
    Ok(())
}

#[derive(Debug)]
struct Fixture {
    value: Value,
}

impl Fixture {
    fn cases(&self) -> &[Value] {
        self.value
            .get("cases")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn cases_len(&self) -> usize {
        self.cases().len()
    }
}

fn verify_fixture(path: &Path, probe: &Probe) -> Result<Fixture> {
    let fixture: Value = serde_json::from_slice(&fs::read(path)?)?;
    if fixture.get("schema").and_then(Value::as_str) != Some("uor-r4.stack-probe/1") {
        return Err(format!("{}: wrong fixture schema", path.display()).into());
    }
    if fixture.get("probe").and_then(Value::as_str) != Some(probe.id.as_str()) {
        return Err(format!("{}: fixture names another probe", path.display()).into());
    }
    let count = fixture
        .get("cases")
        .and_then(Value::as_array)
        .map(Vec::len)
        .filter(|count| *count > 0)
        .ok_or_else(|| format!("{}: no executable cases", path.display()))?;
    for case in fixture
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if case.get("id").and_then(Value::as_str).is_none()
            || case.get("script").is_none()
            || case.get("expect").is_none()
        {
            return Err(format!(
                "{}: each of the {count} cases needs id, script and expect",
                path.display()
            )
            .into());
        }
    }
    Ok(Fixture { value: fixture })
}

fn run_fixture(
    report: &mut Report,
    fixture_path: &Path,
    fixture: &Fixture,
    probe: &Probe,
    chat: &Path,
    artifact: &Path,
    tokenizer: &Path,
) -> Result<()> {
    for case in fixture.cases() {
        let case_id = case["id"].as_str().expect("validated case id");
        let id = format!("probe:{}:{case_id}", probe.id);
        if !probe.layers.iter().any(|layer| layer.as_str() == "chat") {
            report.record(
                id,
                AuditStatus::Unavailable,
                Value::String("chat".to_owned()),
                Value::String(
                    probe
                        .layers
                        .iter()
                        .map(|layer| layer.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                fixture_path.display().to_string(),
                "this runner only executes the chat layer; the contract asks for another layer",
            );
            continue;
        }
        let scratch = std::env::temp_dir().join(format!(
            "uor-stack-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir_all(&scratch)?;
        let script_path = scratch.join("script.json");
        let script = match case.get("repeat_user") {
            Some(repeat) => {
                let text = repeat
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("{case_id}: repeat_user.text is not a string"))?;
                let count = repeat
                    .get("count")
                    .and_then(Value::as_u64)
                    .filter(|count| (1..=100_000).contains(count))
                    .ok_or_else(|| format!("{case_id}: repeat_user.count is not 1..=100000"))?;
                serde_json::json!({
                    "schema": "uor-r4.chat-script/1",
                    "turns": [{"user": text.repeat(count as usize)}]
                })
            }
            None => case["script"].clone(),
        };
        fs::write(&script_path, serde_json::to_vec(&script)?)?;
        let mut args = vec![
            "--stack".to_owned(),
            artifact.display().to_string(),
            "--tokenizer".to_owned(),
            tokenizer.display().to_string(),
            "--script".to_owned(),
            script_path.display().to_string(),
        ];
        if let Some(extra) = case.get("args").and_then(Value::as_array) {
            args.extend(extra.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        let output = Command::new(chat).args(&args).output();
        let _ = fs::remove_dir_all(&scratch);
        match output {
            Ok(output) => {
                let expectation = &case["expect"];
                let want_status = expectation
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("record");
                if want_status == "error" {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let contains = expectation
                        .get("stderr_contains")
                        .and_then(Value::as_str)
                        .is_none_or(|needle| stderr.contains(needle));
                    let ok = !output.status.success() && contains;
                    report.record(
                        id,
                        if ok {
                            AuditStatus::Match
                        } else {
                            AuditStatus::Mismatch
                        },
                        expectation.clone(),
                        Value::String(stderr.into_owned()),
                        fixture_path.display().to_string(),
                        "the product path refused the malformed or over-boundary case as declared",
                    );
                    continue;
                }
                let record: Value = match serde_json::from_slice(&output.stdout) {
                    Ok(record) => record,
                    Err(error) => {
                        report.record(
                            id,
                            AuditStatus::Mismatch,
                            Value::String("a JSON script record".to_owned()),
                            Value::String(error.to_string()),
                            fixture_path.display().to_string(),
                            "the product chat path did not produce a script record",
                        );
                        continue;
                    }
                };
                let failures = check_expectation(&record, expectation);
                report.record(
                    id,
                    if output.status.success() && failures.is_empty() {
                        AuditStatus::Match
                    } else {
                        AuditStatus::Mismatch
                    },
                    expectation.clone(),
                    record,
                    fixture_path.display().to_string(),
                    if failures.is_empty() {
                        "the product chat path met the declared structural expectation".to_owned()
                    } else {
                        failures.join("; ")
                    },
                );
            }
            Err(error) => report.record(
                id,
                AuditStatus::Unavailable,
                Value::String("executable chat path".to_owned()),
                Value::String(error.to_string()),
                chat.display().to_string(),
                "probe could not start the chat executable",
            ),
        }
    }
    Ok(())
}

fn check_expectation(record: &Value, expectation: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    let turns = record
        .get("transcript")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if let Some(required) = expectation.get("turns").and_then(Value::as_u64) {
        if turns.len() as u64 != required {
            failures.push(format!("turns: got {}, expected {required}", turns.len()));
        }
    }
    if let Some(required) = expectation.get("turn_count").and_then(Value::as_u64) {
        if turns.len() as u64 != required {
            failures.push(format!(
                "turn_count: got {}, expected {required}",
                turns.len()
            ));
        }
    }
    if expectation
        .get("all_reply_ids_nonempty")
        .and_then(Value::as_bool)
        == Some(true)
    {
        for (index, turn) in turns.iter().enumerate() {
            if turn
                .get("reply_ids")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
            {
                failures.push(format!("turn {index} produced no reply token"));
            }
        }
    }
    if let Some(allowed) = expectation.get("stops").and_then(Value::as_array) {
        let allowed: BTreeSet<&str> = allowed.iter().filter_map(Value::as_str).collect();
        for (index, turn) in turns.iter().enumerate() {
            let stop = turn.get("stop").and_then(Value::as_str).unwrap_or("");
            if !allowed.contains(stop) {
                failures.push(format!("turn {index} stop {stop:?} is not allowed"));
            }
        }
    }
    if let Some(memory) = expectation.get("memory").and_then(Value::as_array) {
        let expected: Vec<&str> = memory.iter().filter_map(Value::as_str).collect();
        if expected.len() != turns.len() {
            failures.push(format!(
                "memory expectation has {} rows for {} turns",
                expected.len(),
                turns.len()
            ));
        } else {
            for (index, (turn, expected)) in turns.iter().zip(expected).enumerate() {
                let actual = turn.get("memory").and_then(Value::as_str).unwrap_or("");
                if actual != expected {
                    failures.push(format!(
                        "turn {index} memory {actual:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    if let Some(minimum) = expectation
        .get("distinct_replies_min")
        .and_then(Value::as_u64)
    {
        let distinct: BTreeSet<&str> = turns
            .iter()
            .filter_map(|turn| turn.get("reply_text").and_then(Value::as_str))
            .collect();
        if distinct.len() < minimum as usize {
            failures.push(format!(
                "distinct replies: got {}, expected at least {minimum}",
                distinct.len()
            ));
        }
    }
    failures
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_never_promotes_unavailable_or_empty_to_match() {
        let options = Options {
            contract: PathBuf::new(),
            artifact: None,
            tokenizer: None,
            config: None,
            checkpoint: None,
            chat_bin: None,
            probes: None,
            out: None,
        };
        let contract: StackContract = toml::from_str(include_str!(
            "../../conformance/stack/fbs-b64l-baseline.toml"
        ))
        .expect("contract parses");
        let mut report = Report::new(&options, &contract);
        report.finish();
        assert_eq!(report.overall, "NOT_RUN");
        report.record(
            "missing",
            AuditStatus::Unavailable,
            Value::Bool(true),
            Value::Null,
            "test",
            "missing",
        );
        report.finish();
        assert_eq!(report.overall, "UNAVAILABLE");
        report.record(
            "wrong",
            AuditStatus::Mismatch,
            Value::Bool(true),
            Value::Bool(false),
            "test",
            "wrong",
        );
        report.finish();
        assert_eq!(report.overall, "MISMATCH");
    }

    #[test]
    fn probe_expectations_check_turns_memory_stops_and_reply_ids() {
        let record = serde_json::json!({
            "transcript": [
                {
                    "reply_ids": [7],
                    "stop": "eos",
                    "memory": "none",
                    "reply_text": "Hello."
                },
                {
                    "reply_ids": [8, 9],
                    "stop": "max_new_tokens",
                    "memory": "explicit",
                    "reply_text": "It's 47."
                }
            ]
        });
        let expectation = serde_json::json!({
            "turns": 2,
            "all_reply_ids_nonempty": true,
            "stops": ["eos", "max_new_tokens", "short_cycle"],
            "memory": ["none", "explicit"],
            "distinct_replies_min": 2
        });
        assert!(check_expectation(&record, &expectation).is_empty());

        let wrong = serde_json::json!({
            "turns": 1,
            "stops": ["eos"],
            "memory": ["sieve", "none"],
            "distinct_replies_min": 3
        });
        let failures = check_expectation(&record, &wrong);
        assert!(failures.iter().any(|failure| failure.contains("turns")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("max_new_tokens")));
        assert!(failures.iter().any(|failure| failure.contains("memory")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("distinct replies")));
    }

    #[test]
    fn a_truncated_artifact_header_is_unavailable_not_match() {
        let path =
            std::env::temp_dir().join(format!("uor-stack-audit-bad-{}.uorlut", std::process::id()));
        fs::write(&path, b"UORLUT01").expect("bad artifact fixture");
        assert!(parse_artifact(&path).is_err());
        fs::remove_file(path).expect("bad artifact cleanup");
    }
}
