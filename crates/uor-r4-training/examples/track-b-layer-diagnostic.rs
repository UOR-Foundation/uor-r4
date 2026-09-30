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

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let layer12 = args.len() == 4 && args[3] == "replay-layer12";
    let block11 = args.len() == 4 && args[3] == "replay-block11";
    let replay = layer12 || block11;
    if args.len() != 3 && !replay {
        return Err(
            "usage: track-b-layer-diagnostic MODEL SEALED_PARENT NEW_REPORT [replay-layer12|replay-block11]"
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
    let result = if block11 {
        replay_block11(Path::new(&args[0]), Path::new(&args[1]), out)
    } else if layer12 {
        replay_layer12(Path::new(&args[0]), Path::new(&args[1]), out)
    } else {
        run(Path::new(&args[0]), Path::new(&args[1]), out)
    };
    let (value, code) = match result {
        Ok(v) => {
            let code = if v["anchors_match_bitwise"] == true
                || (replay && v["parent_anchors_verified"] == true)
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
