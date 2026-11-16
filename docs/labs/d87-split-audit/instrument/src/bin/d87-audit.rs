//! `d87-audit --tokenizer <tokenizer.json> --split <dir> --out <dir> [--threads T]
//!  [--bloom-log2 B] <shard>...`
//!
//! Re-derives the split from `<split>/split-summary.json`'s config over the same shards and
//! refuses unless it reproduces that summary byte for byte; then tokenizes every kept record
//! with student-ingest and writes `<out>/audit-summary.json`: the exact 13-gram TRAIN/DEV
//! overlap census, the P0(d) gate verdict and the per-split target counts.

use std::path::PathBuf;
use std::process::ExitCode;

use d87_split_audit::{audit, plan, Splitter};
use m2_student_corpus_ingest::{hex, Coder};
use sha2::{Digest, Sha256};

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut tok, mut split, mut out, mut threads, mut bloom, mut shards) =
        (None, None, None, 12usize, 30u32, vec![]);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--tokenizer" => tok = args.next().map(PathBuf::from),
            "--split" => split = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--threads" => threads = args.next().and_then(|s| s.parse().ok()).ok_or("--threads")?,
            "--bloom-log2" => bloom = args.next().and_then(|s| s.parse().ok()).ok_or("--bloom-log2")?,
            s if s.starts_with("--") => return Err(format!("unknown flag {s}")),
            s => shards.push(PathBuf::from(s)),
        }
    }
    let (Some(tok), Some(split), Some(out)) = (tok, split, out) else {
        return Err("--tokenizer, --split and --out are required".into());
    };
    if !(6..=34).contains(&bloom) {
        return Err("--bloom-log2 must be 6..=34".into());
    }
    let summary_path = split.join("split-summary.json");
    let text = std::fs::read_to_string(&summary_path).map_err(|e| format!("{}: {e}", summary_path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let sp = Splitter::from_config_json(&v["config"])?;
    let p = plan(&sp, &shards, threads)?;
    let again = serde_json::to_string_pretty(&p.summary()).map_err(|e| e.to_string())? + "\n";
    if again != text {
        return Err("the shards do not reproduce split-summary.json".into());
    }
    let json = std::fs::read(&tok).map_err(|e| format!("{}: {e}", tok.display()))?;
    let coder = Coder::new(&json)?;
    let mut a = audit(&p, &coder, threads, bloom)?;
    a["tokenizer_sha256"] = hex(&Sha256::digest(&json)).into();
    a["split_summary_sha256"] = hex(&Sha256::digest(text.as_bytes())).into();
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let body = serde_json::to_string_pretty(&a).map_err(|e| e.to_string())? + "\n";
    std::fs::write(out.join("audit-summary.json"), body).map_err(|e| e.to_string())?;
    eprintln!("d87-audit: gate {}; written to {}", a["gate"], out.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("d87-audit: {e}");
            ExitCode::FAILURE
        }
    }
}
