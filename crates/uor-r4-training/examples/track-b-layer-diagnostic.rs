//! Diagnostic only: preserve the failed parity window's batch/time shape and
//! check final-logit identity before interpreting captured layer differences.
use candle_core::{Device, Tensor};
use serde_json::{json, Value};
use std::{fs, io::Write, num::NonZeroUsize, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_model_source::{
    HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherOracle, TraceCaptureRequest,
    TraceCaptureSinks,
};
use uor_r4_training::{
    sha256_file,
    track_b::model::{AttentionKernel, DenseAttention, TrackBModel},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const T: usize = 8;
const V: usize = 49152;

fn floats(path: &Path) -> Result<Vec<f32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 4 != 0 {
        return Err("unaligned F32 file".into());
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
fn write_floats(path: &Path, values: &[f32]) -> Result<()> {
    let mut f = fs::File::create_new(path)?;
    for x in values {
        f.write_all(&x.to_le_bytes())?;
    }
    Ok(())
}
fn compare(a: &[f32], b: &[f32]) -> Result<Value> {
    if a.is_empty() || a.len() != b.len() || a.iter().chain(b).any(|x| !x.is_finite()) {
        return Err("nonfinite or mismatched diagnostic rows".into());
    }
    let mut max = 0.0_f64;
    let mut squared = 0.0;
    let mut energy = 0.0;
    let mut bits = 0;
    for (&a, &b) in a.iter().zip(b) {
        let d = f64::from(b) - f64::from(a);
        max = max.max(d.abs());
        squared += d * d;
        energy += f64::from(a) * f64::from(a);
        bits += usize::from(a.to_bits() != b.to_bits());
    }
    Ok(json!({"max_abs":max,"rms":(squared/a.len() as f64).sqrt(),
        "reference_rms":(energy/a.len() as f64).sqrt(),"different_bits":bits,"elements":a.len()}))
}
fn time_major(x: &Tensor) -> candle_core::Result<Vec<f32>> {
    x.transpose(1, 2)?
        .contiguous()?
        .flatten_all()?
        .to_vec1::<f32>()
}
fn run(model_path: &Path, parent: &Path, out: &Path) -> Result<Value> {
    if std::env::var("TLESS_CANONICAL_DETERMINISTIC").as_deref() != Ok("0")
        || std::env::var_os("TLESS_EXACT_SCALAR").is_some()
    {
        return Err("use canonical math=0 and unset exact scalar, matching parent".into());
    }
    report_output::verify(parent)?;
    let input: Value = serde_json::from_slice(&fs::read(parent.join("inputs.json"))?)?;
    if input["windows"][2] != json!([1, 2, 3, 4, 5, 6, 7, 8]) || input["dtype"] != "F32" {
        return Err("parent does not carry the fixed failing window".into());
    }
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        if input["sha256"][name] != sha256_file(&model_path.join(name))? {
            return Err(format!("parent input mismatch: {name}").into());
        }
    }
    let source = option_env!("TRACK_B_SOURCE_REVISION").ok_or("missing source revision")?;
    let diff = option_env!("TRACK_B_SOURCE_DIFF_SHA256").ok_or("missing source diff")?;
    fs::write(
        out.join("inputs.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision":source,"source_diff_sha256":diff,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "parent_manifest_sha256":sha256_file(&parent.join("manifest.json"))?,
            "parent_inputs":input,"tokens":[1,2,3,4,5,6,7,8],"batch":1,"time":8,
            "scope":"diagnostic only; original full shape; no gate or production arithmetic change",
            "trace_layout":"QKV: layer, position, concatenated Q/K/V in native head order; residual: layer, position, width"
        }))?,
    )?;
    let start = Instant::now();
    let workers = NonZeroUsize::new(2).ok_or("zero workers")?;
    let mut oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
        model_path,
        32,
        TeacherExecutionConfig::fixed_workers(workers),
    )?;
    if oracle.exact_backend_report().arithmetic_owner != "uor-matmul exact GEMM" {
        return Err("wrong reference arithmetic".into());
    }
    let geometry = oracle.trace_capture_geometry().ok_or("no trace geometry")?;
    let width = geometry.residual_width;
    let kv = width * geometry.kv_heads / geometry.heads;
    let stride = width + 2 * kv;
    let layers: Vec<usize> = (0..geometry.layers).collect();
    let mut reference = vec![0f32; geometry.layers * T * stride];
    let mut residual = vec![0f32; geometry.layers * T * width];
    let mut reference_logits = vec![0f32; T * V];
    let request = TraceCaptureRequest {
        residual_layers: &layers,
        qkv_layers: &layers,
        attention_layers: &[],
    };
    for pos in 0..T {
        let mut qkv_count = 0;
        let mut residual_count = 0;
        let mut qkv_sink = |layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
            let offset = (layer * T + pos) * stride;
            reference[offset..offset + width].copy_from_slice(q);
            reference[offset + width..offset + width + kv].copy_from_slice(k);
            reference[offset + width + kv..offset + stride].copy_from_slice(v);
            qkv_count += 1;
        };
        let mut residual_sink = |layer: usize, x: &[f32]| {
            let offset = (layer * T + pos) * width;
            residual[offset..offset + width].copy_from_slice(x);
            residual_count += 1;
        };
        let mut attention_sink = |_: usize, _: usize, _: &[f32]| {};
        if !oracle.step_with_trace_capture(
            pos + 1,
            pos,
            &mut reference_logits[pos * V..(pos + 1) * V],
            &request,
            &mut TraceCaptureSinks {
                residual: &mut residual_sink,
                qkv: &mut qkv_sink,
                attention: &mut attention_sink,
            },
        ) {
            return Err("reference trace unavailable".into());
        }
        if qkv_count != geometry.layers || residual_count != geometry.layers {
            return Err("incomplete reference capture".into());
        }
        eprintln!(
            "reference position={pos} elapsed_s={:.3}",
            start.elapsed().as_secs_f64()
        );
    }
    drop(oracle);
    write_floats(&out.join("reference-qkv.f32le"), &reference)?;
    write_floats(&out.join("reference-residual.f32le"), &residual)?;
    write_floats(&out.join("reference-logits.f32le"), &reference_logits)?;
    let saved = floats(&parent.join("reference-logits.f32le"))?;
    let saved_window = saved.get(5 * V..13 * V).ok_or("short saved reference")?;
    let reference_anchor = compare(saved_window, &reference_logits)?;
    let mut anchors_match = reference_anchor["different_bits"] == 0;
    let mut results = Vec::new();
    for name in ["cpu", "metal"] {
        let device = if name == "cpu" {
            Device::Cpu
        } else {
            Device::new_metal(0)?
        };
        let model = TrackBModel::load(model_path, &device)?;
        if model.shape().width != width
            || model.shape().layers != geometry.layers
            || model.shape().vocab != V
            || model.shape().heads != geometry.heads
            || model.shape().kv_heads != geometry.kv_heads
        {
            return Err("trace geometry mismatch".into());
        }
        let mut captured = vec![0f32; reference.len()];
        let mut count = 0;
        let ids: Vec<u32> = (1..=T as u32).collect();
        let mut dense = DenseAttention;
        let logits = model
            .forward_with_attention_fn(&ids, 1, T, |layer, qkv, positions| {
                let q = time_major(&qkv.query)?;
                let k = time_major(&qkv.key)?;
                let v = time_major(&qkv.value)?;
                for pos in 0..T {
                    let offset = (layer * T + pos) * stride;
                    captured[offset..offset + width]
                        .copy_from_slice(&q[pos * width..(pos + 1) * width]);
                    captured[offset + width..offset + width + kv]
                        .copy_from_slice(&k[pos * kv..(pos + 1) * kv]);
                    captured[offset + width + kv..offset + stride]
                        .copy_from_slice(&v[pos * kv..(pos + 1) * kv]);
                }
                count += 1;
                dense.attend(layer, qkv, positions)
            })?
            .flatten_all()?
            .to_vec1::<f32>()?;
        if count != geometry.layers {
            return Err("incomplete candidate capture".into());
        }
        let saved = floats(&parent.join(format!("{name}-shared-full-logits.f32le")))?;
        let anchor = compare(
            saved.get(5 * V..13 * V).ok_or("short saved candidate")?,
            &logits,
        )?;
        anchors_match &= anchor["different_bits"] == 0;
        let mut rows = Vec::new();
        for layer in 0..geometry.layers {
            for pos in 0..T {
                let offset = (layer * T + pos) * stride;
                for (kind, start, end) in [
                    ("q", 0, width),
                    ("k", width, width + kv),
                    ("v", width + kv, stride),
                ] {
                    rows.push(json!({"layer":layer,"position":pos,"kind":kind,
                    "difference":compare(&reference[offset+start..offset+end],&captured[offset+start..offset+end])?}));
                }
            }
        }
        write_floats(&out.join(format!("{name}-qkv.f32le")), &captured)?;
        write_floats(&out.join(format!("{name}-logits.f32le")), &logits)?;
        results.push(json!({"backend":name,"saved_logits_anchor":anchor,
            "reference_logit_difference":compare(&reference_logits,&logits)?,"rows":rows}));
    }
    Ok(
        json!({"schema":"uor-r4.track-b-layer-diagnostic/1","status":if anchors_match {"DIAGNOSTIC_COMPLETE"} else {"ANCHOR_MISMATCH"},
        "anchors_match_bitwise":anchors_match,"reference_anchor":reference_anchor,"backends":results,
        "elapsed_seconds":start.elapsed().as_secs_f64(),"parity_pass":false,
        "scope":"layer QKV divergence; no isolated-operator causal attribution or parity qualification"}),
    )
}
// Same arithmetic as the fixed shared layer, on saved reference residuals.
// This diagnostic deliberately does not patch a model or change the gate.
fn replay_layer12(model_path: &Path, parent: &Path, out: &Path) -> Result<Value> {
    use candle_core::DType;
    use uor_r4_training::kappa_llama::rope;
    if std::env::var("TLESS_CANONICAL_DETERMINISTIC").as_deref() != Ok("0")
        || std::env::var_os("TLESS_EXACT_SCALAR").is_some()
    {
        return Err("requires native math matching the trace".into());
    }
    report_output::verify(parent)?;
    let inputs: Value = serde_json::from_slice(&fs::read(parent.join("inputs.json"))?)?;
    let prior: Value = serde_json::from_slice(&fs::read(parent.join("result.json"))?)?;
    if prior["anchors_match_bitwise"] != true
        || inputs["tokens"] != json!([1, 2, 3, 4, 5, 6, 7, 8])
        || inputs["batch"] != 1
        || inputs["time"] != 8
    {
        return Err("requires the bitwise-anchored eight-token trace".into());
    }
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        if inputs["parent_inputs"]["sha256"][name] != sha256_file(&model_path.join(name))? {
            return Err(format!("trace input mismatch: {name}").into());
        }
    }
    fs::write(
        out.join("inputs.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision":option_env!("TRACK_B_SOURCE_REVISION").ok_or("missing source")?,
            "source_diff_sha256":option_env!("TRACK_B_SOURCE_DIFF_SHA256").ok_or("missing source diff")?,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "parent_manifest_sha256":sha256_file(&parent.join("manifest.json"))?,
            "parent_inputs":inputs,"layer":12,"input_residual_after_layer":11,"batch":1,"time":8,
            "scope":"same reference residual input, native RMS versus reference scalar RMS; no full forward"
        }))?,
    )?;
    let residual = floats(&parent.join("reference-residual.f32le"))?;
    let reference = floats(&parent.join("reference-qkv.f32le"))?;
    let mut backends = Vec::new();
    for name in ["cpu", "metal"] {
        let device = if name == "cpu" {
            Device::Cpu
        } else {
            Device::new_metal(0)?
        };
        let model = TrackBModel::load(model_path, &device)?;
        let shape = model.shape();
        if shape.layers != 30
            || shape.width != 576
            || shape.heads != 9
            || shape.kv_heads != 3
            || shape.head_dim != 64
            || shape.rms_eps != 1e-5
        {
            return Err("unexpected fixed replay geometry or epsilon".into());
        }
        let width = shape.width;
        let kv = shape.kv_heads * shape.head_dim;
        let stride = width + 2 * kv;
        if residual.len() != 30 * T * width || reference.len() != 30 * T * stride {
            return Err("incomplete trace arrays".into());
        }
        let x = &residual[11 * T * width..12 * T * width];
        let target = &reference[12 * T * stride..13 * T * stride];
        let observed = floats(&parent.join(format!("{name}-qkv.f32le")))?;
        if observed.len() != reference.len() {
            return Err("incomplete candidate trace".into());
        }
        let observed = &observed[12 * T * stride..13 * T * stride];
        let weights = model.attention_weights(12)?;
        let gain = weights.input_norm.to_vec1::<f32>()?;
        // Exact operation order from model-source rmsnorm_with_mode, native
        // math mode: sequential F32 sum, reciprocal sqrt, weight*(r*x).
        let mut scalar_norm = vec![0f32; T * width];
        for (row, output) in x
            .chunks_exact(width)
            .zip(scalar_norm.chunks_exact_mut(width))
        {
            let mut ss = row.iter().map(|v| v * v).sum::<f32>();
            ss /= width as f32;
            ss += 1e-5f32;
            ss = 1.0f32 / ss.sqrt();
            for ((output, value), weight) in output.iter_mut().zip(row).zip(&gain) {
                *output = *weight * (ss * *value);
            }
        }
        let xt = Tensor::from_slice(x, (T, width), &device)?;
        let denominator = xt
            .sqr()?
            .mean_keepdim(1)?
            .affine(1.0, shape.rms_eps)?
            .sqrt()?;
        let native_norm = xt
            .broadcast_div(&denominator)?
            .broadcast_mul(&weights.input_norm)?;
        let normalization_difference =
            compare(&scalar_norm, &native_norm.flatten_all()?.to_vec1::<f32>()?)?;
        // Match model.rs candle_f32_rope_tables, not the older F64 Kappa table helper.
        let frequency: Vec<f32> = (0..shape.head_dim)
            .step_by(2)
            .map(|index| {
                1.0f32 / (shape.rope_theta as f32).powf(index as f32 / shape.head_dim as f32)
            })
            .collect();
        let frequency = Tensor::from_vec(frequency, (1, shape.head_dim / 2), &device)?;
        let positions = Tensor::arange(0u32, T as u32, &device)?
            .to_dtype(DType::F32)?
            .reshape((T, 1))?;
        let phase = positions.matmul(&frequency)?;
        let cosine = phase.cos()?.reshape((1, 1, T, shape.head_dim / 2))?;
        let sine = phase.sin()?.reshape((1, 1, T, shape.head_dim / 2))?;
        let mut arms = Vec::new();
        for (arm, norm) in [
            ("native_rms", native_norm),
            (
                "reference_scalar_rms",
                Tensor::from_slice(&scalar_norm, (T, width), &device)?,
            ),
        ] {
            let project = |weight: &Tensor, heads: usize| -> Result<Tensor> {
                Ok(norm
                    .matmul(&weight.t()?)?
                    .reshape((1, T, heads, shape.head_dim))?
                    .transpose(1, 2)?
                    .contiguous()?)
            };
            let q = time_major(&rope(
                &project(&weights.query, shape.heads)?,
                &cosine,
                &sine,
            )?)?;
            let k = time_major(&rope(
                &project(&weights.key, shape.kv_heads)?,
                &cosine,
                &sine,
            )?)?;
            let v = time_major(&project(&weights.value, shape.kv_heads)?)?;
            let mut packed = Vec::with_capacity(T * stride);
            for pos in 0..T {
                packed.extend_from_slice(&q[pos * width..(pos + 1) * width]);
                packed.extend_from_slice(&k[pos * kv..(pos + 1) * kv]);
                packed.extend_from_slice(&v[pos * kv..(pos + 1) * kv]);
            }
            write_floats(&out.join(format!("{name}-{arm}-qkv.f32le")), &packed)?;
            let mut rows = Vec::new();
            for pos in 0..T {
                for (kind, start, end) in [
                    ("q", 0, width),
                    ("k", width, width + kv),
                    ("v", width + kv, stride),
                ] {
                    let start = pos * stride + start;
                    let end = pos * stride + end;
                    rows.push(json!({"position":pos,"kind":kind,
                    "replay_vs_reference":compare(&target[start..end],&packed[start..end])?,
                    "original_candidate_vs_reference":compare(&target[start..end],&observed[start..end])?,
                    "replay_vs_original_candidate":compare(&observed[start..end],&packed[start..end])?}));
                }
            }
            arms.push(json!({"arm":arm,"rows":rows}));
        }
        write_floats(
            &out.join(format!("{name}-reference-normalized.f32le")),
            &scalar_norm,
        )?;
        backends.push(
            json!({"backend":name,"normalization_difference":normalization_difference,"arms":arms}),
        );
    }
    Ok(
        json!({"schema":"uor-r4.track-b-layer12-replay/1","status":"DIAGNOSTIC_COMPLETE",
        "parent_anchors_verified":true,"parity_pass":false,"backends":backends,
        "scope":"common-input local replay; no correction or full parity qualification"}),
    )
}

// A local replay, not a replacement reference executor. Only the final residual
// and QKV have saved reference anchors; other stages report substitution effects.
fn replay_block11(model_path: &Path, parent: &Path, out: &Path) -> Result<Value> {
    use candle_core::DType;
    use uor_r4_model_source::attention::{
        head_attention_value_aggregate, standard_head_attention_weights,
    };
    use uor_r4_training::{
        kappa_llama::{load_checkpoint, rope},
        track_b::model::{dense_attention, AttentionPositions, AttentionQkv},
    };
    if std::env::var("TLESS_CANONICAL_DETERMINISTIC").as_deref() != Ok("0")
        || std::env::var_os("TLESS_EXACT_SCALAR").is_some()
    {
        return Err("requires native math matching the trace".into());
    }
    report_output::verify(parent)?;
    let inputs: Value = serde_json::from_slice(&fs::read(parent.join("inputs.json"))?)?;
    let prior: Value = serde_json::from_slice(&fs::read(parent.join("result.json"))?)?;
    if prior["anchors_match_bitwise"] != true
        || inputs["tokens"] != json!([1, 2, 3, 4, 5, 6, 7, 8])
        || inputs["batch"] != 1
        || inputs["time"] != 8
    {
        return Err("requires the bitwise-anchored eight-token trace".into());
    }
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        if inputs["parent_inputs"]["sha256"][name] != sha256_file(&model_path.join(name))? {
            return Err(format!("trace input mismatch: {name}").into());
        }
    }
    fs::write(
        out.join("inputs.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision":option_env!("TRACK_B_SOURCE_REVISION").ok_or("missing source")?,
            "source_diff_sha256":option_env!("TRACK_B_SOURCE_DIFF_SHA256").ok_or("missing diff")?,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "parent_manifest_sha256":sha256_file(&parent.join("manifest.json"))?,
            "parent_inputs":inputs,"layer":11,"input_residual_after_layer":10,"batch":1,"time":8,
            "scope":"common reference input; native, reference QKV, reference QKV and source attention; no full forward"
        }))?,
    )?;
    let residual = floats(&parent.join("reference-residual.f32le"))?;
    let reference = floats(&parent.join("reference-qkv.f32le"))?;
    const W: usize = 576;
    const KV: usize = 192;
    const STRIDE: usize = W + 2 * KV;
    if residual.len() != 30 * T * W || reference.len() != 30 * T * STRIDE {
        return Err("incomplete fixed trace arrays".into());
    }
    let x = &residual[10 * T * W..11 * T * W];
    let target = &residual[11 * T * W..12 * T * W];
    let ref_qkv = &reference[11 * T * STRIDE..12 * T * STRIDE];
    let rq: Vec<f32> = ref_qkv
        .chunks_exact(STRIDE)
        .flat_map(|r| r[..W].iter().copied())
        .collect();
    let rk: Vec<f32> = ref_qkv
        .chunks_exact(STRIDE)
        .flat_map(|r| r[W..W + KV].iter().copied())
        .collect();
    let rv: Vec<f32> = ref_qkv
        .chunks_exact(STRIDE)
        .flat_map(|r| r[W + KV..].iter().copied())
        .collect();
    let mut source_attended = vec![0f32; T * W];
    for pos in 0..T {
        for head in 0..9 {
            let mut att = vec![0f32; pos + 1];
            standard_head_attention_weights(
                &mut att,
                &rq[pos * W + head * 64..pos * W + (head + 1) * 64],
                &rk,
                (head / 3) * 64,
                KV,
                false,
            );
            head_attention_value_aggregate(
                &mut source_attended[pos * W + head * 64..pos * W + (head + 1) * 64],
                &att,
                &rv,
                (head / 3) * 64,
                KV,
            );
        }
    }
    write_floats(&out.join("source-attended.f32le"), &source_attended)?;
    let mut backends = Vec::new();
    for name in ["cpu", "metal"] {
        let device = if name == "cpu" {
            Device::Cpu
        } else {
            Device::new_metal(0)?
        };
        let checkpoint = load_checkpoint(model_path, &device)?;
        let shape = &checkpoint.shape;
        if shape.layers != 30
            || shape.width != W
            || shape.heads != 9
            || shape.kv_heads != 3
            || shape.head_dim != 64
            || shape.rms_eps != 1e-5
        {
            return Err("unexpected fixed replay geometry".into());
        }
        let weight = |suffix: &str| -> Result<&Tensor> {
            checkpoint
                .tensors
                .get(&format!("model.layers.11.{suffix}.weight"))
                .ok_or_else(|| format!("missing weight {suffix}").into())
        };
        let rms = |input: &Tensor, gain: &Tensor| -> Result<Tensor> {
            let denominator = input
                .sqr()?
                .mean_keepdim(1)?
                .affine(1.0, shape.rms_eps)?
                .sqrt()?;
            Ok(input.broadcast_div(&denominator)?.broadcast_mul(gain)?)
        };
        let xt = Tensor::from_slice(x, (T, W), &device)?;
        let norm = rms(&xt, weight("input_layernorm")?)?;
        let frequencies: Vec<f32> = (0..64)
            .step_by(2)
            .map(|i| 1f32 / (shape.rope_theta as f32).powf(i as f32 / 64f32))
            .collect();
        let frequency = Tensor::from_vec(frequencies, (1, 32), &device)?;
        let positions = Tensor::arange(0u32, T as u32, &device)?
            .to_dtype(DType::F32)?
            .reshape((T, 1))?;
        let phase = positions.matmul(&frequency)?;
        let cosine = phase.cos()?.reshape((1, 1, T, 32))?;
        let sine = phase.sin()?.reshape((1, 1, T, 32))?;
        let project = |suffix: &str, heads: usize| -> Result<Tensor> {
            Ok(norm
                .matmul(&weight(suffix)?.t()?)?
                .reshape((1, T, heads, 64))?
                .transpose(1, 2)?
                .contiguous()?)
        };
        let nq = rope(&project("self_attn.q_proj", 9)?, &cosine, &sine)?;
        let nk = rope(&project("self_attn.k_proj", 3)?, &cosine, &sine)?;
        let nv = project("self_attn.v_proj", 3)?;
        let saved = |values: &[f32], heads: usize| -> Result<Tensor> {
            Ok(Tensor::from_slice(values, (1, T, heads, 64), &device)?
                .transpose(1, 2)?
                .contiguous()?)
        };
        let mut qkv_rows = Vec::new();
        for (kind, expected, actual, width) in
            [("q", &rq, &nq, W), ("k", &rk, &nk, KV), ("v", &rv, &nv, KV)]
        {
            let actual = time_major(actual)?;
            for pos in 0..T {
                qkv_rows.push(json!({"kind":kind,"position":pos,"difference":compare(&expected[pos*width..(pos+1)*width],&actual[pos*width..(pos+1)*width])?}));
            }
        }
        let mask: Vec<u8> = (0..T)
            .flat_map(|i| (0..T).map(move |j| u8::from(j > i)))
            .collect();
        let excluded = Tensor::from_vec(mask, (1, 1, T, T), &device)?;
        let attend = |query: Tensor, key: Tensor, value: Tensor| -> Result<Tensor> {
            let qkv = AttentionQkv {
                query,
                key,
                value,
                excluded: excluded.clone(),
                normalized_input: norm.reshape((1, T, W))?,
                cosine: cosine.clone(),
                sine: sine.clone(),
            };
            Ok(dense_attention(
                &qkv,
                &AttentionPositions {
                    query: 0..T,
                    key: 0..T,
                },
            )?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((T, W))?)
        };
        let native_attended = attend(nq, nk, nv)?;
        let saved_attended = attend(saved(&rq, 9)?, saved(&rk, 3)?, saved(&rv, 3)?)?;
        let mut baseline: Vec<Vec<f32>> = Vec::new();
        let mut arms = Vec::new();
        for (arm, attended) in [
            ("native", native_attended),
            ("reference_qkv", saved_attended),
            (
                "reference_qkv_source_attention",
                Tensor::from_slice(&source_attended, (T, W), &device)?,
            ),
        ] {
            let mapped = attended.matmul(&weight("self_attn.o_proj")?.t()?)?;
            let after_attention = xt.add(&mapped)?;
            let post_norm = rms(&after_attention, weight("post_attention_layernorm")?)?;
            let gate_pre = post_norm.matmul(&weight("mlp.gate_proj")?.t()?)?;
            let gate = gate_pre.silu()?;
            let up = post_norm.matmul(&weight("mlp.up_proj")?.t()?)?;
            let gated = gate.mul(&up)?;
            let down = gated.matmul(&weight("mlp.down_proj")?.t()?)?;
            let output = after_attention.add(&down)?;
            let mut stages = Vec::new();
            for (index, (stage, tensor)) in [
                ("attended", &attended),
                ("attention_output", &mapped),
                ("after_attention", &after_attention),
                ("post_norm", &post_norm),
                ("gate_pre", &gate_pre),
                ("gate", &gate),
                ("up", &up),
                ("gated", &gated),
                ("down", &down),
                ("output", &output),
            ]
            .into_iter()
            .enumerate()
            {
                let values = tensor.flatten_all()?.to_vec1::<f32>()?;
                let stage_width = values.len() / T;
                write_floats(&out.join(format!("{name}-{arm}-{stage}.f32le")), &values)?;
                if arm == "native" {
                    baseline.push(values.clone());
                }
                let mut rows = Vec::new();
                for pos in 0..T {
                    let range = pos * stage_width..(pos + 1) * stage_width;
                    rows.push(json!({"position":pos,"vs_native_arm":compare(&baseline[index][range.clone()],&values[range.clone()])?,
                        "vs_saved_reference":if stage=="output" {compare(&target[range.clone()],&values[range])?} else {Value::Null}}));
                }
                stages.push(json!({"stage":stage,"width":stage_width,"rows":rows}));
            }
            arms.push(json!({"arm":arm,"stages":stages}));
        }
        backends.push(json!({"backend":name,"common_input_qkv":qkv_rows,"arms":arms}));
    }
    Ok(
        json!({"schema":"uor-r4.track-b-block11-replay/1","status":"DIAGNOSTIC_COMPLETE","parent_anchors_verified":true,
        "parity_pass":false,"backends":backends,"scope":"common-input block and attention substitutions; only QKV and final residual have saved reference anchors"}),
    )
}

fn replay_down(model_path: &Path, parent: &Path, out: &Path) -> Result<Value> {
    use uor_r4_training::kappa_llama::load_checkpoint;
    report_output::verify(parent)?;
    let inputs: Value = serde_json::from_slice(&fs::read(parent.join("inputs.json"))?)?;
    let prior: Value = serde_json::from_slice(&fs::read(parent.join("result.json"))?)?;
    if prior["schema"] != "uor-r4.track-b-block11-replay/1"
        || prior["parent_anchors_verified"] != true
        || prior["status"] != "DIAGNOSTIC_COMPLETE"
        || inputs["layer"] != 11
        || inputs["batch"] != 1
        || inputs["time"] != 8
    {
        return Err("requires completed fixed block11 replay".into());
    }
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        if inputs["parent_inputs"]["parent_inputs"]["sha256"][name]
            != sha256_file(&model_path.join(name))?
        {
            return Err(format!("parent input mismatch: {name}").into());
        }
    }
    fs::write(
        out.join("inputs.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision":option_env!("TRACK_B_SOURCE_REVISION").ok_or("missing source")?,
            "source_diff_sha256":option_env!("TRACK_B_SOURCE_DIFF_SHA256").ok_or("missing diff")?,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "parent_manifest_sha256":sha256_file(&parent.join("manifest.json"))?,
            "parent_inputs":inputs,"exact_revision":uor_r4_model_source::UOR_MATMUL_REVISION,
            "batch":1,"time":8,"scope":"same saved gated inputs through unchanged CPU/Metal and pinned exact down projection"
        }))?,
    )?;
    let checkpoint = load_checkpoint(model_path, &Device::Cpu)?;
    const W: usize = 576;
    const K: usize = 1536;
    if checkpoint.shape.width != W || checkpoint.shape.ffn != K || checkpoint.shape.layers != 30 {
        return Err("unexpected down replay geometry".into());
    }
    let weight = checkpoint
        .tensors
        .get("model.layers.11.mlp.down_proj.weight")
        .ok_or("missing down weight")?
        .clone();
    if weight.dims() != [W, K] {
        return Err("unexpected down weight shape".into());
    }
    let weight_bits = weight.flatten_all()?.to_vec1::<f32>()?;
    drop(checkpoint);
    let metal = Device::new_metal(0)?;
    let mut rows_by_input = Vec::new();
    let mut all_exact = Vec::new();
    let mut all_inputs = Vec::new();
    let mut anchors_match = true;
    for origin in ["cpu", "metal"] {
        let x = floats(&parent.join(format!("{origin}-native-gated.f32le")))?;
        let original = floats(&parent.join(format!("{origin}-native-down.f32le")))?;
        if x.len() != T * K
            || original.len() != T * W
            || x.iter().chain(&weight_bits).any(|v| !v.is_finite())
        {
            return Err("invalid saved projection inputs".into());
        }
        // Exact executor's portable shared-weight layout: W[W,K] * X^T[K,T],
        // followed by transpose to the model's [T,W] output. Each dot rounds once.
        let mut xt = vec![0f32; K * T];
        for pos in 0..T {
            for depth in 0..K {
                xt[depth * T + pos] = x[pos * K + depth];
            }
        }
        let mut exact_t = vec![0f32; W * T];
        let mut pa = vec![uor_matmul::PackedCode::default(); K];
        let mut pb = vec![uor_matmul::PackedCode::default(); K * T];
        uor_matmul::slice::gemm_float(W, K, T, &weight_bits, &xt, &mut exact_t, &mut pa, &mut pb)
            .map_err(|e| format!("exact projection failed: {e:?}"))?;
        let mut exact = vec![0f32; T * W];
        for pos in 0..T {
            for row in 0..W {
                exact[pos * W + row] = exact_t[row * T + pos];
            }
        }
        write_floats(
            &out.join(format!("{origin}-input-exact-down.f32le")),
            &exact,
        )?;
        let mut backends = Vec::new();
        for (name, device) in [("cpu", &Device::Cpu), ("metal", &metal)] {
            let actual = Tensor::from_slice(&x, (T, K), device)?
                .matmul(&weight.to_device(device)?.t()?)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            write_floats(
                &out.join(format!("{origin}-input-{name}-down.f32le")),
                &actual,
            )?;
            let anchor = if name == origin {
                let a = compare(&original, &actual)?;
                anchors_match &= a["different_bits"] == 0;
                a
            } else {
                Value::Null
            };
            let mut rows = Vec::new();
            for pos in 0..T {
                rows.push(json!({"position":pos,"local_vs_exact":compare(&exact[pos*W..(pos+1)*W],&actual[pos*W..(pos+1)*W])?}));
            }
            backends.push(json!({"backend":name,"matching_saved_anchor":anchor,"rows":rows}));
        }
        rows_by_input.push(json!({"input_origin":origin,"backends":backends}));
        all_inputs.push(x);
        all_exact.push(exact);
    }
    let mut propagated = Vec::new();
    for pos in 0..T {
        propagated.push(json!({"position":pos,"gated_input_difference":compare(&all_inputs[0][pos*K..(pos+1)*K],&all_inputs[1][pos*K..(pos+1)*K])?,
            "exact_output_difference":compare(&all_exact[0][pos*W..(pos+1)*W],&all_exact[1][pos*W..(pos+1)*W])?}));
    }
    Ok(
        json!({"schema":"uor-r4.track-b-down-replay/1","status":if anchors_match {"DIAGNOSTIC_COMPLETE"} else {"ANCHOR_MISMATCH"},
        "anchors_match_bitwise":anchors_match,"parent_anchors_verified":true,"parity_pass":false,"inputs":rows_by_input,"propagated":propagated,
        "scope":"local down-projection rounding versus exact, and propagation of two saved gated inputs; not full reference block error"}),
    )
}

fn exact_projection(x: &[f32], weight: &Tensor) -> Result<Vec<f32>> {
    let (rows, k) = weight.dims2()?;
    if x.len() != T * k || x.iter().any(|v| !v.is_finite()) {
        return Err("invalid exact input".into());
    }
    let w = weight.flatten_all()?.to_vec1::<f32>()?;
    let mut xt = vec![0f32; k * T];
    for pos in 0..T {
        for d in 0..k {
            xt[d * T + pos] = x[pos * k + d];
        }
    }
    let mut yt = vec![0f32; rows * T];
    let mut pa = vec![uor_matmul::PackedCode::default(); k];
    let mut pb = vec![uor_matmul::PackedCode::default(); k * T];
    uor_matmul::slice::gemm_float(rows, k, T, &w, &xt, &mut yt, &mut pa, &mut pb)
        .map_err(|e| format!("exact projection: {e:?}"))?;
    let mut y = vec![0f32; rows * T];
    for pos in 0..T {
        for row in 0..rows {
            y[pos * rows + row] = yt[row * T + pos];
        }
    }
    Ok(y)
}
fn replay_mlp(model_path: &Path, parent: &Path, trace: &Path, out: &Path) -> Result<Value> {
    use uor_r4_training::kappa_llama::load_checkpoint;
    report_output::verify(parent)?;
    report_output::verify(trace)?;
    let inputs: Value = serde_json::from_slice(&fs::read(parent.join("inputs.json"))?)?;
    let prior: Value = serde_json::from_slice(&fs::read(parent.join("result.json"))?)?;
    if prior["schema"] != "uor-r4.track-b-block11-replay/1"
        || prior["status"] != "DIAGNOSTIC_COMPLETE"
        || prior["parent_anchors_verified"] != true
        || inputs["parent_manifest_sha256"] != sha256_file(&trace.join("manifest.json"))?
        || inputs["layer"] != 11
        || inputs["batch"] != 1
        || inputs["time"] != 8
    {
        return Err("requires linked fixed block11 and trace reports".into());
    }
    for name in [
        "config.json",
        "model.safetensors",
        "tokenizer.json",
        "tokenizer_config.json",
    ] {
        if inputs["parent_inputs"]["parent_inputs"]["sha256"][name]
            != sha256_file(&model_path.join(name))?
        {
            return Err(format!("model hash mismatch: {name}").into());
        }
    }
    fs::write(
        out.join("inputs.json"),
        serde_json::to_vec_pretty(&json!({
            "source_revision":option_env!("TRACK_B_SOURCE_REVISION").ok_or("missing source")?,
            "source_diff_sha256":option_env!("TRACK_B_SOURCE_DIFF_SHA256").ok_or("missing diff")?,
            "executable_sha256":sha256_file(&std::env::current_exe()?)?,"parent_manifest_sha256":sha256_file(&parent.join("manifest.json"))?,
            "parent_inputs":inputs,"trace_manifest_sha256":sha256_file(&trace.join("manifest.json"))?,
            "exact_revision":uor_r4_model_source::UOR_MATMUL_REVISION,"layer":11,"batch":1,"time":8,
            "scope":"anchored exact reference tail reconstruction and isolated common-input projection/activation substitutions"
        }))?,
    )?;
    const W: usize = 576;
    const K: usize = 1536;
    let ck = load_checkpoint(model_path, &Device::Cpu)?;
    if ck.shape.width != W || ck.shape.ffn != K || ck.shape.layers != 30 || ck.shape.rms_eps != 1e-5
    {
        return Err("unexpected geometry".into());
    }
    let weight = |suffix: &str| -> Result<&Tensor> {
        ck.tensors
            .get(&format!("model.layers.11.{suffix}.weight"))
            .ok_or_else(|| format!("missing {suffix}").into())
    };
    let residual = floats(&trace.join("reference-residual.f32le"))?;
    let attended = floats(&parent.join("source-attended.f32le"))?;
    if residual.len() != 30 * T * W || attended.len() != T * W {
        return Err("incomplete reference arrays".into());
    }
    let x = &residual[10 * T * W..11 * T * W];
    let target = &residual[11 * T * W..12 * T * W];
    let mapped = exact_projection(&attended, weight("self_attn.o_proj")?)?;
    let after: Vec<f32> = x.iter().zip(&mapped).map(|(a, b)| a + b).collect();
    let gain = weight("post_attention_layernorm")?.to_vec1::<f32>()?;
    let reference_rms = |values: &[f32]| -> Result<Vec<f32>> {
        if values.len() != T * W {
            return Err("incomplete RMS input".into());
        }
        let mut norm = vec![0f32; T * W];
        for (r, o) in values.chunks_exact(W).zip(norm.chunks_exact_mut(W)) {
            let mut ss = r.iter().map(|v| v * v).sum::<f32>();
            ss /= W as f32;
            ss += 1e-5f32;
            ss = 1f32 / ss.sqrt();
            for ((v, x), g) in o.iter_mut().zip(r).zip(&gain) {
                *v = *g * (ss * *x);
            }
        }
        Ok(norm)
    };
    let norm = reference_rms(&after)?;
    let gate_pre = exact_projection(&norm, weight("mlp.gate_proj")?)?;
    let up = exact_projection(&norm, weight("mlp.up_proj")?)?;
    let reference_silu = |x: &[f32]| -> Vec<f32> {
        x.iter()
            .map(|v| *v * (1f32 / (1f32 + (-*v).exp())))
            .collect()
    };
    let gate = reference_silu(&gate_pre);
    let gated: Vec<f32> = gate.iter().zip(&up).map(|(a, b)| a * b).collect();
    let down = exact_projection(&gated, weight("mlp.down_proj")?)?;
    let output: Vec<f32> = after.iter().zip(&down).map(|(a, b)| a + b).collect();
    let anchor = compare(target, &output)?;
    let anchored = anchor["different_bits"] == 0;
    let stages = [
        ("attended", &attended),
        ("attention_output", &mapped),
        ("after_attention", &after),
        ("post_norm", &norm),
        ("gate_pre", &gate_pre),
        ("gate", &gate),
        ("up", &up),
        ("gated", &gated),
        ("down", &down),
        ("output", &output),
    ];
    let mut original_rows = Vec::new();
    for (stage, values) in stages {
        write_floats(&out.join(format!("reference-{stage}.f32le")), values)?;
        let width = values.len() / T;
        for backend in ["cpu", "metal"] {
            let saved = floats(&parent.join(format!("{backend}-native-{stage}.f32le")))?;
            if saved.len() != values.len() {
                return Err("incomplete saved stage".into());
            }
            for pos in 0..T {
                original_rows.push(json!({"backend":backend,"stage":stage,"position":pos,"difference":compare(&values[pos*width..(pos+1)*width],&saved[pos*width..(pos+1)*width])?}));
            }
        }
    }
    // Fail closed: reconstructed intermediates are not interpreted if the final
    // residual cannot reproduce the original independent trace.
    if !anchored {
        return Ok(
            json!({"status":"ANCHOR_MISMATCH","anchors_match_bitwise":false,"anchor":anchor,"parity_pass":false}),
        );
    }
    let mut arms: Vec<(String, Vec<f32>, Vec<f32>)> = Vec::new();
    let mut local_rows = Vec::new();
    for name in ["cpu", "metal"] {
        let device = if name == "cpu" {
            Device::Cpu
        } else {
            Device::new_metal(0)?
        };
        let nt = Tensor::from_slice(&norm, (T, W), &device)?;
        let project = |suffix: &str| -> Result<Vec<f32>> {
            Ok(nt
                .matmul(&weight(suffix)?.to_device(&device)?.t()?)?
                .flatten_all()?
                .to_vec1::<f32>()?)
        };
        let gp = project("mlp.gate_proj")?;
        let u = project("mlp.up_proj")?;
        let native_gate = Tensor::from_slice(&gate_pre, (T, K), &device)?
            .silu()?
            .flatten_all()?
            .to_vec1::<f32>()?;
        for (kind, expected, actual) in [
            ("gate_projection", &gate_pre, &gp),
            ("up_projection", &up, &u),
            ("silu", &gate, &native_gate),
        ] {
            write_floats(&out.join(format!("{name}-{kind}.f32le")), actual)?;
            for pos in 0..T {
                local_rows.push(json!({"backend":name,"kind":kind,"position":pos,"difference":compare(&expected[pos*K..(pos+1)*K],&actual[pos*K..(pos+1)*K])?}));
            }
        }
        arms.push((
            format!("{name}-gate-projection-only"),
            reference_silu(&gp),
            up.clone(),
        ));
        arms.push((format!("{name}-up-projection-only"), gate.clone(), u));
        arms.push((format!("{name}-silu-only"), native_gate, up.clone()));
        let at = Tensor::from_slice(&after, (T, W), &device)?;
        let den = at.sqr()?.mean_keepdim(1)?.affine(1.0, 1e-5)?.sqrt()?;
        let native_norm = at
            .broadcast_div(&den)?
            .broadcast_mul(&weight("post_attention_layernorm")?.to_device(&device)?)?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let saved_after = floats(&parent.join(format!("{name}-native-after_attention.f32le")))?;
        for (kind, n) in [
            ("rms-only", native_norm),
            ("incoming-attention-only", reference_rms(&saved_after)?),
        ] {
            write_floats(&out.join(format!("{name}-{kind}-norm.f32le")), &n)?;
            let g = reference_silu(&exact_projection(&n, weight("mlp.gate_proj")?)?);
            let u = exact_projection(&n, weight("mlp.up_proj")?)?;
            arms.push((format!("{name}-{kind}"), g, u));
        }
    }
    arms.push((
        "host-division-silu-only".into(),
        gate_pre.iter().map(|v| *v / (1f32 + (-*v).exp())).collect(),
        up.clone(),
    ));
    let mut effects = Vec::new();
    for (name, g, u) in arms {
        let product: Vec<f32> = g.iter().zip(&u).map(|(a, b)| a * b).collect();
        let projected = exact_projection(&product, weight("mlp.down_proj")?)?;
        write_floats(&out.join(format!("{name}-down.f32le")), &projected)?;
        let mut rows = Vec::new();
        for pos in 0..T {
            rows.push(json!({"position":pos,"down_difference":compare(&down[pos*W..(pos+1)*W],&projected[pos*W..(pos+1)*W])?}));
        }
        effects.push(json!({"arm":name,"rows":rows}));
    }
    Ok(
        json!({"schema":"uor-r4.track-b-mlp-replay/1","status":"DIAGNOSTIC_COMPLETE","anchors_match_bitwise":true,"anchor":anchor,
        "parity_pass":false,"original_stage_rows":original_rows,"local_rows":local_rows,"effects":effects,
        "scope":"reference-tail anchor and isolated local substitutions; not a full-model correction"}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let layer12 = args.len() == 4 && args[3] == "replay-layer12";
    let block11 = args.len() == 4 && args[3] == "replay-block11";
    let down = args.len() == 4 && args[3] == "replay-down";
    let mlp = args.len() == 5 && args[3] == "replay-mlp";
    let replay = layer12 || block11 || down || mlp;
    if args.len() != 3 && !replay {
        return Err(
            "usage: track-b-layer-diagnostic MODEL SEALED_PARENT NEW_REPORT [replay-layer12|replay-block11|replay-down|replay-mlp TRACE]"
                .into(),
        );
    }
    let out = Path::new(&args[2]);
    report_output::claim(out)?;
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let wall_seconds = if replay { 60 } else { 180 };
    std::thread::spawn(move || {
        if rx
            .recv_timeout(std::time::Duration::from_secs(wall_seconds))
            .is_err()
        {
            eprintln!("UNSEALED_TIMEOUT: diagnostic {wall_seconds}-second bound");
            std::process::exit(124);
        }
    });
    let result = if mlp {
        replay_mlp(
            Path::new(&args[0]),
            Path::new(&args[1]),
            Path::new(&args[4]),
            out,
        )
    } else if down {
        replay_down(Path::new(&args[0]), Path::new(&args[1]), out)
    } else if block11 {
        replay_block11(Path::new(&args[0]), Path::new(&args[1]), out)
    } else if layer12 {
        replay_layer12(Path::new(&args[0]), Path::new(&args[1]), out)
    } else {
        run(Path::new(&args[0]), Path::new(&args[1]), out)
    };
    let (value, code) = match result {
        Ok(v) => {
            let code = if v["anchors_match_bitwise"] == true
                || (!down && !mlp && replay && v["parent_anchors_verified"] == true)
            {
                0
            } else {
                2
            };
            (v, code)
        }
        Err(e) => (
            json!({"status":"UNAVAILABLE","error":e.to_string(),"parity_pass":false}),
            1,
        ),
    };
    fs::write(out.join("result.json"), serde_json::to_vec_pretty(&value)?)?;
    report_output::seal(out)?;
    report_output::verify(out)?;
    eprintln!(
        "{} anchors_match={}; sealed and verified",
        value["status"], value["anchors_match_bitwise"]
    );
    let _ = tx.send(());
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}
