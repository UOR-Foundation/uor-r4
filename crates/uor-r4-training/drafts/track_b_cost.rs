//! DRAFT / NOT_RUN / ANALYTIC ONLY: source-derived B2 cost accounting.
//!
//! This file has no model, Candle, allocator, timing, or filesystem dependency.
//! It is not module-registered. Formula fixtures below are authored, not an
//! execution receipt. A later `rustc --test` may check this same std-only source.
//! MAC means one multiply-accumulate, not one FLOP. Scalar arithmetic, copies,
//! reductions, normalization, nonlinearities, sorting and synchronizations are
//! separate; none of these counts establish tokens/s, DRAM traffic, or peak RSS.
//!
//! Source snapshot (2026-09-29; paths relative to crates/uor-r4-training):
//! - src/track_b/model.rs SHA256
//!   a9f518ca9f8703c96ab0033d768c574a4091aa729807a0af37beebf5d769a672
//! - src/track_b/harmonics.rs SHA256
//!   f0a899421a4ad23bff1210b5efa61adca6601606cf82583bd629bdd815dffb71
//! - drafts/track_b_transfer.rs SHA256
//!   ad3936b6fcaa9b9714ac1f45c2d4700638fe131cebc9c42c9673618408916cf5
//!
//! Exact scope: SmolLM2-135M shape W=576,H=9,G=3,D=64,I=1536,V=49152,
//! 30 layers, F32 tensors, rank-4 Q/K/V LoRA. All MAC/position fields refer to
//! ONE sequence position (not B positions), unless explicitly named otherwise.
//! Buffer/state fields include batch. Per-layer fields exclude other layers.
//! "Full-model" means a hypothetical all-layer replacement plus the retained
//! MLP and vocabulary projection; the current isolated fit does not execute it.
//!
//! The current default training path forms Q'K'^T at width d, applies
//! delta+((1+t)/2)^L, then AV. It does NOT construct F packed features. Its
//! forward MACs/window are B*H*T*T*(d+D), with full square GEMMs before masking.
//! Backward is NOT estimated as a fixed multiple: frozen WO still propagates
//! an input gradient, LoRA/map factors need gradients, source capture is detached,
//! and retained autograd buffers/backend kernels require separate measurement.
//!
//! The detached recurrence builds both packed maps at each position. Its
//! M[H,F,D],z[H,F] state is independent of T, but the actual shared caller still
//! supplies a T*T U8 causal mask, full Q/K/V and projected sequences, and keeps
//! all outputs until cat. This is not an incremental constant-memory API.
//! Native GQA does NOT divide its state by three: independent K maps are applied
//! after repeating the 3 KV heads into 9 heads. `shared_kv_counterfactual` changes
//! that trainable sharing assumption and is explicitly NOT_IMPLEMENTED.
//!
//! Read/write fields are logical tensor-array passes, not hardware bytes. The
//! recurrence's out-of-place M update holds old M, outer product, and new M.
//! Eager core reads=3*(M+z), writes=2*(M+z), excluding q/k/v broadcasts, feature
//! construction, outputs, parameters and allocation/launch overhead. An ideal
//! fused streaming kernel could read/write the state once; no such kernel is
//! claimed. Weight bytes describe stored payload / one logical visit, not DRAM
//! misses. Batch/tile reuse and the memory hierarchy can change actual traffic.
//!
//! Hybrid status: NOT_INTEGRATED. drafts/track_b_hybrid.rs corrects supplied
//! causal unique support with RAW a_i=1/(rank+1), c_i=a_i-h_i; both numerator and
//! mass receive the correction. Its quadratic helper allocates dense masks and
//! scores and copies scores/output to the host. It does not select support.
//! DeepSeek's uncommitted b0-flock-20260929 kappa_llama.rs:209-317 scans all keys,
//! sorts the rest, then sorts selected ranks. Its current flock_probabilities
//! materializes dot/Lorentz/scaled/probability T*T tensors and host arrays, even
//! in rank mode. A committed shared raw-mass/ranked-support adapter is unresolved.
//! `hybrid` counts a proposed streaming correction plus that full-scan policy;
//! it does not claim O(k) selection or a measured indexed implementation.
//!
//! Fit integration should record capture_calls/captured_positions, student
//! forward_positions/backward_steps/eval_positions; arm/source/config/window
//! hashes; B/T/layer/rank/seed/path; denominator and norm fallback diagnostics;
//! observed wall time, RSS, logical tokens and model work separately. Recompute
//! teacher capture and base QKV every update/pre/post evaluation, according to
//! the current fit design; no all-dataset activation cache is assumed. Final
//! held-out document-disjoint MSE and composed all-layer NLL remain separate
//! evaluation work. Step-0 and zero/mean energy baselines are proposed diagnostics,
//! not additional owner gates. Two sequential seeds double
//! total fit/evaluation work and retained checkpoints, not simultaneous state.

use std::fmt;

pub const STATUS: &str = "DRAFT_NOT_RUN_ANALYTIC_ONLY";
pub const WIDTH: u64 = 576;
pub const HEADS: u64 = 9;
pub const KV_HEADS: u64 = 3;
pub const HEAD_DIM: u64 = 64;
pub const INTERMEDIATE: u64 = 1536;
pub const VOCAB: u64 = 49_152;
pub const LAYERS: u64 = 30;
pub const RANK: u64 = 4;
pub const F32_BYTES: u64 = 4;
pub const CONTEXTS: [u64; 3] = [512, 2048, 8192];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostError {
    Invalid(&'static str),
    Overflow,
}

impl fmt::Display for CostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(f, "invalid cost request: {reason}"),
            Self::Overflow => f.write_str("analytic cost exceeds u64"),
        }
    }
}

impl std::error::Error for CostError {}
pub type Result<T> = std::result::Result<T, CostError>;

fn product(values: &[u64]) -> Result<u64> {
    values.iter().try_fold(1u64, |n, &x| {
        n.checked_mul(x).ok_or(CostError::Overflow)
    })
}

fn sum(values: &[u64]) -> Result<u64> {
    values.iter().try_fold(0u64, |n, &x| {
        n.checked_add(x).ok_or(CostError::Overflow)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Traffic {
    pub read_bytes: u64,
    pub write_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedFeatureCost {
    pub dimensions: u64,
    pub degree: u64,
    pub features: u64,
    /// Per ONE input row. Includes band scales; does not call these MACs.
    pub multiplications: u64,
    pub essential_add_subtract: u64,
    /// affine(scale,0) may or may not execute the zero addition in the backend.
    pub affine_zero_additions: u64,
    /// Shared map constants, not one copy per attention head; excludes headers.
    pub device_constant_bytes: u64,
    pub host_band_weight_bytes: u64,
    /// Largest one-row scratch payload among the packed intermediate shapes.
    /// This is a shape inventory, not simultaneous scratch or a peak-RSS bound.
    pub largest_intermediate_row_bytes: u64,
    /// Source computes these scalar F64 sqrts per forward CALL, not per row.
    pub host_sqrt_per_forward_call: u64,
}

/// Packed harmonic dimensions: N2=C(d,2)+d-1, N3=C(d,3)+d(d-1).
/// L2 multiplication cost is 3d+2*C(d,2)+N2 (diagonal, Householder,
/// off-diagonals, scale); L3 adds 3*C(d,3)+5*d^2+N3. Then all F coordinates
/// receive a band scale. Householder adds/subtracts: 2d-1 and 2d^2-d.
pub fn packed_features(dimensions: u64, degree: u64) -> Result<PackedFeatureCost> {
    if !matches!(dimensions, 16 | 32) || !(1..=3).contains(&degree) {
        return Err(CostError::Invalid("registered arms are d=16/32, L=1/2/3"));
    }
    let d = dimensions;
    let pairs = product(&[d, d - 1])? / 2;
    let triples = product(&[d, d - 1, d - 2])? / 6;
    let d2 = product(&[d, d])?;
    let n2 = sum(&[pairs, d - 1])?;
    let n3 = sum(&[triples, product(&[d, d - 1])?])?;
    let features = sum(&[
        1, d, if degree >= 2 { n2 } else { 0 },
        if degree >= 3 { n3 } else { 0 },
    ])?;
    let mut multiplications = features;
    let mut essential_add_subtract = 0;
    let mut affine_zero_additions = features;
    let mut constants = 0;
    let mut largest = features;
    if degree >= 2 {
        multiplications = sum(&[
            multiplications, product(&[3, d])?, product(&[2, pairs])?, n2,
        ])?;
        essential_add_subtract = sum(&[essential_add_subtract, product(&[2, d])? - 1])?;
        affine_zero_additions = sum(&[affine_zero_additions, pairs, n2])?;
        // Two u32 gather arrays and one F32 reflection vector.
        constants = sum(&[constants, product(&[8, pairs])?, product(&[4, d])?])?;
        largest = largest.max(n2).max(d);
    }
    if degree >= 3 {
        multiplications = sum(&[
            multiplications, product(&[3, triples])?, product(&[5, d2])?, n3,
        ])?;
        essential_add_subtract = sum(&[essential_add_subtract, product(&[2, d2])? - d])?;
        affine_zero_additions = sum(&[affine_zero_additions, triples, n3])?;
        // Three distinct u32 arrays; two group u32 arrays; group F32 scale;
        // one F32 reflection vector shared by all d groups.
        constants = sum(&[
            constants, product(&[12, triples])?, product(&[12, d2])?, product(&[4, d])?,
        ])?;
        largest = largest.max(triples).max(d2).max(n3);
    }
    Ok(PackedFeatureCost {
        dimensions, degree, features, multiplications, essential_add_subtract,
        affine_zero_additions, device_constant_bytes: constants,
        host_band_weight_bytes: product(&[8, degree + 1])?,
        largest_intermediate_row_bytes: product(&[4, largest])?,
        host_sqrt_per_forward_call: product(&[3, degree])? - 1,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommonCost {
    pub batch: u64,
    pub context: u64,
    pub positions: u64,
    pub base_qkv_mac_per_layer_position: u64,
    pub output_projection_mac_per_layer_position: u64,
    pub mlp_mac_per_layer_position: u64,
    pub lora_parameters_per_layer: u64,
    pub lora_mac_per_layer_position: u64,
    pub frozen_matrix_mac_per_model_position: u64,
    /// Tied embedding/head counted once; includes all RMS gains.
    pub unique_frozen_f32_weight_bytes: u64,
    pub dense_core_mac_per_layer_position: u64,
    /// All layers + rank4 LoRA + retained MLP + vocabulary output projection.
    pub dense_control_forward_mac_per_model_position: u64,
    pub native_kv_history_bytes_per_layer: u64,
    pub native_kv_history_bytes_all_layers: u64,
    pub repeated_kv_prefix_bytes_per_layer: u64,
    /// One F32 [B,H,T,T] tensor, not a training memory estimate.
    pub one_gram_bytes: u64,
    /// Current caller keeps one shared [1,1,T,T] U8 mask even for recurrence.
    pub shared_causal_mask_bytes: u64,
    pub shared_rope_table_bytes: u64,
    pub full_vocabulary_logits_bytes: u64,
    /// Base QKV (and separately the adapted copy) has this payload.
    pub qkv_prefix_bytes: u64,
    pub normalized_input_prefix_bytes: u64,
}

pub fn common(batch: u64, context: u64) -> Result<CommonCost> {
    if batch == 0 || context == 0 {
        return Err(CostError::Invalid("batch and context must be nonzero"));
    }
    let positions = product(&[batch, context])?;
    let qkv_width = sum(&[WIDTH, product(&[2, KV_HEADS, HEAD_DIM])?])?;
    let qkv = product(&[WIDTH, qkv_width])?;
    let wo = product(&[WIDTH, WIDTH])?;
    let mlp = product(&[3, WIDTH, INTERMEDIATE])?;
    let matrices = sum(&[
        product(&[LAYERS, sum(&[qkv, wo, mlp])?])?, product(&[VOCAB, WIDTH])?,
    ])?;
    let norms = product(&[sum(&[product(&[2, LAYERS])?, 1])?, WIDTH])?;
    let lora = product(&[RANK, sum(&[product(&[3, WIDTH])?, qkv_width])?])?;
    let dense_core = product(&[HEADS, context, 2, HEAD_DIM])?;
    let kv = product(&[4, batch, KV_HEADS, context, 2, HEAD_DIM])?;
    Ok(CommonCost {
        batch, context, positions,
        base_qkv_mac_per_layer_position: qkv,
        output_projection_mac_per_layer_position: wo,
        mlp_mac_per_layer_position: mlp,
        lora_parameters_per_layer: lora,
        lora_mac_per_layer_position: lora,
        frozen_matrix_mac_per_model_position: matrices,
        unique_frozen_f32_weight_bytes: product(&[4, sum(&[matrices, norms])?])?,
        dense_core_mac_per_layer_position: dense_core,
        dense_control_forward_mac_per_model_position: sum(&[
            matrices, product(&[LAYERS, sum(&[lora, dense_core])?])?,
        ])?,
        native_kv_history_bytes_per_layer: kv,
        native_kv_history_bytes_all_layers: product(&[LAYERS, kv])?,
        repeated_kv_prefix_bytes_per_layer: product(&[4, positions, 2, HEADS, HEAD_DIM])?,
        one_gram_bytes: product(&[4, batch, HEADS, context, context])?,
        shared_causal_mask_bytes: product(&[context, context])?,
        shared_rope_table_bytes: product(&[4, context, HEAD_DIM])?,
        full_vocabulary_logits_bytes: product(&[4, positions, VOCAB])?,
        qkv_prefix_bytes: product(&[4, positions, qkv_width])?,
        normalized_input_prefix_bytes: product(&[4, positions, WIDTH])?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HarmonicCost {
    pub common: CommonCost,
    pub packed: PackedFeatureCost,
    pub projection_parameters_and_mac_per_layer_position: u64,
    pub trainable_parameters_per_layer: u64,
    pub trainable_f32_bytes_all_layers: u64,
    /// Parameters + gradients + Adam first/second moments; excludes backend
    /// temporaries, master copies, optimizer metadata, and frozen weights.
    pub nominal_adam_f32_bytes_one_layer: u64,
    pub quadratic_core_mac_per_layer_position: u64,
    pub quadratic_core_mac_per_layer_window: u64,
    pub quadratic_forward_mac_per_model_position: u64,
    /// HF(2D+1): update M, query M, and query z. z update adds are separate.
    pub recurrent_core_mac_per_layer_position: u64,
    pub recurrent_z_adds_per_layer_position: u64,
    pub recurrent_output_divisions_per_layer_position: u64,
    pub recurrent_forward_mac_per_model_position: u64,
    /// Packed scalar multiplications/additions excluded from MAC fields above.
    pub packed_multiplications_per_layer_position: u64,
    pub packed_add_subtract_per_layer_position: u64,
    pub pre_and_post_normalization_squares_per_layer_position: u64,
    pub pre_and_post_normalization_sum_adds_per_layer_position: u64,
    pub pre_and_post_normalization_divisions_per_layer_position: u64,
    pub pre_and_post_normalization_sqrts_per_layer_position: u64,
    pub state_bytes_per_layer: u64,
    pub state_bytes_all_layers: u64,
    pub ideal_state_traffic_per_layer_batch_step: Traffic,
    pub eager_core_traffic_per_layer_batch_step: Traffic,
    /// Old M + outer product + new M + z, before extra feature/input/output
    /// buffers. This is an eager tensor payload inventory, not a peak upper bound.
    pub update_live_state_payload_bytes: u64,
    pub projected_qk_prefix_bytes: u64,
    pub repeated_v_prefix_bytes: u64,
    pub outputs_then_cat_payload_bytes: u64,
    pub latest_pre_norm_host_diagnostic_bytes: u64,
    /// Four norm-row copies plus one denominator-row copy; not all transfers.
    pub norm_and_mass_host_copy_bytes_per_layer_window: u64,
    pub recurrent_norm_and_mass_host_sync_calls_per_layer_window: u64,
    pub quadratic_norm_and_mass_host_sync_calls_per_layer_window: u64,
}

pub fn harmonic(batch: u64, context: u64, dimensions: u64, degree: u64) -> Result<HarmonicCost> {
    let c = common(batch, context)?;
    let p = packed_features(dimensions, degree)?;
    let projection = product(&[2, HEADS, HEAD_DIM, dimensions])?;
    let parameters = sum(&[c.lora_parameters_per_layer, projection])?;
    let quadratic = product(&[HEADS, context, sum(&[dimensions, HEAD_DIM])?])?;
    let recurrent = product(&[HEADS, p.features, sum(&[product(&[2, HEAD_DIM])?, 1])?])?;
    let matrix_state = product(&[4, batch, HEADS, p.features, HEAD_DIM])?;
    let mass_state = product(&[4, batch, HEADS, p.features])?;
    let state = sum(&[matrix_state, mass_state])?;
    let normalization_widths = sum(&[HEAD_DIM, dimensions])?;
    let normalized_elements = product(&[2, HEADS, normalization_widths])?;
    let full_model = |core| -> Result<u64> {
        sum(&[c.frozen_matrix_mac_per_model_position,
            product(&[LAYERS, sum(&[c.lora_mac_per_layer_position, projection, core])?])?])
    };
    Ok(HarmonicCost {
        common: c, packed: p,
        projection_parameters_and_mac_per_layer_position: projection,
        trainable_parameters_per_layer: parameters,
        trainable_f32_bytes_all_layers: product(&[4, LAYERS, parameters])?,
        nominal_adam_f32_bytes_one_layer: product(&[16, parameters])?,
        quadratic_core_mac_per_layer_position: quadratic,
        quadratic_core_mac_per_layer_window: product(&[c.positions, quadratic])?,
        quadratic_forward_mac_per_model_position: full_model(quadratic)?,
        recurrent_core_mac_per_layer_position: recurrent,
        recurrent_z_adds_per_layer_position: product(&[HEADS, p.features])?,
        recurrent_output_divisions_per_layer_position: product(&[HEADS, HEAD_DIM])?,
        recurrent_forward_mac_per_model_position: full_model(recurrent)?,
        packed_multiplications_per_layer_position: product(&[2, HEADS, p.multiplications])?,
        packed_add_subtract_per_layer_position: product(&[2, HEADS, p.essential_add_subtract])?,
        pre_and_post_normalization_squares_per_layer_position: normalized_elements,
        pre_and_post_normalization_sum_adds_per_layer_position: product(&[2, HEADS, normalization_widths - 2])?,
        pre_and_post_normalization_divisions_per_layer_position: normalized_elements,
        pre_and_post_normalization_sqrts_per_layer_position: product(&[4, HEADS])?,
        state_bytes_per_layer: state,
        state_bytes_all_layers: product(&[LAYERS, state])?,
        ideal_state_traffic_per_layer_batch_step: Traffic { read_bytes: state, write_bytes: state },
        eager_core_traffic_per_layer_batch_step: Traffic {
            read_bytes: product(&[3, state])?, write_bytes: product(&[2, state])?,
        },
        update_live_state_payload_bytes: sum(&[product(&[3, matrix_state])?, mass_state])?,
        projected_qk_prefix_bytes: product(&[4, c.positions, 2, HEADS, dimensions])?,
        repeated_v_prefix_bytes: product(&[4, c.positions, HEADS, HEAD_DIM])?,
        outputs_then_cat_payload_bytes: product(&[4, c.positions, 2, HEADS, HEAD_DIM])?,
        latest_pre_norm_host_diagnostic_bytes: product(&[4, c.positions, 2, HEADS])?,
        norm_and_mass_host_copy_bytes_per_layer_window: product(&[4, c.positions, 5, HEADS])?,
        recurrent_norm_and_mass_host_sync_calls_per_layer_window: sum(&[4, context])?,
        quadratic_norm_and_mass_host_sync_calls_per_layer_window: 5,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedKvCounterfactual {
    pub status: &'static str,
    pub projection_parameters_and_mac_per_layer_position: u64,
    pub state_bytes_per_layer: u64,
    pub recurrent_core_mac_per_layer_position: u64,
    pub packed_multiplications_per_layer_position: u64,
}

/// NOT IMPLEMENTED: tie K projections within each group of three query heads.
/// This reduces K maps/state/updates; queries and reads still have all 9 heads.
pub fn shared_kv_counterfactual(cost: &HarmonicCost) -> Result<SharedKvCounterfactual> {
    let head_sum = sum(&[HEADS, KV_HEADS])?;
    Ok(SharedKvCounterfactual {
        status: "NOT_IMPLEMENTED_DIFFERENT_PARAMETER_SHARING",
        projection_parameters_and_mac_per_layer_position: product(&[head_sum, HEAD_DIM, cost.packed.dimensions])?,
        state_bytes_per_layer: product(&[4, cost.common.batch, KV_HEADS, cost.packed.features, HEAD_DIM + 1])?,
        recurrent_core_mac_per_layer_position: product(&[
            cost.packed.features, sum(&[product(&[HEAD_DIM, head_sum])?, HEADS])?,
        ])?,
        packed_multiplications_per_layer_position: product(&[head_sum, cost.packed.multiplications])?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HybridCost {
    pub status: &'static str,
    pub final_row_support: u64,
    pub final_row_rest_sort_len: u64,
    /// Q.K and q0*k0, excluding norms, subtractions and sort comparisons.
    pub full_scan_mac_per_layer_final_position: u64,
    /// Proposed direct polynomial residual path; projected keys must be stored
    /// or recomputed. Not the cost of invoking the full quadratic helper.
    pub direct_polynomial_correction_mac_per_layer_final_position: u64,
    /// Alternative: feature-dot subtraction plus correction weighted values.
    pub packed_dot_correction_mac_per_layer_final_position: u64,
    pub state_plus_native_kv_history_bytes_all_layers: u64,
    pub optional_projected_key_history_bytes_all_layers: u64,
    pub optional_packed_key_history_bytes_all_layers: u64,
    pub optional_native_key_norm_history_bytes_all_layers: u64,
    /// Three F32 host T*T arrays in current flock_probabilities: Lorentz,
    /// scaled dot, and probability. Device tensors are ADDITIONAL.
    pub current_flock_three_host_gram_payload_bytes: u64,
    /// Current helper copies both Lorentz and scaled dot to host even rank-only.
    pub current_flock_host_copy_bytes: u64,
    /// Supplied-support quadratic hybrid helper separately copies scores, mass
    /// and attended output to host; selector copies above are additional.
    pub quadratic_hybrid_validation_host_copy_bytes: u64,
    /// One U8 keep mask + one F32 replacement tensor, excluding other scores,
    /// autograd retention, host construction copies and support metadata.
    pub quadratic_hybrid_device_mask_and_replacement_bytes: u64,
}

/// Max support at final query: sink + recent window + top-k of remaining keys.
/// The raw hybrid ranks this entire support; normalized flock probabilities
/// cannot be substituted at the same scale. Real-arithmetic polynomial equality
/// does not establish F32 subtraction parity with packed recurrence scores.
pub fn hybrid(cost: &HarmonicCost, k: u64, window: u64) -> Result<HybridCost> {
    if k == 0 || window == 0 {
        return Err(CostError::Invalid("flock k/window must be nonzero"));
    }
    let c = cost.common;
    let forced = c.context.min(sum(&[window, 1])?);
    let rest = c.context - forced;
    let selected = sum(&[forced, rest.min(k)])?;
    let entries = product(&[c.batch, HEADS, c.context, c.context])?;
    let residual_mac = |width| product(&[HEADS, selected, sum(&[width, HEAD_DIM])?]);
    Ok(HybridCost {
        status: "PROPOSED_CORRECTION_UNRESOLVED_SHARED_SELECTOR_NO_INDEX",
        final_row_support: selected,
        final_row_rest_sort_len: rest,
        full_scan_mac_per_layer_final_position: product(&[HEADS, c.context, HEAD_DIM + 1])?,
        direct_polynomial_correction_mac_per_layer_final_position: residual_mac(cost.packed.dimensions)?,
        packed_dot_correction_mac_per_layer_final_position: residual_mac(cost.packed.features)?,
        state_plus_native_kv_history_bytes_all_layers: sum(&[cost.state_bytes_all_layers, c.native_kv_history_bytes_all_layers])?,
        optional_projected_key_history_bytes_all_layers: product(&[4, LAYERS, c.positions, HEADS, cost.packed.dimensions])?,
        optional_packed_key_history_bytes_all_layers: product(&[4, LAYERS, c.positions, HEADS, cost.packed.features])?,
        optional_native_key_norm_history_bytes_all_layers: product(&[4, LAYERS, c.positions, KV_HEADS])?,
        current_flock_three_host_gram_payload_bytes: product(&[3, c.one_gram_bytes])?,
        current_flock_host_copy_bytes: product(&[2, c.one_gram_bytes])?,
        quadratic_hybrid_validation_host_copy_bytes: sum(&[
            c.one_gram_bytes, product(&[4, c.positions, HEADS])?,
            product(&[4, c.positions, HEADS, HEAD_DIM])?,
        ])?,
        quadratic_hybrid_device_mask_and_replacement_bytes: product(&[5, entries])?,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IsolatedFitForwardCost {
    pub teacher_capture_mac_per_window: u64,
    pub reconstructed_base_qkv_mac_per_window: u64,
    /// Includes delta LoRA, learned maps when present, attention, frozen WO.
    pub student_forward_mac_per_window: u64,
    /// Excludes BACKWARD/optimizer, normalization, softmax/polynomial scalar
    /// work, validation copies and optional baseline/evaluation extra passes.
    pub combined_forward_mac_per_window: u64,
}

/// Current streaming fit: one detached teacher capture, reconstructed QKV,
/// one student forward. `layer` is zero-based. Prior teacher layers execute MLP;
/// selected layer stops after WO, with no final norm/vocabulary head.
/// `None` selects the paired dense control; Some(d,L) selects quadratic training.
pub fn isolated_fit_forward(
    batch: u64, context: u64, layer: u64, arm: Option<(u64, u64)>,
) -> Result<IsolatedFitForwardCost> {
    if layer >= LAYERS {
        return Err(CostError::Invalid("capture layer must be 0..29"));
    }
    let c = common(batch, context)?;
    let attention_matrices = sum(&[
        c.base_qkv_mac_per_layer_position, c.output_projection_mac_per_layer_position,
    ])?;
    let capture_per_position = sum(&[
        product(&[layer, sum(&[attention_matrices, c.mlp_mac_per_layer_position])?])?,
        attention_matrices,
        product(&[layer + 1, c.dense_core_mac_per_layer_position])?,
    ])?;
    let (projection, core) = match arm {
        None => (0, c.dense_core_mac_per_layer_position),
        Some((d, degree)) => {
            let h = harmonic(batch, context, d, degree)?;
            (h.projection_parameters_and_mac_per_layer_position, h.quadratic_core_mac_per_layer_position)
        }
    };
    let capture = product(&[c.positions, capture_per_position])?;
    let qkv = product(&[c.positions, c.base_qkv_mac_per_layer_position])?;
    let student = product(&[c.positions, sum(&[
        c.lora_mac_per_layer_position, projection, core,
        c.output_projection_mac_per_layer_position,
    ])?])?;
    Ok(IsolatedFitForwardCost {
        teacher_capture_mac_per_window: capture,
        reconstructed_base_qkv_mac_per_window: qkv,
        student_forward_mac_per_window: student,
        combined_forward_mac_per_window: sum(&[capture, qkv, student])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_dimensions_and_source_arithmetic() -> Result<()> {
        for (d, l, f, scalar_mul) in [
            (16, 1, 17, 17), (16, 2, 152, 575), (16, 3, 952, 5135),
            (32, 1, 33, 33), (32, 2, 560, 2175), (32, 3, 6512, 34079),
        ] {
            let p = packed_features(d, l)?;
            assert_eq!(p.features, f);
            assert_eq!(p.multiplications, scalar_mul);
        }
        Ok(())
    }

    #[test]
    fn persistent_state_uses_nine_heads_not_three() -> Result<()> {
        let h = harmonic(1, 2048, 16, 2)?;
        assert_eq!(h.state_bytes_per_layer, 355_680);
        assert_eq!(h.state_bytes_all_layers, 10_670_400);
        assert_eq!(h.recurrent_core_mac_per_layer_position, 176_472);
        let tied = shared_kv_counterfactual(&h)?;
        assert_eq!(product(&[3, tied.state_bytes_per_layer])?, h.state_bytes_per_layer);
        assert_eq!(tied.recurrent_core_mac_per_layer_position, 118_104);
        Ok(())
    }

    #[test]
    fn quadratic_allocations_scale_as_square_even_when_state_is_constant() -> Result<()> {
        let small = harmonic(1, 512, 32, 3)?;
        let large = harmonic(1, 8192, 32, 3)?;
        assert_eq!(small.state_bytes_per_layer, large.state_bytes_per_layer);
        assert_eq!(large.common.one_gram_bytes, 2_415_919_104);
        assert_eq!(large.common.shared_causal_mask_bytes, 67_108_864);
        assert_eq!(large.common.one_gram_bytes, product(&[256, small.common.one_gram_bytes])?);
        assert_eq!(large.common.native_kv_history_bytes_all_layers, 377_487_360);
        Ok(())
    }

    #[test]
    fn frozen_weights_and_parameters_do_not_confuse_tied_embedding_with_two_copies() -> Result<()> {
        let h = harmonic(1, 512, 16, 1)?;
        assert_eq!(h.common.unique_frozen_f32_weight_bytes, 538_060_032);
        assert_eq!(h.common.frozen_matrix_mac_per_model_position, 134_479_872);
        assert_eq!(h.common.lora_parameters_per_layer, 10_752);
        assert_eq!(h.trainable_parameters_per_layer, 29_184);
        assert_eq!(h.trainable_f32_bytes_all_layers, 3_502_080);
        Ok(())
    }

    #[test]
    fn degree_changes_recurrence_not_quadratic_gemm_width() -> Result<()> {
        let a = harmonic(1, 2048, 32, 1)?;
        let b = harmonic(1, 2048, 32, 3)?;
        assert_eq!(a.quadratic_core_mac_per_layer_window, b.quadratic_core_mac_per_layer_window);
        assert_ne!(a.recurrent_core_mac_per_layer_position, b.recurrent_core_mac_per_layer_position);
        assert_eq!(b.quadratic_core_mac_per_layer_position, 1_769_472);
        Ok(())
    }

    #[test]
    fn batch_changes_total_memory_and_work_not_per_position_cost() -> Result<()> {
        let one = harmonic(1, 512, 16, 2)?;
        let two = harmonic(2, 512, 16, 2)?;
        assert_eq!(two.state_bytes_per_layer, product(&[2, one.state_bytes_per_layer])?);
        assert_eq!(two.quadratic_core_mac_per_layer_window, product(&[2, one.quadratic_core_mac_per_layer_window])?);
        assert_eq!(two.recurrent_core_mac_per_layer_position, one.recurrent_core_mac_per_layer_position);
        assert_eq!(two.common.shared_causal_mask_bytes, one.common.shared_causal_mask_bytes);
        Ok(())
    }

    #[test]
    fn hybrid_keeps_history_and_does_not_turn_selection_into_top_k_only() -> Result<()> {
        let h = harmonic(1, 2048, 16, 2)?;
        let x = hybrid(&h, 7, 64)?;
        assert_eq!(x.final_row_support, 72);
        assert_eq!(x.final_row_rest_sort_len, 1983);
        assert_eq!(x.full_scan_mac_per_layer_final_position, 1_198_080);
        assert_eq!(x.direct_polynomial_correction_mac_per_layer_final_position, 51_840);
        assert_eq!(x.state_plus_native_kv_history_bytes_all_layers, 105_042_240);
        assert_eq!(hybrid(&harmonic(1, 1, 16, 1)?, 7, 64)?.final_row_support, 1);
        Ok(())
    }

    #[test]
    fn capture_stops_after_selected_output_projection() -> Result<()> {
        let c = isolated_fit_forward(1, 512, 0, None)?;
        assert_eq!(c.teacher_capture_mac_per_window, 754_974_720);
        let c = isolated_fit_forward(1, 512, 15, Some((16, 2)))?;
        assert_eq!(c.teacher_capture_mac_per_window, 32_463_912_960);
        Ok(())
    }

    #[test]
    fn invalid_and_overflow_requests_fail_before_wrapping() {
        assert!(common(0, 512).is_err());
        assert!(packed_features(64, 2).is_err());
        assert!(packed_features(16, 4).is_err());
        assert_eq!(common(u64::MAX, 8192), Err(CostError::Overflow));
        assert_eq!(common(1, u64::MAX), Err(CostError::Overflow));
        assert!(isolated_fit_forward(1, 512, 30, None).is_err());
    }
}
