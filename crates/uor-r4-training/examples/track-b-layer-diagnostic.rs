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
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: track-b-layer-diagnostic MODEL SEALED_PARENT NEW_REPORT".into());
    }
    let out = Path::new(&args[2]);
    report_output::claim(out)?;
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        if rx
            .recv_timeout(std::time::Duration::from_secs(180))
            .is_err()
        {
            eprintln!("UNSEALED_TIMEOUT: diagnostic 180-second bound");
            std::process::exit(124);
        }
    });
    let result = run(Path::new(&args[0]), Path::new(&args[1]), out);
    let (value, code) = match result {
        Ok(v) => {
            let code = if v["anchors_match_bitwise"] == true {
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
