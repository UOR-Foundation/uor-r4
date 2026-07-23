//! From-text observation driver tests (issue #72): the observation
//! manifest stays byte-compatible with the generation path, and the
//! `observe-text` CLI turns the sealed natural-text corpus (or a small
//! synthesized stand-in) into sharded v3 observation records with the D3
//! split rule applied at write time. The CLI smoke runs against the
//! legacy llama2.c fixture checkpoint when present and skips with a note
//! otherwise.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use uor_r4_core::transformerless::command;
use uor_r4_core::transformerless::compiler::load_corpus_from;
use uor_r4_core::transformerless::observe::{
    merge_shards, shard_file_name, ObservationManifest, PartitionCounts, RecordPartition,
    ShardEntry, RECORD_SIZE,
};
use uor_r4_core::transformerless::observe_text::{
    partition_of, StoryIndex, PARTITION_RULE, STORIES_FILE,
};

const LEGACY_CHECKPOINT: &str = "/tmp/ref/out/model.bin";
const LEGACY_TOKENIZER: &str = "/tmp/ref/tokenizer.bin";
const SEALED_CORPUS: &str = ".uor-models/corpora/simple-wiki-20231101/articles.jsonl";

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("uor-r4-observe-text-cli-{name}-{nanos}"))
}

// ------------------------------------------------- manifest byte-compat --

/// The partition extension is additive: a manifest without partition
/// information serializes exactly as the generation path always has, and
/// the legacy bytes parse into the extended struct.
#[test]
fn manifest_without_partitions_is_byte_compatible() {
    let mut manifest = ObservationManifest::new(3);
    manifest.completed.insert(
        2,
        ShardEntry {
            records: 2,
            kappa: "blake3:x".to_owned(),
            partitions: None,
        },
    );
    manifest.total_records = 2;
    let bytes = serde_json::to_vec_pretty(&manifest).expect("serialize");
    let expected = r#"{
  "schema": 1,
  "shard_bits": 3,
  "completed": {
    "2": {
      "records": 2,
      "kappa": "blake3:x"
    }
  },
  "total_records": 2
}"#;
    assert_eq!(
        String::from_utf8(bytes).expect("utf-8"),
        expected,
        "generation-path manifest bytes changed"
    );

    let parsed: ObservationManifest =
        serde_json::from_str(expected).expect("legacy manifest parses");
    assert_eq!(parsed.partition_rule, None);
    assert_eq!(parsed.completed[&2].partitions, None);

    // The partitioned form round-trips with the rule and per-shard counts.
    let mut partitioned = ObservationManifest::new(3);
    partitioned.partition_rule = Some(PARTITION_RULE.to_owned());
    partitioned.completed.insert(
        5,
        ShardEntry {
            records: 3,
            kappa: "blake3:y".to_owned(),
            partitions: Some(PartitionCounts {
                construction: 2,
                held_out: 1,
            }),
        },
    );
    partitioned.total_records = 3;
    let bytes = serde_json::to_vec_pretty(&partitioned).expect("serialize partitioned");
    let parsed: ObservationManifest =
        serde_json::from_slice(&bytes).expect("partitioned manifest parses");
    assert_eq!(parsed, partitioned);
    let text = String::from_utf8(bytes).expect("utf-8");
    assert!(text.contains("partition_rule"));
    assert!(text.contains("\"construction\": 2"));
    assert!(text.contains("\"held_out\": 1"));
}

// ------------------------------------------------------------- CLI smoke --

/// Ten short articles: a char-safe prefix of the sealed D3 corpus when
/// present, otherwise a synthesized stand-in. Short texts keep the debug
/// teacher fast; the release smoke run uses full articles.
fn smoke_articles() -> Option<Vec<String>> {
    if let Ok(corpus) = std::fs::read_to_string(SEALED_CORPUS) {
        let mut lines = Vec::new();
        for line in corpus.lines().take(10) {
            let mut article: serde_json::Value = serde_json::from_str(line).expect("article json");
            let text = article["text"].as_str().expect("article text");
            let prefix: String = text.chars().take(120).collect();
            article["text"] = serde_json::Value::String(prefix);
            lines.push(serde_json::to_string(&article).expect("article line"));
        }
        return Some(lines);
    }
    let texts = [
        "The cat sat on the mat and looked at the dog with great care.",
        "A little dog ran across the park chasing a red ball all day.",
        "Mary had a small lamb whose fleece was white as the new snow.",
        "The sun rises in the east and sets in the west every single day.",
        "One day a bird found a shiny key hidden under the garden bench.",
        "Tom saw a big truck outside his house early in the morning.",
        "Water is made of hydrogen and oxygen in every small drop we drink.",
        "The school opens at eight and the children arrive by the big gate.",
        "A farmer grows wheat and corn in the field behind the old barn.",
        "The library has many books about ships and the sea on the shelf.",
    ];
    let lines = texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            format!(
                "{{\"id\":\"smoke-{index}\",\"url\":\"https://example.test/{index}\",\"title\":\"Smoke {index}\",\"text\":{}}}",
                serde_json::to_string(text).expect("text json")
            )
        })
        .collect();
    Some(lines)
}

fn directory_fingerprint(dir: &Path) -> String {
    let mut hasher = blake3::Hasher::new();
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .expect("read dir")
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            continue;
        }
        hasher.update(entry.file_name().expect("file name").as_encoded_bytes());
        hasher.update(&std::fs::read(&entry).expect("file bytes"));
    }
    hasher.finalize().to_hex().to_string()
}

#[test]
fn observe_text_cli_smoke_with_legacy_checkpoint() {
    if std::fs::metadata(LEGACY_CHECKPOINT).is_err() || std::fs::metadata(LEGACY_TOKENIZER).is_err()
    {
        eprintln!(
            "skipping: legacy checkpoint or tokenizer not found at {LEGACY_CHECKPOINT} / {LEGACY_TOKENIZER}"
        );
        return;
    }
    let Some(lines) = smoke_articles() else {
        eprintln!("skipping: no smoke articles available");
        return;
    };
    let input = unique_path("articles.jsonl");
    std::fs::write(&input, lines.join("\n") + "\n").expect("write smoke articles");
    let article_ids: Vec<String> = lines
        .iter()
        .map(|line| {
            let article: serde_json::Value = serde_json::from_str(line).expect("article json");
            article["id"].as_str().expect("article id").to_owned()
        })
        .collect();
    let dir = unique_path("smoke");
    let args: Vec<String> = [
        "observe-text",
        "--input",
        input.to_str().expect("utf-8 input path"),
        "--checkpoint",
        LEGACY_CHECKPOINT,
        "--tokenizer",
        LEGACY_TOKENIZER,
        "--out",
        dir.to_str().expect("utf-8 temp path"),
        "--shards",
        "3",
        "--seconds",
        "120",
    ]
    .iter()
    .map(|arg| (*arg).to_string())
    .collect();
    command::run(&args).expect("observe-text run 1");

    // Shards + manifest produced; the rule is recorded.
    let manifest = ObservationManifest::load(&dir)
        .expect("manifest io")
        .expect("manifest present");
    assert_eq!(manifest.shard_bits, 3);
    assert_eq!(manifest.completed.len(), 8, "all shards finalized");
    assert_eq!(manifest.partition_rule.as_deref(), Some(PARTITION_RULE));
    assert!(manifest.total_records > 0);

    // The story mapping covers every article with the rule's partition.
    let index = StoryIndex::load(&dir.join(STORIES_FILE))
        .expect("story mapping io")
        .expect("story mapping");
    assert_eq!(index.len(), article_ids.len());
    let (mut construction_articles, mut held_out_articles) = (0u64, 0u64);
    for (ordinal, id) in article_ids.iter().enumerate() {
        let entry = index.get(ordinal as u32).expect("story entry");
        assert_eq!(entry.id, *id);
        assert_eq!(entry.partition, partition_of(id));
        if entry.partition == RecordPartition::Construction {
            construction_articles += 1;
        } else {
            held_out_articles += 1;
        }
    }
    assert!(
        held_out_articles > 0,
        "smoke must exercise held-out articles"
    );
    assert!(construction_articles > 0);

    // Partition counts in each shard entry match the rule applied to
    // stories.jsonl, recounted record by record.
    let (mut construction_records, mut held_out_records) = (0u64, 0u64);
    for shard in 0..8u32 {
        let bytes = std::fs::read(dir.join(shard_file_name(3, shard))).expect("shard file bytes");
        assert_eq!(bytes.len() % RECORD_SIZE, 0);
        let (mut shard_construction, mut shard_held_out) = (0u64, 0u64);
        for record in bytes.chunks_exact(RECORD_SIZE) {
            let story = u32::from_le_bytes(record[0..4].try_into().expect("story field"));
            match index.partition_of(story) {
                Some(RecordPartition::Construction) => shard_construction += 1,
                Some(RecordPartition::HeldOut) => shard_held_out += 1,
                None => panic!("record story {story} missing from the story mapping"),
            }
        }
        let entry = manifest.completed.get(&shard).expect("shard entry");
        let partitions = entry.partitions.expect("partition counts recorded");
        assert_eq!(
            (partitions.construction, partitions.held_out),
            (shard_construction, shard_held_out),
            "shard {shard} partition counts"
        );
        assert_eq!(partitions.total(), entry.records);
        construction_records += partitions.construction;
        held_out_records += partitions.held_out;
    }
    assert_eq!(
        construction_records + held_out_records,
        manifest.total_records
    );

    // Records parse via the shared v3 corpus loader.
    let merged = merge_shards(&dir).expect("merge");
    assert_eq!(
        merged.len() as u64,
        manifest.total_records * RECORD_SIZE as u64
    );
    let meta = unique_path("corpus.meta");
    let recs = unique_path("corpus.records");
    let mut header = [0u8; 25];
    header[0..8].copy_from_slice(&manifest.total_records.to_le_bytes());
    header[8..16].copy_from_slice(&(article_ids.len() as u64).to_le_bytes());
    header[24] = 1;
    std::fs::write(&meta, header).expect("meta");
    std::fs::write(&recs, &merged).expect("recs");
    let corpus = load_corpus_from(
        meta.to_str().expect("meta utf-8"),
        recs.to_str().expect("recs utf-8"),
    )
    .expect("observation records parse as a v3 corpus");
    assert_eq!(corpus.n, manifest.total_records as usize);
    assert_eq!(corpus.stories, article_ids.len() as u64);

    // Byte anchors chain within each story: dense spans from 0 and
    // contiguous byte ranges (real token byte lengths, not u32::MAX).
    let mut stories: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for position in 0..corpus.n {
        stories
            .entry(corpus.story[position])
            .or_default()
            .push(position);
    }
    for (story, positions) in &stories {
        let mut positions = positions.clone();
        positions.sort_by_key(|&position| corpus.span_start[position]);
        for (rank, &position) in positions.iter().enumerate() {
            assert_eq!(corpus.span_start[position] as usize, rank, "story {story}");
            assert_ne!(corpus.byte_start[position], u32::MAX, "story {story}");
            if rank > 0 {
                let previous = positions[rank - 1];
                assert_eq!(
                    corpus.byte_start[position], corpus.byte_end[previous],
                    "story {story} byte anchors must chain"
                );
            } else {
                assert_eq!(corpus.byte_start[position], 0, "story {story}");
            }
        }
    }

    // Rerun is idempotent: no byte changes anywhere in the directory.
    let fingerprint = directory_fingerprint(&dir);
    command::run(&args).expect("observe-text run 2");
    assert_eq!(
        directory_fingerprint(&dir),
        fingerprint,
        "completed observation directory changed on rerun"
    );

    // A held-out-only merge contains only records whose story ids the
    // rule marks held-out.
    let held_out_merged: Vec<&[u8]> = merged
        .chunks_exact(RECORD_SIZE)
        .filter(|record| {
            let story = u32::from_le_bytes(record[0..4].try_into().expect("story field"));
            index.partition_of(story) == Some(RecordPartition::HeldOut)
        })
        .collect();
    assert_eq!(held_out_merged.len() as u64, held_out_records);
    for record in &held_out_merged {
        let story = u32::from_le_bytes(record[0..4].try_into().expect("story field"));
        let entry = index.get(story).expect("story entry");
        assert_eq!(partition_of(&entry.id), RecordPartition::HeldOut);
    }

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&meta);
    let _ = std::fs::remove_file(&recs);
}
