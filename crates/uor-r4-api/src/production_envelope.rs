//! Filesystem-free schema-2 production-envelope admission.
//!
//! Native startup and browser startup must make the same decision from the
//! same bytes. This module owns that portable decision: component CIDs,
//! tokenizer adapter identity, canonical held-out population, deployed-quality
//! bindings, and the token-free-D4 plus R4G1Runtime serving composition are all
//! checked before a generation can become active.

use serde::Deserialize;
use uor_r4_core::transformerless::{compiler, hf_bpe::TokenizerAdapter};
use uor_r4_graph_format::{GraphView, SectionId};
use uor_r4_model_source::SourceUnavailable;

use crate::deployed_quality::{
    derive_deployed_quality_bindings, DeployedQualityBindingMaterial, DeployedQualityBindings,
    DeployedQualityReport, LABEL_SHUFFLED_CONTROL_ID,
};
use crate::engine::{AbiVersion, EngineParts};
use crate::release_bundle::{BundleAbi, ReleaseBundleManifest};
use crate::serving::{validate_production_serving_parts, ProductionServingParts};

/// Exact bytes that make up one settled schema-2 production generation.
///
/// No path or ambient process state participates in verification. Callers may
/// capture these bytes from guarded native file handles or fetch them into a
/// browser, then pass the same immutable generation here.
#[derive(Debug, Clone, Copy)]
pub struct ProductionEnvelopeParts<'a> {
    pub graph: &'a [u8],
    pub sections_absent_graph: &'a [u8],
    pub label_shuffled_graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tokenizer: &'a [u8],
    pub score_report: &'a [u8],
    pub compile_report: &'a [u8],
    pub deployed_quality_report: &'a [u8],
    pub cross_surface_parity: &'a [u8],
    pub witness_replay: &'a [u8],
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
    pub tokenizer_adapter: &'a [u8],
    pub release_manifest: &'a [u8],
}

/// Raw evidence whose contents, not merely file digests, must agree with one
/// deployed-quality report. Kept separate from [`ProductionEnvelopeParts`]
/// so native packagers can validate a staging generation before its manifest
/// has been serialized.
#[derive(Debug, Clone, Copy)]
pub struct ProductionEvidenceParts<'a> {
    pub graph: &'a [u8],
    pub sections_absent_graph: &'a [u8],
    pub label_shuffled_graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tokenizer: &'a [u8],
    pub score_report: &'a [u8],
    pub deployed_quality_report: &'a [u8],
    pub cross_surface_parity: &'a [u8],
    pub witness_replay: &'a [u8],
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
}

/// Unforgeable result of verifying one complete production envelope.
///
/// The fields are deliberately private: production-serving constructors take
/// this value as a capability, so a caller cannot substitute a manifest CID
/// or a hand-built set of quality bindings after verification.
#[derive(Debug)]
pub struct VerifiedProductionEnvelope {
    manifest: ReleaseBundleManifest,
    loaded_bindings: DeployedQualityBindings,
    _verified: VerifiedEnvelopeSeal,
}

#[derive(Debug)]
struct VerifiedEnvelopeSeal;

impl VerifiedProductionEnvelope {
    /// Schema-2 manifest whose exact bytes participated in verification.
    pub fn manifest(&self) -> &ReleaseBundleManifest {
        &self.manifest
    }

    /// Independently reproduced identities retained for serving admission.
    pub fn loaded_bindings(&self) -> &DeployedQualityBindings {
        &self.loaded_bindings
    }
}

/// Verify one immutable schema-2 generation without filesystem access.
pub fn verify_production_envelope(
    parts: ProductionEnvelopeParts<'_>,
) -> Result<VerifiedProductionEnvelope, SourceUnavailable> {
    let manifest: ReleaseBundleManifest = serde_json::from_slice(parts.release_manifest)
        .map_err(|error| unavailable(format!("invalid production release-bundle.json: {error}")))?;
    if let Some(reason) = manifest.validate() {
        return Err(unavailable(format!(
            "production release-bundle.json is not schema-2 admissible: {reason}"
        )));
    }

    let expected_abi = BundleAbi::from(AbiVersion::current());
    if manifest.abi != expected_abi {
        return Err(unavailable(format!(
            "production release ABI {:?} does not equal this runtime ABI {:?}",
            manifest.abi, expected_abi
        )));
    }

    require_digest("graph/score.r4g1", parts.graph, &manifest.components.graph)?;
    require_schema_two_digest(
        "graph/score_sections_absent.r4g1",
        parts.sections_absent_graph,
        manifest.components.sections_absent_graph.as_deref(),
    )?;
    require_schema_two_digest(
        "graph/score_label_shuffled.r4g1",
        parts.label_shuffled_graph,
        manifest.components.label_shuffled_graph.as_deref(),
    )?;
    require_digest(
        "tless_artifacts.bin",
        parts.signature_artifact,
        &manifest.components.signature_artifact,
    )?;
    require_digest(
        "graph/score_report.json",
        parts.score_report,
        &manifest.components.score_report,
    )?;
    require_digest(
        "graph-cover/cover_report.json",
        parts.compile_report,
        &manifest.components.compile_report,
    )?;
    let declared_quality_cid = manifest
        .components
        .deployed_quality_report
        .as_deref()
        .ok_or_else(|| unavailable("schema-2 manifest omitted deployed-quality report CID"))?;
    require_digest(
        "graph/deployed_quality_report.json",
        parts.deployed_quality_report,
        declared_quality_cid,
    )?;
    require_schema_two_digest(
        "graph/cross_surface_parity.json",
        parts.cross_surface_parity,
        manifest.components.cross_surface_parity.as_deref(),
    )?;
    require_schema_two_digest(
        "graph/witness_replay.json",
        parts.witness_replay,
        manifest.components.witness_replay.as_deref(),
    )?;
    let declared_tokenizer_cid = manifest
        .components
        .tokenizer
        .as_deref()
        .ok_or_else(|| unavailable("schema-2 production admission requires tokenizer.bin"))?;
    require_digest("tokenizer.bin", parts.tokenizer, declared_tokenizer_cid)?;

    serde_json::from_slice::<serde_json::Value>(parts.score_report)
        .map_err(|error| unavailable(format!("invalid graph/score_report.json: {error}")))?;
    serde_json::from_slice::<serde_json::Value>(parts.compile_report)
        .map_err(|error| unavailable(format!("invalid graph-cover/cover_report.json: {error}")))?;

    let adapter: TokenizerAdapter = serde_json::from_slice(parts.tokenizer_adapter)
        .map_err(|error| unavailable(format!("invalid tokenizer_adapter.json: {error}")))?;
    if adapter != manifest.tokenizer_adapter {
        return Err(unavailable(
            "tokenizer_adapter.json does not equal the schema-2 release manifest adapter",
        ));
    }
    let reproduced_adapter_digest = adapter.declared_digest();
    if adapter.adapter_digest != reproduced_adapter_digest {
        return Err(unavailable(format!(
            "tokenizer_adapter.json declares digest {}, but its fields reproduce {reproduced_adapter_digest}",
            adapter.adapter_digest
        )));
    }

    let corpus = compiler::load_corpus_bytes(parts.corpus_meta, parts.corpus_records, None)
        .ok_or_else(|| unavailable("corpus.meta/corpus.records do not parse as one corpus"))?;
    let (_, held_out) = compiler::split_positions(&corpus);
    let certification_positions = held_out
        .into_iter()
        .map(|position| {
            u64::try_from(position).map_err(|_| {
                unavailable("certification corpus position does not fit the u64 wire identity")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let manifest_selector = manifest
        .selector
        .as_ref()
        .ok_or_else(|| unavailable("schema-2 release manifest omitted selector identity"))?;
    let manifest_compiler = manifest
        .compiler
        .as_ref()
        .ok_or_else(|| unavailable("schema-2 release manifest omitted compiler identity"))?;
    let loaded_bindings = derive_deployed_quality_bindings(DeployedQualityBindingMaterial {
        graph: parts.graph,
        teacher_artifact: parts.signature_artifact,
        corpus_meta: parts.corpus_meta,
        corpus_records: parts.corpus_records,
        tokenizer: parts.tokenizer,
        tokenizer_adapter: parts.tokenizer_adapter,
        score_report: parts.score_report,
        compile_report: parts.compile_report,
        compiler_revision: &manifest_compiler.revision,
        full_population_positions: &certification_positions,
        evaluated_positions: &certification_positions,
    })
    .map_err(|error| unavailable(error.to_string()))?;
    if manifest_selector != &loaded_bindings.selector {
        return Err(unavailable(format!(
            "schema-2 manifest selector {:?} does not equal the selector derived by this runtime {:?}",
            manifest_selector, loaded_bindings.selector
        )));
    }
    if manifest_compiler != &loaded_bindings.compiler {
        return Err(unavailable(format!(
            "schema-2 manifest compiler identity {:?} does not equal the captured compiler/configuration identity {:?}",
            manifest_compiler, loaded_bindings.compiler
        )));
    }

    let verified = VerifiedProductionEnvelope {
        manifest,
        loaded_bindings,
        _verified: VerifiedEnvelopeSeal,
    };
    validate_production_serving_parts(&ProductionServingParts {
        engine: EngineParts {
            graph: parts.graph,
            signature_artifact: parts.signature_artifact,
            tokenizer: Some(parts.tokenizer),
            score_report: Some(parts.score_report),
        },
        deployed_quality_report: parts.deployed_quality_report,
        verified_envelope: &verified,
    })?;
    validate_production_evidence_links(
        ProductionEvidenceParts {
            graph: parts.graph,
            sections_absent_graph: parts.sections_absent_graph,
            label_shuffled_graph: parts.label_shuffled_graph,
            signature_artifact: parts.signature_artifact,
            tokenizer: parts.tokenizer,
            score_report: parts.score_report,
            deployed_quality_report: parts.deployed_quality_report,
            cross_surface_parity: parts.cross_surface_parity,
            witness_replay: parts.witness_replay,
            corpus_meta: parts.corpus_meta,
            corpus_records: parts.corpus_records,
        },
        &certification_positions,
    )?;

    Ok(verified)
}

fn require_schema_two_digest(
    label: &str,
    bytes: &[u8],
    declared: Option<&str>,
) -> Result<(), SourceUnavailable> {
    let declared = declared.ok_or_else(|| {
        unavailable(format!(
            "schema-2 production manifest omitted required component {label}"
        ))
    })?;
    require_digest(label, bytes, declared)
}

/// Runtime-readable projection of the canonical witness artifact. Native
/// packaging independently replays every row; production hosts retain and
/// check the exact generation identities and aggregate claims consumed by the
/// deployed-quality report.
#[derive(Deserialize)]
struct WitnessReplaySummary {
    schema: String,
    graph_cid: String,
    signature_artifact_cid: String,
    tokenizer_cid: String,
    score_report_cid: Option<String>,
    corpus_meta_cid: String,
    corpus_records_cid: String,
    evaluated_positions_cid: String,
    sample_positions_cid: String,
    requested: u64,
    replayed: u64,
    failures: u64,
    records: Vec<serde_json::Value>,
}

pub fn validate_production_evidence_links(
    parts: ProductionEvidenceParts<'_>,
    certification_positions: &[u64],
) -> Result<(), SourceUnavailable> {
    validate_control_graphs(
        parts.graph,
        parts.sections_absent_graph,
        parts.label_shuffled_graph,
    )?;
    let report: DeployedQualityReport = serde_json::from_slice(parts.deployed_quality_report)
        .map_err(|error| unavailable(format!("invalid deployed-quality report: {error}")))?;
    let measurements = report.evaluation.measurements.as_ref().ok_or_else(|| {
        unavailable("production deployed-quality report omitted measured evidence")
    })?;

    const ABSENT_VERSION: &str = "r4g1-sections-absent/1";
    let expected_absent = tagged_cid(
        b"r4-deployed-quality-sections-absent-comparator/1",
        &[ABSENT_VERSION.as_bytes(), parts.sections_absent_graph],
    );
    if measurements
        .versus_sections_absent
        .comparator
        .definition_cid
        != expected_absent
    {
        return Err(unavailable(
            "deployed-quality report is not bound to graph/score_sections_absent.r4g1",
        ));
    }

    let label_control = report
        .negative_controls
        .iter()
        .find(|control| control.id == LABEL_SHUFFLED_CONTROL_ID)
        .ok_or_else(|| unavailable("deployed-quality report omitted label-shuffled control"))?;
    const SHUFFLED_VERSION: &str = "train-target-rotation-half-plus-one/1";
    let expected_shuffled = tagged_cid(
        b"r4-deployed-quality-label-shuffled-control/1",
        &[
            SHUFFLED_VERSION.as_bytes(),
            parts.sections_absent_graph,
            parts.label_shuffled_graph,
            report.bindings.partition.manifest_cid.as_bytes(),
        ],
    );
    if label_control.identity_cid != expected_shuffled {
        return Err(unavailable(
            "deployed-quality report is not bound to graph/score_label_shuffled.r4g1",
        ));
    }

    let cross =
        crate::serving::CrossSurfaceParityEvidence::parse_and_validate_for_production_bundle(
            parts.cross_surface_parity,
            parts.graph,
            parts.signature_artifact,
            parts.tokenizer,
            parts.score_report,
        )?;
    let expected_combined_checks = cross
        .checks
        .checked_add(measurements.internal_base_control_checks)
        .ok_or_else(|| unavailable("combined cross-surface check count overflow"))?;
    let expected_combined_mismatches = cross
        .mismatches
        .checked_add(measurements.internal_base_control_mismatches)
        .ok_or_else(|| unavailable("combined cross-surface mismatch count overflow"))?;
    if measurements.cross_surface_checks != expected_combined_checks
        || measurements.cross_surface_mismatches != expected_combined_mismatches
    {
        return Err(unavailable(
            "combined cross-surface counts do not equal raw external plus declared internal evidence",
        ));
    }
    if measurements.internal_base_control_checks != report.evaluation.evaluated_positions
        || measurements.internal_base_control_mismatches != 0
    {
        return Err(unavailable(format!(
            "full-census sections-absent identity covered {}/{} evaluated positions with {} mismatches",
            measurements.internal_base_control_checks,
            report.evaluation.evaluated_positions,
            measurements.internal_base_control_mismatches
        )));
    }
    let expected_cross = tagged_cid(
        b"r4-deployed-quality-cross-surface-evidence/1",
        &[
            &cross.checks.to_le_bytes(),
            &cross.mismatches.to_le_bytes(),
            &measurements.internal_base_control_checks.to_le_bytes(),
            &measurements.internal_base_control_mismatches.to_le_bytes(),
            cross.graph_cid.as_bytes(),
            cross.signature_artifact_cid.as_bytes(),
            parts.cross_surface_parity,
        ],
    );
    if measurements.cross_surface_evidence_cid != expected_cross {
        return Err(unavailable(
            "deployed-quality report is not transitively bound to graph/cross_surface_parity.json",
        ));
    }

    let witness: WitnessReplaySummary = serde_json::from_slice(parts.witness_replay)
        .map_err(|error| unavailable(format!("invalid witness replay JSON: {error}")))?;
    let expected_positions_cid =
        crate::deployed_quality::deployed_quality_positions_cid(certification_positions);
    let expected_score_cid = bytes_cid(parts.score_report);
    if witness.schema != "uor-r4-normative-witness-replay/1"
        || witness.graph_cid != bytes_cid(parts.graph)
        || witness.signature_artifact_cid != bytes_cid(parts.signature_artifact)
        || witness.tokenizer_cid != bytes_cid(parts.tokenizer)
        || witness.score_report_cid.as_deref() != Some(expected_score_cid.as_str())
        || witness.corpus_meta_cid != bytes_cid(parts.corpus_meta)
        || witness.corpus_records_cid != bytes_cid(parts.corpus_records)
        || witness.evaluated_positions_cid != expected_positions_cid
        || witness.requested != witness.records.len() as u64
        || witness.replayed != witness.records.len() as u64
        || witness.sample_positions_cid != report.witness_replay.sample_cid
        || witness.requested != report.witness_replay.requested
        || witness.replayed != report.witness_replay.replayed
        || witness.failures != report.witness_replay.failures
    {
        return Err(unavailable(
            "deployed-quality report and graph/witness_replay.json do not describe the same replay evidence",
        ));
    }
    Ok(())
}

fn validate_control_graphs(
    main: &[u8],
    sections_absent: &[u8],
    label_shuffled: &[u8],
) -> Result<(), SourceUnavailable> {
    let main = parse_control_graph("main", main)?;
    let absent = parse_control_graph("sections-absent", sections_absent)?;
    let shuffled = parse_control_graph("label-shuffled", label_shuffled)?;
    if main.section(SectionId::SKMX).is_none() || main.section(SectionId::PSIB).is_none() {
        return Err(unavailable("main graph lacks required SKMX/PSIB sections"));
    }
    if absent.section(SectionId::SKMX).is_some() || absent.section(SectionId::PSIB).is_some() {
        return Err(unavailable(
            "sections-absent control still contains SKMX or PSIB",
        ));
    }
    if shuffled.section(SectionId::SKMX).is_none() || shuffled.section(SectionId::PSIB).is_none() {
        return Err(unavailable("label-shuffled control lacks SKMX or PSIB"));
    }
    require_non_lane_identity(&main, &absent, "sections-absent")?;
    require_non_lane_identity(&main, &shuffled, "label-shuffled")?;
    if [SectionId::SKMX, SectionId::PSIB]
        .iter()
        .all(|&id| main.section(id) == shuffled.section(id))
    {
        return Err(unavailable(
            "label-shuffled control is byte-identical to the main lane",
        ));
    }
    Ok(())
}

fn parse_control_graph<'a>(
    label: &str,
    bytes: &'a [u8],
) -> Result<GraphView<'a>, SourceUnavailable> {
    let view = GraphView::parse(bytes)
        .map_err(|error| unavailable(format!("{label} graph parse failed: {error}")))?;
    view.verify_cids()
        .map_err(|error| unavailable(format!("{label} graph CID check failed: {error}")))?;
    Ok(view)
}

fn require_non_lane_identity(
    main: &GraphView<'_>,
    control: &GraphView<'_>,
    label: &str,
) -> Result<(), SourceUnavailable> {
    for section in main.sections() {
        if section.id == SectionId::SKMX || section.id == SectionId::PSIB {
            continue;
        }
        let Some(other) = control
            .sections()
            .find(|candidate| candidate.id == section.id)
        else {
            return Err(unavailable(format!(
                "{label} control is missing non-lane section 0x{:08x}",
                section.id.raw()
            )));
        };
        if other.flags != section.flags || other.payload != section.payload {
            return Err(unavailable(format!(
                "{label} control changes non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    for section in control.sections() {
        if section.id != SectionId::SKMX
            && section.id != SectionId::PSIB
            && main.section(section.id).is_none()
        {
            return Err(unavailable(format!(
                "{label} control adds non-lane section 0x{:08x}",
                section.id.raw()
            )));
        }
    }
    Ok(())
}

fn bytes_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn tagged_cid(tag: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(tag.len() as u64).to_le_bytes());
    hasher.update(tag);
    for part in parts {
        hasher.update(&(part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn require_digest(label: &str, bytes: &[u8], declared: &str) -> Result<(), SourceUnavailable> {
    let actual = format!("blake3:{}", blake3::hash(bytes).to_hex());
    if actual == declared {
        Ok(())
    } else {
        Err(unavailable(format!(
            "production component {label} digest mismatch: manifest declares {declared}, captured bytes hash to {actual}"
        )))
    }
}

fn unavailable(reason: impl Into<String>) -> SourceUnavailable {
    SourceUnavailable::new(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_manifest_is_refused_before_any_component_can_activate() {
        let empty = &[];
        let error = verify_production_envelope(ProductionEnvelopeParts {
            graph: empty,
            sections_absent_graph: empty,
            label_shuffled_graph: empty,
            signature_artifact: empty,
            tokenizer: empty,
            score_report: empty,
            compile_report: empty,
            deployed_quality_report: empty,
            cross_surface_parity: empty,
            witness_replay: empty,
            corpus_meta: empty,
            corpus_records: empty,
            tokenizer_adapter: empty,
            release_manifest: br#"{}"#,
        })
        .expect_err("an incomplete envelope must fail closed");
        assert!(error.to_string().contains("release-bundle.json"));
    }
}
