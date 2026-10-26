use sha2::Digest;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;
use uor_r4_object_store::address_stage0::{build_manifest, STAGE0_SCHEMA};
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("refused: {error}");
            ExitCode::from(1)
        }
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, tokenizer_path, output] if command == "manifest" => {
            let bytes =
                std::fs::read(tokenizer_path).map_err(|e| format!("{tokenizer_path}: {e}"))?;
            let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
                .ok_or_else(|| format!("{tokenizer_path}: invalid pinned tokenizer"))?;
            let manifest = build_manifest(&tokenizer)?;
            let encoded =
                serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())? + "\n";
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)
                .map_err(|e| format!("{output}: {e} (outputs are never overwritten)"))?;
            file.write_all(encoded.as_bytes())
                .map_err(|e| format!("{output}: {e}"))?;
            println!(
                "{STAGE0_SCHEMA}: 40 rows, sha256 {}",
                sha2::Sha256::digest(encoded.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            Ok(())
        }
        [command, artifact, config, retained_receipt, tokenizer, manifest, out_dir]
            if command == "run" =>
        {
            run_stage0(
                artifact,
                config,
                retained_receipt,
                tokenizer,
                manifest,
                out_dir,
            )
        }
        _ => Err(concat!(
            "usage: address-stage0 manifest TOKENIZER_JSON OUT_JSON | ",
            "address-stage0 run ARTIFACT_D11 CONFIG_JSON ORIGINAL_FINAL_DEV_JSON ",
            "TOKENIZER_JSON MANIFEST_JSON OUT_DIR"
        )
        .into()),
    }
}

#[cfg(feature = "cuda")]
fn run_stage0(
    artifact: &str,
    config: &str,
    retained_receipt: &str,
    tokenizer: &str,
    manifest: &str,
    out_dir: &str,
) -> Result<(), String> {
    use std::path::PathBuf;
    use uor_r4_object_store::address_stage0::runner::{run, RunPaths};
    let paths = RunPaths {
        artifact: PathBuf::from(artifact),
        config: PathBuf::from(config),
        retained_receipt: PathBuf::from(retained_receipt),
        tokenizer: PathBuf::from(tokenizer),
        manifest: PathBuf::from(manifest),
        out_dir: PathBuf::from(out_dir),
    };
    let outcome = run(&paths)?;
    println!(
        "Stage 0 parity PASS; copy decision {} (primary_pass={}); outputs in {}",
        outcome.decision,
        outcome.copy_primary_pass,
        outcome.out_dir.display()
    );
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn run_stage0(
    _artifact: &str,
    _config: &str,
    _retained_receipt: &str,
    _tokenizer: &str,
    _manifest: &str,
    _out_dir: &str,
) -> Result<(), String> {
    Err("address-stage0 run requires a build with --features cuda".into())
}
