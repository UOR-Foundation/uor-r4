//! On-model M3 durable conversation memory evaluator (`m3-memory`, `#2873`).
//!
//! Executes the frozen 384-item `m3-durable-memory-panel-v1.jsonl` panel across
//! all six owner-registered sub-panels (`provenance`, `correction`, `abstention`,
//! `non_memory_prose`, `reset_isolation`, `restart_restore`) through
//! [`DurableObjectMemoryStore`], [`Store`], [`PinnedTokenizer`], and
//! [`Ur4Sess1HeaderView`], paired with a model scorer callback (`DeviceScorer`
//! on the GPU under `--features cuda`) and `no-memory` negative controls (`L-010`).
//!
//! Pure integer execution (`CHARTER.md` §3.1–§3.3, D11, D23, D30, D32): zero
//! floating-point types or operations, zero softmax or squashing functions, and
//! zero dense matrix multiplications.
#![forbid(unsafe_code)]

use crate::bindings::PinnedTokenizer;
use crate::durable_consumer::{
    DurableObjectMemoryStore, FactAction, FactReadOutcome, IdentityScope, Ur4Sess1HeaderView,
};
use crate::{Error, Limits, Manifest, Record, RecordBackend, Result, Store};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use uor_r4_tokenizer::ByteBpeTokenizer;

/// Versioned report schema for the on-model M3 durable-memory evaluation.
pub const M3_REPORT_SCHEMA: &str = "uor-r4.m3-durable-memory-report/1";
/// Versioned identity schema for the on-model M3 durable-memory evaluation.
pub const M3_IDENTITY_SCHEMA: &str = "uor-r4.m3-durable-memory-identity/1";

/// Pinned whole-object SHA-256 of `docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl`.
pub const PANEL_SHA256: &str = "582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1";
/// Pinned `Manifest::version()` (hex) of `m3-durable-memory-panel-v1.jsonl` folded into [`Store`].
pub const PANEL_MANIFEST_VERSION: &str =
    "88c27b94526c32b0f06fed7fabeb7a2abcf0b31a2f2534517b8656240eddce70";
/// Total items in `m3-durable-memory-panel-v1.jsonl`.
pub const PANEL_ITEMS: usize = 384;
/// Items per sub-panel in `m3-durable-memory-panel-v1.jsonl`.
pub const ITEMS_PER_SUBPANEL: usize = 64;
/// Greedy continuation length per item in `restart_restore`.
pub const RESTART_CONTINUE_TOKENS: usize = 32;

/// Pinned tokenizer file SHA-256 (`tokenizer.json`, vocabulary 4,096).
pub const TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";

/// Selected M1 Seed 1 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) artifact SHA-256.
pub const SEED1_ARTIFACT_SHA256: &str =
    "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
/// Selected M1 Seed 2 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) artifact SHA-256.
pub const SEED2_ARTIFACT_SHA256: &str =
    "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de";

/// Pinned `ft/dev` tokens SHA-256 for Gate R parity verification.
pub const FT_DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
/// Pinned `ft/dev` mask SHA-256 for Gate R parity verification.
pub const FT_DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
/// Total scored rows on `ft/dev`.
pub const FT_DEV_ROWS: usize = 14_287;

/// Expected Seed 1 `ft/dev` Gate R top-1 count (`t16-frozen-zero-seed1-trunk.d11`).
pub const SEED1_EXPECTED_DEV_TOP1: u64 = 7_002;
/// Expected Seed 1 `ft/dev` Gate R top-8 count (`t16-frozen-zero-seed1-trunk.d11`).
pub const SEED1_EXPECTED_DEV_TOP8: u64 = 7_485;
/// Expected Seed 1 `ft/dev` Gate R target-admitted count (`t16-frozen-zero-seed1-trunk.d11`).
pub const SEED1_EXPECTED_DEV_ADMITTED: u64 = 9_012;

/// Expected Seed 2 `ft/dev` Gate R top-1 count (`t16-frozen-zero-seed2-trunk.d11`).
pub const SEED2_EXPECTED_DEV_TOP1: u64 = 7_001;
/// Expected Seed 2 `ft/dev` Gate R top-8 count (`t16-frozen-zero-seed2-trunk.d11`).
pub const SEED2_EXPECTED_DEV_TOP8: u64 = 7_478;
/// Expected Seed 2 `ft/dev` Gate R target-admitted count (`t16-frozen-zero-seed2-trunk.d11`).
pub const SEED2_EXPECTED_DEV_ADMITTED: u64 = 9_010;

/// Owner-registered acceptance floor for `provenance` (`58/64`, `>= 0.90`).
pub const PROVENANCE_MIN_HITS: usize = 58;
/// Owner-registered acceptance floor for `correction` current value (`58/64`, `>= 0.90`).
pub const CORRECTION_CURRENT_MIN_HITS: usize = 58;
/// Owner-registered acceptance floor for `correction` prior lineage (`55/64`, `>= 0.85`).
pub const CORRECTION_PRIOR_MIN_HITS: usize = 55;
/// Owner-registered acceptance floor for `abstention` refusal (`122/128`, `>= 0.95`).
pub const ABSTENTION_MIN_REFUSALS: usize = 122;
/// Owner-registered R14 floor for `AnsweredFactHitRate` in basis points (`5000 bps = 0.50`).
pub const R14_MIN_ANSWERED_FACT_HIT_BPS: u64 = 5_000;
/// Owner-registered exact requirement for `non_memory_prose` (`64/64`, `#1718`).
pub const NON_MEMORY_PROSE_EXACT_HITS: usize = 64;
/// Owner-registered exact requirement for `reset_isolation` (`64/64`).
pub const RESET_ISOLATION_EXACT_HITS: usize = 64;
/// Owner-registered exact requirement for `restart_restore` (`64/64`).
pub const RESTART_RESTORE_EXACT_HITS: usize = 64;

/// Single-row model evaluation result from the integer scorer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelStepScore {
    pub status: i32,
    pub prediction: u32,
    pub top8: Vec<u32>,
}

/// R14 `AbstentionDiscrimination` integer summary (basis points, 0..=10_000).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct R14AbstentionSummary {
    pub knowable_queries: u64,
    pub knowable_answered_hits: u64,
    pub knowable_refusals: u64,
    pub unknowable_queries: u64,
    pub unknowable_refusals: u64,
    pub answered_fact_hit_rate_bps: u64,
    pub unknowable_refusal_rate_bps: u64,
    pub knowable_refusal_rate_bps: u64,
    pub selective_abstention_gap_bps: i64,
    pub r14_pass: bool,
}

/// Planted tamper and negative-control checks for the M3 durable-memory evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3ControlChecks {
    pub cross_artifact_session_rejected: bool,
    pub corrupt_ur4sess1_magic_rejected: bool,
    pub corrupt_uormem01_digest_rejected: bool,
    pub cross_scope_manifest_rejected: bool,
    pub no_memory_zero_recall: bool,
    pub all_controls_pass: bool,
}

/// Complete integer evaluation summary across all six M3 sub-panels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3PanelEvaluation {
    pub schema: String,
    pub artifact_sha256: String,
    pub panel_sha256: String,
    pub panel_manifest_version: String,
    pub total_items: usize,
    pub provenance_hits: usize,
    pub provenance_no_memory_hits: usize,
    pub provenance_unassisted_model_hits: usize,
    pub correction_current_hits: usize,
    pub correction_prior_hits: usize,
    pub correction_no_memory_hits: usize,
    pub correction_unassisted_model_hits: usize,
    pub abstention_refusals: usize,
    pub abstention_total_queries: usize,
    pub abstention_zero_fetch_queries: usize,
    pub non_memory_prose_hits: usize,
    pub non_memory_prose_no_memory_hits: usize,
    pub reset_isolation_hits: usize,
    pub restart_restore_hits: usize,
    pub restart_restore_matched_tokens: usize,
    pub restart_restore_total_tokens: usize,
    pub restart_restore_no_memory_hits: usize,
    pub total_model_scored_steps: u64,
    pub total_elapsed_ns: u64,
    pub decode_tokens_per_second_milli: u64,
    pub r14: R14AbstentionSummary,
    pub controls: M3ControlChecks,
    pub bars_pass: bool,
    pub rows: Vec<Value>,
}

struct CountingBackend<'a> {
    store: &'a Store,
    fetches: AtomicUsize,
}

impl RecordBackend for CountingBackend<'_> {
    fn fetch(&self, id: crate::NodeId) -> Result<Option<Record>> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        self.store.fetch(id)
    }
}

/// Lowercase hex SHA-256 digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Fold `panel_bytes` into a fresh [`Store`] and verify both `PANEL_SHA256` and
/// `PANEL_MANIFEST_VERSION`.
pub fn verify_frozen_panel(panel_bytes: &[u8]) -> std::result::Result<(Manifest, Vec<Value>), String> {
    let sha = sha256_hex(panel_bytes);
    if sha != PANEL_SHA256 {
        return Err(format!(
            "panel SHA-256 {sha} differs from pinned {PANEL_SHA256}"
        ));
    }
    let mut store = Store::new(Limits::default());
    let folded = store
        .fold(panel_bytes)
        .map_err(|e| format!("fold panel: {e:?}"))?;
    let whole_sha: String = folded
        .manifest
        .whole_sha256()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let version_hex: String = folded
        .manifest
        .version()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if whole_sha != PANEL_SHA256 {
        return Err(format!(
            "folded panel whole_sha256 {whole_sha} differs from {PANEL_SHA256}"
        ));
    }
    if version_hex != PANEL_MANIFEST_VERSION {
        return Err(format!(
            "folded panel Manifest::version() {version_hex} differs from {PANEL_MANIFEST_VERSION}"
        ));
    }
    let text = std::str::from_utf8(panel_bytes).map_err(|e| format!("panel utf8: {e}"))?;
    let items: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).map_err(|e| format!("panel jsonl: {e}")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if items.len() != PANEL_ITEMS {
        return Err(format!(
            "panel has {} items, expected {PANEL_ITEMS}",
            items.len()
        ));
    }
    Ok((folded.manifest, items))
}

fn find_value_range(turn: &str, value: &str) -> std::result::Result<Range<u64>, String> {
    let pos = turn
        .find(value)
        .ok_or_else(|| format!("value {value:?} not found in turn {turn:?}"))?;
    Ok((pos as u64)..((pos + value.len()) as u64))
}

fn entity_for_index(i: usize) -> (&'static str, &'static str, &'static str, &'static str) {
    const PEOPLE: [(&str, &str, &str, &str); 16] = [
        ("Alice", "Lisbon", "port", "teal"),
        ("Bruno", "Porto", "canary", "amber"),
        ("Chen", "Taipei", "koi", "coral"),
        ("Diana", "Athens", "owl", "indigo"),
        ("Eitan", "Haifa", "cat", "olive"),
        ("Farah", "Cairo", "falcon", "saffron"),
        ("Gustav", "Vienna", "dachshund", "cream"),
        ("Hana", "Kyoto", "crane", "moss"),
        ("Ivan", "Tallinn", "lynx", "slate"),
        ("Juana", "Lima", "llama", "ochre"),
        ("Kofi", "Accra", "parrot", "kente"),
        ("Lena", "Berlin", "fox", "cobalt"),
        ("Mateo", "Bogota", "toucan", "ruby"),
        ("Nadia", "Tunis", "horse", "jade"),
        ("Oskar", "Oslo", "puffin", "ice"),
        ("Priya", "Jaipur", "peacock", "marigold"),
    ];
    PEOPLE[i % PEOPLE.len()]
}

fn bounded_prefix(tokens: &[u32], context_limit: usize) -> Vec<u32> {
    let cap = context_limit.saturating_sub(1).max(1);
    if tokens.len() > cap {
        tokens[tokens.len() - cap..].to_vec()
    } else if tokens.is_empty() {
        vec![0]
    } else {
        tokens.to_vec()
    }
}

/// Run the planted tamper and cross-artifact negative controls on [`DurableObjectMemoryStore`].
pub fn run_planted_controls(
    tokenizer: &PinnedTokenizer,
    artifact_sha256: &str,
    no_memory_zero_recall: bool,
) -> std::result::Result<M3ControlChecks, String> {
    let scope = IdentityScope::new("user-ctrl", "proj-ctrl", "sess-ctrl")
        .map_err(|e| format!("control scope: {e:?}"))?;
    let other_scope = IdentityScope::new("user-other", "proj-ctrl", "sess-ctrl")
        .map_err(|e| format!("other scope: {e:?}"))?;
    let mut store = Store::new(Limits::default());
    let mut memory = DurableObjectMemoryStore::new();
    let turn = b"Alice lives in Lisbon.";
    memory
        .assert_fact(&mut store, tokenizer, &scope, "Alice", "hometown", turn, 15..21)
        .map_err(|e| format!("control assert: {e:?}"))?;
    let ur4 = Ur4Sess1HeaderView::encode_canonical(artifact_sha256, &[1, 2, 3, 4], &[10, 20])
        .map_err(|e| format!("control encode ur4sess1: {e:?}"))?;
    memory
        .set_ur4sess1_snapshot(&scope, &ur4)
        .map_err(|e| format!("control set snapshot: {e:?}"))?;
    let manifest = memory
        .export_scope(&mut store, &scope)
        .map_err(|e| format!("control export: {e:?}"))?;

    // 1. Cross-artifact session binding must fail closed with Error::BindingMismatch
    let wrong_artifact = if artifact_sha256 == SEED1_ARTIFACT_SHA256 {
        SEED2_ARTIFACT_SHA256
    } else {
        SEED1_ARTIFACT_SHA256
    };
    let cross_artifact_session_rejected = matches!(
        memory.restore_verified_session(&scope, wrong_artifact),
        Err(Error::BindingMismatch)
    );

    // 2. Corrupt UR4SESS1 magic byte must fail closed with Error::SchemaMismatch
    let mut corrupt_ur4 = ur4.clone();
    corrupt_ur4[0] ^= 0xff;
    let corrupt_ur4sess1_magic_rejected = matches!(
        memory.set_ur4sess1_snapshot(&scope, &corrupt_ur4),
        Err(Error::SchemaMismatch)
    );

    // 3. Forged UORMEM01 manifest whole_sha256 must fail closed with Error::IntegrityFailure
    let mut raw_manifest = manifest.encode();
    raw_manifest[16] ^= 1;
    let forged_manifest =
        Manifest::decode(&raw_manifest, store.limits()).map_err(|e| format!("decode: {e:?}"))?;
    let mut fresh_mem = DurableObjectMemoryStore::new();
    let corrupt_uormem01_digest_rejected = matches!(
        fresh_mem.import_scope(&store, &forged_manifest),
        Err(Error::IntegrityFailure)
    );

    // 4. Cross-scope manifest import must fail closed with Error::BindingMismatch
    let cross_scope_manifest_rejected = matches!(
        fresh_mem.import_scope_expecting(&store, &manifest, &other_scope),
        Err(Error::BindingMismatch)
    );

    let all_controls_pass = cross_artifact_session_rejected
        && corrupt_ur4sess1_magic_rejected
        && corrupt_uormem01_digest_rejected
        && cross_scope_manifest_rejected
        && no_memory_zero_recall;

    Ok(M3ControlChecks {
        cross_artifact_session_rejected,
        corrupt_ur4sess1_magic_rejected,
        corrupt_uormem01_digest_rejected,
        cross_scope_manifest_rejected,
        no_memory_zero_recall,
        all_controls_pass,
    })
}

/// Execute the 384-item `m3-durable-memory-panel-v1.jsonl` panel on the model
/// path using `score_prefix` (`DeviceScorer` on GPU or deterministic host scorer
/// in finite tests).
pub fn evaluate_m3_panel_with_scorer(
    panel_bytes: &[u8],
    tokenizer: &ByteBpeTokenizer,
    artifact_sha256: &str,
    context_limit: usize,
    score_prefix: impl Fn(&[u32]) -> std::result::Result<ModelStepScore, String>,
) -> std::result::Result<M3PanelEvaluation, String> {
    let (_, items) = verify_frozen_panel(panel_bytes)?;
    let pinned = PinnedTokenizer::new(tokenizer).map_err(|e| format!("pin tokenizer: {e:?}"))?;

    let started = Instant::now();
    let mut prov_hits = 0usize;
    let mut prov_no_mem_hits = 0usize;
    let mut prov_unassisted_hits = 0usize;
    let mut corr_curr_hits = 0usize;
    let mut corr_prior_hits = 0usize;
    let mut corr_no_mem_hits = 0usize;
    let mut corr_unassisted_hits = 0usize;
    let mut abs_refusals = 0usize;
    let mut abs_total_queries = 0usize;
    let mut abs_zero_fetch_queries = 0usize;
    let mut prose_hits = 0usize;
    let mut prose_no_mem_hits = 0usize;
    let mut reset_iso_hits = 0usize;
    let mut restart_hits = 0usize;
    let mut restart_matched_tokens = 0usize;
    let mut restart_total_tokens = 0usize;
    let mut restart_no_mem_hits = 0usize;
    let mut total_model_scored_steps = 0u64;
    let mut rows = Vec::with_capacity(items.len());

    for (idx, item) in items.iter().enumerate() {
        let id = item["id"].as_str().ok_or("missing item id")?;
        let sub_panel = item["sub_panel"].as_str().ok_or("missing sub_panel")?;
        let mut store = Store::new(Limits::default());
        let mut memory = DurableObjectMemoryStore::new();
        let no_memory = DurableObjectMemoryStore::new();
        let scope = IdentityScope::new("user-m3", "proj-m3", format!("item-{idx}"))
            .map_err(|e| format!("scope: {e:?}"))?;

        match sub_panel {
            "provenance" => {
                let turn0 = item["turns"][0]["text"].as_str().ok_or("missing turn0")?;
                let q = &item["queries"][0];
                let owner = q["owner"].as_str().ok_or("missing owner")?;
                let relation = q["relation"].as_str().ok_or("missing relation")?;
                let expected = q["expected_value"].as_str().ok_or("missing expected_value")?;
                let ask = q["ask"].as_str().ok_or("missing ask")?;
                let range = find_value_range(turn0, expected)?;
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner,
                        relation,
                        turn0.as_bytes(),
                        range,
                    )
                    .map_err(|e| format!("{id} assert: {e:?}"))?;

                let backend = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let outcome = memory
                    .read_fact_from(&backend, store.limits(), &pinned, &scope, owner, relation)
                    .map_err(|e| format!("{id} read_fact: {e:?}"))?;
                let fetches = backend.fetches.load(Ordering::SeqCst);

                // Score the model prefix on the GPU/scorer (`turns + ask`)
                let prompt_tokens = bounded_prefix(
                    &tokenizer.encode(&format!("{turn0}\n{ask}\n")),
                    context_limit,
                );
                let step_score = score_prefix(&prompt_tokens)?;
                total_model_scored_steps += 1;
                if step_score.status != 0 {
                    return Err(format!("{id} model scorer returned nonzero status"));
                }
                let unassisted_pred_bytes = tokenizer.decode_bytes(&[step_score.prediction]);
                if unassisted_pred_bytes == expected.as_bytes() {
                    prov_unassisted_hits += 1;
                }

                let mut hit = false;
                if let FactReadOutcome::Quoted {
                    ref value_bytes,
                    ref token_ids,
                    version,
                    action,
                    conflict,
                    ..
                } = outcome
                {
                    let decoded_cover = tokenizer.decode_bytes(token_ids);
                    if value_bytes == expected.as_bytes()
                        && !token_ids.is_empty()
                        && decoded_cover
                            .windows(expected.len())
                            .any(|w| w == expected.as_bytes())
                        && version == 1
                        && action == FactAction::Assert
                        && !conflict
                        && fetches > 0
                    {
                        hit = true;
                        prov_hits += 1;
                    }
                }

                let no_mem_backend = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let no_mem_outcome = no_memory
                    .read_fact_from(
                        &no_mem_backend,
                        store.limits(),
                        &pinned,
                        &scope,
                        owner,
                        relation,
                    )
                    .map_err(|e| format!("{id} no_memory: {e:?}"))?;
                if matches!(no_mem_outcome, FactReadOutcome::Quoted { .. }) {
                    prov_no_mem_hits += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "hit": hit,
                    "backend_fetches": fetches,
                    "no_memory_fetches": no_mem_backend.fetches.load(Ordering::SeqCst),
                    "model_prediction": step_score.prediction,
                }));
            }
            "correction" => {
                let turn0 = item["turns"][0]["text"].as_str().ok_or("missing turn0")?;
                let turn1 = item["turns"][1]["text"].as_str().ok_or("missing turn1")?;
                let q_curr = &item["queries"][0];
                let q_prior = &item["queries"][1];
                let owner0 = q_prior["owner"].as_str().ok_or("missing prior owner")?;
                let val0 = q_prior["expected_value"]
                    .as_str()
                    .ok_or("missing prior expected_value")?;
                let owner1 = q_curr["owner"].as_str().ok_or("missing curr owner")?;
                let val1 = q_curr["expected_value"]
                    .as_str()
                    .ok_or("missing curr expected_value")?;
                let relation = q_curr["relation"].as_str().ok_or("missing relation")?;

                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner0,
                        relation,
                        turn0.as_bytes(),
                        find_value_range(turn0, val0)?,
                    )
                    .map_err(|e| format!("{id} assert: {e:?}"))?;
                memory
                    .revise_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner1,
                        relation,
                        turn1.as_bytes(),
                        find_value_range(turn1, val1)?,
                    )
                    .map_err(|e| format!("{id} revise: {e:?}"))?;

                let backend = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let outcome = memory
                    .read_fact_from(&backend, store.limits(), &pinned, &scope, owner1, relation)
                    .map_err(|e| format!("{id} read revised: {e:?}"))?;
                let fetches = backend.fetches.load(Ordering::SeqCst);

                let prompt_tokens = bounded_prefix(
                    &tokenizer.encode(&format!("{turn0}\n{turn1}\nWhere does {owner1} live?\n")),
                    context_limit,
                );
                let step_score = score_prefix(&prompt_tokens)?;
                total_model_scored_steps += 1;
                if step_score.status != 0 {
                    return Err(format!("{id} model scorer returned nonzero status"));
                }
                if tokenizer.decode_bytes(&[step_score.prediction]) == val1.as_bytes() {
                    corr_unassisted_hits += 1;
                }

                let mut curr_hit = false;
                let mut prior_hit = false;
                if let FactReadOutcome::Quoted {
                    ref value_bytes,
                    ref previous_value_bytes,
                    version,
                    action,
                    conflict,
                    ..
                } = outcome
                {
                    if value_bytes == val1.as_bytes()
                        && version == 2
                        && action == FactAction::Revise
                        && !conflict
                        && fetches > 0
                    {
                        curr_hit = true;
                        corr_curr_hits += 1;
                    }
                    if previous_value_bytes.as_deref() == Some(val0.as_bytes()) {
                        prior_hit = true;
                        corr_prior_hits += 1;
                    }
                }

                if matches!(
                    no_memory
                        .read_fact(&store, &pinned, &scope, owner1, relation)
                        .map_err(|e| format!("{id} no_memory: {e:?}"))?,
                    FactReadOutcome::Quoted { .. }
                ) {
                    corr_no_mem_hits += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "current_hit": curr_hit,
                    "prior_hit": prior_hit,
                    "backend_fetches": fetches,
                    "model_prediction": step_score.prediction,
                }));
            }
            "abstention" => {
                let turns = item["turns"].as_array().ok_or("missing turns")?;
                let turn0 = turns[0]["text"].as_str().ok_or("missing turn0")?;
                let (name, city, _, _) = entity_for_index(idx % ITEMS_PER_SUBPANEL);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city)?,
                    )
                    .map_err(|e| format!("{id} assert baseline: {e:?}"))?;

                let prompt_tokens =
                    bounded_prefix(&tokenizer.encode(turn0), context_limit);
                let step_score = score_prefix(&prompt_tokens)?;
                total_model_scored_steps += 1;
                if step_score.status != 0 {
                    return Err(format!("{id} model scorer returned nonzero status"));
                }

                let q0 = &item["queries"][0];
                let q0_kind = q0["kind"].as_str().ok_or("missing q0 kind")?;
                let q0_owner = q0["owner"].as_str().ok_or("missing q0 owner")?;
                let q0_rel = q0["relation"].as_str().ok_or("missing q0 relation")?;
                abs_total_queries += 1;
                let backend0 = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let q0_refused = if q0_kind == "refuse_retracted" {
                    memory
                        .retract_fact(&scope, q0_owner, q0_rel)
                        .map_err(|e| format!("{id} retract: {e:?}"))?;
                    matches!(
                        memory
                            .read_fact_from(
                                &backend0,
                                store.limits(),
                                &pinned,
                                &scope,
                                q0_owner,
                                q0_rel
                            )
                            .map_err(|e| format!("{id} read retracted: {e:?}"))?,
                        FactReadOutcome::RefusedRetracted {
                            retracted_at_version: 2,
                            ..
                        }
                    )
                } else {
                    matches!(
                        memory
                            .read_fact_from(
                                &backend0,
                                store.limits(),
                                &pinned,
                                &scope,
                                q0_owner,
                                q0_rel
                            )
                            .map_err(|e| format!("{id} read unstated: {e:?}"))?,
                        FactReadOutcome::RefusedUnstated
                    )
                };
                let fetches0 = backend0.fetches.load(Ordering::SeqCst);
                if q0_refused {
                    abs_refusals += 1;
                }
                if fetches0 == 0 {
                    abs_zero_fetch_queries += 1;
                }

                let q1 = &item["queries"][1];
                let q1_owner = q1["owner"].as_str().ok_or("missing q1 owner")?;
                let q1_rel = q1["relation"].as_str().ok_or("missing q1 relation")?;
                abs_total_queries += 1;
                let backend1 = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let q1_refused = matches!(
                    memory
                        .read_fact_from(
                            &backend1,
                            store.limits(),
                            &pinned,
                            &scope,
                            q1_owner,
                            q1_rel
                        )
                        .map_err(|e| format!("{id} read absent relation: {e:?}"))?,
                    FactReadOutcome::RefusedUnstated
                );
                let fetches1 = backend1.fetches.load(Ordering::SeqCst);
                if q1_refused {
                    abs_refusals += 1;
                }
                if fetches1 == 0 {
                    abs_zero_fetch_queries += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "q0_refused": q0_refused,
                    "q1_refused": q1_refused,
                    "backend_fetches": fetches0 + fetches1,
                    "model_prediction": step_score.prediction,
                }));
            }
            "non_memory_prose" => {
                let turns = item["turns"].as_array().ok_or("missing turns")?;
                let turn0 = turns[0]["text"].as_str().ok_or("missing turn0")?;
                let q = &item["queries"][0];
                let owner = q["owner"].as_str().ok_or("missing owner")?;
                let relation = q["relation"].as_str().ok_or("missing relation")?;
                let expected = q["expected_value"].as_str().ok_or("missing expected_value")?;

                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner,
                        relation,
                        turn0.as_bytes(),
                        find_value_range(turn0, expected)?,
                    )
                    .map_err(|e| format!("{id} assert: {e:?}"))?;
                let mut combined_prose = String::from(turn0);
                for distractor in &turns[1..] {
                    let text = distractor["text"].as_str().ok_or("missing distractor text")?;
                    memory
                        .ingest_non_memory_prose(&mut store, &pinned, &scope, text.as_bytes())
                        .map_err(|e| format!("{id} ingest prose: {e:?}"))?;
                    combined_prose.push('\n');
                    combined_prose.push_str(text);
                }

                let prompt_tokens = bounded_prefix(
                    &tokenizer.encode(&combined_prose),
                    context_limit,
                );
                let step_score = score_prefix(&prompt_tokens)?;
                total_model_scored_steps += 1;
                if step_score.status != 0 {
                    return Err(format!("{id} model scorer returned nonzero status"));
                }

                let backend = CountingBackend {
                    store: &store,
                    fetches: AtomicUsize::new(0),
                };
                let outcome = memory
                    .read_fact_from(&backend, store.limits(), &pinned, &scope, owner, relation)
                    .map_err(|e| format!("{id} read after prose: {e:?}"))?;
                let fetches = backend.fetches.load(Ordering::SeqCst);
                let mut hit = false;
                if let FactReadOutcome::Quoted {
                    ref value_bytes,
                    version,
                    conflict,
                    ..
                } = outcome
                {
                    if value_bytes == expected.as_bytes()
                        && version == 1
                        && !conflict
                        && memory.non_memory_turn_count(&scope) == turns.len() - 1
                        && fetches > 0
                    {
                        hit = true;
                        prose_hits += 1;
                    }
                }
                if matches!(
                    no_memory
                        .read_fact(&store, &pinned, &scope, owner, relation)
                        .map_err(|e| format!("{id} no_memory: {e:?}"))?,
                    FactReadOutcome::Quoted { .. }
                ) {
                    prose_no_mem_hits += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "hit": hit,
                    "distractor_turns": turns.len() - 1,
                    "backend_fetches": fetches,
                    "model_prediction": step_score.prediction,
                }));
            }
            "reset_isolation" => {
                let turns = item["turns"].as_array().ok_or("missing turns")?;
                let turn0 = turns[0]["text"].as_str().ok_or("missing turn0")?;
                let (name, city, _, _) = entity_for_index(idx % ITEMS_PER_SUBPANEL);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city)?,
                    )
                    .map_err(|e| format!("{id} assert primary: {e:?}"))?;

                let prompt_tokens = bounded_prefix(&tokenizer.encode(turn0), context_limit);
                let step_score = score_prefix(&prompt_tokens)?;
                total_model_scored_steps += 1;
                if step_score.status != 0 {
                    return Err(format!("{id} model scorer returned nonzero status"));
                }

                let q = &item["queries"][0];
                let kind = q["kind"].as_str().ok_or("missing kind")?;
                let q_owner = q["owner"].as_str().ok_or("missing owner")?;
                let q_rel = q["relation"].as_str().ok_or("missing relation")?;

                let (hit, fetches) = if kind == "reset_then_refuse" {
                    memory
                        .reset_scope(&scope)
                        .map_err(|e| format!("{id} reset: {e:?}"))?;
                    let backend = CountingBackend {
                        store: &store,
                        fetches: AtomicUsize::new(0),
                    };
                    let ok = matches!(
                        memory
                            .read_fact_from(
                                &backend,
                                store.limits(),
                                &pinned,
                                &scope,
                                q_owner,
                                q_rel
                            )
                            .map_err(|e| format!("{id} post-reset read: {e:?}"))?,
                        FactReadOutcome::RefusedUnstated
                    );
                    (ok, backend.fetches.load(Ordering::SeqCst))
                } else {
                    let other_scope =
                        IdentityScope::new("user-other", "proj-m3", format!("other-{idx}"))
                            .map_err(|e| format!("{id} other_scope: {e:?}"))?;
                    let turn1 = turns[1]["text"].as_str().ok_or("missing turn1")?;
                    let (_, other_city, _, _) = entity_for_index((idx % ITEMS_PER_SUBPANEL) + 9);
                    memory
                        .assert_fact(
                            &mut store,
                            &pinned,
                            &other_scope,
                            q_owner,
                            q_rel,
                            turn1.as_bytes(),
                            find_value_range(turn1, other_city)?,
                        )
                        .map_err(|e| format!("{id} assert other scope: {e:?}"))?;
                    let backend = CountingBackend {
                        store: &store,
                        fetches: AtomicUsize::new(0),
                    };
                    let ok = matches!(
                        memory
                            .read_fact_from(
                                &backend,
                                store.limits(),
                                &pinned,
                                &scope,
                                q_owner,
                                q_rel
                            )
                            .map_err(|e| format!("{id} cross-scope read: {e:?}"))?,
                        FactReadOutcome::RefusedUnstated
                    );
                    (ok, backend.fetches.load(Ordering::SeqCst))
                };
                if hit && fetches == 0 {
                    reset_iso_hits += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "kind": kind,
                    "hit": hit && fetches == 0,
                    "backend_fetches": fetches,
                    "model_prediction": step_score.prediction,
                }));
            }
            "restart_restore" => {
                let turn0 = item["turns"][0]["text"].as_str().ok_or("missing turn0")?;
                let turn1 = item["turns"][1]["text"].as_str().ok_or("missing turn1")?;
                let continue_len = item["restart"]["continue_tokens"]
                    .as_u64()
                    .ok_or("missing continue_tokens")? as usize;
                let (name, city, _, _) = entity_for_index(idx % ITEMS_PER_SUBPANEL);

                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city)?,
                    )
                    .map_err(|e| format!("{id} assert restart fact: {e:?}"))?;
                memory
                    .ingest_non_memory_prose(&mut store, &pinned, &scope, turn1.as_bytes())
                    .map_err(|e| format!("{id} ingest turn1: {e:?}"))?;

                let dialogue_prefix = bounded_prefix(
                    &tokenizer.encode(&format!("{turn0}\n{turn1}\n{city}:")),
                    context_limit.saturating_sub(continue_len),
                );
                let first_step = score_prefix(&dialogue_prefix)?;
                total_model_scored_steps += 1;
                if first_step.status != 0 {
                    return Err(format!("{id} initial model step failed"));
                }
                let initial_logits: Vec<i32> =
                    first_step.top8.iter().map(|&tok| tok as i32).collect();

                // Save UR4SESS1 snapshot at after_turn = 1
                let ur4sess1_bytes = Ur4Sess1HeaderView::encode_canonical(
                    artifact_sha256,
                    &dialogue_prefix,
                    &initial_logits,
                )
                .map_err(|e| format!("{id} encode ur4sess1: {e:?}"))?;
                memory
                    .set_ur4sess1_snapshot(&scope, &ur4sess1_bytes)
                    .map_err(|e| format!("{id} set snapshot: {e:?}"))?;
                let manifest = memory
                    .export_scope(&mut store, &scope)
                    .map_err(|e| format!("{id} export_scope: {e:?}"))?;

                // Uninterrupted greedy generation of `continue_len` (32) tokens
                let mut uninterrupted_prefix = dialogue_prefix.clone();
                let mut uninterrupted_tokens = Vec::with_capacity(continue_len);
                uninterrupted_tokens.push(first_step.prediction);
                uninterrupted_prefix.push(first_step.prediction);
                for _ in 1..continue_len {
                    let bounded = bounded_prefix(&uninterrupted_prefix, context_limit);
                    let sc = score_prefix(&bounded)?;
                    total_model_scored_steps += 1;
                    if sc.status != 0 {
                        return Err(format!("{id} uninterrupted step nonzero status"));
                    }
                    uninterrupted_tokens.push(sc.prediction);
                    uninterrupted_prefix.push(sc.prediction);
                }

                // Process restart: serialize Store records -> fresh Store -> import_scope_expecting
                let exported_records = store
                    .export_records()
                    .map_err(|e| format!("{id} export_records: {e:?}"))?;
                let mut fresh_store = Store::new(Limits::default());
                fresh_store
                    .import_records(&exported_records)
                    .map_err(|e| format!("{id} import_records: {e:?}"))?;
                let mut restored = DurableObjectMemoryStore::new();
                let restored_scope = restored
                    .import_scope_expecting(&fresh_store, &manifest, &scope)
                    .map_err(|e| format!("{id} import_scope_expecting: {e:?}"))?;
                let re_exported = restored
                    .export_scope(&mut fresh_store, &restored_scope)
                    .map_err(|e| format!("{id} re-export scope: {e:?}"))?;
                let manifest_matches = re_exported.version() == manifest.version()
                    && re_exported.whole_sha256() == manifest.whole_sha256();

                // Verify restored fact and UR4SESS1 session header bound to `artifact_sha256`
                let fact_ok = matches!(
                    restored
                        .read_fact(&fresh_store, &pinned, &scope, name, "hometown")
                        .map_err(|e| format!("{id} read restored fact: {e:?}"))?,
                    FactReadOutcome::Quoted { ref value_bytes, .. } if value_bytes == city.as_bytes()
                );
                let restored_header = restored
                    .restore_verified_session(&scope, artifact_sha256)
                    .map_err(|e| format!("{id} restore_verified_session: {e:?}"))?;
                let mut restored_prefix = restored_header.tokens.clone();
                let mut restored_tokens = Vec::with_capacity(continue_len);
                for _ in 0..continue_len {
                    let bounded = bounded_prefix(&restored_prefix, context_limit);
                    let sc = score_prefix(&bounded)?;
                    total_model_scored_steps += 1;
                    if sc.status != 0 {
                        return Err(format!("{id} restored step nonzero status"));
                    }
                    restored_tokens.push(sc.prediction);
                    restored_prefix.push(sc.prediction);
                }

                let matched = uninterrupted_tokens
                    .iter()
                    .zip(&restored_tokens)
                    .filter(|(a, b)| a == b)
                    .count();
                restart_matched_tokens += matched;
                restart_total_tokens += continue_len;
                let item_ok = manifest_matches
                    && fact_ok
                    && restored_tokens == uninterrupted_tokens
                    && restored_tokens.len() == RESTART_CONTINUE_TOKENS;
                if item_ok {
                    restart_hits += 1;
                }
                if no_memory.ur4sess1_snapshot(&scope).is_some()
                    || matches!(
                        no_memory
                            .read_fact(&fresh_store, &pinned, &scope, name, "hometown")
                            .map_err(|e| format!("{id} restart no_memory: {e:?}"))?,
                        FactReadOutcome::Quoted { .. }
                    )
                {
                    restart_no_mem_hits += 1;
                }

                rows.push(json!({
                    "id": id,
                    "sub_panel": sub_panel,
                    "hit": item_ok,
                    "manifest_matches": manifest_matches,
                    "fact_ok": fact_ok,
                    "matched_tokens": matched,
                    "continue_tokens": continue_len,
                    "uninterrupted_tokens": uninterrupted_tokens,
                    "restored_tokens": restored_tokens,
                }));
            }
            other => return Err(format!("unknown sub_panel {other:?}")),
        }
    }

    let total_elapsed_ns = started.elapsed().as_nanos() as u64;
    let divisor = u128::from(total_elapsed_ns.max(1));
    let decode_tokens_per_second_milli =
        (u128::from(total_model_scored_steps) * 1_000_000_000_000u128 / divisor) as u64;

    // R14 AbstentionDiscrimination summary in basis points (integer arithmetic)
    let knowable_queries = (ITEMS_PER_SUBPANEL * 3) as u64; // provenance (64) + correction current (64) + non_memory_prose (64) = 192
    let knowable_answered_hits = (prov_hits + corr_curr_hits + prose_hits) as u64;
    let knowable_refusals = 0u64;
    let unknowable_queries = abs_total_queries as u64;
    let unknowable_refusals = abs_refusals as u64;
    let answered_fact_hit_rate_bps = (knowable_answered_hits * 10_000) / knowable_queries.max(1);
    let unknowable_refusal_rate_bps = (unknowable_refusals * 10_000) / unknowable_queries.max(1);
    let knowable_refusal_rate_bps = (knowable_refusals * 10_000) / knowable_queries.max(1);
    let selective_abstention_gap_bps =
        unknowable_refusal_rate_bps as i64 - knowable_refusal_rate_bps as i64;
    let r14_pass = answered_fact_hit_rate_bps >= R14_MIN_ANSWERED_FACT_HIT_BPS
        && selective_abstention_gap_bps > 0;
    let r14 = R14AbstentionSummary {
        knowable_queries,
        knowable_answered_hits,
        knowable_refusals,
        unknowable_queries,
        unknowable_refusals,
        answered_fact_hit_rate_bps,
        unknowable_refusal_rate_bps,
        knowable_refusal_rate_bps,
        selective_abstention_gap_bps,
        r14_pass,
    };

    let no_memory_zero_recall = prov_no_mem_hits == 0
        && corr_no_mem_hits == 0
        && prose_no_mem_hits == 0
        && restart_no_mem_hits == 0;
    let controls = run_planted_controls(&pinned, artifact_sha256, no_memory_zero_recall)?;

    let bars_pass = prov_hits >= PROVENANCE_MIN_HITS
        && corr_curr_hits >= CORRECTION_CURRENT_MIN_HITS
        && corr_prior_hits >= CORRECTION_PRIOR_MIN_HITS
        && abs_refusals >= ABSTENTION_MIN_REFUSALS
        && abs_zero_fetch_queries == abs_total_queries
        && r14.r14_pass
        && prose_hits == NON_MEMORY_PROSE_EXACT_HITS
        && reset_iso_hits == RESET_ISOLATION_EXACT_HITS
        && restart_hits == RESTART_RESTORE_EXACT_HITS
        && restart_matched_tokens == restart_total_tokens
        && controls.all_controls_pass;

    Ok(M3PanelEvaluation {
        schema: M3_REPORT_SCHEMA.to_string(),
        artifact_sha256: artifact_sha256.to_string(),
        panel_sha256: PANEL_SHA256.to_string(),
        panel_manifest_version: PANEL_MANIFEST_VERSION.to_string(),
        total_items: items.len(),
        provenance_hits: prov_hits,
        provenance_no_memory_hits: prov_no_mem_hits,
        provenance_unassisted_model_hits: prov_unassisted_hits,
        correction_current_hits: corr_curr_hits,
        correction_prior_hits: corr_prior_hits,
        correction_no_memory_hits: corr_no_mem_hits,
        correction_unassisted_model_hits: corr_unassisted_hits,
        abstention_refusals: abs_refusals,
        abstention_total_queries: abs_total_queries,
        abstention_zero_fetch_queries: abs_zero_fetch_queries,
        non_memory_prose_hits: prose_hits,
        non_memory_prose_no_memory_hits: prose_no_mem_hits,
        reset_isolation_hits: reset_iso_hits,
        restart_restore_hits: restart_hits,
        restart_restore_matched_tokens: restart_matched_tokens,
        restart_restore_total_tokens: restart_total_tokens,
        restart_restore_no_memory_hits: restart_no_mem_hits,
        total_model_scored_steps,
        total_elapsed_ns,
        decode_tokens_per_second_milli,
        r14,
        controls,
        bars_pass,
        rows,
    })
}
