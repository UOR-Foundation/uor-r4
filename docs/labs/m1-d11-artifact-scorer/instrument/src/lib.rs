//! Stage 1 of `m1-d11-artifact-scorer` (#3248, claude-116; ORDER
//! `docs/loop/tasks/m1-d11-artifact-scorer/claude-116/ORDER.md`): the D11
//! exporter CLI and the DEV-panel row builder that M1 step 6 needs.
//!
//! `export` turns a saved discrete checkpoint into a `UORLUT01` file through the
//! crate's own `export_d11`, and checks that the clean-room integer reader
//! accepts it. `rows` rebuilds exactly the (prefix, target) rows that
//! `discrete-stack eval-corpus` scores on a config's DEV split, by the same
//! public batch walk. Both are host data preparation (D24): neither computes a
//! model quantity.
use serde_json::{json, Value};
use uor_r4_discrete::corpus::{sha256, PreparedCorpus};
use uor_r4_discrete::d11_export::export_d11;
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::training::{DriverConfig, CHECKPOINT_SCHEMA};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

/// A `UORLUT01` artifact and its export report.
pub struct Exported {
    pub bytes: Vec<u8>,
    pub report: Value,
}

/// Exports a snapshot and confirms that the clean-room integer reader accepts
/// the bytes. The bytes are `export_d11`'s, unchanged.
pub fn export_snapshot(snapshot: &Snapshot) -> Result<Exported, String> {
    let bytes =
        export_d11(snapshot).map_err(|e| format!("export_d11 refused the snapshot: {e}"))?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)
        .map_err(|e| format!("the integer reader refused the export: {e}"))?;
    let shape = serde_json::to_value(model.shape()).map_err(|e| e.to_string())?;
    let report = json!({
        "schema": "uor-r4.d11-scorer-export/1",
        "file_sha256": sha256(&bytes),
        "artifact_sha256": model.artifact_sha256(),
        "bytes": bytes.len(),
        "read_mode": format!("{:?}", model.read_mode()),
        "shape": shape,
    });
    Ok(Exported { bytes, report })
}

/// Exports a `DriverCheckpoint` JSON (`training.rs:469`). The checkpoint's
/// schema, its model storage hash (as `training.rs:905` checks it) and its
/// cursor's accepted updates against the snapshot epoch are all checked first.
pub fn export_checkpoint(checkpoint: &Value) -> Result<Exported, String> {
    if checkpoint["schema"] != CHECKPOINT_SCHEMA {
        return Err(format!("not a {CHECKPOINT_SCHEMA} checkpoint"));
    }
    let model = checkpoint.get("model").ok_or("checkpoint lacks model")?;
    let stored = checkpoint["model_sha256"]
        .as_str()
        .ok_or("checkpoint lacks model_sha256")?;
    let model_bytes = serde_json::to_vec(model).map_err(|e| e.to_string())?;
    if sha256(&model_bytes) != stored {
        return Err("checkpoint model storage hash mismatch".into());
    }
    let accepted = checkpoint["cursor"]["accepted_updates"]
        .as_u64()
        .ok_or("checkpoint lacks cursor.accepted_updates")?;
    let snapshot: Snapshot =
        serde_json::from_value(model.clone()).map_err(|e| format!("checkpoint model: {e}"))?;
    if snapshot.epoch != accepted {
        return Err(format!(
            "snapshot epoch {} differs from the cursor's {accepted} accepted updates",
            snapshot.epoch
        ));
    }
    let mut exported = export_snapshot(&snapshot)?;
    exported.report["checkpoint_model_sha256"] = json!(stored);
    exported.report["accepted_updates"] = json!(accepted);
    Ok(exported)
}

/// One scored DEV row: the causal prefix the evaluator reads and its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevRow {
    pub ordinal: u64,
    pub target_position: u64,
    pub target_token: u32,
    pub prefix: Vec<u32>,
}

/// The rows `eval-corpus` scores on a corpus: its own loop, `first = 0`, then
/// `batch(first, context, batch_rows)` until the eligible targets run out,
/// advancing by `next_target` (`training.rs:3411-3428`).
pub fn rows_from_corpus(
    corpus: &PreparedCorpus,
    context: u32,
    batch_rows: u32,
) -> Result<Vec<DevRow>, String> {
    let mut rows = Vec::new();
    let mut first = 0u64;
    while first < corpus.eligible_len() as u64 {
        let batch = corpus
            .batch(first, context, batch_rows)
            .map_err(|e| e.to_string())?;
        for ((window, &target), &position) in batch
            .windows
            .iter()
            .zip(&batch.targets)
            .zip(&batch.target_positions)
        {
            rows.push(DevRow {
                ordinal: rows.len() as u64,
                target_position: position,
                target_token: target,
                prefix: window.iter().map(|identity| identity.token).collect(),
            });
        }
        if batch.next_target <= first {
            return Err("the corpus batch cursor did not advance".into());
        }
        first = batch.next_target;
    }
    Ok(rows)
}

/// A driver config's DEV rows, loaded and pinned exactly as the driver loads
/// them (`training.rs:499`).
pub fn dev_rows(config: &DriverConfig) -> Result<Vec<DevRow>, String> {
    config.validate().map_err(|e| e.to_string())?;
    let dev = PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    rows_from_corpus(&dev, config.spec.context, config.batch_rows)
}

/// sha256 over every row as little-endian `ordinal u64, target_position u64,
/// target_token u32, prefix_len u32, prefix u32 x prefix_len`.
pub fn rows_sha256(rows: &[DevRow]) -> String {
    let mut bytes = Vec::new();
    for row in rows {
        bytes.extend_from_slice(&row.ordinal.to_le_bytes());
        bytes.extend_from_slice(&row.target_position.to_le_bytes());
        bytes.extend_from_slice(&row.target_token.to_le_bytes());
        bytes.extend_from_slice(&(row.prefix.len() as u32).to_le_bytes());
        for token in &row.prefix {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
    }
    sha256(&bytes)
}

/// The row report: the count, the hash, and per-row metadata (not prefixes).
pub fn rows_report(rows: &[DevRow]) -> Value {
    let listed: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({"ordinal": row.ordinal, "target_position": row.target_position,
                   "target_token": row.target_token, "prefix_len": row.prefix.len()})
        })
        .collect();
    json!({
        "schema": "uor-r4.d11-scorer-rows/1",
        "rows": rows.len(),
        "rows_sha256": rows_sha256(rows),
        "row_metadata": listed,
    })
}
