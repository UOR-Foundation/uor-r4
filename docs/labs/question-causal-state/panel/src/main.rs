use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, io::Write, path::Path};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const TRAIN: &str = "bfbf581256a1ef332c005134cfaeefc3bcd0d9653281fa248cab84c0bc0bfc24";
const TOK: &str = "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const U: &[u32] = &[55, 2728, 28];
const A: &[u32] = &[201, 35, 560, 652, 714, 28];
const QUESTIONS: [[&str; 2]; 5] = [
    ["What is my current job?", "Where do I currently live?"],
    [
        "Which position do I work in?",
        "Which place do I call home?",
    ],
    [
        "Give the occupation I mentioned earlier.",
        "Give the place I said I inhabit.",
    ],
    [
        "Leave out my address. What work do I do?",
        "Leave out my occupation. Which place houses me?",
    ],
    [
        "Tell me what I do for work. Omit where I live.",
        "Tell me the place I live. Omit what I do for work.",
    ],
];
const VALUES: [[&str; 2]; 4] = [
    ["amber ridge", "cedar bay"],
    ["silver brook", "copper grove"],
    ["maple field", "willow glen"],
    ["pine valley", "open hill"],
];
fn checked(path: &str, sum: &str) -> Result<Vec<u8>> {
    let b = fs::read(path)?;
    if format!("{:x}", Sha256::digest(&b)) != sum {
        return Err("input digest mismatch".into());
    }
    Ok(b)
}
fn save(dir: &Path, name: &str, v: &Value) -> Result<()> {
    let mut f = fs::File::options()
        .create_new(true)
        .write(true)
        .open(dir.join(name))?;
    serde_json::to_writer(&mut f, v)?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [tokenizer, stream, dir] = args.as_slice() else {
        return Err("TOKENIZER TRAIN_UORT FRESH_DIR".into());
    };
    let codec = ByteBpeTokenizer::from_tokenizer_json_bytes(&checked(tokenizer, TOK)?)
        .ok_or("invalid tokenizer")?;
    let bytes = checked(stream, TRAIN)?;
    if bytes.get(..4) != Some(b"UORT") {
        return Err("bad UORT".into());
    }
    let train = bytes[64..]
        .chunks_exact(2)
        .map(|v| u32::from(u16::from_le_bytes([v[0], v[1]])))
        .collect::<Vec<_>>();
    let mut inputs = vec![];
    let mut references = vec![];
    let mut strata = vec![];
    let mut collisions = vec![];
    for (family, variants) in QUESTIONS.iter().enumerate() {
        let questions = variants.map(|text| codec.encode(text));
        for (role, q) in questions.iter().enumerate() {
            let n = train
                .windows(q.len())
                .filter(|w| *w == q.as_slice())
                .count();
            if (family == 0) != (n > 0) || codec.decode(q) != variants[role] {
                return Err(
                    format!("wording gate failed family={family} role={role} matches={n}").into(),
                );
            }
            collisions.push(
                json!({"family":family,"role":role,"full_query_occurrences_in_reader_train":n}),
            );
        }
        for (values, pair) in VALUES.iter().enumerate() {
            for swap in 0..2 {
                for order in 0..2 {
                    let by_role = [pair[swap], pair[1 - swap]];
                    let facts = [
                        format!("My current job is {}.", by_role[0]),
                        format!("I currently live in {}.", by_role[1]),
                    ];
                    for role in 0..2 {
                        let id = format!("w{:04}", inputs.len());
                        let mut prompt = vec![0];
                        for f in [order, 1 - order] {
                            prompt.extend(U);
                            prompt.extend(codec.encode(&facts[f]));
                            prompt.extend(A);
                            prompt.extend([1, 201]);
                        }
                        prompt.extend(U);
                        prompt.extend(&questions[role]);
                        prompt.extend(A);
                        let opening = [" Your job is", " You live in"][role];
                        let answer = format!("{opening} {}.", by_role[role]);
                        let mut gold = codec.encode(&answer);
                        let lead = codec.encode(opening);
                        if !gold.starts_with(&lead) || codec.decode(&gold) != answer {
                            return Err("gold encoding gate".into());
                        }
                        gold.push(1);
                        inputs.push(json!({"id":id,"prompt":prompt}));
                        references.push(json!({"id":id,"reference":gold}));
                        strata.push(json!({"id":id,"family":family,"role":role,"values":values,"swap":swap,"order":order,"value_offset":lead.len(),"fact_text":[facts[order],facts[1-order]],"question":variants[role],"answer":answer}));
                    }
                }
            }
        }
    }
    let dir = Path::new(dir);
    fs::create_dir(dir)?;
    save(dir, "prompts.json", &json!(inputs))?;
    save(dir, "reference.json", &json!(references))?;
    save(dir, "strata.json", &json!(strata))?;
    save(dir, "training-overlap.json", &json!(collisions))?;
    println!(
        "{} total, 32 known-wording controls, 128 unseen-wording rows",
        inputs.len()
    );
    Ok(())
}
