//! End-to-end: `generate`, resume, a torn `.part` shard, `seal` and `verify` against a
//! mock OpenAI-compatible server on the loopback address, through the functions `main` calls.

use m2_donor_corpus::{cmd_generate, cmd_seal, cmd_verify, Args};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const BASE_SEED: u64 = 77;

fn reply(body: &Value) -> Value {
    let msgs = body["messages"].as_array().unwrap();
    let last = msgs.last().unwrap()["content"].as_str().unwrap();
    let seed = body["seed"].as_u64().unwrap();
    let mut answer = format!("ans[{}] s{seed}: {last}", msgs.len());
    if last.contains("MARK") {
        answer.push_str(" <think>");
    }
    let finish = if last.contains("LEN") {
        "length"
    } else {
        "stop"
    };
    let mut message = json!({"role": "assistant", "content": answer});
    if !last.contains("NORS") {
        let field = if seed.is_multiple_of(2) {
            "reasoning_content"
        } else {
            "reasoning"
        };
        message[field] = json!(format!("why {last}"));
    }
    json!({"choices": [{"index": 0, "finish_reason": finish, "message": message}],
        "usage": {"completion_tokens": 10 + last.len(), "prompt_tokens": 5}})
}

fn serve(stream: TcpStream) {
    let mut r = BufReader::new(stream.try_clone().unwrap());
    let mut w = stream;
    loop {
        let mut len = 0usize;
        let mut first = true;
        loop {
            let mut line = String::new();
            if r.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
            if first {
                assert!(line.starts_with("POST /v1/chat/completions"), "{line}");
                first = false;
            }
            let l = line.to_ascii_lowercase();
            if let Some(v) = l.strip_prefix("content-length:") {
                len = v.trim().parse().unwrap();
            }
            if line == "\r\n" {
                break;
            }
        }
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).unwrap();
        let out = reply(&serde_json::from_slice(&body).unwrap()).to_string();
        let head = concat!("HTTP/1", ".1 200 OK\r\nContent-Type: application/json\r\n");
        let msg = format!("{head}Content-Length: {}\r\n\r\n{out}", out.len());
        if w.write_all(msg.as_bytes()).is_err() {
            return;
        }
    }
}

fn mock_server() -> String {
    let l = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let addr = l.local_addr().unwrap();
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            std::thread::spawn(move || serve(s));
        }
    });
    format!("http://{addr}")
}

fn bank_line(i: usize) -> String {
    let mut t0 = format!("p{i} first");
    let two = i.is_multiple_of(3);
    if i % 7 == 3 {
        t0.push_str(" LEN");
    }
    if i % 5 == 4 {
        t0.push_str(" NORS");
    }
    let mut msgs = vec![json!({"role": "user", "content": t0})];
    if two {
        msgs.push(json!({"role": "user", "content": format!("p{i} second")}));
    }
    if i % 11 == 5 {
        let last = msgs.last_mut().unwrap();
        last["content"] = json!(format!("{} MARK", last["content"].as_str().unwrap()));
    }
    let kind = if two { "two-turn" } else { "single" };
    // Spaced and role-first on purpose: records must copy these bytes exactly.
    let raw: Vec<String> = msgs
        .iter()
        .map(|m| format!("{{\"role\": \"user\", \"content\": {}}}", m["content"]))
        .collect();
    format!(
        "{{\"id\": \"q{i:02}\", \"kind\": \"{kind}\", \"source\": \"src{}\", \"messages\": [{}]}}",
        i % 2,
        raw.join(", ")
    )
}

fn args(pairs: &[(&str, String)]) -> Args {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

fn lines(p: &Path) -> Vec<Value> {
    fs::read_to_string(p)
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn files(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            n.starts_with(prefix) && n.ends_with(".jsonl")
        })
        .collect();
    v.sort();
    v
}

/// Ids in shards (checked against counts lines) and in dropped.jsonl.
fn census(dir: &Path) -> (Vec<String>, Vec<String>, u64) {
    let (mut kept, mut kept_tokens) = (vec![], 0u64);
    let shards = files(dir, "shard-");
    let counts = files(dir, "counts-");
    assert_eq!(shards.len(), counts.len());
    for (s, c) in shards.iter().zip(&counts) {
        let (sl, cl) = (lines(s), lines(c));
        assert_eq!(sl.len(), cl.len());
        for (r, k) in sl.iter().zip(&cl) {
            assert_eq!(r["id"], k["id"]);
            kept.push(r["id"].as_str().unwrap().to_string());
            kept_tokens += k["kept_tokens"].as_u64().unwrap();
        }
    }
    let dropped = lines(&dir.join("dropped.jsonl"))
        .iter()
        .map(|d| d["id"].as_str().unwrap().to_string())
        .collect();
    (kept, dropped, kept_tokens)
}

#[test]
fn generate_resume_seal_verify_end_to_end() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("e2e-donor-corpus");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let dir = root.join("run");
    let bank = root.join("bank.jsonl");
    let bank_text: String = (0..40).map(|i| bank_line(i) + "\n").collect();
    fs::write(&bank, &bank_text).unwrap();
    let server = mock_server();
    let gen = |limit: usize| {
        cmd_generate(&args(&[
            ("server", server.clone()),
            ("prompts", bank.display().to_string()),
            ("out", dir.display().to_string()),
            ("budget", "1000000".into()),
            ("instrument-commit", "testcommit".into()),
            ("parallel", "4".into()),
            ("shard-records", "8".into()),
            ("seed", BASE_SEED.to_string()),
            ("limit", limit.to_string()),
        ]))
        .unwrap()
    };

    // Segment one: the first 20 prompts.
    let seg = gen(20);
    assert_eq!(seg["stopped_by"], "exhausted");
    let (kept, dropped, kept_tokens) = census(&dir);
    assert_eq!(kept.len() + dropped.len(), 20);
    let reasons: HashSet<String> = lines(&dir.join("dropped.jsonl"))
        .iter()
        .map(|d| d["reason"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(reasons, ["length", "markup"].map(String::from).into());
    for id in ["q03", "q05", "q10", "q16", "q17"] {
        assert!(dropped.contains(&id.to_string()), "{id} not dropped");
    }
    let metrics: Value =
        serde_json::from_slice(&fs::read(dir.join("metrics.json")).unwrap()).unwrap();
    assert_eq!(metrics["records"], kept.len());
    assert_eq!(metrics["kept_tokens"], kept_tokens);
    assert!(metrics["empty_reasoning_turns"].as_u64().unwrap() > 0);
    assert!(files(&dir, "shard-").len() >= 2);

    // Records copy bank bytes and carry both turns with the right seeds.
    let bank_lines: Vec<&str> = bank_text.lines().collect();
    let all: Vec<Value> = files(&dir, "shard-")
        .iter()
        .flat_map(|p| lines(p))
        .collect();
    let raw_shards: String = files(&dir, "shard-")
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    let q00 = all.iter().find(|r| r["id"] == "q00").unwrap();
    let t1 = q00["turns"][1]["answer"].as_str().unwrap();
    assert!(
        t1.starts_with(&format!("ans[3] s{}:", BASE_SEED + 1)),
        "{t1}"
    );
    let q12 = all.iter().find(|r| r["id"] == "q12").unwrap();
    let want = format!("ans[1] s{}: p12 first", BASE_SEED + 24);
    assert_eq!(q12["turns"][0]["answer"], want.as_str());
    assert!(raw_shards
        .contains(&bank_lines[12][bank_lines[12].find('[').unwrap()..bank_lines[12].len() - 1]));

    // Segment two resumes: nothing repeats and the totals add up.
    gen(40);
    let (kept, dropped, kept_tokens) = census(&dir);
    let ids: HashSet<&String> = kept.iter().chain(&dropped).collect();
    assert_eq!((kept.len() + dropped.len(), ids.len()), (40, 40));
    let metrics: Value =
        serde_json::from_slice(&fs::read(dir.join("metrics.json")).unwrap()).unwrap();
    assert_eq!(metrics["kept_tokens"], kept_tokens);
    assert_eq!(metrics["records"], kept.len());
    assert_eq!(lines(&dir.join("segments.jsonl")).len(), 2);

    // A torn `.part` pair: the last shard reopened, a torn record and an extra counts line.
    let last_shard = files(&dir, "shard-").pop().unwrap();
    let last_counts = files(&dir, "counts-").pop().unwrap();
    let (s_bytes, c_bytes) = (
        fs::read(&last_shard).unwrap(),
        fs::read(&last_counts).unwrap(),
    );
    let part = |p: &Path| PathBuf::from(format!("{}.part", p.display()));
    fs::rename(&last_shard, part(&last_shard)).unwrap();
    fs::rename(&last_counts, part(&last_counts)).unwrap();
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(part(&last_shard))
        .unwrap();
    f.write_all(b"{\"id\":\"q99\",\"ki").unwrap();
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(part(&last_counts))
        .unwrap();
    f.write_all(b"{\"id\":\"q99\",\"kept_tokens\":1,\"generated_tokens\":1,\"prompt_tokens\":1}\n")
        .unwrap();
    for _ in 0..2 {
        let seg = gen(40);
        assert_eq!(seg["records_at_end"], kept.len());
        assert!(!part(&last_shard).exists() && !part(&last_counts).exists());
        assert_eq!(fs::read(&last_shard).unwrap(), s_bytes);
        assert_eq!(fs::read(&last_counts).unwrap(), c_bytes);
    }

    // Seal and verify, then corrupt one shard byte.
    for (k, v) in [
        ("model", "{\"name\": \"m\"}"),
        ("machine", "{\"gpus\": 8}"),
        ("engine", "{\"vllm\": \"x\"}"),
    ] {
        fs::write(root.join(format!("{k}.json")), v).unwrap();
    }
    let seal_args: Vec<(&str, String)> = vec![
        ("dir", dir.display().to_string()),
        ("prompts", bank.display().to_string()),
        ("model", root.join("model.json").display().to_string()),
        ("machine", root.join("machine.json").display().to_string()),
        ("engine", root.join("engine.json").display().to_string()),
    ];
    cmd_seal(&args(&seal_args)).unwrap();
    let ver = args(&seal_args[..2]);
    let v = cmd_verify(&ver).unwrap();
    assert_eq!(v["verified"], true);
    assert_eq!(v["records"], kept.len());
    assert_eq!(v["kept_tokens"], kept_tokens);
    let first = files(&dir, "shard-").remove(0);
    let mut b = fs::read(&first).unwrap();
    b[20] ^= 1;
    fs::write(&first, &b).unwrap();
    assert!(cmd_verify(&ver).is_err());
    fs::remove_dir_all(&root).unwrap();
}
