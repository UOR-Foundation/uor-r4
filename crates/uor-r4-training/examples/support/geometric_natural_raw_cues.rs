//! Shared exact raw-utterance/cue-to-stored-source provenance checks.
use super::*;

pub(crate) fn validate_raw_cues(
    root: &Path,
    es: &[Episode],
    tok: &ByteBpeTokenizer,
    tokenizer_sha: &str,
) -> Result<usize> {
    let report = read_json(&root.join("report.json"))?;
    let receipt_path = root.join("raw-cue-provenance.json");
    if report["raw_cue_provenance_sha256"] != sha256_file(&receipt_path)? {
        return Err(invalid("raw cue provenance SHA differs").into());
    }
    let receipt = read_json(&receipt_path)?;
    if receipt["schema"] != "uor-r4.raw-assertion-cue-provenance/1" {
        return Err(invalid("raw cue provenance schema differs").into());
    }
    let rows = receipt["rows"]
        .as_array()
        .ok_or_else(|| invalid("cue provenance rows absent"))?;
    let mut used = BTreeSet::new();
    let mut count = 0;
    for e in es {
        let segments = e.segments()?;
        for (i, segment) in segments.iter().enumerate() {
            if let SourceBankSegment::Context {
                token_ids, event, ..
            } = segment
            {
                let matching = rows
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| {
                        r["case_id"] == e.packet.id
                            && r["context_segment_index"].as_u64() == Some(i as u64)
                    })
                    .collect::<Vec<_>>();
                if matching.len() != 1 {
                    return Err(
                        invalid("each raw Context needs one recorded-assertion receipt").into(),
                    );
                }
                let (j, r) = matching[0];
                if !used.insert(j) {
                    return Err(invalid("duplicate cue receipt admission").into());
                }
                let text = r["utterance_utf8"]
                    .as_str()
                    .ok_or_else(|| invalid("original utterance absent"))?;
                if r["utterance_sha256"] != sha256_bytes(text.as_bytes())
                    || r["tokenizer_sha256"] != tokenizer_sha
                    || tok.encode(text).as_slice() != *token_ids
                    || r["context_token_ids"] != json!(token_ids)
                    || r["source_event"].as_u64() != Some(*event)
                {
                    return Err(invalid("original cue byte/BPE/event binding differs").into());
                }
                match segments.get(i + 1) {
                    Some(SourceBankSegment::Source {
                        frame,
                        event: source_event,
                        ..
                    }) => {
                        if r["source_segment_index"].as_u64() != Some((i + 1) as u64)
                            || r["record"].as_u64() != Some(frame.identity.record)
                            || r["commit"].as_u64() != Some(frame.identity.commit)
                            || r["stored_original_source_ids"] != json!(frame.token_ids)
                            || *source_event != *event
                        {
                            return Err(invalid(
                                "cue-to-actual-following-store-record receipt differs",
                            )
                            .into());
                        }
                    }
                    _ => return Err(invalid(
                        "natural reader cue Context must immediately precede actual stored Source",
                    )
                    .into()),
                }
                count += 1;
            }
        }
    }
    if count != rows.len() {
        return Err(invalid("extra or omitted cue receipt rows").into());
    }
    Ok(count)
}
