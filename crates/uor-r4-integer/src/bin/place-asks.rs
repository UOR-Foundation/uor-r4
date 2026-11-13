//! place-asks (claude-359): fit the asks on a split by counting, and count what they find.
//!
//! ```text
//! place-asks fit    --tokens TRAIN/tokens.u16 --kit train-places.json --out asks.json
//! place-asks census --tokens PANEL/tokens.u16 --kit panel-places.json --asks asks.json [--show N]
//! ```
//!
//! A token file is the `UORT` format the trainer reads: a 64-byte header holding the token count
//! and the vocabulary size, then the tokens as little-endian 16-bit integers. A kit is the place
//! kit the turn census writes: the marks and each recall reply's `[reply start, place, value
//! start, value length]`. Nothing here reads a model; every number is a count over tokens.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;
use uor_r4_integer::fbs_snapshot::PlaceMarks;
use uor_r4_integer::place_asks::{census, fit, AsksIndex, PlaceAsks};

const HEADER: usize = 64;

#[derive(Deserialize)]
struct Kit {
    tokens_sha256: String,
    marks: PlaceMarks,
    places: Vec<[u32; 4]>,
}

/// A token file's tokens and vocabulary size.
fn read_tokens(path: &Path) -> Result<(Vec<u32>, u32), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() < HEADER || &bytes[..4] != b"UORT" {
        return Err(format!("{}: not a UORT token file", path.display()));
    }
    let word = |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().expect("eight bytes"));
    let (count, vocab) = (word(8), word(16));
    let payload = &bytes[HEADER..];
    if payload.len() as u64 != 2 * count {
        return Err(format!("{}: the header counts {count} tokens and the file holds {} bytes of them", path.display(), payload.len()));
    }
    let tokens = payload.chunks_exact(2).map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]]))).collect();
    let vocab = u32::try_from(vocab).map_err(|_| format!("{}: a vocabulary of {vocab}", path.display()))?;
    Ok((tokens, vocab))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn flag(args: &[String], name: &str) -> Result<PathBuf, String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(PathBuf::from).ok_or_else(|| format!("{name} PATH is required"))
}

fn run(args: &[String]) -> Result<(), String> {
    let mode = args.first().map(String::as_str).ok_or("usage: place-asks fit|census ...")?;
    let (tokens, vocab) = read_tokens(&flag(args, "--tokens")?)?;
    let kit: Kit = read_json(&flag(args, "--kit")?)?;
    match mode {
        "fit" => {
            let out = flag(args, "--out")?;
            if out.exists() {
                return Err(format!("{}: exists; a fit never writes over a table", out.display()));
            }
            let (asks, lines) = fit(&tokens, &kit.marks, vocab, &kit.places, kit.tokens_sha256.clone())?;
            for line in &lines {
                println!("{line}");
            }
            let index = AsksIndex::new(&asks)?;
            let (counts, _) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, 0);
            for line in counts.lines("fitted") {
                println!("{line}");
            }
            let text = serde_json::to_string(&asks).map_err(|e| e.to_string())?;
            std::fs::write(&out, text).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("asks-fit wrote {} fitted_on={}", out.display(), asks.tokens_sha256);
        }
        "census" => {
            let asks: PlaceAsks = read_json(&flag(args, "--asks")?)?;
            if asks.marks != kit.marks || asks.vocab != vocab {
                return Err("the asks were fitted with other marks or another vocabulary than this split's".to_owned());
            }
            let show = match args.iter().position(|a| a == "--show") {
                Some(i) => args.get(i + 1).and_then(|n| n.parse().ok()).ok_or("--show N")?,
                None => 0,
            };
            let index = AsksIndex::new(&asks)?;
            let (counts, missed) = census(&tokens, &kit.marks, &kit.places, &index, &asks.forms, show);
            println!(
                "asks-census tokens={} fitted_on={} margin={} floor={} floor_other={} asking_floor={}",
                kit.tokens_sha256, asks.tokens_sha256, asks.margin, asks.floor, asks.floor_other, asks.asking_floor
            );
            for line in counts.lines("held") {
                println!("{line}");
            }
            for line in missed {
                println!("{line}");
            }
        }
        other => return Err(format!("unknown mode {other}: fit or census")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("place-asks: {e}");
            ExitCode::FAILURE
        }
    }
}
