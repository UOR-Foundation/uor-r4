use sha2::Digest;
use std::{error::Error, fs::OpenOptions, io::Write};
use uor_r4_object_store::address_stage0::{build_manifest, STAGE0_SCHEMA};
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, command, tokenizer_path, output] if command == "manifest" => {
            let bytes = std::fs::read(tokenizer_path)?;
            let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
                .ok_or("invalid pinned tokenizer")?;
            let manifest = build_manifest(&tokenizer)?;
            let encoded = serde_json::to_string_pretty(&manifest)? + "\n";
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            file.write_all(encoded.as_bytes())?;
            println!(
                "{STAGE0_SCHEMA}: 40 rows, sha256 {}",
                sha2::Sha256::digest(encoded.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            Ok(())
        }
        _ => Err("usage: address-stage0 manifest TOKENIZER_JSON OUT_JSON".into()),
    }
}
