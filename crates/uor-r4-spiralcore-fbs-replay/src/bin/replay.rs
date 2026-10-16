//! Measurement and replay harness for the SpiralCore Recursive FBS Engine
//! specification.
//!
//! For every corpus it records: exact round-trip recovery, the encoded size
//! against the input size, the reference baselines (gzip -9, xz -9, zstd -19),
//! the order-0 entropy bound, the dictionary-reuse statistics, the 16-step
//! nested literal-tail size from Section 6, and the Section 7 anchor+manifest
//! absorption chain. Output is a machine-readable receipt plus a markdown table.
//!
//! Usage:
//!   spiralcore-fbs-replay [--size BYTES] [--nested N] [--absorb K] [--out DIR] [FILE...]
//!
//! Reporting statistics use f64 (ratio, entropy) and shell out to standard
//! compressors; the codec path itself is exact integer/byte arithmetic.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use uor_r4_spiralcore_fbs_replay::corpus::{builtin_corpora, order0_entropy_bits, Corpus};
use uor_r4_spiralcore_fbs_replay::sha256::sha256_hex;
use uor_r4_spiralcore_fbs_replay::{
    absorb, decode, encode_literal, nest, sigma_inv, ATOM_CAPACITY,
};

/// A 334-byte manifest record, mirroring the paper's "334-byte manifest" role
/// (five identifying fields as JSON, padded to the same length). NOTE: the
/// developer's exact manifest bytes are not published; this is the review's
/// stand-in and is recorded with its own SHA-256.
fn manifest_334(input_sha: &str, wire_sha: &str) -> Vec<u8> {
    let mut s = format!(
        "{{\"rule\":\"recursive-literal-tail\",\"stop\":16,\"start_sha256\":\"{wire_sha}\",\"descendant_sha256\":\"{wire_sha}\",\"original_file_sha256\":\"{input_sha}\"}}"
    );
    while s.len() < 334 {
        s.push(' ');
    }
    s.truncate(334);
    s.into_bytes()
}

/// Run an external compressor over `data` and return the compressed size.
///
/// The write runs on its own thread. Writing the whole input from the calling
/// thread while the child blocks on a full stdout pipe deadlocks as soon as the
/// compressed form exceeds the pipe buffer; that was measured the hard way on
/// multi-megabyte corpora, and it is why this function is not a straight
/// write-then-wait.
fn compressed_size(program: &str, args: &[&str], data: &[u8]) -> Option<usize> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdin = child.stdin.take()?;
    let owned = data.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&owned);
        // stdin is dropped here, closing the child's input
    });
    let out = child.wait_with_output().ok()?;
    let _ = writer.join();
    if out.status.success() {
        Some(out.stdout.len())
    } else {
        None
    }
}

struct Row {
    name: String,
    source: String,
    input_bytes: usize,
    input_sha: String,
    wire_bytes: usize,
    wire_sha: String,
    dict_bytes: usize,
    verbatim_bytes: u64,
    atoms: u64,
    carries: u64,
    leaf_defs: u64,
    leaf_reuse: u64,
    parent_defs: u64,
    parent_reuse: u64,
    round_trip: bool,
    nested_bytes: usize,
    nested_round_trip: bool,
    absorb_bytes: Vec<usize>,
    absorb_round_trip: bool,
    gzip: Option<usize>,
    xz: Option<usize>,
    zstd: Option<usize>,
    entropy_bits: f64,
}

fn measure(c: &Corpus, nested_n: usize, absorb_k: usize) -> Row {
    let input_sha = sha256_hex(&c.bytes);
    let (w, packet, stats) = encode_literal(&c.bytes);
    let wire_sha = sha256_hex(&w);
    let dict_bytes = packet.dict.serialize().len();
    // Input bytes that travel verbatim inside the packet's dictionary: every
    // distinct leaf definition stores its atom's bytes literally (Appendix A).
    let verbatim_bytes: u64 = packet
        .dict
        .definitions()
        .iter()
        .map(|d| match d {
            uor_r4_spiralcore_fbs_replay::Def::Leaf(a) => a.len() as u64,
            uor_r4_spiralcore_fbs_replay::Def::Parent(_, _, _) => 0,
        })
        .sum();
    let round_trip = decode(&w).map(|d| d == c.bytes).unwrap_or(false);

    // Section 6: nested literal tails.
    let nested = nest(&packet, nested_n);
    let nested_bytes = nested.wires[0].len();
    let mut nested_round_trip = decode(&nested.wires[0])
        .map(|d| d == c.bytes)
        .unwrap_or(false);
    for j in 1..=nested_n {
        let ok = decode(&nested.wires[j])
            .map(|d| d == nested.dictionaries[j - 1])
            .unwrap_or(false);
        nested_round_trip &= ok;
    }

    // Section 7: absorb anchor + manifest repeatedly.
    let manifest = manifest_334(&input_sha, &wire_sha);
    let mut anchor = w.clone();
    let mut sizes = Vec::new();
    let mut absorb_ok = true;
    for _ in 0..absorb_k {
        let next = absorb(&anchor, &manifest);
        sizes.push(next.len());
        let recovered = decode(&next).ok().and_then(|d| sigma_inv(&d).ok());
        match recovered {
            Some((a, m)) => absorb_ok &= a == anchor && m == manifest,
            None => absorb_ok = false,
        }
        anchor = next;
    }

    Row {
        name: c.name.clone(),
        source: c.source.clone(),
        input_bytes: c.bytes.len(),
        input_sha,
        wire_bytes: w.len(),
        wire_sha,
        dict_bytes,
        verbatim_bytes,
        atoms: stats.atoms,
        carries: stats.carries,
        leaf_defs: stats.leaf_defs,
        leaf_reuse: stats.leaf_reuse,
        parent_defs: stats.parent_defs,
        parent_reuse: stats.parent_reuse,
        round_trip,
        nested_bytes,
        nested_round_trip,
        absorb_bytes: sizes,
        absorb_round_trip: absorb_ok,
        gzip: compressed_size("gzip", &["-9", "-c"], &c.bytes),
        xz: compressed_size("xz", &["-9", "-c", "-T1"], &c.bytes),
        zstd: compressed_size("zstd", &["-19", "-c", "-q"], &c.bytes),
        entropy_bits: order0_entropy_bits(&c.bytes),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut size = 64 * 1024usize;
    let mut nested_n = 16usize;
    let mut absorb_k = 3usize;
    let mut out_dir: Option<PathBuf> = None;
    let mut files: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                i += 1;
                size = args[i].parse().expect("--size");
            }
            "--nested" => {
                i += 1;
                nested_n = args[i].parse().expect("--nested");
            }
            "--absorb" => {
                i += 1;
                absorb_k = args[i].parse().expect("--absorb");
            }
            "--out" => {
                i += 1;
                out_dir = Some(PathBuf::from(&args[i]));
            }
            other => files.push(other.to_string()),
        }
        i += 1;
    }

    let mut corpora = builtin_corpora(size);
    for f in &files {
        match fs::read(f) {
            Ok(bytes) => corpora.push(Corpus {
                name: PathBuf::from(f)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| f.clone()),
                source: format!("file {f}"),
                bytes,
            }),
            Err(e) => eprintln!("skip {f}: {e}"),
        }
    }

    let rows: Vec<Row> = corpora
        .iter()
        .map(|c| measure(c, nested_n, absorb_k))
        .collect();

    println!(
        "atom capacity q={ATOM_CAPACITY}; nested steps n={nested_n}; absorb steps k={absorb_k}"
    );
    println!();
    println!("| corpus | input B | E(x) B | ratio | dict B | verbatim B | leaves (distinct/reused) | parents (distinct/reused) | carries | round-trip | Q0(n={nested_n}) B | gzip -9 | xz -9 | zstd -19 | H0 bits |");
    println!("|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|");
    for r in &rows {
        let ratio = if r.input_bytes > 0 {
            r.wire_bytes as f64 / r.input_bytes as f64
        } else {
            f64::NAN
        };
        println!(
            "| {} | {} | {} | {:.4} | {} | {} | {}/{} | {}/{} | {} | {} | {} | {} | {} | {} | {:.0} |",
            r.name,
            r.input_bytes,
            r.wire_bytes,
            ratio,
            r.dict_bytes,
            r.verbatim_bytes,
            r.leaf_defs,
            r.leaf_reuse,
            r.parent_defs,
            r.parent_reuse,
            r.carries,
            if r.round_trip { "exact" } else { "FAIL" },
            r.nested_bytes,
            r.gzip.map(|v| v.to_string()).unwrap_or_else(|| "n/a".into()),
            r.xz.map(|v| v.to_string()).unwrap_or_else(|| "n/a".into()),
            r.zstd.map(|v| v.to_string()).unwrap_or_else(|| "n/a".into()),
            r.entropy_bits,
        );
    }
    println!();
    for r in &rows {
        println!(
            "{}: nested round-trip {} | absorb k={} sizes {:?} round-trip {} | sha256(in)={} sha256(E(x))={}",
            r.name,
            if r.nested_round_trip { "exact" } else { "FAIL" },
            absorb_k,
            r.absorb_bytes,
            if r.absorb_round_trip { "exact" } else { "FAIL" },
            r.input_sha,
            r.wire_sha
        );
    }

    if let Some(dir) = out_dir {
        fs::create_dir_all(&dir).expect("create out dir");
        let mut json = String::from("{\n  \"instrument\": \"uor-r4-spiralcore-fbs-replay\",\n  \"spec\": \"SpiralCore Recursive FBS Engine v1.0, 15 October 2026\",\n");
        json.push_str(&format!(
            "  \"atom_capacity\": {ATOM_CAPACITY},\n  \"nested_steps\": {nested_n},\n  \"absorb_steps\": {absorb_k},\n  \"rows\": [\n"
        ));
        for (idx, r) in rows.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"corpus\": \"{}\",\n", r.name));
            json.push_str(&format!(
                "      \"source\": \"{}\",\n",
                r.source.replace('\\', "/")
            ));
            json.push_str(&format!("      \"input_bytes\": {},\n", r.input_bytes));
            json.push_str(&format!("      \"input_sha256\": \"{}\",\n", r.input_sha));
            json.push_str(&format!("      \"wire_bytes\": {},\n", r.wire_bytes));
            json.push_str(&format!("      \"wire_sha256\": \"{}\",\n", r.wire_sha));
            json.push_str(&format!("      \"dict_bytes\": {},\n", r.dict_bytes));
            json.push_str(&format!(
                "      \"verbatim_payload_bytes\": {},\n",
                r.verbatim_bytes
            ));
            json.push_str(&format!(
                "      \"atoms\": {}, \"carries\": {}, \"leaf_defs\": {}, \"leaf_reuse\": {}, \"parent_defs\": {}, \"parent_reuse\": {},\n",
                r.atoms, r.carries, r.leaf_defs, r.leaf_reuse, r.parent_defs, r.parent_reuse
            ));
            json.push_str(&format!("      \"round_trip_exact\": {},\n", r.round_trip));
            json.push_str(&format!("      \"nested_bytes\": {},\n", r.nested_bytes));
            json.push_str(&format!(
                "      \"nested_round_trip_exact\": {},\n",
                r.nested_round_trip
            ));
            json.push_str(&format!(
                "      \"absorb_sizes\": {:?},\n      \"absorb_round_trip_exact\": {},\n",
                r.absorb_bytes, r.absorb_round_trip
            ));
            json.push_str(&format!(
                "      \"gzip9\": {}, \"xz9\": {}, \"zstd19\": {},\n",
                r.gzip
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".into()),
                r.xz.map(|v| v.to_string()).unwrap_or_else(|| "null".into()),
                r.zstd
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "null".into())
            ));
            json.push_str(&format!(
                "      \"order0_entropy_bits\": {:.1}\n",
                r.entropy_bits
            ));
            json.push_str(if idx + 1 == rows.len() {
                "    }\n"
            } else {
                "    },\n"
            });
        }
        json.push_str("  ]\n}\n");
        fs::write(dir.join("receipt.json"), json).expect("write receipt");
        println!("\nwrote {}", dir.join("receipt.json").display());
    }
}
